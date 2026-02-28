import json

import pytest

from omnifocus_mcp.jxa import escape_for_jxa


@pytest.mark.parametrize(
    "value",
    [
        'He said "hello"',
        r"C:\Users\test",
        "line1\nline2\tend",
        "こんにちは",
        "emoji: 🚀✨",
        "",
        "a" * 10000,
    ],
)
def test_escape_for_jxa_round_trips(value: str) -> None:
    escaped = escape_for_jxa(value)
    assert json.loads(escaped) == value
