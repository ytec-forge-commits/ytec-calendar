"""Exercise generated manual links and page numbering, not source strings."""
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[1]


class ManualContract(unittest.TestCase):
    def check_manual(self, language):
        script = ROOT / "scripts" / ("generate-operation-manual.py" if language == "ja" else "generate-operation-manual-en.py")
        self.assertTrue(script.is_file(), "English manual generator is required")
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "manual.pdf"
            result = subprocess.run([sys.executable, str(script), "--output", str(output)], capture_output=True)
            self.assertEqual(result.returncode, 0, "Manual generation failed")
            reader = PdfReader(output)
            links = {str(a.get_object().get("/A", {}).get("/URI", "")) for page in reader.pages for a in page.get("/Annots", [])}
            self.assertIn("https://ytec.cloudfree.jp/forge/contact/", links)
            official = "https://ytec.cloudfree.jp/forge/" + ("projects/koyomado/" if language == "ja" else "en/projects/koyomado/")
            self.assertIn(official, links)
            for index, page in enumerate(reader.pages[1:], 2):
                self.assertIn(f"{index} / {len(reader.pages)}", page.extract_text())

    def test_japanese_manual_has_working_contact_link(self):
        self.check_manual("ja")

    def test_english_manual_has_working_contact_link(self):
        self.check_manual("en")


if __name__ == "__main__":
    unittest.main()
