import json
from typing import Any, Callable, TypeVar, cast

from mcp.server.fastmcp import FastMCP  # type: ignore[import-not-found]

from omnifocus_mcp.jxa import run_omnijs


F = TypeVar("F", bound=Callable[..., Any])


def _typed_tool(server: Any) -> Callable[[F], F]:
    return cast(Callable[[F], F], server.tool())


mcp = FastMCP("omnifocus-mcp")


@_typed_tool(mcp)
async def ping() -> dict[str, str]:
    return {"status": "ok", "message": "pong"}


@_typed_tool(mcp)
async def get_inbox(limit: int = 100) -> str:
    """get inbox tasks from omnifocus.

    returns unprocessed inbox tasks with id, name, note, flagged state, due/defer
    dates, tag names, and estimated minutes. limit controls max returned tasks.
    """
    if limit < 1:
        raise ValueError("limit must be greater than 0.")

    script = f"""
const tasks = inbox
  .filter(task => !task.completed)
  .slice(0, {limit});

return tasks.map(task => {{
  const tags = task.tags.map(tag => tag.name);
  return {{
    id: task.id.primaryKey,
    name: task.name,
    note: task.note,
    flagged: task.flagged,
    dueDate: task.dueDate ? task.dueDate.toISOString() : null,
    deferDate: task.deferDate ? task.deferDate.toISOString() : null,
    tags: tags,
    estimatedMinutes: task.estimatedMinutes
  }};
}});
""".strip()
    result = await run_omnijs(script)
    return json.dumps(result)
