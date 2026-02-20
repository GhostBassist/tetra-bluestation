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
DEFAULT_SHEET_ID = "1VEaGJtokDDVlZ-_JD3SBQX52dQJeUbAxWxOAU8-qZsk"
DEFAULT_SHEET_TAB = "Bug Tracker"
DEFAULT_SHEET_GID = "1671453665"


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
    sheet_id = os.getenv("GOOGLE_SHEET_ID", "").strip() or DEFAULT_SHEET_ID
    sheet_tab = os.getenv("GOOGLE_SHEET_TAB", "").strip() or DEFAULT_SHEET_TAB
    sheet_gid = os.getenv("GOOGLE_SHEET_GID", "").strip() or DEFAULT_SHEET_GID

    return Config(
        github_token=must_getenv("GITHUB_TOKEN"),
        github_repository=must_getenv("GITHUB_REPOSITORY"),
        sheet_id=sheet_id,
        sheet_tab=sheet_tab,
        sheet_gid=sheet_gid,
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


def get_sheet_metadata(service, spreadsheet_id: str) -> list[dict[str, Any]]:
    metadata = (
        service.spreadsheets()
        .get(
            spreadsheetId=spreadsheet_id,
            fields="sheets(properties(sheetId,title,index),bandedRanges(bandedRangeId,range))",
        )
        .execute()
    )
    return metadata.get("sheets", [])


def resolve_sheet_tab_and_id(
    service, spreadsheet_id: str, preferred_tab: str, gid: str
) -> tuple[str, int]:
    sheets = get_sheet_metadata(service, spreadsheet_id)

    if gid:
        for sheet in sheets:
            props = sheet.get("properties", {})
            if str(props.get("sheetId")) == gid:
                return props.get("title", preferred_tab), int(props["sheetId"])

    for sheet in sheets:
        props = sheet.get("properties", {})
        if props.get("title") == preferred_tab:
            return preferred_tab, int(props["sheetId"])

    raise RuntimeError(
        f"Unable to find sheet tab '{preferred_tab}'. Set GOOGLE_SHEET_TAB or GOOGLE_SHEET_GID correctly."
    )


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


def apply_sheet_formatting(
    service,
    spreadsheet_id: str,
    sheet_id: int,
    headers: list[str],
    row_count: int,
) -> None:
    col_count = len(headers)
    metadata = get_sheet_metadata(service, spreadsheet_id)
    existing_banding_ids: list[int] = []
    for sheet in metadata:
        props = sheet.get("properties", {})
        if int(props.get("sheetId", -1)) == sheet_id:
            for banded in sheet.get("bandedRanges", []):
                band_id = banded.get("bandedRangeId")
                if isinstance(band_id, int):
                    existing_banding_ids.append(band_id)

    def col_index(name: str) -> int:
        return headers.index(name) if name in headers else -1

    requests: list[dict[str, Any]] = []

    for band_id in existing_banding_ids:
        requests.append({"deleteBanding": {"bandedRangeId": band_id}})

    requests.extend([
        {
            "updateSheetProperties": {
                "properties": {"sheetId": sheet_id, "gridProperties": {"frozenRowCount": 1}},
                "fields": "gridProperties.frozenRowCount",
            }
        },
        {
            "repeatCell": {
                "range": {
                    "sheetId": sheet_id,
                    "startRowIndex": 0,
                    "endRowIndex": 1,
                    "startColumnIndex": 0,
                    "endColumnIndex": col_count,
                },
                "cell": {
                    "userEnteredFormat": {
                        "backgroundColor": {"red": 0.09, "green": 0.29, "blue": 0.53},
                        "textFormat": {"foregroundColor": {"red": 1, "green": 1, "blue": 1}, "bold": True},
                        "horizontalAlignment": "CENTER",
                        "verticalAlignment": "MIDDLE",
                        "wrapStrategy": "WRAP",
                    }
                },
                "fields": "userEnteredFormat(backgroundColor,textFormat,horizontalAlignment,verticalAlignment,wrapStrategy)",
            }
        },
        {
            "repeatCell": {
                "range": {
                    "sheetId": sheet_id,
                    "startRowIndex": 1,
                    "endRowIndex": max(row_count, 2),
                    "startColumnIndex": 0,
                    "endColumnIndex": col_count,
                },
                "cell": {"userEnteredFormat": {"verticalAlignment": "MIDDLE", "wrapStrategy": "WRAP"}},
                "fields": "userEnteredFormat(verticalAlignment,wrapStrategy)",
            }
        },
        {
            "addBanding": {
                "bandedRange": {
                    "range": {
                        "sheetId": sheet_id,
                        "startRowIndex": 0,
                        "endRowIndex": max(row_count, 2),
                        "startColumnIndex": 0,
                        "endColumnIndex": col_count,
                    },
                    "headerColor": {"red": 0.09, "green": 0.29, "blue": 0.53},
                    "firstBandColor": {"red": 0.96, "green": 0.98, "blue": 1.0},
                    "secondBandColor": {"red": 1.0, "green": 1.0, "blue": 1.0},
                }
            }
        },
        {
            "setBasicFilter": {
                "filter": {
                    "range": {
                        "sheetId": sheet_id,
                        "startRowIndex": 0,
                        "endRowIndex": max(row_count, 2),
                        "startColumnIndex": 0,
                        "endColumnIndex": col_count,
                    }
                }
            }
        },
        {
            "autoResizeDimensions": {
                "dimensions": {
                    "sheetId": sheet_id,
                    "dimension": "COLUMNS",
                    "startIndex": 0,
                    "endIndex": col_count,
                }
            }
        },
    ])

    priority_idx = col_index("Priority")
    triage_idx = col_index("Triage Status")

    if priority_idx >= 0:
        requests.append(
            {
                "repeatCell": {
                    "range": {
                        "sheetId": sheet_id,
                        "startRowIndex": 1,
                        "endRowIndex": max(row_count, 2000),
                        "startColumnIndex": priority_idx,
                        "endColumnIndex": priority_idx + 1,
                    },
                    "cell": {
                        "dataValidation": {
                            "condition": {
                                "type": "ONE_OF_LIST",
                                "values": [
                                    {"userEnteredValue": "Critical"},
                                    {"userEnteredValue": "High"},
                                    {"userEnteredValue": "Medium"},
                                    {"userEnteredValue": "Low"},
                                ],
                            },
                            "strict": True,
                            "showCustomUi": True,
                        }
                    },
                    "fields": "dataValidation",
                }
            }
        )

    if triage_idx >= 0:
        requests.append(
            {
                "repeatCell": {
                    "range": {
                        "sheetId": sheet_id,
                        "startRowIndex": 1,
                        "endRowIndex": max(row_count, 2000),
                        "startColumnIndex": triage_idx,
                        "endColumnIndex": triage_idx + 1,
                    },
                    "cell": {
                        "dataValidation": {
                            "condition": {
                                "type": "ONE_OF_LIST",
                                "values": [
                                    {"userEnteredValue": "New"},
                                    {"userEnteredValue": "Confirmed"},
                                    {"userEnteredValue": "In Progress"},
                                    {"userEnteredValue": "Blocked"},
                                    {"userEnteredValue": "Done"},
                                ],
                            },
                            "strict": True,
                            "showCustomUi": True,
                        }
                    },
                    "fields": "dataValidation",
                }
            }
        )

    service.spreadsheets().batchUpdate(
        spreadsheetId=spreadsheet_id,
        body={"requests": requests},
    ).execute()


def main() -> int:
    try:
        cfg = load_config()
        issues = fetch_all_issues(cfg.github_repository, cfg.github_token)

        sheets = sheet_service(cfg.service_account_json)
        target_tab, sheet_id = resolve_sheet_tab_and_id(
            sheets, cfg.sheet_id, cfg.sheet_tab, cfg.sheet_gid
        )
        values = get_sheet_values(sheets, cfg.sheet_id, target_tab)

        existing_headers = values[0] if values else []
        existing_rows = values[1:] if len(values) > 1 else []
        headers = ensure_headers(existing_headers)

        rows = upsert_rows(headers, existing_rows, issues)
        clear_and_write_table(sheets, cfg.sheet_id, target_tab, [headers, *rows])
        apply_sheet_formatting(
            sheets,
            cfg.sheet_id,
            sheet_id=sheet_id,
            headers=headers,
            row_count=len(rows) + 1,
        )

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
