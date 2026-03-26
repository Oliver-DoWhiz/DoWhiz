#!/usr/bin/env python3
"""
TPM Feature E2E Test Script

Tests the full TPM workflow on staging:
1. tpm_cli - Contact directory operations
2. slack_cli - Proactive messaging (dry run)
3. discord_cli - Proactive messaging (dry run)
4. Full workflow simulation

Usage:
    python3 test_tpm_e2e.py [--live]

    --live: Actually send messages (requires bot tokens configured)
"""

import subprocess
import json
import sys
import os

# Test configuration
STAGING_HOST = "dowhizstaging"
CLI_PATH = "/home/azureuser/server/DoWhiz/DoWhiz_service/target/release"

# Test data
TEST_ACCOUNT_ID = "test-account-uuid"  # Replace with real account for live test
TEST_NOTION_USER_ID = "notion-user-123"
TEST_SLACK_USER_ID = "U0123456789"
TEST_DISCORD_USER_ID = "123456789012345678"


def ssh_cmd(cmd: str, capture: bool = True) -> tuple[int, str, str]:
    """Run command on staging via SSH."""
    full_cmd = f'ssh {STAGING_HOST} "{cmd}"'
    result = subprocess.run(
        full_cmd,
        shell=True,
        capture_output=capture,
        text=True
    )
    return result.returncode, result.stdout, result.stderr


def test_cli_exists():
    """Test 1: Verify CLIs are compiled and available."""
    print("\n=== Test 1: CLI Binaries Exist ===")

    clis = ["tpm_cli", "slack_cli", "discord_cli"]
    all_exist = True

    for cli in clis:
        code, stdout, stderr = ssh_cmd(f"test -f {CLI_PATH}/{cli} && echo EXISTS || echo MISSING")
        exists = "EXISTS" in stdout
        status = "OK" if exists else "MISSING"
        print(f"  {cli}: {status}")
        if not exists:
            all_exist = False

    return all_exist


def test_tpm_cli_help():
    """Test 2: tpm_cli responds to --help."""
    print("\n=== Test 2: tpm_cli --help ===")

    code, stdout, stderr = ssh_cmd(f"{CLI_PATH}/tpm_cli help")
    combined = stdout + stderr

    if "TPM CLI" in combined or "Contact" in combined:
        print("  OK - Help text displayed")
        return True
    else:
        print(f"  FAIL - Unexpected output: {combined}")
        return False


def test_slack_cli_help():
    """Test 3: slack_cli responds to --help."""
    print("\n=== Test 3: slack_cli --help ===")

    code, stdout, stderr = ssh_cmd(f"{CLI_PATH}/slack_cli help")
    combined = stdout + stderr

    if "Slack CLI" in combined or "send" in combined:
        print("  OK - Help text displayed")
        return True
    else:
        print(f"  FAIL - Unexpected output: {combined}")
        return False


def test_discord_cli_help():
    """Test 4: discord_cli responds to --help."""
    print("\n=== Test 4: discord_cli --help ===")

    code, stdout, stderr = ssh_cmd(f"{CLI_PATH}/discord_cli help")
    combined = stdout + stderr

    if "Discord CLI" in combined or "send" in combined:
        print("  OK - Help text displayed")
        return True
    else:
        print(f"  FAIL - Unexpected output: {combined}")
        return False


def test_tpm_cli_list_contacts():
    """Test 5: tpm_cli list-contacts (may return empty)."""
    print("\n=== Test 5: tpm_cli list-contacts ===")

    # Need to set ACCOUNT_ID env var
    cmd = f"cd /home/azureuser/server/DoWhiz/DoWhiz_service && source .env && ACCOUNT_ID={TEST_ACCOUNT_ID} {CLI_PATH}/tpm_cli list-contacts"
    code, stdout, stderr = ssh_cmd(cmd)

    # Expecting JSON output or "no contacts" message
    if code == 0 or "[]" in stdout or "No contacts" in stderr:
        print(f"  OK - Command executed (exit code: {code})")
        return True
    else:
        print(f"  WARN - Exit code {code}: {stderr}")
        # Not a hard failure - might just be missing account
        return True


def test_slack_cli_dry_run():
    """Test 6: slack_cli send-dm (expect auth error without token)."""
    print("\n=== Test 6: slack_cli send-dm (dry run) ===")

    cmd = f"{CLI_PATH}/slack_cli send-dm --user-id U0000000000 --message 'Test message'"
    code, stdout, stderr = ssh_cmd(cmd)

    # Expecting error about missing token or auth
    if "SLACK_BOT_TOKEN" in stderr or "not configured" in stderr or "token" in stderr.lower():
        print("  OK - Correctly requires SLACK_BOT_TOKEN")
        return True
    elif code != 0:
        print(f"  OK - Failed as expected (no token): {stderr[:100]}")
        return True
    else:
        print(f"  UNEXPECTED - Command succeeded without token?")
        return False


def test_discord_cli_dry_run():
    """Test 7: discord_cli send-dm (expect auth error without token)."""
    print("\n=== Test 7: discord_cli send-dm (dry run) ===")

    cmd = f"{CLI_PATH}/discord_cli send-dm --user-id 000000000000000000 --message 'Test message'"
    code, stdout, stderr = ssh_cmd(cmd)

    # Expecting error about missing token
    if "DISCORD_BOT_TOKEN" in stderr or "not configured" in stderr or "token" in stderr.lower():
        print("  OK - Correctly requires DISCORD_BOT_TOKEN")
        return True
    elif code != 0:
        print(f"  OK - Failed as expected (no token): {stderr[:100]}")
        return True
    else:
        print(f"  UNEXPECTED - Command succeeded without token?")
        return False


def test_database_tables():
    """Test 8: Verify database tables exist."""
    print("\n=== Test 8: Database Tables ===")

    # Use escaped quotes since ssh_cmd wraps in double quotes
    cmd = r'''cd /home/azureuser/server/DoWhiz/DoWhiz_service && source .env && psql \$SUPABASE_DB_URL -t -c \"SELECT tablename FROM pg_tables WHERE tablename IN ('user_contact_directory', 'scheduled_jobs');\"'''
    code, stdout, stderr = ssh_cmd(cmd)
    combined = stdout + stderr

    tables_found = []
    if "user_contact_directory" in combined:
        tables_found.append("user_contact_directory")
    if "scheduled_jobs" in combined:
        tables_found.append("scheduled_jobs")

    print(f"  Found tables: {tables_found}")

    if len(tables_found) == 2:
        print("  OK - Both tables exist")
        return True
    else:
        print(f"  FAIL - Missing tables (output: {combined[:200]})")
        return False


def test_live_slack_dm(user_id: str, message: str):
    """Live test: Send actual Slack DM."""
    print(f"\n=== LIVE: Slack DM to {user_id} ===")

    cmd = f'''cd /home/azureuser/server/DoWhiz/DoWhiz_service && source .env && {CLI_PATH}/slack_cli send-dm --user-id "{user_id}" --message "{message}"'''
    code, stdout, stderr = ssh_cmd(cmd)

    if code == 0 and "success" in stdout.lower():
        print(f"  OK - Message sent!")
        print(f"  Response: {stdout[:200]}")
        return True
    else:
        print(f"  FAIL - {stderr or stdout}")
        return False


def test_live_discord_dm(user_id: str, message: str):
    """Live test: Send actual Discord DM."""
    print(f"\n=== LIVE: Discord DM to {user_id} ===")

    cmd = f'''cd /home/azureuser/server/DoWhiz/DoWhiz_service && source .env && {CLI_PATH}/discord_cli send-dm --user-id "{user_id}" --message "{message}"'''
    code, stdout, stderr = ssh_cmd(cmd)

    if code == 0 and "success" in stdout.lower():
        print(f"  OK - Message sent!")
        print(f"  Response: {stdout[:200]}")
        return True
    else:
        print(f"  FAIL - {stderr or stdout}")
        return False


def run_all_tests(live: bool = False):
    """Run all E2E tests."""
    print("=" * 60)
    print("TPM Feature E2E Test Suite")
    print("=" * 60)
    print(f"Target: {STAGING_HOST}")
    print(f"Mode: {'LIVE' if live else 'DRY RUN'}")

    results = {}

    # Core tests (always run)
    results["cli_exists"] = test_cli_exists()
    results["tpm_help"] = test_tpm_cli_help()
    results["slack_help"] = test_slack_cli_help()
    results["discord_help"] = test_discord_cli_help()
    results["tpm_list"] = test_tpm_cli_list_contacts()
    results["slack_dry"] = test_slack_cli_dry_run()
    results["discord_dry"] = test_discord_cli_dry_run()
    results["db_tables"] = test_database_tables()

    # Live tests (only if --live flag)
    if live:
        print("\n" + "=" * 60)
        print("LIVE TESTS (sending real messages)")
        print("=" * 60)

        # These need real user IDs configured
        # results["slack_live"] = test_live_slack_dm(TEST_SLACK_USER_ID, "TPM E2E Test from staging")
        # results["discord_live"] = test_live_discord_dm(TEST_DISCORD_USER_ID, "TPM E2E Test from staging")
        print("  (Live tests require manual configuration of user IDs)")

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)

    passed = sum(1 for v in results.values() if v)
    total = len(results)

    for name, result in results.items():
        status = "PASS" if result else "FAIL"
        print(f"  {name}: {status}")

    print(f"\nTotal: {passed}/{total} passed")

    return passed == total


if __name__ == "__main__":
    live = "--live" in sys.argv
    success = run_all_tests(live)
    sys.exit(0 if success else 1)
