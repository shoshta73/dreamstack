import datetime as dt
import importlib.machinery
import importlib.util
from pathlib import Path
import unittest


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "scripts" / "next-dev-tag"
LOADER = importlib.machinery.SourceFileLoader("next_dev_tag", str(SCRIPT_PATH))
SPEC = importlib.util.spec_from_loader("next_dev_tag", LOADER)
next_dev_tag = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(next_dev_tag)


class ChangelogHeadingTest(unittest.TestCase):
    def test_updates_first_unreleased_heading(self) -> None:
        changelog = """# Changelog

## Unreleased

- New change.

## Unreleased

- Older literal heading.
"""

        updated = next_dev_tag.changelog_with_dev_heading(
            changelog,
            "26w25g",
            dt.date(2026, 6, 17),
        )

        self.assertEqual(
            updated,
            """# Changelog

## 26w25g [2026-06-17]

- New change.

## Unreleased

- Older literal heading.
""",
        )

    def test_requires_unreleased_heading(self) -> None:
        with self.assertRaises(ValueError):
            next_dev_tag.changelog_with_dev_heading(
                "# Changelog\n\n## 26w25f [2026-06-17]\n",
                "26w25g",
                dt.date(2026, 6, 17),
            )


if __name__ == "__main__":
    unittest.main()
