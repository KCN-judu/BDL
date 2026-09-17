// The reference VS Code integration of bdl-lsp: a thin client.  Every
// language fact — diagnostics, navigation, rename, completion, formatting,
// inlay hints, semantic tokens — comes from the server; this file only
// starts it, points it at the project root, forwards file changes under
// `src/`, and shows the server's virtual documents (explain, kernel Core,
// generated Rust) in read-only editors.

import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

type VirtualKind = "explain" | "core" | "rust";

interface VirtualDocumentResult {
  uri: string;
  language: string;
  text: string;
}

/** The server's read-only documents, refreshed on every request. */
class VirtualDocuments implements vscode.TextDocumentContentProvider {
  private readonly contents = new Map<string, string>();
  private readonly emitter = new vscode.EventEmitter<vscode.Uri>();
  readonly onDidChange = this.emitter.event;

  provideTextDocumentContent(uri: vscode.Uri): string {
    return this.contents.get(uri.toString()) ?? "";
  }

  async show(kind: VirtualKind, at?: vscode.TextEditor): Promise<void> {
    if (!client) {
      void vscode.window.showWarningMessage("BDL: the language server is not running.");
      return;
    }
    const params: Record<string, unknown> = { kind };
    if (kind === "explain" && at) {
      params.textDocument = { uri: at.document.uri.toString() };
      params.position = at.selection.active;
    }
    const result = await client.sendRequest<VirtualDocumentResult>("bdl/virtualDocument", params);
    const uri = vscode.Uri.parse(result.uri);
    this.contents.set(uri.toString(), result.text);
    this.emitter.fire(uri);
    const doc = await vscode.workspace.openTextDocument(uri);
    await vscode.languages.setTextDocumentLanguage(doc, languageOf(result.language));
    await vscode.window.showTextDocument(doc, { preview: true, viewColumn: vscode.ViewColumn.Beside });
  }
}

function languageOf(serverLanguage: string): string {
  switch (serverLanguage) {
    case "rust":
      return "rust";
    case "markdown":
      return "markdown";
    default:
      return "bdl-core";
  }
}

/** The project root: the nearest folder with a `bdl.toml` above the
 *  active file, else the first workspace folder. */
function projectRoot(): string | undefined {
  const active = vscode.window.activeTextEditor?.document.uri;
  if (active && active.scheme === "file") {
    let dir = path.dirname(active.fsPath);
    for (let i = 0; i < 16; i++) {
      if (fs.existsSync(path.join(dir, "bdl.toml"))) {
        return dir;
      }
      const parent = path.dirname(dir);
      if (parent === dir) {
        break;
      }
      dir = parent;
    }
  }
  const folder = vscode.workspace.workspaceFolders?.[0];
  return folder?.uri.fsPath;
}

async function start(): Promise<void> {
  const config = vscode.workspace.getConfiguration("bdl");
  const command = config.get<string>("serverPath", "bdl-lsp");
  const root = projectRoot();

  const serverOptions: ServerOptions = {
    command,
    transport: TransportKind.stdio,
    options: { env: { ...process.env, RUST_LOG: process.env.RUST_LOG ?? "info" } },
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "bdl" }],
    initializationOptions: { projectRoot: root },
    synchronize: {
      // The server reloads the project when a source file or the
      // identity sidecar changes outside an open buffer.
      fileEvents: vscode.workspace.createFileSystemWatcher("**/{src/**/*.bdl,.bdl/identities.json,bdl.toml}"),
    },
    outputChannelName: "BDL Language Server",
  };
  client = new LanguageClient("bdl", "BDL Language Server", serverOptions, clientOptions);
  try {
    await client.start();
  } catch (e) {
    client = undefined;
    void vscode.window.showErrorMessage(
      `BDL: could not start ${command}. Build it with \`cargo build -p bdl-lsp\` and set bdl.serverPath. (${e})`,
    );
  }
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const virtual = new VirtualDocuments();
  for (const scheme of ["bdl-core", "bdl-explain", "bdl-generated"]) {
    context.subscriptions.push(vscode.workspace.registerTextDocumentContentProvider(scheme, virtual));
  }
  context.subscriptions.push(
    vscode.commands.registerCommand("bdl.showCore", () => virtual.show("core")),
    vscode.commands.registerCommand("bdl.showRust", () => virtual.show("rust")),
    vscode.commands.registerCommand("bdl.explain", () =>
      virtual.show("explain", vscode.window.activeTextEditor),
    ),
    vscode.commands.registerCommand("bdl.restartServer", async () => {
      if (client) {
        await client.stop();
        client = undefined;
      }
      await start();
    }),
  );
  await start();
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}
