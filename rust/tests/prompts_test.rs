use std::{future::Future, pin::Pin};

use omnifocus_mcp::{
    error::OmniFocusError,
    jxa::JxaRunner,
    prompts::{daily_review, inbox_processing, project_planning, weekly_review},
};
use serde_json::{json, Value};

#[derive(Clone, Default)]
struct RoutingRunner;

impl JxaRunner for RoutingRunner {
    fn run_omnijs<'a>(
        &'a self,
        script: &'a str,
    ) -> Pin<Box<dyn Future<Output = omnifocus_mcp::error::Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            if script.contains("const tasks = inbox") {
                return Ok(json!([{ "id": "i1", "name": "inbox item" }]));
            }
            if script.contains("const statusFilter = \"due_soon\"") {
                return Ok(json!([{ "id": "d1", "name": "due soon task" }]));
            }
            if script.contains("const statusFilter = \"overdue\"") {
                return Ok(json!([{ "id": "o1", "name": "overdue task" }]));
            }
            if script.contains("const flaggedFilter = true")
                && script.contains("const statusFilter = \"all\"")
            {
                return Ok(json!([{ "id": "f1", "name": "flagged task" }]));
            }
            if script.contains("const statusFilter = \"active\"")
                && script.contains("document.flattenedProjects")
            {
                return Ok(json!([{ "id": "p1", "name": "active project" }]));
            }
            if script.contains("const statusFilter = \"available\"")
                && script.contains("document.flattenedTasks")
                && script.contains("projectFilter")
            {
                return Ok(json!([{ "id": "a1", "name": "next action" }]));
            }
            if script.contains("const projectFilter =")
                && script.contains("const rootTasks = project.tasks.map")
            {
                return Ok(json!({
                    "id": "p1",
                    "name": "roadmap project",
                    "status": "active",
                    "tasks": []
                }));
            }
            Ok(json!([]))
        })
    }
}

#[tokio::test]
async fn daily_review_renders_expected_sections() {
    let runner = RoutingRunner;
    let prompt = daily_review(&runner).await.expect("daily review");
    assert!(prompt.contains("overdue_tasks_json"));
    assert!(prompt.contains("due_soon_tasks_json"));
    assert!(prompt.contains("flagged_tasks_json"));
    assert!(prompt.contains("overdue task"));
    assert!(prompt.contains("due soon task"));
    assert!(prompt.contains("flagged task"));
}

#[tokio::test]
async fn weekly_review_renders_expected_sections() {
    let runner = RoutingRunner;
    let prompt = weekly_review(&runner).await.expect("weekly review");
    assert!(prompt.contains("active_projects_json"));
    assert!(prompt.contains("available_tasks_json"));
    assert!(prompt.contains("active project"));
    assert!(prompt.contains("next action"));
}

#[tokio::test]
async fn inbox_processing_renders_expected_sections() {
    let runner = RoutingRunner;
    let prompt = inbox_processing(&runner).await.expect("inbox processing");
    assert!(prompt.contains("inbox_items_json"));
    assert!(prompt.contains("inbox item"));
}

#[tokio::test]
async fn project_planning_renders_expected_sections() {
    let runner = RoutingRunner;
    let prompt = project_planning(&runner, "roadmap project")
        .await
        .expect("project planning");
    assert!(prompt.contains("project_details_json"));
    assert!(prompt.contains("project_available_tasks_json"));
    assert!(prompt.contains("roadmap project"));
    assert!(prompt.contains("next action"));
}

#[tokio::test]
async fn project_planning_rejects_empty_project() {
    let runner = RoutingRunner;
    let result = project_planning(&runner, "   ").await;
    assert!(matches!(result, Err(OmniFocusError::Validation(_))));
}
use std::{future::Future, pin::Pin};

use omnifocus_mcp::{
    error::OmniFocusError,
    jxa::JxaRunner,
    prompts::{daily_review, inbox_processing, project_planning, weekly_review},
};
use serde_json::{json, Value};

#[derive(Clone)]
struct MockRunner;

impl JxaRunner for MockRunner {
    fn run_omnijs<'a>(
        &'a self,
        script: &'a str,
    ) -> Pin<Box<dyn Future<Output = omnifocus_mcp::error::Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            if script.contains("const statusFilter = \"due_soon\"") {
                return Ok(json!([{
                    "id": "task-due-soon-1",
                    "name": "due soon task",
                    "note": null,
                    "flagged": false,
                    "completed": false,
                    "projectName": null,
                    "dueDate": null,
                    "deferDate": null,
                    "completionDate": null,
                    "tags": [],
                    "estimatedMinutes": null,
                    "inInbox": true,
                    "hasChildren": false,
                    "sequential": false
                }]));
            }

            if script.contains("const statusFilter = \"overdue\"") {
                return Ok(json!([{
                    "id": "task-overdue-1",
                    "name": "overdue task",
                    "note": null,
                    "flagged": false,
                    "completed": false,
                    "projectName": null,
                    "dueDate": null,
                    "deferDate": null,
                    "completionDate": null,
                    "tags": [],
                    "estimatedMinutes": null,
                    "inInbox": true,
                    "hasChildren": false,
                    "sequential": false
                }]));
            }

            if script.contains("const flaggedFilter = true") {
                return Ok(json!([{
                    "id": "task-flagged-1",
                    "name": "flagged task",
                    "note": null,
                    "flagged": true,
                    "completed": false,
                    "projectName": null,
                    "dueDate": null,
                    "deferDate": null,
                    "completionDate": null,
                    "tags": [],
                    "estimatedMinutes": null,
                    "inInbox": true,
                    "hasChildren": false,
                    "sequential": false
                }]));
            }

            if script.contains("const inboxTasks = inbox.tasks") {
                return Ok(json!([{
                    "id": "task-inbox-1",
                    "name": "inbox item",
                    "note": null,
                    "flagged": false,
                    "completed": false,
                    "projectName": null,
                    "dueDate": null,
                    "deferDate": null,
                    "completionDate": null,
                    "tags": [],
                    "estimatedMinutes": null,
                    "inInbox": true,
                    "hasChildren": false,
                    "sequential": false
                }]));
            }

            if script.contains("const projectFilter =")
                && script.contains("document.flattenedProjects.find")
            {
                return Ok(json!({
                    "id": "project-1",
                    "name": "Launch",
                    "status": "active"
                }));
            }

            if script.contains("const projectFilter =")
                && script.contains("document.flattenedTasks")
            {
                return Ok(json!([{
                    "id": "task-available-1",
                    "name": "available task",
                    "note": null,
                    "flagged": false,
                    "completed": false,
                    "projectName": "Launch",
                    "dueDate": null,
                    "deferDate": null,
                    "completionDate": null,
                    "tags": [],
                    "estimatedMinutes": 15,
                    "inInbox": false,
                    "hasChildren": false,
                    "sequential": false
                }]));
            }

            if script.contains("document.flattenedProjects") {
                return Ok(json!([{
                    "id": "project-1",
                    "name": "Launch",
                    "status": "active",
                    "note": null,
                    "folder": null,
                    "dueDate": null,
                    "deferDate": null,
                    "completionDate": null,
                    "sequential": false,
                    "numberAvailable": 1,
                    "numberRemaining": 2,
                    "flagged": false
                }]));
            }

            Ok(json!([]))
        })
    }
}

#[tokio::test]
async fn prompt_rendering_returns_expected_structure() {
    let runner = MockRunner;

    let daily = daily_review(&runner)
        .await
        .expect("daily_review should render");
    assert!(daily.contains("overdue_tasks_json"));
    assert!(daily.contains("due soon task"));
    assert!(daily.contains("flagged task"));

    let weekly = weekly_review(&runner)
        .await
        .expect("weekly_review should render");
    assert!(weekly.contains("active_projects_json"));
    assert!(weekly.contains("available task"));

    let inbox = inbox_processing(&runner)
        .await
        .expect("inbox_processing should render");
    assert!(inbox.contains("inbox_items_json"));
    assert!(inbox.contains("inbox item"));

    let planning = project_planning(&runner, "Launch")
        .await
        .expect("project_planning should render");
    assert!(planning.contains("project_details_json"));
    assert!(planning.contains("project_available_tasks_json"));
}

#[tokio::test]
async fn project_planning_validates_non_empty_project_argument() {
    let runner = MockRunner;
    let error = project_planning(&runner, "   ")
        .await
        .expect_err("empty project should fail validation");
    assert!(matches!(error, OmniFocusError::Validation(_)));
}
