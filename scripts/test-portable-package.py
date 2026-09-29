"""Run portable packaging in a fixture; verify the actual archive contract."""
import os
import shutil
import subprocess
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class PortablePackage(unittest.TestCase):
    def test_portable_archive_includes_english_guide_and_dependency_notices(self):
        with tempfile.TemporaryDirectory(prefix="koyomado-package-test-", dir=ROOT.parent) as temporary:
            fixture = Path(temporary)
            (fixture / "scripts").mkdir()
            shutil.copy2(ROOT / "scripts/package-portable.ps1", fixture / "scripts/package-portable.ps1")
            for name in ["package.json", "LICENSE.txt", "NOTICE", "CHANGELOG.md", "THIRD_PARTY_NOTICES.md", "PRIVACY.md", "CODE_SIGNING_POLICY.md", "ASSET_PROVENANCE.md"]:
                shutil.copy2(ROOT / name, fixture / name)
            for directory in ["docs", "third_party", "src/assets/fonts"]:
                shutil.copytree(ROOT / directory, fixture / directory)
            executable = ROOT / "src-tauri/target/release/koyomado.exe"
            environment = os.environ.copy()
            for key in list(environment):
                if key.lower() == "psmodulepath":
                    del environment[key]
            result = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(fixture / "scripts/package-portable.ps1"), "-SkipBuild", "-ExecutablePath", str(executable)], capture_output=True, env=environment)
            error = result.stderr.decode("utf-8", errors="replace").replace(str(ROOT), "<candidate>").replace(str(fixture), "<fixture>")
            self.assertEqual(result.returncode, 0, "Packaging must finish in the isolated fixture: " + error)
            with zipfile.ZipFile(fixture / "release/koyomado-v1.0.0-windows-portable.zip") as archive:
                self.assertIsNone(archive.testzip())
                names = set(archive.namelist())
                self.assertIn("Koyomado.en.pdf", names)
                self.assertIn("Koyomado操作説明書.pdf", names)
                self.assertIn("README.en.txt", names)
                self.assertIn("third_party/dependency-licenses/index.json", names)
                self.assertIn("third_party/mpl-source/index.json", names)
                self.assertIn("docs/asset-manifest.json", names)
                self.assertEqual(archive.read("Koyomado.en.pdf"), (ROOT / "docs/Koyomado.en.pdf").read_bytes())
                self.assertIn(b"https://ytec.cloudfree.jp/forge/contact/", archive.read("README.en.txt"))
                self.assertNotIn("data/calendar-data.json", names)
                self.assertFalse(any(name.endswith((".pdb", ".log", ".pfx")) or "/.git/" in name for name in names))


if __name__ == "__main__":
    unittest.main()
