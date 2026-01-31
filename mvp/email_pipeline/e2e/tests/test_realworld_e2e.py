from __future__ import annotations

import imaplib
import os
import shutil
import smtplib
import ssl
import tempfile
import time
import unittest
import uuid
from dataclasses import replace
from email import policy
from email.message import EmailMessage
from email.parser import BytesParser
from email.utils import make_msgid
from pathlib import Path
from typing import Iterable, Tuple

from mvp.email_pipeline.config import load_settings
from mvp.email_pipeline.monitor import _process_incoming_email
from mvp.email_pipeline.task_store import TaskStatus, TaskStore


def _env(name: str, default: str | None = None) -> str | None:
    value = os.getenv(name)
    if value is None or value == "":
        return default
    return value


def _env_int(name: str, default: int) -> int:
    raw = os.getenv(name)
    if raw is None:
        return default
    try:
        return int(raw)
    except ValueError:
        return default


def _env_bool(name: str, default: bool) -> bool:
    raw = os.getenv(name)
    if raw is None:
        return default
    return raw.lower() in {"1", "true", "yes", "y"}


class RealWorldE2ETests(unittest.TestCase):
    smtp_host: str
    smtp_port: int
    smtp_user: str
    smtp_pass: str
    smtp_from: str
    smtp_starttls: bool
    smtp_ssl: bool

    imap_host: str
    imap_port: int
    imap_user: str
    imap_pass: str
    imap_folder: str
    imap_ssl: bool

    poll_interval: int
    inbound_timeout: int
    reply_timeout: int
    imap_delete: bool
    reply_to: str
    require_codex: bool

    @classmethod
    def setUpClass(cls) -> None:
        if not _env_bool("ICEBREW_REALWORLD_TESTS", False):
            raise unittest.SkipTest("Set ICEBREW_REALWORLD_TESTS=1 to run real-world E2E tests.")

        required = [
            "E2E_SMTP_HOST",
            "E2E_SMTP_USER",
            "E2E_SMTP_PASS",
            "E2E_IMAP_HOST",
            "E2E_IMAP_USER",
            "E2E_IMAP_PASS",
            "POSTMARK_SERVER_TOKEN",
            "OUTBOUND_FROM",
            "MONGODB_URI",
        ]
        missing = [name for name in required if not _env(name)]
        if missing:
            raise unittest.SkipTest("Missing required env vars: {}".format(", ".join(missing)))

        cls.smtp_host = _env("E2E_SMTP_HOST", "")
        cls.smtp_port = _env_int("E2E_SMTP_PORT", 587)
        cls.smtp_user = _env("E2E_SMTP_USER", "")
        cls.smtp_pass = _env("E2E_SMTP_PASS", "")
        cls.smtp_from = _env("E2E_SMTP_FROM", cls.smtp_user) or cls.smtp_user
        cls.smtp_starttls = _env_bool("E2E_SMTP_STARTTLS", True)
        cls.smtp_ssl = _env_bool("E2E_SMTP_SSL", False)

        cls.imap_host = _env("E2E_IMAP_HOST", "")
        cls.imap_port = _env_int("E2E_IMAP_PORT", 993)
        cls.imap_user = _env("E2E_IMAP_USER", "")
        cls.imap_pass = _env("E2E_IMAP_PASS", "")
        cls.imap_folder = _env("E2E_IMAP_FOLDER", "INBOX")
        cls.imap_ssl = _env_bool("E2E_IMAP_SSL", True)

        cls.reply_to = _env("E2E_REPLY_TO", cls.imap_user) or cls.imap_user
        cls.poll_interval = _env_int("E2E_POLL_INTERVAL", 5)
        cls.inbound_timeout = _env_int("E2E_INBOUND_TIMEOUT", 90)
        cls.reply_timeout = _env_int("E2E_REPLY_TIMEOUT", 180)
        cls.imap_delete = _env_bool("E2E_IMAP_DELETE", False)
        cls.require_codex = _env_bool("E2E_REQUIRE_CODEX", True)

        if cls.require_codex:
            if _env_bool("CODEX_DISABLED", False):
                raise unittest.SkipTest("CODEX_DISABLED is true; real-world tests require Codex.")
            if not _env("AZURE_OPENAI_API_KEY_BACKUP") or not _env("AZURE_OPENAI_ENDPOINT_BACKUP"):
                raise unittest.SkipTest("Missing AZURE_OPENAI_API_KEY_BACKUP/AZURE_OPENAI_ENDPOINT_BACKUP.")
            if shutil.which("codex") is None:
                raise unittest.SkipTest("codex CLI not found on PATH.")

        try:
            import pymongo  # noqa: F401
        except Exception as exc:
            raise unittest.SkipTest(f"pymongo is required for real-world tests: {exc}")

    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        settings = load_settings()
        mongodb_db = _env("E2E_MONGODB_DB", settings.mongodb_db)
        self.settings = replace(
            settings,
            workspace_root=Path(self.tempdir.name) / "workspaces",
            mongodb_db=mongodb_db,
            use_mongodb=True,
        )
        self.settings.workspace_root.mkdir(parents=True, exist_ok=True)
        self.task_store = TaskStore(self.settings.mongodb_uri, self.settings.mongodb_db)
        self._cleanup_task_ids: list[str] = []
        self._cleanup_headers: list[Tuple[str, str]] = []

    def tearDown(self) -> None:
        try:
            for task_id in self._cleanup_task_ids:
                self.task_store.collection.delete_one({"_id": task_id})
        except Exception:
            pass

        if self.imap_delete and self._cleanup_headers:
            try:
                client = self._imap_connect()
                try:
                    client.select(self.imap_folder)
                    for header, value in self._cleanup_headers:
                        ids = self._imap_search(client, header, value)
                        if not ids:
                            continue
                        for msg_id in ids:
                            client.store(msg_id, "+FLAGS", "\\Deleted")
                    client.expunge()
                finally:
                    client.logout()
            except Exception:
                pass

        try:
            self.task_store.client.close()
        except Exception:
            pass
        self.tempdir.cleanup()

    def test_end_to_end_plain_text_reply(self) -> None:
        subject = self._unique_subject("plain")
        message_id = self._smtp_send(
            subject=subject,
            text_body="Hello from IceBrew E2E.",
            reply_to=self.reply_to,
        )
        raw_inbound = self._wait_for_message("Message-ID", message_id, timeout=self.inbound_timeout)
        result = self._process_raw(raw_inbound)

        self.assertTrue(result["success"], result.get("error"))
        task = self.task_store.get_task(result["message_id"])
        self.assertIsNotNone(task)
        self.assertEqual(task.status, TaskStatus.COMPLETED)

        raw_reply = self._wait_for_message("In-Reply-To", message_id, timeout=self.reply_timeout)
        reply_msg = BytesParser(policy=policy.default).parsebytes(raw_reply)
        self.assertIn("Re:", reply_msg.get("Subject", ""))
        self.assertEqual(reply_msg.get("In-Reply-To"), message_id)

    def test_end_to_end_html_body(self) -> None:
        subject = self._unique_subject("html")
        message_id = self._smtp_send(
            subject=subject,
            html_body="<html><body><p>HTML only body</p></body></html>",
            html_only=True,
            reply_to=self.reply_to,
        )
        raw_inbound = self._wait_for_message("Message-ID", message_id, timeout=self.inbound_timeout)
        result = self._process_raw(raw_inbound)

        self.assertTrue(result["success"], result.get("error"))
        inbox_path = Path(result["workspace_path"]) / "email_inbox.md"
        inbox_text = inbox_path.read_text(encoding="utf-8", errors="replace")
        self.assertIn("HTML only body", inbox_text)
        self.assertNotIn("<html", inbox_text.lower())

        raw_reply = self._wait_for_message("In-Reply-To", message_id, timeout=self.reply_timeout)
        reply_msg = BytesParser(policy=policy.default).parsebytes(raw_reply)
        self.assertEqual(reply_msg.get("In-Reply-To"), message_id)

    def test_end_to_end_attachment_saved(self) -> None:
        subject = self._unique_subject("attachment")
        message_id = self._smtp_send(
            subject=subject,
            text_body="See attachment.",
            attachments=[("report.txt", b"report data", "text", "plain")],
            reply_to=self.reply_to,
        )
        raw_inbound = self._wait_for_message("Message-ID", message_id, timeout=self.inbound_timeout)
        result = self._process_raw(raw_inbound)

        self.assertTrue(result["success"], result.get("error"))
        attachments_dir = Path(result["workspace_path"]) / "email_inbox_attachments"
        self.assertTrue((attachments_dir / "report.txt").exists())

        raw_reply = self._wait_for_message("In-Reply-To", message_id, timeout=self.reply_timeout)
        reply_msg = BytesParser(policy=policy.default).parsebytes(raw_reply)
        self.assertEqual(reply_msg.get("In-Reply-To"), message_id)

    def test_end_to_end_duplicate_detection(self) -> None:
        subject = self._unique_subject("duplicate")
        message_id = self._smtp_send(
            subject=subject,
            text_body="First delivery.",
            reply_to=self.reply_to,
        )
        raw_inbound = self._wait_for_message("Message-ID", message_id, timeout=self.inbound_timeout)
        first = self._process_raw(raw_inbound)
        second = self._process_raw(raw_inbound)

        self.assertTrue(first["success"], first.get("error"))
        self.assertEqual(second.get("error"), "duplicate")

    def test_end_to_end_thread_headers(self) -> None:
        subject = self._unique_subject("thread")
        message_id = self._smtp_send(
            subject=subject,
            text_body="Threading check.",
            reply_to=self.reply_to,
        )
        raw_inbound = self._wait_for_message("Message-ID", message_id, timeout=self.inbound_timeout)
        result = self._process_raw(raw_inbound)

        self.assertTrue(result["success"], result.get("error"))
        raw_reply = self._wait_for_message("In-Reply-To", message_id, timeout=self.reply_timeout)
        reply_msg = BytesParser(policy=policy.default).parsebytes(raw_reply)
        self.assertEqual(reply_msg.get("In-Reply-To"), message_id)
        self.assertIn(message_id, reply_msg.get("References", ""))

    def _unique_subject(self, label: str) -> str:
        return f"[IceBrew E2E] {label} {uuid.uuid4().hex[:10]}"

    def _smtp_send(
        self,
        *,
        subject: str,
        text_body: str | None = None,
        html_body: str | None = None,
        html_only: bool = False,
        attachments: Iterable[Tuple[str, bytes, str, str]] | None = None,
        reply_to: str | None = None,
    ) -> str:
        msg = EmailMessage()
        msg["From"] = self.smtp_from
        msg["To"] = self.imap_user
        msg["Subject"] = subject
        msg["Message-ID"] = make_msgid()
        if reply_to:
            msg["Reply-To"] = reply_to

        if html_body and html_only:
            msg.set_content(html_body, subtype="html")
        elif html_body:
            msg.set_content(text_body or "")
            msg.add_alternative(html_body, subtype="html")
        else:
            msg.set_content(text_body or "")

        if attachments:
            for filename, payload, maintype, subtype in attachments:
                msg.add_attachment(payload, maintype=maintype, subtype=subtype, filename=filename)

        if self.smtp_ssl:
            context = ssl.create_default_context()
            client = smtplib.SMTP_SSL(self.smtp_host, self.smtp_port, context=context, timeout=30)
        else:
            client = smtplib.SMTP(self.smtp_host, self.smtp_port, timeout=30)

        try:
            client.ehlo()
            if self.smtp_starttls and not self.smtp_ssl:
                context = ssl.create_default_context()
                client.starttls(context=context)
                client.ehlo()
            if self.smtp_user:
                client.login(self.smtp_user, self.smtp_pass)
            client.send_message(msg)
        finally:
            try:
                client.quit()
            except Exception:
                client.close()

        return msg["Message-ID"]

    def _imap_connect(self) -> imaplib.IMAP4:
        if self.imap_ssl:
            return imaplib.IMAP4_SSL(self.imap_host, self.imap_port)
        return imaplib.IMAP4(self.imap_host, self.imap_port)

    def _imap_search(self, client: imaplib.IMAP4, header: str, value: str) -> list[bytes]:
        status, data = client.search(None, "HEADER", header, value)
        if status != "OK" or not data or not data[0]:
            return []
        return data[0].split()

    def _wait_for_message(self, header: str, value: str, timeout: int) -> bytes:
        deadline = time.time() + timeout
        while time.time() < deadline:
            client = self._imap_connect()
            try:
                client.select(self.imap_folder)
                ids = self._imap_search(client, header, value)
                if ids:
                    msg_id = ids[-1]
                    status, data = client.fetch(msg_id, "(RFC822)")
                    if status == "OK" and data and data[0]:
                        raw = data[0][1]
                        if self.imap_delete:
                            self._cleanup_headers.append((header, value))
                        return raw
            finally:
                try:
                    client.logout()
                except Exception:
                    pass
            time.sleep(self.poll_interval)
        raise AssertionError(f"Timed out waiting for {header}={value}")

    def _process_raw(self, raw_inbound: bytes) -> dict:
        result = _process_incoming_email(
            raw_email=raw_inbound,
            task_store=self.task_store,
            settings=self.settings,
            max_retries=0,
            postmark_message_id=None,
            sleep_fn=lambda _: None,
        )
        if result.get("message_id"):
            self._cleanup_task_ids.append(result["message_id"])
        return result


if __name__ == "__main__":
    unittest.main()
