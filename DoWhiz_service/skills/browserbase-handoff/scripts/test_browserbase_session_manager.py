import importlib.util
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("browserbase_session_manager.py")
SPEC = importlib.util.spec_from_file_location("browserbase_session_manager", SCRIPT_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class BrowserbaseSessionManagerTests(unittest.TestCase):
    def setUp(self):
        self.env_backup = {key: os.environ.get(key) for key in (
            "BROWSERBASE_API_KEY",
            "BROWSERBASE_PROJECT_ID",
            "BROWSERBASE_STATE_DIR",
        )}
        os.environ["BROWSERBASE_API_KEY"] = "bb_test"
        os.environ["BROWSERBASE_PROJECT_ID"] = "proj_test"

    def tearDown(self):
        for key, value in self.env_backup.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value

    def test_ensure_context_creates_and_persists_default_context(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            state_dir = Path(temp_dir)
            config = MODULE.BrowserbaseConfig(
                api_key="bb_test",
                project_id="proj_test",
                api_base=MODULE.API_BASE_DEFAULT,
                timeout_seconds=3600,
            )

            calls = []

            def fake_http(method, api_base, api_key, path, body=None):
                calls.append((method, path, body))
                return {"id": "ctx_123"}

            original = MODULE.http_json_request
            MODULE.http_json_request = fake_http
            try:
                context_id, created = MODULE.ensure_context(config, state_dir)
            finally:
                MODULE.http_json_request = original

            self.assertEqual(context_id, "ctx_123")
            self.assertTrue(created)
            self.assertEqual(calls[0][0], "POST")
            payload = json.loads((state_dir / MODULE.REGISTRY_FILENAME).read_text(encoding="utf-8"))
            self.assertEqual(payload["default_context_id"], "ctx_123")

    def test_ensure_session_reuses_running_active_session(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            state_dir = Path(temp_dir)
            MODULE.write_json(
                state_dir / MODULE.REGISTRY_FILENAME,
                {"version": 1, "default_context_id": "ctx_keep"},
            )
            MODULE.write_json(
                state_dir / MODULE.ACTIVE_SESSION_FILENAME,
                {
                    "version": 1,
                    "session_id": "sess_keep",
                    "context_id": "ctx_keep",
                    "page_id": "page_prev",
                },
            )
            config = MODULE.BrowserbaseConfig(
                api_key="bb_test",
                project_id="proj_test",
                api_base=MODULE.API_BASE_DEFAULT,
                timeout_seconds=3600,
            )

            original_get = MODULE.get_session
            original_debug = MODULE.get_debug_urls
            original_create = MODULE.create_session
            MODULE.get_session = lambda _config, session_id: {
                "id": session_id,
                "status": "RUNNING",
                "connectUrl": "wss://connect.example.com/session",
                "contextId": "ctx_keep",
                "createdAt": "2026-03-24T00:00:00Z",
                "expiresAt": "2026-03-24T01:00:00Z",
            }
            MODULE.get_debug_urls = lambda *_args, **_kwargs: {
                "pages": [{"id": "page_live"}],
            }
            MODULE.create_session = lambda *_args, **_kwargs: (_ for _ in ()).throw(
                AssertionError("create_session should not be called")
            )
            try:
                payload = MODULE.ensure_session(config, state_dir)
            finally:
                MODULE.get_session = original_get
                MODULE.get_debug_urls = original_debug
                MODULE.create_session = original_create

            self.assertTrue(payload["session_reused"])
            self.assertEqual(payload["session_id"], "sess_keep")
            self.assertEqual(payload["context_id"], "ctx_keep")
            active = json.loads((state_dir / MODULE.ACTIVE_SESSION_FILENAME).read_text(encoding="utf-8"))
            self.assertEqual(active["page_id"], "page_live")

    def test_release_active_removes_active_session_file(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            state_dir = Path(temp_dir)
            MODULE.write_json(
                state_dir / MODULE.ACTIVE_SESSION_FILENAME,
                {"version": 1, "session_id": "sess_release", "context_id": "ctx_release"},
            )
            config = MODULE.BrowserbaseConfig(
                api_key="bb_test",
                project_id="proj_test",
                api_base=MODULE.API_BASE_DEFAULT,
                timeout_seconds=3600,
            )

            calls = []
            original = MODULE.http_json_request
            def fake_http(method, api_base, api_key, path, body=None):
                calls.append((method, path, body))
                return {"status": "REQUEST_RELEASE"}

            MODULE.http_json_request = fake_http
            try:
                payload = MODULE.release_active(config, state_dir)
            finally:
                MODULE.http_json_request = original

            self.assertEqual(payload["status"], "released")
            self.assertFalse((state_dir / MODULE.ACTIVE_SESSION_FILENAME).exists())
            self.assertEqual(calls[0][0], "POST")
            self.assertIn("REQUEST_RELEASE", json.dumps(calls[0][2], sort_keys=True))

    def test_emit_shell_outputs_expected_exports(self):
        payload = {
            "session_id": "sess_test",
            "context_id": "ctx_test",
            "connect_url": "wss://connect.example.com?token=abc",
        }
        with tempfile.TemporaryFile(mode="w+") as handle:
            original_stdout = sys.stdout
            sys.stdout = handle
            try:
                MODULE.emit(payload, "shell")
            finally:
                sys.stdout = original_stdout
            handle.seek(0)
            text = handle.read()

        self.assertIn("export BROWSERBASE_SESSION_ID=sess_test", text)
        self.assertIn("export BROWSERBASE_CONTEXT_ID=ctx_test", text)
        self.assertIn("export PLAYWRIGHT_MCP_CDP_ENDPOINT='wss://connect.example.com?token=abc'", text)


if __name__ == "__main__":
    unittest.main()
