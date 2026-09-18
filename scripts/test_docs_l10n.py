"""Unit tests for the user-guide catalog tooling (scripts/docs_l10n.py)."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import docs_l10n as d  # noqa: E402


class Segmentation(unittest.TestCase):
    def texts(self, src: str) -> list[str]:
        return d.page_texts(d.segment(src.splitlines()))

    def test_paragraph_lines_are_one_message(self) -> None:
        self.assertEqual(self.texts("A first line\nand a second.\n"), ["A first line and a second."])

    def test_headings_lists_quotes_and_cells(self) -> None:
        src = "# Title\n\n- an item\n  continued\n- other\n\n> quoted\n\n| a | b |\n| --- | --- |\n| c | d |\n"
        self.assertEqual(
            self.texts(src), ["Title", "an item continued", "other", "quoted", "a", "b", "c", "d"]
        )

    def test_code_front_matter_images_and_comments_are_not_extracted(self) -> None:
        src = "---\nkind: x\n---\n\n```bdl\nconcept A\n```\n\n![alt text](a.png)\n\n<!-- note -->\n\nText.\n"
        self.assertEqual(self.texts(src), ["Text."])

    def test_bdl_syntax_inside_fences_is_never_a_message(self) -> None:
        src = "```text\nmapping m: () -> B\n```\n"
        self.assertEqual(self.texts(src), [])


class Catalogs(unittest.TestCase):
    def test_po_round_trip(self) -> None:
        tmp = Path(__file__).parent / "_tmp_test.po"
        try:
            d.write_po(tmp, d.HEADER, [('a "quoted" id\nwith newline', ["x.md:1"], "译文")])
            got = d.read_po(tmp)
            self.assertEqual(got['a "quoted" id\nwith newline'], "译文")
            self.assertIn("Content-Type: text/plain; charset=UTF-8", got[""])
        finally:
            tmp.unlink(missing_ok=True)

    def test_only_supported_locales(self) -> None:
        self.assertEqual(d.LANGS, ("zh_Hans", "ja"))

    def test_render_threshold_needs_title_and_half(self) -> None:
        blocks = d.segment("# T\n\nfirst\n\nsecond\n\nthird\n".splitlines())
        self.assertFalse(d.translated_enough(blocks, {"first": "1", "second": "2"}))
        self.assertTrue(d.translated_enough(blocks, {"T": "t", "first": "1"}))

    def test_links_to_untranslated_pages_point_at_english(self) -> None:
        src_dir = d.SOURCE / "studio"
        out_dir = d.OUT / "ja" / "studio"
        text = "[a](canvas.md) [b](library.md#x) ![i](../assets/p.png) [h](https://x)"
        got = d.rewrite_links(text, src_dir, out_dir, {(src_dir / "canvas.md").resolve()})
        self.assertIn("[a](canvas.md)", got)
        self.assertIn("[b](../../../../docs/user-guide/studio/library.md#x)", got)
        self.assertIn("![i](../../../../docs/user-guide/assets/p.png)", got)
        self.assertIn("[h](https://x)", got)


if __name__ == "__main__":
    unittest.main()
