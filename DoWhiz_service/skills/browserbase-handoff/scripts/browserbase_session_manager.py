#!/usr/bin/env python3
"""Manage Browserbase contexts and active sessions for DoWhiz browser tasks."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import sys
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, Optional, Tuple

API_BASE_DEFAULT = "https://api.browserbase.com"
STATE_DIR_DEFAULT = ".secrets/browserbase"
REGISTRY_FILENAME = "registry.json"
ACTIVE_SESSION_FILENAME = "active_session.json"
DEFAULT_TIMEOUT_SECONDS = 3600
RUNNING_SESSION_STATUSES = {"RUNNING"}


class CliError(RuntimeError):
    """Command error with a user-facing message."""


class ApiError(CliError):
    """HTTP/API error with a surfaced status code."""

    def __init__(self, status_code: int, path: str, message: str) -> None:
        self.status_code = status_code
        self.path = path
        self.raw_message = message
        self.body_summary = summarize_api_error_body(message)
        self.error_kind = classify_api_error(status_code, path, self.body_summary)
        self.guidance = api_error_guidance(self.error_kind)
        error_message = f"Browserbase API error {status_code} for {path}: {self.body_summary}"
        if self.guidance:
            error_message = f"{error_message} {self.guidance}"
        super().__init__(error_message)


@dataclass
class BrowserbaseConfig:
    api_key: str
    project_id: Optional[str]
    api_base: str
    timeout_seconds: int


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def isoformat_utc(value: datetime) -> str:
    return value.astimezone(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def summarize_api_error_body(raw: str) -> str:
    text = raw.strip()
    if not text:
        return "empty response body"
    try:
        parsed = json.loads(text)
    except json.JSONDecodeError:
        return text[:800]
    if isinstance(parsed, dict):
        message = str(parsed.get("message", "")).strip()
        error = str(parsed.get("error", "")).strip()
        parts = []
        if error:
            parts.append(error)
        if message and message not in parts:
            parts.append(message)
        if parts:
            return ": ".join(parts)
    return text[:800]


def classify_api_error(status_code: int, path: str, body_summary: str) -> Optional[str]:
    normalized_path = path.strip()
    if status_code == 402 and normalized_path == "/v1/sessions":
        return "browserbase_quota_exhausted"
    if status_code == 429 and normalized_path == "/v1/sessions":
        return "browserbase_rate_limited"
    if status_code == 401:
        return "browserbase_auth_failed"
    if "project not found" in body_summary.lower():
        return "browserbase_project_missing"
    return None


def api_error_guidance(error_kind: Optional[str]) -> Optional[str]:
    if error_kind == "browserbase_quota_exhausted":
        return (
            "Browserbase could not create a live browser session for this task because the "
            "configured project has no remaining browser minutes or billing is inactive. "
            "Upgrade the Browserbase plan, or switch DoWhiz to a Browserbase API key/project "
            "that can create sessions. Until this is fixed, the agent cannot open a live "
            "Browserbase page or send a live browser handoff link."
        )
    if error_kind == "browserbase_rate_limited":
        return (
            "Browserbase is rate limiting session creation right now. Retry after the limit "
            "window, or reduce concurrent Browserbase session starts."
        )
    if error_kind == "browserbase_auth_failed":
        return (
            "Verify that the Browserbase API key configured for this environment is valid and "
            "belongs to the intended project."
        )
    if error_kind == "browserbase_project_missing":
        return (
            "Verify that BROWSERBASE_PROJECT_ID points to a project that exists and is accessible "
            "by the configured API key."
        )
    return None


def read_json(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: Dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp_path_name = ""
    try:
        with tempfile.NamedTemporaryFile(
            "w",
            encoding="utf-8",
            dir=path.parent,
            prefix=f"{path.stem}-",
            suffix=".tmp",
            delete=False,
        ) as handle:
            temp_path_name = handle.name
            handle.write(json.dumps(payload, indent=2, sort_keys=True))
        Path(temp_path_name).replace(path)
    finally:
        if temp_path_name:
            temp_path = Path(temp_path_name)
            if temp_path.exists():
                temp_path.unlink()


def get_env_first(*keys: str) -> Optional[str]:
    for key in keys:
        value = os.environ.get(key, "").strip()
        if value:
            return value
    return None


def state_dir_path(raw_state_dir: str) -> Path:
    path = Path(raw_state_dir).expanduser()
    if path.is_absolute():
        return path
    return (Path.cwd() / path).resolve()


def registry_path(state_dir: Path) -> Path:
    return state_dir / REGISTRY_FILENAME


def active_session_path(state_dir: Path) -> Path:
    return state_dir / ACTIVE_SESSION_FILENAME


def build_config(args: argparse.Namespace) -> BrowserbaseConfig:
    api_key = get_env_first("BROWSERBASE_API_KEY", "BROWSER_BASE_API_KEY")
    if not api_key:
        raise CliError(
            "missing Browserbase API key: set BROWSERBASE_API_KEY (or BROWSER_BASE_API_KEY)"
        )
    project_id = get_env_first("BROWSERBASE_PROJECT_ID", "BROWSER_BASE_PROJECT_ID")
    api_base = (get_env_first("BROWSERBASE_API_BASE_URL") or API_BASE_DEFAULT).rstrip("/")
    timeout_seconds = args.timeout_seconds
    if timeout_seconds is None:
        timeout_raw = get_env_first("BROWSERBASE_SESSION_TIMEOUT_SECONDS")
        timeout_seconds = int(timeout_raw) if timeout_raw else DEFAULT_TIMEOUT_SECONDS
    if timeout_seconds < 60:
        raise CliError("Browserbase session timeout must be at least 60 seconds")
    return BrowserbaseConfig(
        api_key=api_key,
        project_id=project_id,
        api_base=api_base,
        timeout_seconds=timeout_seconds,
    )


def http_json_request(
    method: str,
    api_base: str,
    api_key: str,
    path: str,
    body: Optional[Dict[str, Any]] = None,
) -> Dict[str, Any]:
    url = f"{api_base.rstrip('/')}{path}"
    headers = {
        "Accept": "application/json",
        "X-BB-API-Key": api_key,
    }
    data: Optional[bytes] = None
    if body is not None:
        headers["Content-Type"] = "application/json"
        data = json.dumps(body).encode("utf-8")

    request = urllib.request.Request(url=url, data=data, headers=headers, method=method.upper())

    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            payload = response.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace").strip()
        raise ApiError(exc.code, path, raw[:800]) from exc
    except urllib.error.URLError as exc:
        raise CliError(f"failed to call Browserbase API {path}: {exc.reason}") from exc

    try:
        parsed = json.loads(payload)
    except json.JSONDecodeError as exc:
        raise CliError(f"Browserbase API returned invalid JSON for {path}: {payload[:200]}") from exc

    if not isinstance(parsed, dict):
        raise CliError(f"Browserbase API returned non-object response for {path}")
    return parsed


def load_registry(state_dir: Path) -> Dict[str, Any]:
    path = registry_path(state_dir)
    if not path.exists():
        return {"version": 1}
    payload = read_json(path)
    if not isinstance(payload, dict):
        raise CliError(f"invalid registry payload in {path}")
    return payload


def save_registry(state_dir: Path, payload: Dict[str, Any]) -> None:
    payload["updated_at"] = isoformat_utc(utc_now())
    write_json(registry_path(state_dir), payload)


def load_active_session(state_dir: Path) -> Optional[Dict[str, Any]]:
    path = active_session_path(state_dir)
    if not path.exists():
        return None
    payload = read_json(path)
    if not isinstance(payload, dict):
        raise CliError(f"invalid active session payload in {path}")
    return payload


def save_active_session(state_dir: Path, payload: Dict[str, Any]) -> None:
    payload["updated_at"] = isoformat_utc(utc_now())
    write_json(active_session_path(state_dir), payload)


def remove_active_session(state_dir: Path) -> None:
    path = active_session_path(state_dir)
    if path.exists():
        path.unlink()


def ensure_context(config: BrowserbaseConfig, state_dir: Path) -> Tuple[str, bool]:
    registry = load_registry(state_dir)
    existing = str(registry.get("default_context_id", "")).strip()
    if existing:
        return existing, False

    body: Dict[str, Any] = {}
    if config.project_id:
        body["projectId"] = config.project_id
    response = http_json_request("POST", config.api_base, config.api_key, "/v1/contexts", body=body)
    context_id = str(response.get("id", "")).strip()
    if not context_id:
        raise CliError("Browserbase create context response did not include an id")

    registry["version"] = 1
    registry["default_context_id"] = context_id
    if config.project_id:
        registry["project_id"] = config.project_id
    save_registry(state_dir, registry)
    return context_id, True


def extract_context_id(session: Dict[str, Any]) -> str:
    direct = str(session.get("contextId", "")).strip()
    if direct:
        return direct
    browser_settings = session.get("browserSettings")
    if isinstance(browser_settings, dict):
        context = browser_settings.get("context")
        if isinstance(context, dict):
            nested = str(context.get("id", "")).strip()
            if nested:
                return nested
    return ""


def get_session(config: BrowserbaseConfig, session_id: str) -> Dict[str, Any]:
    encoded = urllib.parse.quote(session_id, safe="")
    return http_json_request("GET", config.api_base, config.api_key, f"/v1/sessions/{encoded}")


def get_debug_urls(config: BrowserbaseConfig, session_id: str) -> Dict[str, Any]:
    encoded = urllib.parse.quote(session_id, safe="")
    return http_json_request(
        "GET",
        config.api_base,
        config.api_key,
        f"/v1/sessions/{encoded}/debug",
    )


def page_debug_url(page: Dict[str, Any]) -> str:
    for key in ("debuggerFullscreenUrl", "debuggerUrl"):
        value = str(page.get(key, "")).strip()
        if value:
            return value
    return ""


def page_text_is_blank(value: Any) -> bool:
    normalized = str(value or "").strip().lower()
    return (
        normalized == ""
        or normalized == "about:blank"
        or normalized == "new tab"
        or normalized.startswith("chrome://newtab")
        or normalized.startswith("edge://newtab")
        or normalized.startswith("chrome-search://local-ntp")
    )


def page_looks_live(page: Dict[str, Any]) -> bool:
    return not page_text_is_blank(page.get("url")) or not page_text_is_blank(page.get("title"))


def select_page_id_from_debug_payload(debug_urls: Dict[str, Any]) -> Optional[str]:
    pages = debug_urls.get("pages")
    if not isinstance(pages, list):
        return None

    def iter_candidates():
        for raw_page in reversed(pages):
            if not isinstance(raw_page, dict):
                continue
            page_id = str(raw_page.get("id", "")).strip()
            if not page_id or not page_debug_url(raw_page):
                continue
            yield raw_page, page_id

    for page, page_id in iter_candidates():
        if page_looks_live(page):
            return page_id

    return None


def create_session(config: BrowserbaseConfig, context_id: str) -> Dict[str, Any]:
    body: Dict[str, Any] = {
        "browserSettings": {
            "context": {
                "id": context_id,
                "persist": True,
            }
        },
        "timeout": config.timeout_seconds,
        "keepAlive": True,
        "userMetadata": {
            "source": "dowhiz",
        },
    }
    if config.project_id:
        body["projectId"] = config.project_id
    response = http_json_request("POST", config.api_base, config.api_key, "/v1/sessions", body=body)
    session_id = str(response.get("id", "")).strip()
    connect_url = str(response.get("connectUrl", "")).strip()
    if not session_id or not connect_url:
        raise CliError("Browserbase create session response is missing id or connectUrl")
    return response


def session_is_reusable(session: Dict[str, Any], expected_context_id: str) -> bool:
    status = str(session.get("status", "")).strip().upper()
    connect_url = str(session.get("connectUrl", "")).strip()
    context_id = extract_context_id(session)
    return (
        bool(connect_url)
        and status in RUNNING_SESSION_STATUSES
        and context_id == expected_context_id
    )


def resolve_page_id_for_session(
    config: BrowserbaseConfig,
    session_id: str,
) -> Tuple[Optional[str], bool]:
    try:
        debug_urls = get_debug_urls(config, session_id)
    except CliError:
        return None, False

    page_id = select_page_id_from_debug_payload(debug_urls)
    if not page_id:
        return None, True
    return page_id, True


def build_active_session_payload(
    session: Dict[str, Any],
    context_id: str,
    previous_payload: Optional[Dict[str, Any]] = None,
    page_id: Optional[str] = None,
    page_id_known: bool = False,
) -> Dict[str, Any]:
    payload = {
        "version": 1,
        "session_id": str(session.get("id", "")).strip(),
        "context_id": context_id,
        "project_id": str(session.get("projectId", "")).strip(),
        "created_at": str(session.get("createdAt", "")).strip() or isoformat_utc(utc_now()),
        "started_at": str(session.get("startedAt", "")).strip(),
        "expires_at": str(session.get("expiresAt", "")).strip(),
        "status": str(session.get("status", "")).strip(),
    }
    if page_id_known:
        if page_id:
            payload["page_id"] = page_id
    elif previous_payload:
        previous_page_id = str(previous_payload.get("page_id", "")).strip()
        if previous_page_id:
            payload["page_id"] = previous_page_id
    return payload


def build_ensure_session_output(
    session: Dict[str, Any],
    context_id: str,
    context_created: bool,
    reused: bool,
    state_dir: Path,
) -> Dict[str, Any]:
    return {
        "status": "ready",
        "context_created": context_created,
        "session_reused": reused,
        "context_id": context_id,
        "session_id": str(session.get("id", "")).strip(),
        "connect_url": str(session.get("connectUrl", "")).strip(),
        "expires_at": str(session.get("expiresAt", "")).strip(),
        "state_dir": str(state_dir),
    }


def ensure_session(config: BrowserbaseConfig, state_dir: Path) -> Dict[str, Any]:
    state_dir.mkdir(parents=True, exist_ok=True)
    context_id, context_created = ensure_context(config, state_dir)
    active = load_active_session(state_dir)

    if active:
        session_id = str(active.get("session_id", "")).strip()
        if session_id:
            try:
                session = get_session(config, session_id)
            except ApiError as exc:
                if exc.status_code not in (404, 410):
                    raise
            else:
                if session_is_reusable(session, context_id):
                    page_id, page_id_known = resolve_page_id_for_session(config, session_id)
                    save_active_session(
                        state_dir,
                        build_active_session_payload(
                            session,
                            context_id,
                            previous_payload=active,
                            page_id=page_id,
                            page_id_known=page_id_known,
                        ),
                    )
                    return build_ensure_session_output(
                        session,
                        context_id,
                        context_created=context_created,
                        reused=True,
                        state_dir=state_dir,
                    )

    session = create_session(config, context_id)
    session_id = str(session.get("id", "")).strip()
    page_id, page_id_known = resolve_page_id_for_session(config, session_id)
    save_active_session(
        state_dir,
        build_active_session_payload(
            session,
            context_id,
            previous_payload=active,
            page_id=page_id,
            page_id_known=page_id_known,
        ),
    )
    return build_ensure_session_output(
        session,
        context_id,
        context_created=context_created,
        reused=False,
        state_dir=state_dir,
    )


def release_active(config: BrowserbaseConfig, state_dir: Path) -> Dict[str, Any]:
    active = load_active_session(state_dir)
    if not active:
        return {"status": "skipped", "reason": "no_active_session"}

    session_id = str(active.get("session_id", "")).strip()
    if not session_id:
        remove_active_session(state_dir)
        return {"status": "skipped", "reason": "missing_session_id"}

    body: Dict[str, Any] = {"status": "REQUEST_RELEASE"}
    if config.project_id:
        body["projectId"] = config.project_id

    encoded = urllib.parse.quote(session_id, safe="")
    try:
        response = http_json_request(
            "POST",
            config.api_base,
            config.api_key,
            f"/v1/sessions/{encoded}",
            body=body,
        )
    except ApiError as exc:
        if exc.status_code in (404, 410):
            remove_active_session(state_dir)
            return {"status": "released", "session_id": session_id, "already_gone": True}
        raise

    remove_active_session(state_dir)
    return {
        "status": "released",
        "session_id": session_id,
        "updated_status": str(response.get("status", "")).strip(),
    }


def emit(payload: Dict[str, Any], output_format: str) -> None:
    if output_format == "shell":
        required = {
            "BROWSERBASE_SESSION_ID": str(payload.get("session_id", "")).strip(),
            "BROWSERBASE_CONTEXT_ID": str(payload.get("context_id", "")).strip(),
            "PLAYWRIGHT_MCP_CDP_ENDPOINT": str(payload.get("connect_url", "")).strip(),
        }
        missing = [key for key, value in required.items() if not value]
        if missing:
            raise CliError(
                f"cannot emit shell exports; missing values for: {', '.join(sorted(missing))}"
            )
        for key, value in required.items():
            print(f"export {key}={shlex.quote(value)}")
        return

    print(json.dumps(payload, ensure_ascii=True, sort_keys=True))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="browserbase_session_manager",
        description="Ensure and release Browserbase sessions for DoWhiz-managed browser tasks.",
    )
    parser.add_argument(
        "--state-dir",
        default=os.environ.get("BROWSERBASE_STATE_DIR", "").strip() or STATE_DIR_DEFAULT,
        help="Directory where Browserbase registry and active session state are stored.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=int,
        default=None,
        help="Override Browserbase session timeout in seconds.",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    ensure_parser = subparsers.add_parser("ensure-session", help="create or reuse the active session")
    ensure_parser.add_argument(
        "--format",
        choices=("json", "shell"),
        default="json",
        help="Output format for the ensured session payload.",
    )
    ensure_parser.set_defaults(func=cmd_ensure_session)

    release_parser = subparsers.add_parser("release-active", help="request release for active session")
    release_parser.add_argument(
        "--format",
        choices=("json",),
        default="json",
        help="Output format for release results.",
    )
    release_parser.set_defaults(func=cmd_release_active)

    return parser


def cmd_ensure_session(args: argparse.Namespace) -> int:
    config = build_config(args)
    state_dir = state_dir_path(args.state_dir)
    payload = ensure_session(config, state_dir)
    emit(payload, args.format)
    return 0


def cmd_release_active(args: argparse.Namespace) -> int:
    state_dir = state_dir_path(args.state_dir)
    if not active_session_path(state_dir).exists():
        emit({"status": "skipped", "reason": "no_active_session"}, args.format)
        return 0
    config = build_config(args)
    payload = release_active(config, state_dir)
    emit(payload, args.format)
    return 0


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return int(args.func(args))
    except CliError as exc:
        payload: Dict[str, Any] = {
            "status": "error",
            "error": str(exc),
        }
        if isinstance(exc, ApiError):
            payload["provider"] = "browserbase"
            payload["provider_path"] = exc.path
            payload["provider_status_code"] = exc.status_code
            if exc.error_kind:
                payload["error_kind"] = exc.error_kind
            if exc.guidance:
                payload["action_required"] = exc.guidance
        print(json.dumps(payload, ensure_ascii=True, sort_keys=True))
        return 1
    except KeyboardInterrupt:
        print(json.dumps({"status": "error", "error": "interrupted"}, ensure_ascii=True, sort_keys=True))
        return 130


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
