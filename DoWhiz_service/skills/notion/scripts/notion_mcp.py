#!/usr/bin/env python3
"""Notion MCP Server - Wraps notion_api_cli for MCP protocol access.

This MCP server provides Notion workspace operations through the MCP protocol,
allowing agents to interact with Notion using standard MCP tool calls.

The server wraps the existing notion_api_cli binary, which handles:
- OAuth token management (per-workspace)
- User isolation via .notion_context.json
- API rate limiting and error handling
"""

import json
import os
import subprocess
import sys
from typing import Any, Dict, List, Optional

from mcp.server.fastmcp import FastMCP
from pydantic import BaseModel, ConfigDict, Field


mcp = FastMCP("notion_mcp")


def find_notion_api_cli() -> str:
    """Find the notion_api_cli binary."""
    candidates = [
        # In workspace (ACI/Docker)
        os.path.join(os.getcwd(), "notion_api_cli"),
        # Relative to DoWhiz_service
        os.path.join(os.path.dirname(__file__), "../../../target/release/notion_api_cli"),
        # System PATH
        "notion_api_cli",
        # Docker container
        "/app/notion_api_cli",
        "/app/DoWhiz_service/target/release/notion_api_cli",
    ]

    for path in candidates:
        if os.path.isfile(path) and os.access(path, os.X_OK):
            return path

    # Try which
    try:
        result = subprocess.run(["which", "notion_api_cli"], capture_output=True, text=True)
        if result.returncode == 0:
            return result.stdout.strip()
    except Exception:
        pass

    return "notion_api_cli"  # Fallback, will fail with clear error


def run_notion_cli(args: List[str]) -> Dict[str, Any]:
    """Run notion_api_cli with given arguments and return parsed result."""
    cli_path = find_notion_api_cli()
    cmd = [cli_path] + args

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=120,
            env={**os.environ},
        )

        if result.returncode != 0:
            return {
                "success": False,
                "error": result.stderr.strip() or f"Command failed with exit code {result.returncode}",
            }

        stdout = result.stdout.strip()
        if not stdout:
            return {"success": True, "data": None}

        try:
            return {"success": True, "data": json.loads(stdout)}
        except json.JSONDecodeError:
            return {"success": True, "data": stdout}

    except subprocess.TimeoutExpired:
        return {"success": False, "error": "Command timed out after 120 seconds"}
    except Exception as e:
        return {"success": False, "error": str(e)}


# ---------------------------------------------------------------------------
# Tool Input Models
# ---------------------------------------------------------------------------

class ReadPageInput(BaseModel):
    """Input for reading a Notion page."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    page_id: str = Field(..., description="The Notion page ID (32-char UUID without dashes)")
    workspace_id: str = Field(default="", description="Workspace ID (optional, auto-detected from context)")


class GetCommentsInput(BaseModel):
    """Input for getting comments on a page."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    page_id: str = Field(..., description="The Notion page ID")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class ReplyInput(BaseModel):
    """Input for replying to a comment thread."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    discussion_id: str = Field(..., description="The discussion ID from the comment thread")
    content: str = Field(..., description="The reply content")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class CreateCommentInput(BaseModel):
    """Input for creating a new comment on a page."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    page_id: str = Field(..., description="The Notion page ID")
    content: str = Field(..., description="The comment content")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class SearchInput(BaseModel):
    """Input for searching pages."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    query: str = Field(..., description="Search query text")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class CreatePageInput(BaseModel):
    """Input for creating a new page."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    parent_id: str = Field(..., description="Parent page ID")
    title: str = Field(..., description="Page title")
    content: str = Field(default="", description="Initial paragraph content (optional)")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class AppendBlocksInput(BaseModel):
    """Input for appending content blocks to a page."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    page_id: str = Field(..., description="Page or block ID to append to")
    blocks: List[Dict[str, Any]] = Field(..., description="JSON array of blocks to append")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class GetDatabaseInput(BaseModel):
    """Input for getting database schema."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    database_id: str = Field(..., description="Database ID")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class QueryDatabaseInput(BaseModel):
    """Input for querying database items."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    database_id: str = Field(..., description="Database ID")
    filter: Optional[Dict[str, Any]] = Field(default=None, description="Filter JSON (optional)")
    sorts: Optional[List[Dict[str, Any]]] = Field(default=None, description="Sorts JSON array (optional)")
    limit: int = Field(default=100, description="Max results (default 100)")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class UpdatePageInput(BaseModel):
    """Input for updating page properties."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    page_id: str = Field(..., description="Page ID")
    properties: Dict[str, Any] = Field(..., description="Properties JSON object")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class ArchivePageInput(BaseModel):
    """Input for archiving a page."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    page_id: str = Field(..., description="Page ID to archive")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class ListPagesInput(BaseModel):
    """Input for listing accessible pages."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    limit: int = Field(default=100, description="Max results (default 100)")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


class GetChildrenInput(BaseModel):
    """Input for getting child pages."""
    model_config = ConfigDict(str_strip_whitespace=True, extra="forbid")

    parent_id: str = Field(..., description="Parent page ID")
    workspace_id: str = Field(default="", description="Workspace ID (optional)")


# ---------------------------------------------------------------------------
# MCP Tools
# ---------------------------------------------------------------------------

@mcp.tool(
    name="notion_read_page",
    annotations={
        "title": "Read Notion Page",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_read_page(params: ReadPageInput) -> Dict[str, Any]:
    """Read a Notion page and its content blocks.

    Returns the page metadata (id, title, url, timestamps) and all content blocks.
    """
    args = ["read-page", "--page-id", params.page_id]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_get_comments",
    annotations={
        "title": "Get Notion Page Comments",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_get_comments(params: GetCommentsInput) -> Dict[str, Any]:
    """Get all comments on a Notion page.

    Returns list of comments with id, discussion_id, author info, timestamp, and text.
    """
    args = ["get-comments", "--page-id", params.page_id]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_reply",
    annotations={
        "title": "Reply to Notion Comment",
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": False,
    },
)
def notion_reply(params: ReplyInput) -> Dict[str, Any]:
    """Reply to an existing comment thread on a Notion page.

    Use get_comments first to find the discussion_id of the thread to reply to.
    """
    args = ["reply", "--discussion-id", params.discussion_id, "--content", params.content]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_create_comment",
    annotations={
        "title": "Create Notion Comment",
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": False,
    },
)
def notion_create_comment(params: CreateCommentInput) -> Dict[str, Any]:
    """Create a new comment on a Notion page.

    This creates a new comment thread, not a reply to existing thread.
    """
    args = ["create-comment", "--page-id", params.page_id, "--content", params.content]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_search",
    annotations={
        "title": "Search Notion Pages",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_search(params: SearchInput) -> Dict[str, Any]:
    """Search for pages in the Notion workspace.

    Returns matching pages with id, title, and url.
    """
    args = ["search", "--query", params.query]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_create_page",
    annotations={
        "title": "Create Notion Page",
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": False,
    },
)
def notion_create_page(params: CreatePageInput) -> Dict[str, Any]:
    """Create a new page under a parent page.

    Optionally include initial paragraph content.
    """
    args = ["create-page", "--parent-id", params.parent_id, "--title", params.title]
    if params.content:
        args.extend(["--content", params.content])
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_append_blocks",
    annotations={
        "title": "Append Blocks to Notion Page",
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": False,
        "openWorldHint": False,
    },
)
def notion_append_blocks(params: AppendBlocksInput) -> Dict[str, Any]:
    """Append content blocks to a Notion page.

    Block types: paragraph, heading_1/2/3, bulleted_list_item, numbered_list_item,
    to_do, quote, callout, code, divider.

    Example blocks:
    [{"type": "paragraph", "text": "Hello world"},
     {"type": "heading_1", "text": "Title"},
     {"type": "to_do", "text": "Task", "checked": false}]
    """
    args = ["append-blocks", "--page-id", params.page_id, "--blocks", json.dumps(params.blocks)]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_get_database",
    annotations={
        "title": "Get Notion Database Schema",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_get_database(params: GetDatabaseInput) -> Dict[str, Any]:
    """Get database schema and properties.

    Returns database id, title, url, and list of properties with their types.
    """
    args = ["get-database", "--database-id", params.database_id]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_query_database",
    annotations={
        "title": "Query Notion Database",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_query_database(params: QueryDatabaseInput) -> Dict[str, Any]:
    """Query items from a Notion database.

    Supports filter and sort options. Returns list of database items with properties.
    """
    args = ["query-database", "--database-id", params.database_id]
    if params.filter:
        args.extend(["--filter", json.dumps(params.filter)])
    if params.sorts:
        args.extend(["--sorts", json.dumps(params.sorts)])
    args.extend(["--limit", str(params.limit)])
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_update_page",
    annotations={
        "title": "Update Notion Page Properties",
        "readOnlyHint": False,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_update_page(params: UpdatePageInput) -> Dict[str, Any]:
    """Update page properties.

    Properties should be a JSON object matching Notion's property format.
    """
    args = ["update-page", "--page-id", params.page_id, "--properties", json.dumps(params.properties)]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_archive_page",
    annotations={
        "title": "Archive Notion Page",
        "readOnlyHint": False,
        "destructiveHint": True,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_archive_page(params: ArchivePageInput) -> Dict[str, Any]:
    """Archive (soft delete) a Notion page.

    The page can be restored from Notion's trash.
    """
    args = ["archive-page", "--page-id", params.page_id]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_list_pages",
    annotations={
        "title": "List Notion Pages",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_list_pages(params: ListPagesInput) -> Dict[str, Any]:
    """List all pages accessible to the integration.

    Returns pages with id, title, url, and last_edited_time.
    """
    args = ["list-pages", "--limit", str(params.limit)]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


@mcp.tool(
    name="notion_get_children",
    annotations={
        "title": "Get Notion Child Pages",
        "readOnlyHint": True,
        "destructiveHint": False,
        "idempotentHint": True,
        "openWorldHint": False,
    },
)
def notion_get_children(params: GetChildrenInput) -> Dict[str, Any]:
    """Get child pages under a parent page.

    Returns list of child pages with id, title, and timestamps.
    """
    args = ["get-children", "--parent-id", params.parent_id]
    if params.workspace_id:
        args.extend(["--workspace-id", params.workspace_id])
    return run_notion_cli(args)


if __name__ == "__main__":
    mcp.run()
