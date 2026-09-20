#!/usr/bin/env python3
"""Preflight: the one validation contract, local and in CI.

Every check CI runs is a named check here, and every CI job runs a
*profile* of this script — so what a developer validates before a push is
what GitHub Actions validates after it, from one list
(``docs/project/ci.md``).  Read-only by design: nothing here formats,
regenerates into the tree or rewrites a file; ``fix`` is the one explicit
mutating profile and says so.

    python scripts/preflight.py fast       before a commit: formatting, docs
                                           structure, l10n consistency,
                                           generated-file freshness, cargo
                                           check, Dart format, flutter analyze
    python scripts/preflight.py full       before a push: everything the
                                           Linux jobs prove
    python scripts/preflight.py platform   what this host can prove of the
                                           Windows job (the real bdld and the
                                           filesystem-sensitive Studio tests;
                                           the native build on Windows only)
    python scripts/preflight.py ci         the Linux Rust + Flutter + proto
                                           contract, exactly
    python scripts/preflight.py list       every check and profile
    python scripts/preflight.py fix        run the formatters (mutates)

CI profiles (one per job): ``rust-ci``, ``flutter-ci``, ``proto-ci``,
``windows-rust-ci``, ``windows-flutter-ci``, ``windows-build-ci``.

Individual checks run by name: ``python scripts/preflight.py rust-format
l10n``.  ``--verbose`` streams every command's output (the default under
``GITHUB_ACTIONS``); otherwise output is shown for failures only.  Each
check is timed and the summary lists the slow ones.

Works from cmd / PowerShell / bash on Windows, macOS and Linux: file
enumeration goes through ``git ls-files`` and comparisons of generated
output are done here, byte for byte, in scratch directories under
``target/preflight`` — never in the tree.
"""

from __future__ import annotations

import filecmp
import os
import platform
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

ROOT = Path(__file__).resolve().parent.parent
STUDIO = ROOT / "apps/studio"
SCRATCH = ROOT / "target/preflight"
WINDOWS = platform.system() == "Windows"
CI = os.environ.get("GITHUB_ACTIONS") == "true"

PRETTIER = ["npx", "--yes", "prettier@3.9.7"]
MARKDOWNLINT = ["npx", "--yes", "markdownlint-cli2@0.23.2"]
DART_WIDTH = ["--page-width", "100"]

# The crates whose tests touch what differs between platforms — paths,
# drive letters, file URIs, process spawning, the filesystem, the host
# toolchain — and so run on the Windows job as well as on Linux
# (docs/project/ci.md).  Everything else in the workspace is
# platform-independent semantics that Linux proves once.
WINDOWS_RUST_CRATES = {
    "bdl-daemon": "spawns bdld.exe in its CLI, stdio, text, system and deploy e2e tests; project files on disk",
    "bdl-lsp": "file:// URIs with drive letters and backslashes (position, e2e, text_workspace)",
    "bdl-text": "source discovery and write-back over real paths and line endings",
    "bdl-model": "project persistence: manifest, layout, identities on the filesystem",
    "bdl-ide-db": "document URIs and text workspaces",
    "bdl-compiler": "golden files byte for byte (LF checkout) and generated crates built by the host cargo",
    "bdl-runtime-host": "the harness that runs cargo and the generated host binary",
}

# The Dart tags (apps/studio/dart_test.yaml) whose tests can behave
# differently on Windows: the real daemon process, the filesystem.
WINDOWS_FLUTTER_TAGS = "daemon || filesystem"


class Failure(Exception):
    def __init__(self, message: str, hint: str = ""):
        super().__init__(message)
        self.hint = hint


@dataclass
class Check:
    name: str
    title: str
    run: Callable[["Runner"], None]
    tools: tuple[str, ...] = ()
    hint: str = ""
    # `None`: every host; else the platform.system() values it runs on.
    hosts: tuple[str, ...] | None = None
    mutates: bool = False


@dataclass
class Outcome:
    name: str
    title: str
    status: str  # PASS | FAIL | SKIP
    seconds: float
    detail: str = ""


@dataclass
class Runner:
    verbose: bool
    log: list[str] = field(default_factory=list)

    def cmd(
        self,
        args: list[str],
        cwd: Path = ROOT,
        env: dict[str, str] | None = None,
        hint: str = "",
    ) -> str:
        """Run a command; raise Failure with its output on a non-zero exit."""
        shown = " ".join(args)
        if cwd != ROOT:
            shown = f"(cd {cwd.relative_to(ROOT)}; {shown})"
        full_env = {**os.environ, **(env or {})}
        started = time.monotonic()
        if self.verbose:
            print(f"    $ {shown}", flush=True)
            p = subprocess.run(args, cwd=cwd, env=full_env, shell=WINDOWS and args[0] in NPX_LIKE)
            output = ""
        else:
            p = subprocess.run(
                args,
                cwd=cwd,
                env=full_env,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                shell=WINDOWS and args[0] in NPX_LIKE,
            )
            output = (p.stdout or "") + (p.stderr or "")
        elapsed = time.monotonic() - started
        if p.returncode != 0:
            raise Failure(
                f"command failed (exit {p.returncode}, {elapsed:.1f}s):\n  {shown}\n{output.rstrip()}",
                hint,
            )
        return output


# Commands that are .cmd shims on Windows and need the shell to resolve.
NPX_LIKE = {"npx", "npm", "flutter", "dart", "protoc"}


def tracked(pattern: str) -> list[str]:
    """Files matching a git pathspec, cross-platform: the tracked ones and
    the untracked, not-ignored ones — so a page written this session is
    formatted and checked before it is committed, not first by CI."""
    out = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", pattern],
        cwd=ROOT,
        capture_output=True,
        check=True,
    ).stdout
    return sorted({p for p in out.decode("utf-8").split("\0") if p})


def chunked(items: list[str], limit: int = 6000) -> list[list[str]]:
    """Argument lists short enough for a Windows command line."""
    chunks: list[list[str]] = [[]]
    size = 0
    for it in items:
        if size + len(it) + 1 > limit and chunks[-1]:
            chunks.append([])
            size = 0
        chunks[-1].append(it)
        size += len(it) + 1
    return chunks


def compare_trees(expected: Path, actual: Path, what: str) -> None:
    """Byte-for-byte comparison of two directories; Failure names the files."""
    problems: list[str] = []

    def walk(e: Path, a: Path) -> None:
        cmp = filecmp.dircmp(e, a)
        problems.extend(f"only in {what}: {Path(cmp.left).relative_to(expected) / n}" for n in cmp.left_only)
        problems.extend(f"only regenerated: {Path(cmp.right).relative_to(actual) / n}" for n in cmp.right_only)
        for n in cmp.common_files:
            if (e / n).read_bytes() != (a / n).read_bytes():
                problems.append(f"differs: {(e / n).relative_to(expected)}")
        for n in cmp.common_dirs:
            walk(e / n, a / n)

    walk(expected, actual)
    if problems:
        raise Failure(f"{what} is stale:\n  " + "\n  ".join(problems))


def scratch(name: str) -> Path:
    d = SCRATCH / name
    if d.exists():
        shutil.rmtree(d)
    d.mkdir(parents=True)
    return d


def bdld_path() -> Path:
    if p := os.environ.get("BDLD_PATH"):
        return Path(p)
    return ROOT / "target/debug" / ("bdld.exe" if WINDOWS else "bdld")


# ---- checks -----------------------------------------------------------------


def rust_format(r: Runner) -> None:
    r.cmd(["cargo", "fmt", "--all", "--", "--check"], hint="cargo fmt --all")


def docs_format(r: Runner) -> None:
    for chunk in chunked(tracked("*.md")):
        r.cmd(PRETTIER + ["--log-level", "warn", "--check"] + chunk, hint="just docs-fmt")


def docs_lint(r: Runner) -> None:
    for chunk in chunked(tracked("*.md")):
        r.cmd(MARKDOWNLINT + chunk, hint="just docs-fmt")


def docs_validate(r: Runner) -> None:
    r.cmd([sys.executable, "scripts/validate_docs.py"])
    r.cmd([sys.executable, "-m", "unittest", "scripts/test_validate_docs.py"])


def ci_scripts(r: Runner) -> None:
    """The helper scripts the workflow calls besides this one
    (scripts/slim_flutter_sdk.py) keep their contract."""
    r.cmd([sys.executable, "-m", "unittest", "scripts/test_slim_flutter_sdk.py"])


def screenshots(r: Runner) -> None:
    r.cmd([sys.executable, "scripts/check_screenshots.py"])


def l10n(r: Runner) -> None:
    r.cmd([sys.executable, "scripts/check_l10n.py"])
    # the Standard Library's presentation strings in the ARB catalogs and
    # lib/l10n/library_strings.dart follow library/std/concepts.toml and
    # locale/library/std.json
    r.cmd([sys.executable, "scripts/gen_library_l10n.py", "--check"], hint="just library-l10n")
    r.cmd([sys.executable, "-m", "unittest", "scripts/test_docs_l10n.py", "scripts/test_gen_library_l10n.py"])
    # the rendered user-guide catalogs and pages must be current: regenerate
    # into a scratch copy and compare, never into the tree
    out = scratch("locale-user-guide")
    shutil.rmtree(out)
    shutil.copytree(ROOT / "locale/user-guide", out)
    r.cmd(
        [sys.executable, "scripts/docs_l10n.py", "--out", str(out), "all"],
        hint="just docs-l10n, then commit locale/",
    )
    compare_trees(ROOT / "locale/user-guide", out, "locale/user-guide")


def rust_check(r: Runner) -> None:
    r.cmd(["cargo", "check", "--workspace", "--all-targets"])


def rust_clippy(r: Runner) -> None:
    r.cmd(["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])


def rust_test(r: Runner) -> None:
    r.cmd(["cargo", "test", "--workspace"])


def rust_build_daemon(r: Runner) -> None:
    r.cmd(["cargo", "build", "-p", "bdl-daemon"])


def flutter_deps(r: Runner) -> None:
    r.cmd(["flutter", "pub", "get"], cwd=STUDIO)


def dart_format(r: Runner) -> None:
    r.cmd(
        ["dart", "format", *DART_WIDTH, "--output=none", "--set-exit-if-changed", "lib", "test"],
        cwd=STUDIO,
        hint="cd apps/studio && dart format --page-width 100 lib test",
    )


def flutter_analyze(r: Runner) -> None:
    r.cmd(["flutter", "analyze"], cwd=STUDIO)


def studio_l10n_generated(r: Runner) -> None:
    """`flutter gen-l10n` output checked in is current: generate into a
    scratch directory and compare."""
    out = scratch("studio-l10n")
    r.cmd(["flutter", "gen-l10n", "--output-dir", str(out)], cwd=STUDIO, hint="just studio-l10n")
    for f in sorted(out.glob("app_localizations*.dart")):
        checked_in = STUDIO / "lib/l10n" / f.name
        if not checked_in.exists() or checked_in.read_bytes() != f.read_bytes():
            raise Failure(
                f"apps/studio/lib/l10n/{f.name} is stale", "just studio-l10n, then commit lib/l10n/"
            )


def flutter_test(r: Runner) -> None:
    r.cmd(["flutter", "test"], cwd=STUDIO, env={"BDLD_PATH": str(bdld_path())})


def proto_generated(r: Runner) -> None:
    """The checked-in Dart protobuf code is what protoc + dart format give
    for the current .proto (Rust regenerates at build time)."""
    out = scratch("proto")
    proto = ROOT / "crates/bdl-protocol/proto"
    r.cmd(
        ["protoc", "-I", str(proto), f"--dart_out={out}", str(proto / "bdl/v1/bdl.proto")],
        hint="just proto (needs protoc and `dart pub global activate protoc_plugin` on PATH)",
    )
    r.cmd(["dart", "format", *DART_WIDTH, str(out)], cwd=STUDIO)
    compare_trees(STUDIO / "lib/protocol/gen/bdl", out / "bdl", "apps/studio/lib/protocol/gen/bdl")


def windows_rust(r: Runner) -> None:
    """The Windows compatibility set of the Rust workspace (WINDOWS_RUST_CRATES):
    the crates whose tests meet paths, URIs, processes, files or the host
    toolchain.  bdld.exe is built with them for the Studio tests."""
    args = ["cargo", "test"]
    for crate in WINDOWS_RUST_CRATES:
        args += ["-p", crate]
    r.cmd(args)


def windows_flutter(r: Runner) -> None:
    """The Studio tests tagged `daemon` or `filesystem` (apps/studio/dart_test.yaml)
    against the real bdld of this host."""
    bdld = bdld_path()
    if not bdld.exists():
        raise Failure(f"no daemon at {bdld}", "cargo build -p bdl-daemon, or set BDLD_PATH")
    r.cmd(
        ["flutter", "test", "--tags", WINDOWS_FLUTTER_TAGS],
        cwd=STUDIO,
        env={"BDLD_PATH": str(bdld)},
    )


def windows_build(r: Runner) -> None:
    """The Windows native target compiles and links (a debug build is the
    cheapest command that proves it; `flutter analyze` does not)."""
    r.cmd(["flutter", "build", "windows", "--debug"], cwd=STUDIO)


def fix_formatting(r: Runner) -> None:
    r.cmd(["cargo", "fmt", "--all"])
    r.cmd(["dart", "format", *DART_WIDTH, "lib", "test"], cwd=STUDIO)
    files = tracked("*.md")
    for chunk in chunked(files):
        r.cmd(PRETTIER + ["--log-level", "warn", "--write"] + chunk)
    for chunk in chunked(files):
        r.cmd([sys.executable, "scripts/md_normalize.py"] + chunk)
    for chunk in chunked(files):
        r.cmd(MARKDOWNLINT + ["--fix"] + chunk)


CHECKS: dict[str, Check] = {
    c.name: c
    for c in [
        Check("rust-format", "Rust formatting", rust_format, ("cargo",), "cargo fmt --all"),
        Check("docs-format", "Markdown formatting (Prettier)", docs_format, ("npx", "git"), "just docs-fmt"),
        Check("docs-lint", "Markdown lint", docs_lint, ("npx", "git"), "just docs-fmt"),
        Check("docs-validate", "Engineering records", docs_validate),
        Check("ci-scripts", "CI helper scripts", ci_scripts),
        Check("screenshots", "Screenshot ledger", screenshots),
        Check("l10n", "Localization catalogs and rendered pages", l10n, hint="just docs-l10n"),
        Check("rust-check", "cargo check (all targets)", rust_check, ("cargo",)),
        Check("rust-clippy", "Clippy (-D warnings)", rust_clippy, ("cargo",)),
        Check("rust-test", "Rust tests (workspace)", rust_test, ("cargo",)),
        Check("rust-build-daemon", "Build bdld", rust_build_daemon, ("cargo",)),
        Check("flutter-deps", "flutter pub get", flutter_deps, ("flutter",)),
        Check("dart-format", "Dart formatting", dart_format, ("dart",)),
        Check("flutter-analyze", "flutter analyze", flutter_analyze, ("flutter",)),
        Check("studio-l10n-generated", "Studio localization classes current", studio_l10n_generated, ("flutter",), "just studio-l10n"),
        Check("flutter-test", "Flutter tests (all)", flutter_test, ("flutter",)),
        Check("proto-generated", "Generated Dart protobuf current", proto_generated, ("protoc", "dart"), "just proto"),
        Check("windows-rust", "Windows-compatibility Rust tests", windows_rust, ("cargo",)),
        Check("windows-flutter", "Daemon and filesystem Studio tests", windows_flutter, ("flutter",)),
        Check("windows-build", "Windows native build (debug)", windows_build, ("flutter",), hosts=("Windows",)),
        Check("fix-formatting", "Formatters (rewrites files)", fix_formatting, ("cargo", "dart", "npx"), mutates=True),
    ]
}

# What each profile proves is recorded in docs/project/ci.md; the CI jobs
# call the `*-ci` profiles by name.
PROFILES: dict[str, list[str]] = {
    "fast": [
        "rust-format",
        "docs-validate",
        "ci-scripts",
        "l10n",
        "proto-generated",
        "rust-check",
        "dart-format",
        "flutter-analyze",
        "studio-l10n-generated",
    ],
    "full": [
        "rust-format",
        "docs-format",
        "docs-lint",
        "docs-validate",
        "ci-scripts",
        "screenshots",
        "l10n",
        "rust-clippy",
        "rust-test",
        "flutter-deps",
        "dart-format",
        "flutter-analyze",
        "studio-l10n-generated",
        "flutter-test",
        "proto-generated",
    ],
    "platform": ["rust-build-daemon", "windows-rust", "flutter-deps", "windows-flutter", "windows-build"],
    # the CI jobs, one profile each
    "rust-ci": [
        "rust-format",
        "docs-format",
        "docs-lint",
        "docs-validate",
        "ci-scripts",
        "screenshots",
        "l10n",
        "rust-clippy",
        "rust-test",
    ],
    "flutter-ci": ["flutter-deps", "dart-format", "flutter-analyze", "studio-l10n-generated", "flutter-test"],
    "proto-ci": ["proto-generated"],
    "windows-rust-ci": ["windows-rust"],
    "windows-flutter-ci": ["flutter-deps", "windows-flutter"],
    "windows-build-ci": ["flutter-deps", "windows-build"],
    "fix": ["fix-formatting"],
}
PROFILES["ci"] = PROFILES["rust-ci"] + PROFILES["flutter-ci"] + PROFILES["proto-ci"]


# ---- driver -----------------------------------------------------------------


def tool_available(name: str) -> bool:
    return shutil.which(name) is not None or (WINDOWS and shutil.which(name + ".cmd") is not None)


def missing_tools(checks: list[Check]) -> dict[str, list[str]]:
    missing: dict[str, list[str]] = {}
    for c in checks:
        for t in c.tools:
            if not tool_available(t):
                missing.setdefault(t, []).append(c.name)
    return missing


TOOL_HELP = {
    "cargo": "install Rust (https://rustup.rs); the workspace pins the toolchain in CI to 1.89.0",
    "flutter": "install Flutter 3.47.4 (stable) and put `flutter` on PATH",
    "dart": "comes with Flutter; put its bin on PATH",
    "npx": "install Node 22; Prettier and markdownlint run through npx",
    "protoc": "install protoc 25.x and `dart pub global activate protoc_plugin` (~/.pub-cache/bin on PATH)",
    "git": "install Git",
}


def fmt_seconds(s: float) -> str:
    if s < 60:
        return f"{s:.1f}s"
    m, sec = divmod(int(round(s)), 60)
    return f"{m}m {sec:02d}s"


def run(names: list[str], verbose: bool, allow_mutation: bool) -> int:
    checks = [CHECKS[n] for n in names]
    if any(c.mutates for c in checks) and not allow_mutation:
        print("preflight is read-only; `python scripts/preflight.py fix` runs the formatters explicitly")
        return 2
    runnable = [c for c in checks if c.hosts is None or platform.system() in c.hosts]
    missing = missing_tools(runnable)
    # tools are checked up front: in CI a missing tool is an error before
    # anything runs; locally the checks that need it are skipped, visibly,
    # and the rest still run
    if missing:
        for tool, users in missing.items():
            print(f"missing tool `{tool}` (needed by {', '.join(users)}): {TOOL_HELP.get(tool, '')}")
        if CI:
            return 2
    outcomes: list[Outcome] = []
    width = len(checks)
    started_all = time.monotonic()
    failed = False
    for i, c in enumerate(checks, 1):
        label = f"[{i}/{width}] {c.title} ..."
        if c.hosts is not None and platform.system() not in c.hosts:
            print(f"{label} SKIP (runs on {', '.join(c.hosts)} only; this is {platform.system()})")
            outcomes.append(Outcome(c.name, c.title, "SKIP", 0.0))
            continue
        absent = [t for t in c.tools if not tool_available(t)]
        if absent:
            print(f"{label} SKIP (no `{absent[0]}` on this machine)")
            outcomes.append(Outcome(c.name, c.title, "SKIP", 0.0))
            continue
        print(label, flush=True)
        r = Runner(verbose=verbose)
        started = time.monotonic()
        try:
            c.run(r)
        except Failure as f:
            elapsed = time.monotonic() - started
            print(f"{label} FAIL {fmt_seconds(elapsed)}")
            print(f"  {f}")
            hint = f.hint or c.hint
            if hint:
                print(f"  hint: {hint}")
            outcomes.append(Outcome(c.name, c.title, "FAIL", elapsed, str(f)))
            failed = True
            continue
        elapsed = time.monotonic() - started
        print(f"{label} PASS {fmt_seconds(elapsed)}")
        outcomes.append(Outcome(c.name, c.title, "PASS", elapsed))
    total = time.monotonic() - started_all
    print()
    print(f"Total: {fmt_seconds(total)}")
    for o in sorted(outcomes, key=lambda o: -o.seconds):
        if o.status != "SKIP":
            print(f"  {o.title:<44} {fmt_seconds(o.seconds):>8}  {o.status}")
    skipped = [o for o in outcomes if o.status == "SKIP"]
    if skipped:
        print("  skipped: " + ", ".join(o.name for o in skipped))
    if failed:
        print("\nFAILED: " + ", ".join(o.name for o in outcomes if o.status == "FAIL"))
        return 1
    return 0


def list_all() -> None:
    print("checks:")
    for c in CHECKS.values():
        where = "" if c.hosts is None else f"  [{', '.join(c.hosts)} only]"
        print(f"  {c.name:<24} {c.title}{where}")
    print("\nprofiles:")
    for p, names in PROFILES.items():
        print(f"  {p:<20} {' '.join(names)}")


def main(argv: list[str]) -> int:
    args = [a for a in argv if not a.startswith("--")]
    flags = {a for a in argv if a.startswith("--")}
    verbose = "--verbose" in flags or CI
    if not args or "--help" in flags or "-h" in argv:
        print(__doc__)
        return 0
    if args == ["list"]:
        list_all()
        return 0
    names: list[str] = []
    for a in args:
        if a in PROFILES:
            names.extend(n for n in PROFILES[a] if n not in names)
        elif a in CHECKS:
            if a not in names:
                names.append(a)
        else:
            print(f"unknown check or profile `{a}`; `python scripts/preflight.py list`")
            return 2
    return run(names, verbose, allow_mutation=args == ["fix"] or "fix-formatting" in args and "--fix" in flags)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
