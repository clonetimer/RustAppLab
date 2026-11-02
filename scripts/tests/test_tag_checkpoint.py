"""Exercise real lightweight remote tags without credentials or GitHub access."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HELPER = Path(__file__).resolve().parents[1] / 'tag_checkpoint.py'


class CheckpointTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.remote = self.root / 'remote.git'
        self.repo = self.root / 'work'
        self.git(self.root, 'init', '--bare', str(self.remote))
        self.git(self.root, 'init', str(self.repo))
        self.git(self.repo, 'config', 'user.name', 'Test')
        self.git(self.repo, 'config', 'user.email', 'test@example.invalid')
        (self.repo/'VERSION').write_text('0.19.0-dev.2\n')
        self.git(self.repo, 'add', 'VERSION')
        self.git(self.repo, 'commit', '-m', 'initial')
        self.sha = self.git(self.repo, 'rev-parse', 'HEAD').strip()
        self.git(self.repo, 'remote', 'add', 'origin', str(self.remote))
        self.tag = 'checkpoint/v0.19.0-dev.2-' + self.sha[:12]
        self.env = dict(os.environ, GITHUB_EVENT_NAME='push', GITHUB_REF='refs/heads/main', GITHUB_SHA=self.sha)

    def git(self, directory, *args):
        return subprocess.run(['git', '-C', str(directory), *args], text=True, capture_output=True, check=True).stdout

    def helper(self, **env):
        return subprocess.run([sys.executable, str(HELPER), '--repository', str(self.repo)],
                              env=dict(self.env, **env), text=True, capture_output=True)

    def remote_tag(self):
        return self.git(self.repo, 'ls-remote', '--refs', 'origin', 'refs/tags/' + self.tag).strip()

    def test_creates_exact_lightweight_tag_and_is_idempotent(self):
        # No signing identity or author configuration is needed for lightweight tags.
        self.git(self.repo, 'config', '--unset', 'user.name')
        self.git(self.repo, 'config', '--unset', 'user.email')
        first = self.helper()
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(self.remote_tag(), self.sha + '\trefs/tags/' + self.tag)
        self.assertEqual(self.git(self.remote, 'cat-file', '-t', 'refs/tags/' + self.tag).strip(), 'commit')
        second = self.helper()
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertIn('already', second.stdout.lower())
        self.assertEqual(self.remote_tag(), self.sha + '\trefs/tags/' + self.tag)

    def test_rejects_wrong_event_branch_or_sha_without_creating_tag(self):
        for values in ({'GITHUB_EVENT_NAME':'pull_request'}, {'GITHUB_EVENT_NAME':'workflow_dispatch'},
                       {'GITHUB_REF':'refs/heads/feature'}, {'GITHUB_SHA':'a'*40}, {'GITHUB_SHA':self.sha[:12]}):
            with self.subTest(values=values):
                result = self.helper(**values)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.remote_tag(), '')

    def test_rejects_changed_head(self):
        self.git(self.repo, 'commit', '--allow-empty', '-m', 'later')
        result = self.helper()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('HEAD', result.stderr)
        self.assertEqual(self.remote_tag(), '')

    def test_conflicting_remote_tag_is_never_overwritten(self):
        self.git(self.repo, 'commit', '--allow-empty', '-m', 'other')
        other = self.git(self.repo, 'rev-parse', 'HEAD').strip()
        self.git(self.repo, 'push', 'origin', other + ':refs/tags/' + self.tag)
        self.git(self.repo, 'checkout', '--detach', self.sha)
        result = self.helper()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('conflict', result.stderr.lower())
        self.assertEqual(self.remote_tag(), other + '\trefs/tags/' + self.tag)

    def test_unsafe_version_cannot_become_remote_ref(self):
        for version in ('../bad', '0.19.0 dev', '0.19.0\nextra', '-force', '0.19.0..oops'):
            with self.subTest(version=version):
                (self.repo/'VERSION').write_text(version)
                result = self.helper()
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.remote_tag(), '')


if __name__ == '__main__':
    unittest.main()
