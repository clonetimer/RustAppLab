"""Keep rerun artifact selection deterministic without weakening report gates."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import unittest
from unittest.mock import patch
from urllib.error import HTTPError, URLError

HELPER = Path(__file__).resolve().parents[1] / 'select_ci_artifacts.py'


def artifact(identifier, name='delivery-windows-latest',
             created_at='2026-11-18T06:26:47Z', **extra):
    return dict(id=identifier, name=name, created_at=created_at, expired=False, **extra)


class SelectorTestCase(unittest.TestCase):
    def setUp(self):
        self.assertTrue(HELPER.is_file(), 'The current-run artifact selector is not implemented.')
        spec = importlib.util.spec_from_file_location('select_ci_artifacts', HELPER)
        self.selector = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.selector)


class SelectorTests(SelectorTestCase):
    def test_rerun_selects_newer_lower_id_and_preserves_unrelated_artifacts(self):
        # Run 226: the first failed Windows upload has the HIGHER artifact ID.
        old = artifact(14404276696, status='FAIL')
        new = artifact(14404059996, created_at='2026-11-18T06:43:41Z', status='PASS')
        artifacts = [old, artifact(3, 'native-ubuntu-latest'), new,
                     artifact(4, 'book-practice-windows-latest')]
        original = [dict(item) for item in artifacts]
        self.assertEqual(self.selector.select_artifact_ids(artifacts),
                         [4, 14404059996, 3])
        self.assertEqual(artifacts, original)

    def test_latest_failed_artifact_is_selected_without_fallback_to_pass(self):
        old = artifact(9, status='PASS', conclusion='success')
        new = artifact(2, created_at='2026-11-18T06:43:41Z',
                       status='FAIL', conclusion='failure')
        self.assertEqual(self.selector.select_artifact_ids([old, new]), [2])

    def test_compares_instants_across_timezones_and_ignores_input_order(self):
        earlier = artifact(99, created_at='2026-11-18T08:00:00+02:00')
        later = artifact(2, created_at='2026-11-18T01:01:00-05:00')
        for artifacts in ([earlier, later], [later, earlier]):
            with self.subTest(artifacts=artifacts):
                self.assertEqual(self.selector.select_artifact_ids(artifacts), [2])

    def test_groups_exact_names_and_orders_names_deterministically(self):
        artifacts = [artifact(4, 'native'), artifact(2, 'Native'),
                     artifact(7, 'native-extra'), artifact(8, 'native ')]
        self.assertEqual(self.selector.select_artifact_ids(artifacts), [2, 4, 8, 7])

    def test_equal_latest_timestamps_fail_even_with_different_offsets(self):
        artifacts = [artifact(1), artifact(2, created_at='2026-11-18T08:26:47+02:00')]
        for values in (artifacts, list(reversed(artifacts))):
            with self.subTest(values=values), self.assertRaises(ValueError):
                self.selector.select_artifact_ids(values)

    def test_equal_older_timestamps_do_not_hide_unique_newest(self):
        artifacts = [artifact(1), artifact(2),
                     artifact(3, created_at='2026-11-18T06:43:41Z')]
        self.assertEqual(self.selector.select_artifact_ids(artifacts), [3])

    def test_expired_latest_fails_without_fallback(self):
        newest = artifact(2, created_at='2026-11-18T06:43:41Z')
        newest['expired'] = True
        with self.assertRaises(ValueError):
            self.selector.select_artifact_ids([artifact(9), newest])

    def test_expired_older_artifact_does_not_block_live_newest(self):
        old = artifact(9)
        old['expired'] = True
        self.assertEqual(self.selector.select_artifact_ids([
            old, artifact(2, created_at='2026-11-18T06:43:41Z')]), [2])

    def test_duplicate_ids_fail_even_across_names(self):
        for second in (artifact(1), artifact(1, 'another-name')):
            with self.subTest(second=second), self.assertRaises(ValueError):
                self.selector.select_artifact_ids([artifact(1), second])

    def test_invalid_metadata_is_not_ignored_on_superseded_artifacts(self):
        cases = {
            'id': [None, True, 0, -1, '12', 1.5],
            'name': [None, 12, '', ' \t'],
            'created_at': [None, 12, '', 'not-a-date', '2026-11-18',
                           '2026-11-18T06:26:47', '2026-02-30T06:26:47Z'],
            'expired': [None, 0, 1, 'false'],
        }
        for field, values in cases.items():
            for value in values:
                invalid = artifact(1)
                invalid[field] = value
                with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                    self.selector.select_artifact_ids([
                        invalid, artifact(2, created_at='2026-11-18T06:43:41Z')])
            missing = artifact(1)
            del missing[field]
            with self.subTest(missing=field), self.assertRaises(ValueError):
                self.selector.select_artifact_ids([missing])

    def test_empty_or_malformed_artifact_lists_fail(self):
        for value in ([], None, {}, 'artifacts', [None], [[]]):
            with self.subTest(value=value), self.assertRaises(ValueError):
                self.selector.select_artifact_ids(value)


class ApiTests(SelectorTestCase):
    # HTTP is the only substituted boundary: parsing, pagination, selection,
    # command-line handling, and output formatting all execute real code.
    def response(self, total, artifacts):
        return io.BytesIO(json.dumps({'total_count': total, 'artifacts': artifacts}).encode())

    def run_cli(self, responses, arguments=None, token='test-token-never-log'):
        stdout, stderr = io.StringIO(), io.StringIO()
        arguments = arguments or ['--repository', 'owner/repo', '--run-id', '226']
        with patch.dict(os.environ, {'GH_TOKEN': token}), \
                patch.object(self.selector, 'urlopen', side_effect=responses) as http, \
                contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            result = self.selector.main(arguments)
        return result, stdout.getvalue(), stderr.getvalue(), http

    def test_paginated_current_run_selects_newer_duplicate_on_next_page(self):
        page_one = [artifact(14404276696, status='FAIL')]
        page_one += [artifact(index + 1, f'native-{index:03d}') for index in range(99)]
        page_two = [artifact(14404059996, created_at='2026-11-18T06:43:41Z',
                             status='PASS', digest='sha256:' + 'b' * 64)]
        result, stdout, stderr, http = self.run_cli([
            self.response(101, page_one), self.response(101, page_two)])
        self.assertEqual(result, 0, stderr)
        self.assertEqual(json.loads(stdout), {
            'schema': 1, 'repository': 'owner/repo', 'run_id': 226,
            'artifacts': [page_two[0], *page_one[1:]],
        })
        self.assertEqual(stderr, '')
        self.assertEqual(http.call_count, 2)
        for page, call in enumerate(http.call_args_list, 1):
            request = call.args[0]
            self.assertEqual(request.full_url,
                             f'https://api.github.com/repos/owner/repo/actions/runs/226/artifacts?per_page=100&page={page}')
            self.assertEqual(request.get_method(), 'GET')
            self.assertEqual(request.get_header('Authorization'), 'Bearer test-token-never-log')
            self.assertEqual(request.get_header('Accept'), 'application/vnd.github+json')
            self.assertEqual(request.get_header('X-github-api-version'), '2022-11-28')
            self.assertEqual(call.kwargs['timeout'], 30)

    def test_exact_page_boundary_needs_no_extra_request(self):
        artifacts = [artifact(index + 1, f'native-{index:03d}') for index in range(100)]
        result, stdout, stderr, http = self.run_cli([self.response(100, artifacts)])
        self.assertEqual(result, 0, stderr)
        self.assertEqual(len(json.loads(stdout)['artifacts']), 100)
        self.assertEqual(http.call_count, 1)

    def test_cli_excludes_only_consumers_own_candidate_release_artifact(self):
        own_output = artifact(9, 'candidate-release')
        own_output['expired'] = True
        another_name = artifact(7, 'candidate-release-extra')
        delivery = artifact(2, digest='sha256:' + 'a' * 64)
        result, stdout, stderr, _ = self.run_cli([
            self.response(3, [own_output, delivery, another_name])])
        self.assertEqual(result, 0, stderr)
        self.assertEqual(json.loads(stdout)['artifacts'], [another_name, delivery])

    def test_cli_does_not_emit_partial_manifest_when_newest_is_unusable(self):
        expired = artifact(2, created_at='2026-11-18T06:43:41Z')
        expired['expired'] = True
        for artifacts in ([artifact(9), expired], [artifact(9, 'candidate-release')]):
            with self.subTest(artifacts=artifacts):
                result, stdout, stderr, _ = self.run_cli([self.response(len(artifacts), artifacts)])
                self.assertNotEqual(result, 0)
                self.assertEqual(stdout, '')
                self.assertTrue(stderr)

    def test_inconsistent_or_incomplete_pagination_fails_without_output(self):
        full_page = [artifact(index + 1, f'native-{index:03d}') for index in range(100)]
        cases = [
            [self.response(2, [artifact(1)])],
            [self.response(0, [artifact(1)])],
            [self.response(101, full_page), self.response(102, [artifact(101)])],
            [self.response(101, full_page), self.response(101, [])],
            [self.response(101, full_page), self.response(101, [artifact(1)])],
        ]
        for responses in cases:
            with self.subTest(pages=len(responses)):
                result, stdout, stderr, _ = self.run_cli(responses)
                self.assertNotEqual(result, 0)
                self.assertEqual(stdout, '')
                self.assertTrue(stderr)

    def test_invalid_api_responses_fail_without_output(self):
        payloads = [b'not-json', b'\xff', b'[]', b'{}',
                    b'{"total_count": true, "artifacts": []}',
                    b'{"total_count": -1, "artifacts": []}',
                    b'{"total_count": 1, "artifacts": {}}',
                    b'{"total_count": 0, "artifacts": []}']
        for payload in payloads:
            with self.subTest(payload=payload):
                result, stdout, stderr, _ = self.run_cli([io.BytesIO(payload)])
                self.assertNotEqual(result, 0)
                self.assertEqual(stdout, '')
                self.assertTrue(stderr)

    def test_http_and_transport_errors_do_not_disclose_token_or_response_body(self):
        token = 'secret-test-token'
        for error in (HTTPError('https://api.github.com/', 403, token, {},
                                io.BytesIO(token.encode())), URLError(token), TimeoutError(token)):
            with self.subTest(error=type(error).__name__):
                result, stdout, stderr, _ = self.run_cli([error], token=token)
                self.assertNotEqual(result, 0)
                self.assertEqual(stdout, '')
                self.assertTrue(stderr)
                self.assertNotIn(token, stderr)

    def test_missing_token_fails_before_request(self):
        result, stdout, stderr, http = self.run_cli([], token='')
        self.assertNotEqual(result, 0)
        self.assertEqual(stdout, '')
        self.assertIn('GH_TOKEN', stderr)
        self.assertEqual(http.call_count, 0)

    def test_invalid_repository_or_run_id_fails_before_request(self):
        for repository, run_id in (('owner/repo/extra', '226'), ('../repo', '226'),
                                   ('owner/repo?x=1', '226'), ('https://example.invalid/repo', '226'),
                                   ('owner/repo', '0'), ('owner/repo', '-1'),
                                   ('owner/repo', 'true'), ('owner/repo', '1.5')):
            with self.subTest(repository=repository, run_id=run_id), \
                    self.assertRaises(SystemExit) as raised:
                self.run_cli([], ['--repository', repository, '--run-id', run_id])
            self.assertEqual(raised.exception.code, 2)


if __name__ == '__main__':
    unittest.main()
