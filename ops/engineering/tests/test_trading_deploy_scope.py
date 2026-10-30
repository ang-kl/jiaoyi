"""Deployment filtering must retain real Rust inputs without restarting for docs."""
import fnmatch
from pathlib import Path
import re
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[3]


class TradingDeployScopeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        config = tomllib.loads((ROOT / "railway.toml").read_text())
        cls.patterns = config.get("build", {}).get("watchPatterns") or ["/**"]

    def watched(self, name):
        # The contract intentionally uses only root-anchored positive globs.
        self.assertTrue(all(p.startswith("/") and not p.startswith("!") for p in self.patterns))
        path = "/" + name
        for pattern in self.patterns:
            if pattern.endswith("/**") and path.startswith(pattern[:-2]):
                return True
            parts, globs = path.split("/"), pattern.split("/")
            if len(parts) == len(globs) and all(fnmatch.fnmatchcase(p, g) for p, g in zip(parts, globs)):
                return True
        return False

    def test_actual_rust_build_inputs_and_embedded_files_trigger_deployment(self):
        inputs = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "Dockerfile",
                  ".dockerignore", "railway.toml", ".cargo/config.toml"}
        for directory in ("crates", "migrations", "config"):
            inputs.update(str(p.relative_to(ROOT)) for p in (ROOT / directory).rglob("*") if p.is_file())
        # Discover literal includes from the source; do not duplicate a fixed
        # registration list that can silently omit the next embedded rulebook.
        for source in (ROOT / "crates").rglob("*.rs"):
            for relative in re.findall(r'include_(?:str|bytes)!\s*\(\s*"([^"\n]+)"', source.read_text()):
                target = (source.parent / relative).resolve()
                inputs.add(str(target.relative_to(ROOT)))
        inputs.add("docs/preregistration-next-version.md")
        for name in sorted(inputs):
            with self.subTest(path=name):
                self.assertTrue(self.watched(name), "a Rust/image input must not be skipped")

    def test_engineering_and_nonembedded_document_changes_do_not_restart_trading(self):
        for name in ("ops/engineering/worker.py", "ops/engineering/Dockerfile",
                     "ops/engineering/prompt.md", "README.md", "log/CODEX_Log.txt",
                     "journal/2026-10-30.md", "docs/dividend-reporting-contract.md",
                     "docs/current-system-contract.md", ".github/workflows/ci.yml"):
            with self.subTest(path=name):
                self.assertFalse(self.watched(name), "unrelated changes must not redeploy the trader")


if __name__ == "__main__":
    unittest.main()
