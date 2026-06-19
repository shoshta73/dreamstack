import datetime as dt
import importlib.machinery
import importlib.util
from pathlib import Path

import pytest


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "scripts" / "next-dev-tag"
EXAMPLE_TAG = "99w01a"
EXAMPLE_DATE = dt.date(2099, 1, 2)
LOADER = importlib.machinery.SourceFileLoader("next_dev_tag", str(SCRIPT_PATH))
SPEC = importlib.util.spec_from_loader("next_dev_tag", LOADER)
next_dev_tag = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(next_dev_tag)


def test_updates_first_unreleased_heading() -> None:
    changelog = """# Changelog

## Unreleased

- New change.

## Unreleased

- Older literal heading.
"""

    updated = next_dev_tag.changelog_with_dev_heading(
        changelog,
        EXAMPLE_TAG,
        EXAMPLE_DATE,
    )

    assert updated == """# Changelog

## 99w01a [2099-01-02]

- New change.

## Unreleased

- Older literal heading.
"""


def test_requires_unreleased_heading() -> None:
    with pytest.raises(ValueError):
        next_dev_tag.changelog_with_dev_heading(
            "# Changelog\n\n## 99w01a [2099-01-02]\n",
            EXAMPLE_TAG,
            EXAMPLE_DATE,
        )


def test_update_changelog_heading_writes_changelog(tmp_path, monkeypatch) -> None:
    class FixedDate(dt.date):
        @classmethod
        def today(cls):
            return cls(2099, 1, 2)

    changelog_path = tmp_path / "CHANGELOG.md"
    changelog_path.write_text("# Changelog\n\n## Unreleased\n\n- New change.\n")
    monkeypatch.setattr(next_dev_tag, "CHANGELOG_PATH", changelog_path)
    monkeypatch.setattr(next_dev_tag.dt, "date", FixedDate)

    updated = next_dev_tag.update_changelog_heading(EXAMPLE_TAG)

    assert updated
    assert changelog_path.read_text().splitlines()[2] == "## 99w01a [2099-01-02]"


def test_update_changelog_heading_skips_without_unreleased(tmp_path, monkeypatch) -> None:
    changelog_path = tmp_path / "CHANGELOG.md"
    changelog = "# Changelog\n\n## 99w01a [2099-01-02]\n"
    changelog_path.write_text(changelog)
    monkeypatch.setattr(next_dev_tag, "CHANGELOG_PATH", changelog_path)

    updated = next_dev_tag.update_changelog_heading(EXAMPLE_TAG)

    assert not updated
    assert changelog_path.read_text() == changelog


def test_create_tag_commits_changelog_before_tag(monkeypatch) -> None:
    git_calls = []

    monkeypatch.setattr(next_dev_tag, "ensure_clean_worktree", lambda force: None)
    monkeypatch.setattr(next_dev_tag, "tag_exists", lambda tag: False)
    monkeypatch.setattr(next_dev_tag, "update_changelog_heading", lambda tag: True)
    monkeypatch.setattr(next_dev_tag, "require_git", lambda args: git_calls.append(args) or "")

    next_dev_tag.create_tag(EXAMPLE_TAG, force=False)

    assert git_calls == [
        ["add", "CHANGELOG.md"],
        ["commit", "-m", "chore: update changelog"],
        ["tag", EXAMPLE_TAG],
    ]


def test_create_tag_without_changelog_update_only_tags(monkeypatch) -> None:
    git_calls = []

    monkeypatch.setattr(next_dev_tag, "ensure_clean_worktree", lambda force: None)
    monkeypatch.setattr(next_dev_tag, "tag_exists", lambda tag: False)
    monkeypatch.setattr(next_dev_tag, "update_changelog_heading", lambda tag: False)
    monkeypatch.setattr(next_dev_tag, "require_git", lambda args: git_calls.append(args) or "")

    next_dev_tag.create_tag(EXAMPLE_TAG, force=False)

    assert git_calls == [["tag", EXAMPLE_TAG]]
