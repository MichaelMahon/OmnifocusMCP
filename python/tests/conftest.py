from collections.abc import Callable
from typing import Any

import pytest


@pytest.fixture
def sample_omnijs_payload() -> list[dict[str, str]]:
    return [{"id": "abc123", "name": "Test Task"}]


@pytest.fixture
def mock_run_omnijs(
    monkeypatch: pytest.MonkeyPatch,
) -> Callable[[Any], dict[str, Any]]:
    state: dict[str, Any] = {"result": [{"id": "abc123", "name": "Test Task"}]}
    calls: list[dict[str, Any]] = []

    async def fake_run_omnijs(script: str, timeout_seconds: float = 30.0) -> Any:
        calls.append({"script": script, "timeout_seconds": timeout_seconds})
        return state["result"]

    monkeypatch.setattr("omnifocus_mcp.jxa.run_omnijs", fake_run_omnijs)

    def configure(result: Any) -> dict[str, Any]:
        state["result"] = result
        return {"state": state, "calls": calls}

    return configure
