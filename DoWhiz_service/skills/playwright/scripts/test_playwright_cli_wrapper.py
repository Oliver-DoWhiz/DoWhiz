import os
import stat
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("playwright_cli.sh")


def write_executable(path: Path, contents: str) -> None:
    path.write_text(contents, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


class PlaywrightCliWrapperTests(unittest.TestCase):
    def test_browserbase_state_dir_is_passed_before_subcommand(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            bin_dir = temp / "bin"
            bin_dir.mkdir()
            state_dir = temp / "browserbase-state"
            log_path = temp / "wrapper.log"

            write_executable(
                bin_dir / "browserbase_session_manager",
                textwrap.dedent(
                    f"""\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    printf '%s\n' "$*" > "{log_path}"
                    if [[ "${{1:-}}" != "--state-dir" ]]; then
                      echo "state-dir must be the first argument" >&2
                      exit 17
                    fi
                    if [[ "${{2:-}}" != "{state_dir}" ]]; then
                      echo "unexpected state dir: ${{2:-}}" >&2
                      exit 18
                    fi
                    if [[ "${{3:-}}" != "ensure-session" ]]; then
                      echo "missing ensure-session subcommand" >&2
                      exit 19
                    fi
                    if [[ "${{4:-}}" != "--format" || "${{5:-}}" != "shell" ]]; then
                      echo "missing shell format arguments" >&2
                      exit 20
                    fi
                    printf "export PLAYWRIGHT_MCP_CDP_ENDPOINT='ws://example.test'\\n"
                    """
                ),
            )

            write_executable(
                bin_dir / "npx",
                textwrap.dedent(
                    """\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    printf '%s\n' "$*" >&2
                    """
                ),
            )

            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}:{env.get('PATH', '')}"
            env["BROWSERBASE_API_KEY"] = "bb_test"
            env["BROWSERBASE_STATE_DIR"] = str(state_dir)

            result = subprocess.run(
                [str(SCRIPT_PATH), "open", "https://example.com"],
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            self.assertEqual(
                log_path.read_text(encoding="utf-8").strip(),
                f"--state-dir {state_dir} ensure-session --format shell",
            )
            self.assertIn("--session dowhiz-", result.stderr)
            self.assertIn("playwright-cli --session dowhiz-", result.stderr)
            self.assertIn("goto https://example.com", result.stderr)

    def test_browserbase_json_error_is_rendered_as_plain_text(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            bin_dir = temp / "bin"
            bin_dir.mkdir()

            write_executable(
                bin_dir / "browserbase_session_manager",
                textwrap.dedent(
                    """\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    printf '%s' '{"status":"error","error":"Browserbase API error 402 for /v1/sessions: Free plan browser minutes limit reached."}'
                    exit 1
                    """
                ),
            )

            write_executable(
                bin_dir / "npx",
                textwrap.dedent(
                    """\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    echo "npx should not be called" >&2
                    exit 99
                    """
                ),
            )

            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}:{env.get('PATH', '')}"
            env["BROWSERBASE_API_KEY"] = "bb_test"

            result = subprocess.run(
                [str(SCRIPT_PATH), "open", "https://example.com"],
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 1)
            self.assertIn("Free plan browser minutes limit reached", result.stderr)
            self.assertNotIn('"status":"error"', result.stderr)

    def test_browserbase_reuses_existing_session_with_goto_instead_of_reopen(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp = Path(temp_dir)
            bin_dir = temp / "bin"
            bin_dir.mkdir()
            log_path = temp / "npx.log"

            write_executable(
                bin_dir / "browserbase_session_manager",
                textwrap.dedent(
                    """\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    printf "export PLAYWRIGHT_MCP_CDP_ENDPOINT='ws://browserbase.test'\\n"
                    """
                ),
            )

            write_executable(
                bin_dir / "npx",
                textwrap.dedent(
                    f"""\
                    #!/usr/bin/env bash
                    set -euo pipefail
                    printf '%s\\n' "$*" >> "{log_path}"
                    shift 4
                    if [[ "${{1:-}}" == "--session" ]]; then
                      shift 2
                    fi
                    if [[ "${{1:-}}" == "eval" ]]; then
                      exit 0
                    fi
                    if [[ "${{1:-}}" == "goto" && "${{2:-}}" == "https://example.com" ]]; then
                      exit 0
                    fi
                    echo "unexpected command: $*" >&2
                    exit 92
                    """
                ),
            )

            env = os.environ.copy()
            env["PATH"] = f"{bin_dir}:{env.get('PATH', '')}"
            env["BROWSERBASE_API_KEY"] = "bb_test"

            result = subprocess.run(
                [str(SCRIPT_PATH), "open", "https://example.com"],
                env=env,
                capture_output=True,
                text=True,
                check=False,
            )

            self.assertEqual(result.returncode, 0, msg=result.stderr)
            log_lines = log_path.read_text(encoding="utf-8").splitlines()
            self.assertEqual(len(log_lines), 2)
            self.assertIn("playwright-cli --session dowhiz-", log_lines[0])
            self.assertTrue(log_lines[0].endswith("eval location.href"), log_lines[0])
            self.assertIn("playwright-cli --session dowhiz-", log_lines[1])
            self.assertTrue(log_lines[1].endswith("goto https://example.com"), log_lines[1])


if __name__ == "__main__":
    unittest.main()
