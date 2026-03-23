#!/usr/bin/env python3
import argparse
import json
import pathlib
import sys
import urllib.error
import urllib.request
from typing import Optional


SCOPE_FILE = ".chat_history_scope.json"


def find_scope_file(start: pathlib.Path) -> pathlib.Path:
    current = start.resolve()
    for path in [current, *current.parents]:
        candidate = path / SCOPE_FILE
        if candidate.exists():
            return candidate
    raise FileNotFoundError(f"could not find {SCOPE_FILE} from {start}")


def load_scope(scope_path: pathlib.Path) -> dict:
    payload = json.loads(scope_path.read_text())
    platform = payload.get("platform")
    if platform != "slack":
        raise RuntimeError(f"scope file is for platform={platform!r}, not slack")
    return payload


def print_status(scope: dict) -> None:
    print(f"Platform: {scope.get('platform')}")
    print(f"Scope mode: {scope.get('scope_mode')}")
    print(f"Description: {scope.get('description')}")
    print(f"Channel ID: {scope.get('channel_id')}")
    if scope.get("team_id"):
        print(f"Team ID: {scope.get('team_id')}")
    if scope.get("thread_id"):
        print(f"Current thread ID: {scope.get('thread_id')}")
    print(f"Expires at: {scope.get('expires_at')}")
    print(f"Endpoint: {scope.get('search_endpoint')}")


def run_search(
    scope: dict,
    query: str,
    limit: int,
    output: Optional[str],
    emit_json: bool,
) -> int:
    endpoint = scope.get("search_endpoint")
    token = scope.get("token")
    if not endpoint or not token:
        raise RuntimeError("scope file is missing search_endpoint or token")

    request_body = {"query": query, "limit": limit}
    request = urllib.request.Request(
        endpoint,
        data=json.dumps(request_body).encode("utf-8"),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {token}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request) as response:
            payload = json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8", errors="replace")
        sys.stderr.write(f"history search failed: HTTP {exc.code}: {body}\n")
        return 1

    if output:
        output_path = pathlib.Path(output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(payload, indent=2) + "\n")

    if emit_json:
        json.dump(payload, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0

    print(
        f"Found {len(payload.get('results', []))} Slack matches "
        f"across {payload.get('searched_channels', 0)} searchable conversation(s)."
    )
    for warning in payload.get("warnings", []):
        print(f"Warning: {warning}")
    for item in payload.get("results", []):
        author = item.get("author_name") or item.get("author_id") or "unknown"
        print(
            f"- [{item.get('timestamp')}] {author}: {item.get('text')}"
        )
    if output:
        print(f"Saved full JSON to {output}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Search scoped Slack history")
    parser.add_argument(
        "--scope-file",
        default=None,
        help="Optional path to .chat_history_scope.json",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    subparsers.add_parser("status", help="Show the current scoped Slack history grant")

    search = subparsers.add_parser("search", help="Search the current Slack conversation history")
    search.add_argument("--query", required=True, help="Keyword or phrase to search for")
    search.add_argument("--limit", type=int, default=10, help="Max results to return")
    search.add_argument("--output", help="Optional JSON output file path")
    search.add_argument("--json", action="store_true", help="Print raw JSON instead of a summary")

    args = parser.parse_args()
    scope_path = (
        pathlib.Path(args.scope_file)
        if args.scope_file
        else find_scope_file(pathlib.Path.cwd())
    )
    scope = load_scope(scope_path)

    if args.command == "status":
        print_status(scope)
        return 0
    if args.command == "search":
        return run_search(scope, args.query, args.limit, args.output, args.json)
    raise AssertionError(f"unexpected command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
