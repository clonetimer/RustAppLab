import contextlib
import importlib.util
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location(
    'check_error_labs', Path(__file__).resolve().parents[1] / 'check_error_labs.py'
)
labs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(labs)
REAL_RUN = subprocess.run


class ErrorLabTests(unittest.TestCase):
    def run_labs(self, compiler_error=None, compiled_failure=False,
                 corrected_compile_failure=False, runtime_failure=False):
        def run(command, **kwargs):
            if command[0] == 'rustc':
                index = int(Path(command[3]).stem.removeprefix('lab'))
                if index % 2 == 0:
                    code = ['E0382', 'E0502', 'E0308', 'E0597'][index // 2]
                    diagnostic = compiler_error if index == 0 and compiler_error else f'error[{code}]: cannot borrow\n'
                    return subprocess.CompletedProcess(
                        command, 0 if compiled_failure else 1, '', diagnostic
                    )
                return subprocess.CompletedProcess(
                    command, 1 if corrected_compile_failure else 0, '', 'compile failed'
                )
            return subprocess.CompletedProcess(
                command, 1 if runtime_failure else 0, '', 'program failed'
            )

        with patch.object(labs.subprocess, 'run', side_effect=run):
            with contextlib.redirect_stdout(io.StringIO()):
                return labs.verify()

    def test_utf8_subprocess_output_works_with_windows_legacy_default(self):
        # Keep real pipe decoding: a mocked CompletedProcess would hide the CI bug.
        def run(command, **kwargs):
            if command[0] == 'rustc':
                index = int(Path(command[3]).stem.removeprefix('lab'))
                code = ['E0382', 'E0502', 'E0308', 'E0597'][index // 2]
                failing = index % 2 == 0
                output = f'error[{code}]: 不能借用\r\n' if failing else ''
                child = (
                    'import sys; '
                    f'sys.stderr.buffer.write({output.encode("utf-8")!r}); '
                    f'sys.exit({1 if failing else 0})'
                )
            else:
                child = 'import sys; sys.stdout.buffer.write("新增任务：不能借用\\r\\n".encode("utf-8"))'
            return REAL_RUN([sys.executable, '-c', child], **kwargs)

        with patch.object(labs.subprocess, 'run', side_effect=run):
            with patch.object(subprocess, '_text_encoding', return_value='cp1252'):
                with contextlib.redirect_stdout(io.StringIO()):
                    try:
                        result = labs.verify()
                    except (UnicodeError, TypeError) as exc:
                        self.fail(f'UTF-8 child output must work with a cp1252 default: {exc}')
                    self.assertEqual(result, (4, 4))

    def test_error_code_in_source_excerpt_is_not_a_matching_diagnostic(self):
        diagnostic = 'error[E0425]: cannot find value\n  | println!("{missing}"); // E0382\n'
        with self.assertRaisesRegex(RuntimeError, 'must fail with E0382'):
            self.run_labs(compiler_error=diagnostic)

    def test_expected_errors_and_repaired_programs_pass_with_crlf_document(self):
        with tempfile.TemporaryDirectory() as tmp:
            doc = Path(tmp) / 'labs.md'
            doc.write_bytes(labs.DOC.read_bytes().replace(b'\r\n', b'\n').replace(b'\n', b'\r\n'))
            with patch.object(labs, 'DOC', doc):
                self.assertEqual(self.run_labs(), (4, 4))

    def test_intentional_error_must_not_compile_successfully(self):
        with self.assertRaisesRegex(RuntimeError, 'must fail with E0382'):
            self.run_labs(compiled_failure=True)

    def test_repaired_example_must_compile(self):
        with self.assertRaisesRegex(RuntimeError, 'Corrected lab 1 did not compile'):
            self.run_labs(corrected_compile_failure=True)

    def test_repaired_example_must_run_successfully(self):
        with self.assertRaisesRegex(RuntimeError, 'Corrected lab 1 did not run'):
            self.run_labs(runtime_failure=True)


if __name__ == '__main__':
    unittest.main()
