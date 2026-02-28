from mcp.server.fastmcp import FastMCP  # type: ignore[import-not-found]

from omnifocus_mcp.registration import typed_tool


mcp = FastMCP("omnifocus-mcp")


@typed_tool(mcp)
async def ping() -> dict[str, str]:
    return {"status": "ok", "message": "pong"}


import omnifocus_mcp.tools.tasks as tasks  # noqa: E402
import omnifocus_mcp.tools.projects as projects  # noqa: E402
import omnifocus_mcp.tools.tags as tags  # noqa: E402
import omnifocus_mcp.tools.folders as folders  # noqa: E402
import omnifocus_mcp.tools.forecast as forecast  # noqa: E402
import omnifocus_mcp.tools.perspectives as perspectives  # noqa: E402
import omnifocus_mcp.resources as resources  # noqa: E402
import omnifocus_mcp.prompts as prompts  # noqa: E402


get_inbox = tasks.get_inbox
list_tasks = tasks.list_tasks
get_task = tasks.get_task
search_tasks = tasks.search_tasks
create_task = tasks.create_task
create_tasks_batch = tasks.create_tasks_batch
complete_task = tasks.complete_task
update_task = tasks.update_task
delete_task = tasks.delete_task
move_task = tasks.move_task
list_projects = projects.list_projects
get_project = projects.get_project
create_project = projects.create_project
complete_project = projects.complete_project
list_tags = tags.list_tags
create_tag = tags.create_tag
list_folders = folders.list_folders
get_forecast = forecast.get_forecast
list_perspectives = perspectives.list_perspectives
inbox_resource = resources.inbox_resource
today_resource = resources.today_resource
projects_resource = resources.projects_resource
daily_review = prompts.daily_review
weekly_review = prompts.weekly_review
inbox_processing = prompts.inbox_processing
project_planning = prompts.project_planning
