import contextlib
import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from base64 import urlsafe_b64decode
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("human_approval_gate.py")
SPEC = importlib.util.spec_from_file_location("human_approval_gate", SCRIPT_PATH)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)

MCP_SCRIPT_PATH = Path(__file__).with_name("human_approval_gate_mcp.py")
MCP_SPEC = importlib.util.spec_from_file_location("human_approval_gate_mcp", MCP_SCRIPT_PATH)
MCP_MODULE = importlib.util.module_from_spec(MCP_SPEC)
assert MCP_SPEC.loader is not None
sys.modules[MCP_SPEC.name] = MCP_MODULE
MCP_SPEC.loader.exec_module(MCP_MODULE)


class HumanApprovalGateTests(unittest.TestCase):
    def decode_token_claims(self, token: str):
        _header, claims, _signature = token.split(".")
        padding = "=" * (-len(claims) % 4)
        return json.loads(urlsafe_b64decode(claims + padding).decode("utf-8"))

    def create_screenshot(self, directory: str, name: str = "screen.png") -> str:
        path = Path(directory) / name
        path.write_bytes(
            b"\x89PNG\r\n\x1a\n"
            b"\x00\x00\x00\rIHDR"
            b"\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00"
            b"\x90wS\xde"
            b"\x00\x00\x00\x0cIDATx\x9cc```\x00\x00\x00\x04\x00\x01"
            b"\x0b\xe7\x02\x9d"
            b"\x00\x00\x00\x00IEND\xaeB`\x82"
        )
        return str(path)

    def parse_request(self, *extra_args: str):
        parser = MODULE.build_parser()
        return parser.parse_args(["request", *extra_args])

    def test_request_requires_screenshot(self):
        args = self.parse_request("--challenge-type", "captcha")
        with self.assertRaises(MODULE.CliError):
            MODULE.build_request_state(args)

    def test_two_factor_requires_explicit_page_state(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            args = self.parse_request(
                "--challenge-type",
                "two_factor",
                "--scope",
                "admin",
                "--two-factor-method",
                "sms",
                "--verification-destination",
                "phone ending in 9315",
                "--screenshot",
                screenshot,
            )
            with self.assertRaises(MODULE.CliError):
                MODULE.build_request_state(args)

    def test_two_factor_body_reports_method_and_destination(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            args = self.parse_request(
                "--challenge-type",
                "two_factor",
                "--scope",
                "admin",
                "--page-state",
                "waiting_for_code_input",
                "--two-factor-method",
                "sms",
                "--verification-destination",
                "phone ending in 9315",
                "--account-label",
                "Oliver Google account",
                "--screenshot",
                screenshot,
            )
            state = MODULE.build_request_state(args)
            rendered = state["_rendered_email"]

            self.assertEqual(state["challenge_type"], "two_factor")
            self.assertEqual(state["page_state"], "waiting_for_code_input")
            self.assertIn(
                "Blocked on: 2FA for Oliver Google account (SMS code to phone ending in 9315)",
                rendered["text_body"],
            )
            self.assertIn(
                "Help needed: Reply with the required SMS code to phone ending in 9315.",
                rendered["text_body"],
            )
            self.assertIn(
                "Current page: Browser is waiting for a verification code to be typed",
                rendered["text_body"],
            )
            self.assertIn("Screenshot attached: screen.png", rendered["text_body"])
            self.assertEqual(rendered["attachments"][0]["Name"], "screen.png")
            self.assertEqual(state["request_attachments"][0]["content_type"], "image/png")

    def test_password_body_reports_env_lookup(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            args = self.parse_request(
                "--challenge-type",
                "password",
                "--scope",
                "admin",
                "--password-env-key",
                "GOOGLE_PASSWORD",
                "--password-lookup-status",
                "Checked workspace .env for GOOGLE_PASSWORD; no value was present.",
                "--account-label",
                "Oliver Google account",
                "--screenshot",
                screenshot,
            )
            state = MODULE.build_request_state(args)
            rendered = state["_rendered_email"]

            self.assertIn("Blocked on: Password entry for Oliver Google account", rendered["text_body"])
            self.assertIn(
                "Help needed: Reply with the password for Oliver Google account.",
                rendered["text_body"],
            )
            self.assertNotIn("Password env key checked", rendered["text_body"])
            self.assertNotIn("Checked workspace .env", rendered["text_body"])
            self.assertIn("Password needed", state["subject"])

    def test_password_request_rejects_when_password_exists_in_current_environment(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            previous = os.environ.get("GOOGLE_PASSWORD")
            os.environ["GOOGLE_PASSWORD"] = "already-available"
            try:
                args = self.parse_request(
                    "--challenge-type",
                    "password",
                    "--scope",
                    "admin",
                    "--password-env-key",
                    "GOOGLE_PASSWORD",
                    "--account-label",
                    "Oliver Google account",
                    "--screenshot",
                    screenshot,
                )
                with self.assertRaises(MODULE.CliError) as ctx:
                    MODULE.build_request_state(args)
            finally:
                if previous is None:
                    os.environ.pop("GOOGLE_PASSWORD", None)
                else:
                    os.environ["GOOGLE_PASSWORD"] = previous

            self.assertIn("GOOGLE_PASSWORD is already available", str(ctx.exception))

    def test_password_request_rejects_when_password_exists_in_workspace_env(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            workspace_env = Path(temp_dir) / ".env"
            workspace_env.write_text("GOOGLE_PASSWORD=workspace-secret\n", encoding="utf-8")
            previous_password = os.environ.pop("GOOGLE_PASSWORD", None)
            previous_cwd = os.getcwd()
            os.chdir(temp_dir)
            try:
                args = self.parse_request(
                    "--challenge-type",
                    "password",
                    "--scope",
                    "admin",
                    "--password-env-key",
                    "GOOGLE_PASSWORD",
                    "--account-label",
                    "Oliver Google account",
                    "--screenshot",
                    screenshot,
                )
                with self.assertRaises(MODULE.CliError) as ctx:
                    MODULE.build_request_state(args)
            finally:
                os.chdir(previous_cwd)
                if previous_password is not None:
                    os.environ["GOOGLE_PASSWORD"] = previous_password

            self.assertIn("workspace .env", str(ctx.exception))

    def test_record_send_event_writes_attachment_details(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            args = self.parse_request(
                "--challenge-type",
                "captcha",
                "--scope",
                "admin",
                "--account-label",
                "Oliver Google account",
                "--screenshot",
                screenshot,
            )
            state = MODULE.build_request_state(args)
            state["outbound_message_id"] = "msg-123"
            state_dir = Path(temp_dir) / ".human_approval_gate" / "challenges"

            MODULE.record_send_event(state_dir, state)

            events_path = state_dir.parent / "events.jsonl"
            lines = events_path.read_text(encoding="utf-8").strip().splitlines()
            payload = json.loads(lines[-1])
            self.assertEqual(payload["event"], "hag_request_sent")
            self.assertEqual(payload["challenge_type"], "captcha")
            self.assertEqual(payload["attachment_count"], 1)
            self.assertEqual(payload["attachments"][0]["name"], "screen.png")
            self.assertGreater(payload["attachments"][0]["size_bytes"], 0)

    def test_captcha_body_reports_blocked_state_without_solve_attempt_claim(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            args = self.parse_request(
                "--challenge-type",
                "captcha",
                "--scope",
                "admin",
                "--account-label",
                "Oliver Google account",
                "--screenshot",
                screenshot,
            )
            state = MODULE.build_request_state(args)
            rendered = state["_rendered_email"]

            self.assertIn(
                "Current page: Browser is currently blocked on a CAPTCHA challenge",
                rendered["text_body"],
            )
            self.assertNotIn("attempted one built-in visual solve", rendered["text_body"])

    def test_browser_handoff_link_is_included_when_active_session_exists(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            active_session_path = Path(temp_dir) / "active_session.json"
            active_session_path.write_text(
                json.dumps({"session_id": "sess_live_123", "page_id": "page_live_456"}),
                encoding="utf-8",
            )

            previous = {
                MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY: os.environ.get(MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY),
                MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY: os.environ.get(MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY),
                MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY: os.environ.get(MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY),
            }
            os.environ[MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY] = "https://api.example.com/service"
            os.environ[MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY] = "secret-key"
            os.environ[MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY] = str(active_session_path)
            original_fetch = MODULE.fetch_browserbase_debug_payload
            MODULE.fetch_browserbase_debug_payload = lambda session_id: {
                "pages": [
                    {
                        "id": "page_live_456",
                        "url": "https://example.com/captcha",
                        "title": "Captcha",
                        "debuggerFullscreenUrl": "https://live.example.com",
                    }
                ]
            }
            try:
                args = self.parse_request(
                    "--challenge-type",
                    "captcha",
                    "--scope",
                    "admin",
                    "--account-label",
                    "Oliver Google account",
                    "--screenshot",
                    screenshot,
                )
                state = MODULE.build_request_state(args)
            finally:
                MODULE.fetch_browserbase_debug_payload = original_fetch
                for key, value in previous.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value

            self.assertEqual(state["browser_session_id"], "sess_live_123")
            self.assertEqual(state["browser_page_id"], "page_live_456")
            self.assertIn(
                "https://api.example.com/service/auth/browser-handoff?token=",
                state["browser_handoff_url"],
            )
            token = state["browser_handoff_url"].split("token=", 1)[1]
            claims = self.decode_token_claims(token)
            self.assertEqual(claims["session_id"], "sess_live_123")
            self.assertEqual(claims["page_id"], "page_live_456")
            self.assertIn("Open the real browser page:", state["_rendered_email"]["text_body"])
            self.assertIn("This opens the exact page where the agent is stuck.", state["_rendered_email"]["text_body"])
            self.assertIn(
                "Help needed: Open the real browser page and solve the CAPTCHA.",
                state["_rendered_email"]["text_body"],
            )
            self.assertIn("Open the real browser page", state["_rendered_email"]["html_body"])
            self.assertLess(
                state["_rendered_email"]["html_body"].index("Open the real browser page"),
                state["_rendered_email"]["html_body"].index("Blocked on:"),
            )

    def test_browser_handoff_refreshes_single_live_page_id_when_missing_from_state(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            active_session_path = Path(temp_dir) / "active_session.json"
            active_session_path.write_text(
                json.dumps({"session_id": "sess_live_123"}),
                encoding="utf-8",
            )

            previous = {
                MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY: os.environ.get(MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY),
                MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY: os.environ.get(MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY),
                MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY: os.environ.get(MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY),
            }
            os.environ[MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY] = "https://api.example.com/service"
            os.environ[MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY] = "secret-key"
            os.environ[MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY] = str(active_session_path)

            original_fetch = MODULE.fetch_browserbase_debug_payload
            MODULE.fetch_browserbase_debug_payload = lambda session_id: {
                "pages": [
                    {
                        "id": "page_from_debug",
                        "url": "https://example.com/verify",
                        "title": "Verify code",
                        "debuggerFullscreenUrl": "https://live.example.com",
                    }
                ]
            }
            try:
                args = self.parse_request(
                    "--challenge-type",
                    "captcha",
                    "--scope",
                    "admin",
                    "--account-label",
                    "Oliver Google account",
                    "--screenshot",
                    screenshot,
                )
                state = MODULE.build_request_state(args)
            finally:
                MODULE.fetch_browserbase_debug_payload = original_fetch
                for key, value in previous.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value

            self.assertEqual(state["browser_session_id"], "sess_live_123")
            self.assertEqual(state["browser_page_id"], "page_from_debug")
            token = state["browser_handoff_url"].split("token=", 1)[1]
            claims = self.decode_token_claims(token)
            self.assertEqual(claims["page_id"], "page_from_debug")

    def test_browser_handoff_refreshes_live_page_id_from_multi_tab_debug_payload(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            active_session_path = Path(temp_dir) / "active_session.json"
            active_session_path.write_text(
                json.dumps({"session_id": "sess_live_123"}),
                encoding="utf-8",
            )

            previous = {
                MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY: os.environ.get(MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY),
                MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY: os.environ.get(MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY),
                MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY: os.environ.get(MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY),
            }
            os.environ[MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY] = "https://api.example.com/service"
            os.environ[MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY] = "secret-key"
            os.environ[MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY] = str(active_session_path)

            original_fetch = MODULE.fetch_browserbase_debug_payload
            MODULE.fetch_browserbase_debug_payload = lambda session_id: {
                "pages": [
                    {
                        "id": "page_blank",
                        "url": "about:blank",
                        "title": "about:blank",
                        "debuggerFullscreenUrl": "https://blank.example.com",
                    },
                    {
                        "id": "page_from_debug",
                        "url": "https://example.com/verify",
                        "title": "Verify code",
                        "debuggerFullscreenUrl": "https://live.example.com",
                    },
                    {
                        "id": "page_blank_tail",
                        "url": "about:blank",
                        "title": "about:blank",
                        "debuggerFullscreenUrl": "https://blank-tail.example.com",
                    },
                ]
            }
            try:
                args = self.parse_request(
                    "--challenge-type",
                    "captcha",
                    "--scope",
                    "admin",
                    "--account-label",
                    "Oliver Google account",
                    "--screenshot",
                    screenshot,
                )
                state = MODULE.build_request_state(args)
            finally:
                MODULE.fetch_browserbase_debug_payload = original_fetch
                for key, value in previous.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value

            self.assertEqual(state["browser_session_id"], "sess_live_123")
            self.assertEqual(state["browser_page_id"], "page_from_debug")
            token = state["browser_handoff_url"].split("token=", 1)[1]
            claims = self.decode_token_claims(token)
            self.assertEqual(claims["page_id"], "page_from_debug")

    def test_browser_handoff_replaces_stored_blank_page_id_with_live_debug_page(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            active_session_path = Path(temp_dir) / "active_session.json"
            active_session_path.write_text(
                json.dumps({"session_id": "sess_live_123", "page_id": "page_blank"}),
                encoding="utf-8",
            )

            previous = {
                MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY: os.environ.get(MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY),
                MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY: os.environ.get(MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY),
                MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY: os.environ.get(MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY),
            }
            os.environ[MODULE.BROWSER_HANDOFF_BASE_URL_ENV_KEY] = "https://api.example.com/service"
            os.environ[MODULE.BROWSER_HANDOFF_SIGNING_SECRET_ENV_KEY] = "secret-key"
            os.environ[MODULE.BROWSERBASE_ACTIVE_SESSION_PATH_ENV_KEY] = str(active_session_path)

            original_fetch = MODULE.fetch_browserbase_debug_payload
            MODULE.fetch_browserbase_debug_payload = lambda session_id: {
                "pages": [
                    {
                        "id": "page_blank",
                        "url": "about:blank",
                        "title": "about:blank",
                        "debuggerFullscreenUrl": "https://blank.example.com",
                    },
                    {
                        "id": "page_live",
                        "url": "https://accounts.google.com/signin/v2/challenge/ipp",
                        "title": "2-Step Verification",
                        "debuggerFullscreenUrl": "https://live.example.com",
                    },
                ]
            }
            try:
                args = self.parse_request(
                    "--challenge-type",
                    "two_factor",
                    "--page-state",
                    "waiting_for_code_input",
                    "--scope",
                    "admin",
                    "--account-label",
                    "dowhiz@deep-tutor.com Google account",
                    "--screenshot",
                    screenshot,
                    "--verification-destination",
                    "phone ending in 15",
                    "--two-factor-method",
                    "sms",
                )
                state = MODULE.build_request_state(args)
            finally:
                MODULE.fetch_browserbase_debug_payload = original_fetch
                for key, value in previous.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value

            self.assertEqual(state["browser_session_id"], "sess_live_123")
            self.assertEqual(state["browser_page_id"], "page_live")
            token = state["browser_handoff_url"].split("token=", 1)[1]
            claims = self.decode_token_claims(token)
            self.assertEqual(claims["page_id"], "page_live")

    def test_poll_for_reply_once_retries_without_recipient_filter_for_alias_deliveries(self):
        challenge = {
            "challenge_id": "abc-123",
            "created_at": "2026-03-25T17:48:42Z",
            "expected_reply_from": "deep-tutor+bb-handoff-123@deep-tutor.com",
            "reply_to": "dowhiz@deep-tutor.com",
            "seen_inbound_message_ids": [],
        }
        recorded_queries = []

        original_http = MODULE.http_json_request

        def fake_http(method, api_base, token, path, query=None, body=None):
            recorded_queries.append((path, dict(query or {})))
            if path == "/messages/inbound":
                if query and query.get("recipient") == "dowhiz@deep-tutor.com":
                    return {"TotalCount": 0, "InboundMessages": []}
                return {
                    "TotalCount": 1,
                    "InboundMessages": [{"MessageID": "msg-1"}],
                }
            if path == "/messages/inbound/msg-1/details":
                return {
                    "From": "deep-tutor+bb-handoff-123@deep-tutor.com",
                    "Subject": "Re: [HAG:abc-123] 2FA help needed for Browserbase handoff demo",
                    "TextBody": "424242",
                    "Date": "2026-03-25T17:50:00Z",
                    "OriginalRecipient": "alias@inbound.postmarkapp.com",
                    "To": "alias@inbound.postmarkapp.com",
                }
            raise AssertionError(f"unexpected request path: {path}")

        MODULE.http_json_request = fake_http
        try:
            reply, new_seen = MODULE.poll_for_reply_once(
                api_base="https://api.postmarkapp.com",
                token="token",
                challenge=challenge,
            )
        finally:
            MODULE.http_json_request = original_http

        self.assertIsNotNone(reply)
        assert reply is not None
        self.assertEqual(reply["from_email"], "deep-tutor+bb-handoff-123@deep-tutor.com")
        self.assertEqual(reply["text_body"], "424242")
        self.assertEqual(new_seen, ["msg-1"])

        inbound_queries = [query for path, query in recorded_queries if path == "/messages/inbound"]
        self.assertEqual(len(inbound_queries), 2)
        self.assertEqual(inbound_queries[0]["recipient"], "dowhiz@deep-tutor.com")
        self.assertNotIn("recipient", inbound_queries[1])

    def test_cli_rejects_shell_usage_when_mcp_required(self):
        previous = os.environ.get(MODULE.HAG_REQUIRE_MCP_ENV_KEY)
        os.environ[MODULE.HAG_REQUIRE_MCP_ENV_KEY] = "1"
        stdout = io.StringIO()
        try:
            with contextlib.redirect_stdout(stdout):
                exit_code = MODULE.main(["status", "--challenge-id", "missing"])
        finally:
            if previous is None:
                os.environ.pop(MODULE.HAG_REQUIRE_MCP_ENV_KEY, None)
            else:
                os.environ[MODULE.HAG_REQUIRE_MCP_ENV_KEY] = previous

        self.assertEqual(exit_code, 2)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["status"], "error")
        self.assertIn(MODULE.HAG_BLOCKING_MCP_TOOL_NAME, payload["error"])

    def test_mcp_wrapper_reuses_blocking_request_flow(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            screenshot = self.create_screenshot(temp_dir)
            params = MCP_MODULE.BlockingHumanApprovalGateInput(
                scope="admin",
                challenge_type="captcha",
                screenshot=[screenshot],
                account_label="Oliver Google account",
                state_dir=str(Path(temp_dir) / ".human_approval_gate" / "challenges"),
                dry_run=True,
            )

            state = MCP_MODULE.execute_blocking_hag_request(params)

            self.assertEqual(state["status"], "pending")
            self.assertEqual(state["challenge_type"], "captcha")
            self.assertEqual(state["outbound_message_id"], "DRY_RUN")
            self.assertEqual(state["request_attachments"][0]["name"], "screen.png")


if __name__ == "__main__":
    unittest.main()
