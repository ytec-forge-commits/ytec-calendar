"""Exercise signed release assembly with an explicitly supplied public certificate."""
import hashlib
import os
import shutil
import subprocess
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class SignedRelease(unittest.TestCase):
    def test_signed_release_includes_a_consistent_english_manual(self):
        certificate_file = os.environ.get("KOYOMADO_TEST_PUBLIC_CERTIFICATE")
        if not certificate_file:
            self.skipTest("An approved existing signing certificate's public file is required")
        with tempfile.TemporaryDirectory(prefix="koyomado-signed-test-", dir=ROOT.parent) as temporary:
            fixture = Path(temporary)
            (fixture / "scripts").mkdir()
            for name in ["package-self-signed-direct.ps1", "package-portable.ps1", "sign-windows-artifact.ps1", "verify-windows-signature.ps1", "verify-direct-release.ps1", "windows-sdk-tools.ps1", "code-signing-certificate-policy.ps1"]:
                shutil.copy2(ROOT / "scripts" / name, fixture / "scripts" / name)
            for name in ["package.json", "LICENSE.txt", "NOTICE", "CHANGELOG.md", "THIRD_PARTY_NOTICES.md", "PRIVACY.md", "CODE_SIGNING_POLICY.md", "ASSET_PROVENANCE.md"]:
                shutil.copy2(ROOT / name, fixture / name)
            for directory in ["docs", "third_party", "src/assets/fonts"]:
                shutil.copytree(ROOT / directory, fixture / directory)
            executable = fixture / "src-tauri/target/release/koyomado.exe"
            executable.parent.mkdir(parents=True)
            shutil.copy2(ROOT / "src-tauri/target/release/koyomado.exe", executable)
            environment = {key: value for key, value in os.environ.items() if key.lower() != "psmodulepath"}
            command = "$public = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new($env:KOYOMADO_TEST_PUBLIC_CERTIFICATE); & './scripts/package-self-signed-direct.ps1' -SkipBuild -CertificateThumbprint $public.Thumbprint"
            assembled = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", command], cwd=fixture, capture_output=True, env=environment)
            error = assembled.stderr.decode("utf-8", errors="replace").replace(str(fixture), "<fixture>")
            self.assertEqual(assembled.returncode, 0, "Signed assembly failed: " + error)
            release = fixture / "release"
            english = release / "Koyomado.en.pdf"
            self.assertTrue(english.is_file(), "Signed Release must publish the English PDF too")
            self.assertEqual(english.read_bytes(), (ROOT / "docs/Koyomado.en.pdf").read_bytes())
            hashes = dict(line.split("  ", 1)[::-1] for line in (release / "SHA256SUMS.txt").read_text().splitlines())
            self.assertEqual(set(hashes), {"koyomado-v1.0.0-windows-portable.zip", "Koyomado.pdf", "Koyomado.en.pdf", "Y-TEC-CodeSigning-Public.cer"})
            with zipfile.ZipFile(release / "koyomado-v1.0.0-windows-portable.zip") as archive:
                self.assertEqual(archive.read("Koyomado.en.pdf"), english.read_bytes())
            english.write_bytes(b"mismatched English PDF")
            hashes["Koyomado.en.pdf"] = hashlib.sha256(english.read_bytes()).hexdigest()
            (release / "SHA256SUMS.txt").write_text("\n".join(f"{value}  {name}" for name, value in hashes.items()) + "\n", encoding="utf-8")
            verify = "$public = [System.Security.Cryptography.X509Certificates.X509Certificate2]::new($env:KOYOMADO_TEST_PUBLIC_CERTIFICATE); & './scripts/verify-direct-release.ps1' -ArchivePath './release/koyomado-v1.0.0-windows-portable.zip' -ManualPath './release/Koyomado.pdf' -EnglishManualPath './release/Koyomado.en.pdf' -CertificatePath './release/Y-TEC-CodeSigning-Public.cer' -HashPath './release/SHA256SUMS.txt' -ExpectedVersion '1.0.0' -ExpectedThumbprint $public.Thumbprint"
            rejected = subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", verify], cwd=fixture, capture_output=True, env=environment)
            self.assertNotEqual(rejected.returncode, 0, "A standalone English PDF differing from the ZIP must be rejected")


if __name__ == "__main__":
    unittest.main()
