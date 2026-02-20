#!/usr/bin/env python3
"""Sync GitHub issues into a Google Sheets bug tracker tab.

This script is designed for GitHub Actions. It upserts issue rows keyed by
"Issue #" and preserves manual planning columns (team assignment, priority, etc).
"""

from __future__ import annotations

import json
import os
import sys
from dataclasses import dataclass
from typing import Any

import requests
from google.oauth2.service_account import Credentials
from googleapiclient.discovery import build

SCOPES = ["https://www.googleapis.com/auth/spreadsheets"]


REQUIRED_COLUMNS = [
    "Issue #",
    "Title",
    "State",
    "Labels",
    "GitHub Assignees",
    "Team Assignee",
    "Priority",
    "Triage Status",
    "Reporter",
    "Created At",
    "Updated At",
    "Issue URL",
    "Sprint",
    "Notes",
]

MANAGED_COLUMNS = {
    "Issue #",
    "Title",
    "State",
    "Labels",
    "GitHub Assignees",
    "Reporter",
    "Created At",
    "Updated At",
    "Issue URL",
}


@dataclass
class Config:
    github_token: str
    github_repository: str
    sheet_id: str
    sheet_tab: str
    sheet_gid: str
    service_account_json: str


def must_getenv(name: str) -> str:
    value = os.getenv(name, "").strip()
    if not value:
        raise RuntimeError(f"Missing required environment variable: {name}")
    return value


def load_config() -> Config:
    return Config(
        github_token=must_getenv("GITHUB_TOKEN"),
        github_repository=must_getenv("GITHUB_REPOSITORY"),
        sheet_id=must_getenv("GOOGLE_SHEET_ID"),
        sheet_tab=os.getenv("GOOGLE_SHEET_TAB", "Bug Tracker").strip() or "Bug Tracker",
        sheet_gid=os.getenv("GOOGLE_SHEET_GID", "").strip(),
        service_account_json=must_getenv("GOOGLE_SERVICE_ACCOUNT_JSON"),
    )


def github_headers(token: str) -> dict[str, str]:
    return {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "User-Agent": "tetra-bluestation-sheet-sync",
    }


def fetch_all_issues(repo: str, token: str) -> list[dict[str, Any]]:
    issues: list[dict[str, Any]] = []
    url = f"https://api.github.com/repos/{repo}/issues"
    params = {
        "state": "all",
        "per_page": 100,
        "sort": "updated",
        "direction": "desc",
    }

    while True:
        response = requests.get(url, headers=github_headers(token), params=params, timeout=30)
        response.raise_for_status()
        page_data = response.json()
        if not page_data:
            break

        for item in page_data:
            if "pull_request" in item:
                continue
            issues.append(item)

        next_url = response.links.get("next", {}).get("url")
        if not next_url:
            break
        url = next_url
        params = None

    return issues


def sheet_service(service_account_json: str):
    creds_info = json.loads(service_account_json)
    creds = Credentials.from_service_account_info(creds_info, scopes=SCOPES)
    return build("sheets", "v4", credentials=creds, cache_discovery=False)


def get_sheet_values(service, spreadsheet_id: str, sheet_tab: str) -> list[list[str]]:
    result = (
        service.spreadsheets()
        .values()
        .get(spreadsheetId=spreadsheet_id, range=f"{sheet_tab}!A1:ZZ")
        .execute()
    )
    return result.get("values", [])


def resolve_sheet_tab(service, spreadsheet_id: str, preferred_tab: str, gid: str) -> str:
    if gid:
        metadata = (
            service.spreadsheets()
            .get(spreadsheetId=spreadsheet_id, fields="sheets(properties(sheetId,title))")
            .execute()
        )
        sheets = metadata.get("sheets", [])
        for sheet in sheets:
            props = sheet.get("properties", {})
            if str(props.get("sheetId")) == gid:
                return props.get("title", preferred_tab)

    return preferred_tab


def ensure_headers(existing_headers: list[str]) -> list[str]:
    headers = [h.strip() for h in existing_headers if h is not None]
    if not headers:
        return REQUIRED_COLUMNS.copy()

    for required in REQUIRED_COLUMNS:
        if required not in headers:
            headers.append(required)
    return headers


def normalize_row(row: list[str], width: int) -> list[str]:
    normalized = [str(x) for x in row[:width]]
    if len(normalized) < width:
        normalized.extend([""] * (width - len(normalized)))
    return normalized


def issue_to_values(issue: dict[str, Any]) -> dict[str, str]:
    labels = ", ".join(label["name"] for label in issue.get("labels", []))
    assignees = ", ".join(a["login"] for a in issue.get("assignees", []))
    return {
        "Issue #": str(issue["number"]),
        "Title": issue.get("title", ""),
        "State": issue.get("state", ""),
        "Labels": labels,
        "GitHub Assignees": assignees,
        "Reporter": (issue.get("user") or {}).get("login", ""),
        "Created At": issue.get("created_at", ""),
        "Updated At": issue.get("updated_at", ""),
        "Issue URL": issue.get("html_url", ""),
    }


def build_row_map(headers: list[str], existing_rows: list[list[str]]) -> dict[str, list[str]]:
    issue_col = headers.index("Issue #")
    by_issue: dict[str, list[str]] = {}
    for raw_row in existing_rows:
        row = normalize_row(raw_row, len(headers))
        issue_num = row[issue_col].strip()
        if issue_num:
            by_issue[issue_num] = row
    return by_issue


def upsert_rows(
    headers: list[str],
    existing_rows: list[list[str]],
    issues: list[dict[str, Any]],
) -> list[list[str]]:
    by_issue = build_row_map(headers, existing_rows)
    new_table_rows: list[list[str]] = []
    seen: set[str] = set()

    for issue in issues:
        mapped = issue_to_values(issue)
        issue_number = mapped["Issue #"]
        row = by_issue.get(issue_number, [""] * len(headers))

        for col, value in mapped.items():
            if col in MANAGED_COLUMNS and col in headers:
                row[headers.index(col)] = value

        new_table_rows.append(row)
        seen.add(issue_number)

    for issue_number, row in by_issue.items():
        if issue_number not in seen:
            new_table_rows.append(row)

    return new_table_rows


def clear_and_write_table(service, spreadsheet_id: str, sheet_tab: str, values: list[list[str]]) -> None:
    service.spreadsheets().values().clear(
        spreadsheetId=spreadsheet_id,
        range=f"{sheet_tab}!A1:ZZ",
        body={},
    ).execute()

    service.spreadsheets().values().update(
        spreadsheetId=spreadsheet_id,
        range=f"{sheet_tab}!A1",
        valueInputOption="RAW",
        body={"values": values},
    ).execute()


def main() -> int:
    try:
        cfg = load_config()
        issues = fetch_all_issues(cfg.github_repository, cfg.github_token)

        sheets = sheet_service(cfg.service_account_json)
        target_tab = resolve_sheet_tab(sheets, cfg.sheet_id, cfg.sheet_tab, cfg.sheet_gid)
        values = get_sheet_values(sheets, cfg.sheet_id, target_tab)

        existing_headers = values[0] if values else []
        existing_rows = values[1:] if len(values) > 1 else []
        headers = ensure_headers(existing_headers)

        rows = upsert_rows(headers, existing_rows, issues)
        clear_and_write_table(sheets, cfg.sheet_id, target_tab, [headers, *rows])

        print(f"Synced {len(issues)} GitHub issues into '{target_tab}'.")
        return 0
    except requests.HTTPError as exc:
        status = exc.response.status_code if exc.response is not None else "unknown"
        print(f"GitHub API request failed (status={status}): {exc}", file=sys.stderr)
        return 1
    except Exception as exc:  # pragma: no cover - defensive guard for Action logs
        print(f"Sync failed: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
