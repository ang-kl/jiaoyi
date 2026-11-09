"""Verify immutable pre-change Rust archive independently of modified sources."""
import hashlib
import json
from pathlib import Path
import unittest

class BackupTests(unittest.TestCase):
    def test_every_prechange_source_matches_manifest(self):
        root = Path(__file__).resolve().parents[2]
        directory = root / 'archive/strategies_rust_backup_2026_11_09_18403'
        manifest = json.loads((directory / 'manifest.json').read_text())
        self.assertTrue(manifest['files'])
        paths = [row['original_path'] for row in manifest['files']]
        self.assertEqual(len(paths), len(set(paths)))
        self.assertIn('crates/jiaoyi/src/strategy3.rs', paths)
        self.assertIn('crates/jiaoyi/src/trading.rs', paths)
        for row in manifest['files']:
            backup = root / row['backup_path']
            self.assertTrue(backup.is_relative_to(directory))
            self.assertTrue(str(backup).endswith('.rs.bak'))
            data = backup.read_bytes()
            self.assertEqual(len(data), row['byte_size'])
            self.assertEqual(hashlib.sha256(data).hexdigest(), row['sha256'])
