import json
from datetime import UTC, datetime, timedelta
import importlib
import sys
import types
from uuid import uuid4

import pytest
import pytest_asyncio

from omnifocus_mcp.jxa import escape_for_jxa, run_omnijs


class _FakeFastMCP:
    def __init__(self, name: str):
        self.name = name

    def tool(self):
        def decorator(func):
            return func

        return decorator

    def resource(self, _: str):
        def decorator(func):
            return func

        return decorator

    def prompt(self):
        def decorator(func):
            return func

        return decorator


_mcp_module = types.ModuleType("mcp")
_mcp_server_module = types.ModuleType("mcp.server")
_mcp_fastmcp_module = types.ModuleType("mcp.server.fastmcp")
_mcp_fastmcp_module.FastMCP = _FakeFastMCP
_mcp_server_module.fastmcp = _mcp_fastmcp_module
_mcp_module.server = _mcp_server_module
sys.modules.setdefault("mcp", _mcp_module)
sys.modules.setdefault("mcp.server", _mcp_server_module)
sys.modules.setdefault("mcp.server.fastmcp", _mcp_fastmcp_module)

server = importlib.import_module("omnifocus_mcp.server")


def _test_name(suffix: str) -> str:
    return f"[TEST-MCP] {suffix} {uuid4().hex[:8]}"


def _json_object(value: str) -> dict[str, object]:
    parsed = json.loads(value)
    assert isinstance(parsed, dict)
    return parsed


def _json_array(value: str) -> list[object]:
    parsed = json.loads(value)
    assert isinstance(parsed, list)
    return parsed


def _assert_has_keys(item: dict[str, object], expected: set[str]) -> None:
    assert expected.issubset(set(item.keys()))


@pytest_asyncio.fixture
async def cleanup_registry() -> dict[str, list[str]]:
    registry: dict[str, list[str]] = {"task_ids": [], "project_ids": []}
    yield registry
    for task_id in reversed(registry["task_ids"]):
        await run_omnijs(
            f"""
const taskId = {escape_for_jxa(task_id)};
const task = document.flattenedTasks.find(item => item.id.primaryKey === taskId);
if (task) task.drop(false);
return true;
""".strip()
        )
    for project_id in reversed(registry["project_ids"]):
        await run_omnijs(
            f"""
const projectId = {escape_for_jxa(project_id)};
const project = document.flattenedProjects.find(item => item.id.primaryKey === projectId);
if (project) project.drop(false);
return true;
""".strip()
        )


@pytest.mark.integration
@pytest.mark.asyncio
async def test_jxa_bridge_connectivity() -> None:
    result = await run_omnijs("return document.flattenedTasks.length;")
    assert isinstance(result, int)
    assert result >= 0


@pytest.mark.integration
@pytest.mark.asyncio
async def test_read_tools_return_valid_json(cleanup_registry: dict[str, list[str]]) -> None:
    task_name = _test_name("Read tool task")
    created_task = _json_object(await create_task(name=task_name, flagged=True))
    created_task_id = created_task["id"]
    assert isinstance(created_task_id, str)
    cleanup_registry["task_ids"].append(created_task_id)

    project_name = _test_name("Read tool project")
    created_project = _json_object(await create_project(name=project_name))
    created_project_id = created_project["id"]
    assert isinstance(created_project_id, str)
    cleanup_registry["project_ids"].append(created_project_id)

    inbox_items = _json_array(await get_inbox(limit=25))
    if inbox_items:
        assert isinstance(inbox_items[0], dict)
        _assert_has_keys(
            inbox_items[0],
            {"id", "name", "note", "flagged", "dueDate", "deferDate", "tags", "estimatedMinutes"},
        )

    task_items = _json_array(await list_tasks(status="all", limit=25))
    if task_items:
        assert isinstance(task_items[0], dict)
        _assert_has_keys(
            task_items[0],
            {
                "id",
                "name",
                "note",
                "flagged",
                "dueDate",
                "deferDate",
                "completed",
                "projectName",
                "tags",
                "estimatedMinutes",
            },
        )

    task_details = _json_object(await get_task(created_task_id))
    _assert_has_keys(
        task_details,
        {
            "id",
            "name",
            "note",
            "flagged",
            "dueDate",
            "deferDate",
            "completed",
            "completionDate",
            "projectName",
            "tags",
            "estimatedMinutes",
            "children",
            "parentName",
            "sequential",
            "repetitionRule",
        },
    )

    task_search = _json_array(await search_tasks("Read tool task", limit=25))
    if task_search:
        assert isinstance(task_search[0], dict)
        _assert_has_keys(
            task_search[0],
            {
                "id",
                "name",
                "note",
                "flagged",
                "dueDate",
                "deferDate",
                "completed",
                "projectName",
                "tags",
                "estimatedMinutes",
            },
        )

    projects = _json_array(await list_projects(status="active", limit=25))
    if projects:
        assert isinstance(projects[0], dict)
        _assert_has_keys(
            projects[0],
            {
                "id",
                "name",
                "status",
                "folderName",
                "taskCount",
                "remainingTaskCount",
                "deferDate",
                "dueDate",
                "note",
                "sequential",
                "reviewInterval",
            },
        )

    project_details = _json_object(await get_project(created_project_id))
    _assert_has_keys(
        project_details,
        {
            "id",
            "name",
            "status",
            "folderName",
            "taskCount",
            "remainingTaskCount",
            "deferDate",
            "dueDate",
            "note",
            "sequential",
            "reviewInterval",
            "rootTasks",
        },
    )

    tags = _json_array(await list_tags(limit=25))
    if tags:
        assert isinstance(tags[0], dict)
        _assert_has_keys(tags[0], {"id", "name", "parent", "availableTaskCount", "status"})

    folders = _json_array(await list_folders(limit=25))
    if folders:
        assert isinstance(folders[0], dict)
        _assert_has_keys(folders[0], {"id", "name", "parentName", "projectCount"})

    forecast = _json_object(await get_forecast(limit=25))
    _assert_has_keys(forecast, {"overdue", "dueToday", "flagged"})

    perspectives = _json_array(await list_perspectives(limit=25))
    if perspectives:
        assert isinstance(perspectives[0], dict)
        _assert_has_keys(perspectives[0], {"id", "name"})


@pytest.mark.integration
@pytest.mark.asyncio
async def test_task_lifecycle(cleanup_registry: dict[str, list[str]]) -> None:
    due_tomorrow = (datetime.now(tz=UTC) + timedelta(days=1)).replace(microsecond=0).isoformat()
    created = _json_object(
        await create_task(
            name=_test_name("Lifecycle task"),
            flagged=True,
            dueDate=due_tomorrow,
        )
    )
    created_id = created["id"]
    assert isinstance(created_id, str)
    cleanup_registry["task_ids"].append(created_id)

    fetched = _json_object(await get_task(created_id))
    assert fetched["id"] == created_id
    assert fetched["flagged"] is True

    updated_name = _test_name("Lifecycle updated")
    updated = _json_object(await update_task(task_id=created_id, name=updated_name))
    assert updated["id"] == created_id
    assert updated["name"] == updated_name

    completed = _json_object(await complete_task(created_id))
    assert completed["id"] == created_id
    assert completed["completed"] is True

    deleted = _json_object(await delete_task(created_id))
    assert deleted["id"] == created_id
    assert deleted["deleted"] is True
    cleanup_registry["task_ids"] = [task_id for task_id in cleanup_registry["task_ids"] if task_id != created_id]


@pytest.mark.integration
@pytest.mark.asyncio
async def test_search_finds_created_task(cleanup_registry: dict[str, list[str]]) -> None:
    unique_token = uuid4().hex[:10]
    name = f"[TEST-MCP] Search {unique_token}"
    created = _json_object(await create_task(name=name, note=f"search-token-{unique_token}"))
    created_id = created["id"]
    assert isinstance(created_id, str)
    cleanup_registry["task_ids"].append(created_id)

    search_results = _json_array(await search_tasks(unique_token, limit=50))
    assert any(isinstance(item, dict) and item.get("id") == created_id for item in search_results)


@pytest.mark.integration
@pytest.mark.asyncio
async def test_project_lifecycle(cleanup_registry: dict[str, list[str]]) -> None:
    created = _json_object(await create_project(name=_test_name("Lifecycle project")))
    project_id = created["id"]
    assert isinstance(project_id, str)
    cleanup_registry["project_ids"].append(project_id)

    fetched = _json_object(await get_project(project_id))
    assert fetched["id"] == project_id

    completed = _json_object(await complete_project(project_id))
    assert completed["id"] == project_id
    assert completed["completed"] is True
import json
from datetime import datetime, timedelta, timezone
import importlib.util
import sys
import types
from uuid import uuid4

import pytest
import pytest_asyncio

if importlib.util.find_spec("mcp") is None:
    class _FakeFastMCP:
        def __init__(self, name: str):
            self.name = name

        def tool(self):
            def decorator(func):
                return func

            return decorator

        def resource(self, _uri: str):
            def decorator(func):
                return func

            return decorator

        def prompt(self):
            def decorator(func):
                return func

            return decorator

    mcp_module = types.ModuleType("mcp")
    mcp_server_module = types.ModuleType("mcp.server")
    mcp_fastmcp_module = types.ModuleType("mcp.server.fastmcp")
    mcp_fastmcp_module.FastMCP = _FakeFastMCP
    mcp_server_module.fastmcp = mcp_fastmcp_module
    mcp_module.server = mcp_server_module
    sys.modules.setdefault("mcp", mcp_module)
    sys.modules.setdefault("mcp.server", mcp_server_module)
    sys.modules.setdefault("mcp.server.fastmcp", mcp_fastmcp_module)

from omnifocus_mcp.jxa import run_omnijs
from omnifocus_mcp.tools.folders import list_folders
from omnifocus_mcp.tools.forecast import get_forecast
from omnifocus_mcp.tools.perspectives import list_perspectives
from omnifocus_mcp.tools.projects import complete_project, create_project, get_project, list_projects
from omnifocus_mcp.tools.tags import list_tags
from omnifocus_mcp.tools.tasks import (
    complete_task,
    create_task,
    delete_task,
    get_inbox,
    get_task,
    list_tasks,
    search_tasks,
    update_task,
)


def _parse_json(payload: str) -> object:
    return json.loads(payload)


def _assert_keys(obj: dict[str, object], required: set[str]) -> None:
    assert required.issubset(obj.keys())


@pytest_asyncio.fixture
async def integration_state() -> dict[str, object]:
    suffix = uuid4().hex[:8]
    state: dict[str, object] = {
        "prefix": f"[TEST-MCP] Integration {suffix}",
        "task_ids": [],
        "project_ids": [],
    }
    try:
        yield state
    finally:
        task_ids = list(reversed(state["task_ids"]))  # type: ignore[arg-type]
        for task_id in task_ids:
            try:
                await delete_task(task_id=task_id)
            except Exception:
                continue

        project_ids = state["project_ids"]  # type: ignore[assignment]
        if project_ids:
            failures = await run_omnijs(
                f"""
const projectIds = {json.dumps(project_ids)};
const failures = [];
projectIds.forEach(projectId => {{
  const project = document.flattenedProjects.find(item => item.id.primaryKey === projectId);
  if (!project) return;
  try {{
    project.drop(false);
  }} catch (firstError) {{
    try {{
      project.drop();
    }} catch (secondError) {{
      const message = secondError && secondError.message ? secondError.message : String(secondError);
      failures.push({{ id: projectId, error: message }});
    }}
  }}
}});
return failures;
""".strip()
            )
            assert isinstance(failures, list)
            assert not failures


@pytest.mark.integration
@pytest.mark.asyncio
async def test_jxa_bridge_connectivity() -> None:
    result = await run_omnijs("return document.flattenedTasks.length;")
    assert isinstance(result, int)
    assert result >= 0


@pytest.mark.integration
@pytest.mark.asyncio
async def test_read_tools_return_valid_json(integration_state: dict[str, object]) -> None:
    prefix = integration_state["prefix"]  # type: ignore[assignment]

    created_task = _parse_json(await create_task(name=f"{prefix} Read Tool Task"))
    assert isinstance(created_task, dict)
    created_task_id = created_task.get("id")
    assert isinstance(created_task_id, str)
    integration_state["task_ids"].append(created_task_id)  # type: ignore[index]

    created_project = _parse_json(await create_project(name=f"{prefix} Read Tool Project"))
    assert isinstance(created_project, dict)
    created_project_id = created_project.get("id")
    assert isinstance(created_project_id, str)
    integration_state["project_ids"].append(created_project_id)  # type: ignore[index]

    inbox = _parse_json(await get_inbox(limit=20))
    assert isinstance(inbox, list)
    if inbox:
        assert isinstance(inbox[0], dict)
        _assert_keys(
            inbox[0],
            {"id", "name", "note", "flagged", "dueDate", "deferDate", "tags", "estimatedMinutes"},
        )

    tasks = _parse_json(await list_tasks(status="all", limit=20))
    assert isinstance(tasks, list)
    if tasks:
        assert isinstance(tasks[0], dict)
        _assert_keys(
            tasks[0],
            {
                "id",
                "name",
                "note",
                "flagged",
                "dueDate",
                "deferDate",
                "completed",
                "projectName",
                "tags",
                "estimatedMinutes",
            },
        )

    task = _parse_json(await get_task(task_id=created_task_id))
    assert isinstance(task, dict)
    _assert_keys(
        task,
        {
            "id",
            "name",
            "note",
            "flagged",
            "dueDate",
            "deferDate",
            "completed",
            "completionDate",
            "projectName",
            "tags",
            "estimatedMinutes",
            "children",
            "parentName",
            "sequential",
            "repetitionRule",
        },
    )

    searched_tasks = _parse_json(await search_tasks(query=prefix, limit=20))
    assert isinstance(searched_tasks, list)
    if searched_tasks:
        assert isinstance(searched_tasks[0], dict)
        _assert_keys(
            searched_tasks[0],
            {
                "id",
                "name",
                "note",
                "flagged",
                "dueDate",
                "deferDate",
                "completed",
                "projectName",
                "tags",
                "estimatedMinutes",
            },
        )

    projects = _parse_json(await list_projects(limit=20))
    assert isinstance(projects, list)
    if projects:
        assert isinstance(projects[0], dict)
        _assert_keys(
            projects[0],
            {
                "id",
                "name",
                "status",
                "folderName",
                "taskCount",
                "remainingTaskCount",
                "deferDate",
                "dueDate",
                "note",
                "sequential",
                "reviewInterval",
            },
        )

    project = _parse_json(await get_project(project_id_or_name=created_project_id))
    assert isinstance(project, dict)
    _assert_keys(
        project,
        {
            "id",
            "name",
            "status",
            "folderName",
            "taskCount",
            "remainingTaskCount",
            "deferDate",
            "dueDate",
            "note",
            "sequential",
            "reviewInterval",
            "rootTasks",
        },
    )

    tags = _parse_json(await list_tags(limit=20))
    assert isinstance(tags, list)
    if tags:
        assert isinstance(tags[0], dict)
        _assert_keys(tags[0], {"id", "name", "parent", "availableTaskCount", "status"})

    folders = _parse_json(await list_folders(limit=20))
    assert isinstance(folders, list)
    if folders:
        assert isinstance(folders[0], dict)
        _assert_keys(folders[0], {"id", "name", "parentName", "projectCount"})

    forecast = _parse_json(await get_forecast(limit=20))
    assert isinstance(forecast, dict)
    _assert_keys(forecast, {"overdue", "dueToday", "flagged"})

    perspectives = _parse_json(await list_perspectives(limit=20))
    assert isinstance(perspectives, list)
    if perspectives:
        assert isinstance(perspectives[0], dict)
        _assert_keys(perspectives[0], {"id", "name"})


@pytest.mark.integration
@pytest.mark.asyncio
async def test_task_lifecycle(integration_state: dict[str, object]) -> None:
    prefix = integration_state["prefix"]  # type: ignore[assignment]
    due_date = (datetime.now(timezone.utc) + timedelta(days=1)).replace(microsecond=0)
    due_date_iso = due_date.isoformat().replace("+00:00", "Z")

    created = _parse_json(
        await create_task(name=f"{prefix} Lifecycle Task", flagged=True, dueDate=due_date_iso)
    )
    assert isinstance(created, dict)
    created_task_id = created.get("id")
    assert isinstance(created_task_id, str)
    integration_state["task_ids"].append(created_task_id)  # type: ignore[index]

    fetched = _parse_json(await get_task(task_id=created_task_id))
    assert isinstance(fetched, dict)
    assert fetched["id"] == created_task_id
    assert fetched["name"] == f"{prefix} Lifecycle Task"
    assert fetched["flagged"] is True
    assert isinstance(fetched["dueDate"], str)

    updated = _parse_json(await update_task(task_id=created_task_id, name=f"{prefix} Updated Task"))
    assert isinstance(updated, dict)
    assert updated["id"] == created_task_id
    assert updated["name"] == f"{prefix} Updated Task"

    completed = _parse_json(await complete_task(task_id=created_task_id))
    assert isinstance(completed, dict)
    assert completed["id"] == created_task_id
    assert completed["completed"] is True

    deleted = _parse_json(await delete_task(task_id=created_task_id))
    assert isinstance(deleted, dict)
    assert deleted["id"] == created_task_id
    assert deleted["deleted"] is True
    integration_state["task_ids"].remove(created_task_id)  # type: ignore[index]


@pytest.mark.integration
@pytest.mark.asyncio
async def test_search_finds_created_task(integration_state: dict[str, object]) -> None:
    prefix = integration_state["prefix"]  # type: ignore[assignment]
    token = uuid4().hex[:10]
    task_name = f"{prefix} Search Task {token}"

    created = _parse_json(await create_task(name=task_name, note=f"search token {token}"))
    assert isinstance(created, dict)
    created_task_id = created.get("id")
    assert isinstance(created_task_id, str)
    integration_state["task_ids"].append(created_task_id)  # type: ignore[index]

    results = _parse_json(await search_tasks(query=token, limit=50))
    assert isinstance(results, list)
    result_ids = {item["id"] for item in results if isinstance(item, dict) and "id" in item}
    assert created_task_id in result_ids


@pytest.mark.integration
@pytest.mark.asyncio
async def test_project_lifecycle(integration_state: dict[str, object]) -> None:
    prefix = integration_state["prefix"]  # type: ignore[assignment]
    project_name = f"{prefix} Project Lifecycle"

    created = _parse_json(await create_project(name=project_name))
    assert isinstance(created, dict)
    project_id = created.get("id")
    assert isinstance(project_id, str)
    integration_state["project_ids"].append(project_id)  # type: ignore[index]

    fetched = _parse_json(await get_project(project_id_or_name=project_id))
    assert isinstance(fetched, dict)
    assert fetched["id"] == project_id
    assert fetched["name"] == project_name

    completed = _parse_json(await complete_project(project_id_or_name=project_id))
    assert isinstance(completed, dict)
    assert completed["id"] == project_id
    assert completed["completed"] is True
