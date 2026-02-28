import asyncio

import pytest

from omnifocus_mcp.jxa import run_jxa


class FakeProcess:
    def __init__(self, stdout: str, stderr: str, returncode: int, delay: float = 0.0):
        self._stdout = stdout.encode("utf-8")
        self._stderr = stderr.encode("utf-8")
        self.returncode = returncode
        self._delay = delay
        self.killed = False

    async def communicate(self) -> tuple[bytes, bytes]:
        if self._delay > 0:
            await asyncio.sleep(self._delay)
        return (self._stdout, self._stderr)

    def kill(self) -> None:
        self.killed = True

    async def wait(self) -> int:
        return self.returncode


@pytest.mark.asyncio
async def test_run_jxa_non_zero_exit_has_clear_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    async def fake_create_subprocess_exec(*args: str, **kwargs: object) -> FakeProcess:
        return FakeProcess("", "syntax error: expected ';'", 1)

    monkeypatch.setattr(asyncio, "create_subprocess_exec", fake_create_subprocess_exec)

    with pytest.raises(RuntimeError, match="syntax error"):
        await run_jxa("invalid script")


@pytest.mark.asyncio
async def test_run_jxa_timeout_raises_timeout_error(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    async def fake_create_subprocess_exec(*args: str, **kwargs: object) -> FakeProcess:
        return FakeProcess("", "", 0, delay=0.2)

    monkeypatch.setattr(asyncio, "create_subprocess_exec", fake_create_subprocess_exec)

    with pytest.raises(TimeoutError, match="timed out"):
        await run_jxa("1 + 1", timeout_seconds=0.01)
