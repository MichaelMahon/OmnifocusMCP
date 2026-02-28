use std::{future::Future, pin::Pin};

use omnifocus_mcp::{
    jxa::JxaRunner,
    resources::{inbox_resource, projects_resource, today_resource},
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
            if script.contains("const tasks = inbox") {
                return Ok(json!([{
                    "id": "task-1",
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
            if script.contains("const now = new Date();")
                && script.contains("const toTaskSummary = (task) =>")
            {
                return Ok(json!({
                    "overdue": [{"id": "task-2", "name": "overdue item"}],
                    "dueToday": [],
                    "flagged": []
                }));
            }
            if script.contains("const projectCounts = new Map();") {
                return Ok(json!([{
                    "id": "project-1",
                    "name": "active project",
                    "status": "active",
                    "folderName": null,
                    "taskCount": 1,
                    "remainingTaskCount": 1,
                    "deferDate": null,
                    "dueDate": null,
                    "note": "",
                    "sequential": false,
                    "reviewInterval": null
                }]));
            }
            Ok(Value::Null)
        })
    }
}

#[tokio::test]
async fn resources_return_json_text_with_expected_keywords() {
    let runner = MockRunner;

    let inbox = inbox_resource(&runner)
        .await
        .expect("inbox resource should render");
    assert!(inbox.contains("inbox item"));
    assert!(inbox.contains("task-1"));

    let today = today_resource(&runner)
        .await
        .expect("today resource should render");
    assert!(today.contains("overdue"));
    assert!(today.contains("overdue item"));

    let projects = projects_resource(&runner)
        .await
        .expect("projects resource should render");
    assert!(projects.contains("active project"));
    assert!(projects.contains("\"status\":\"active\""));
}
use std::{future::Future, pin::Pin};

use omnifocus_mcp::{
    jxa::JxaRunner,
    resources::{inbox_resource, projects_resource, today_resource},
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
            if script.contains("const tasks = inbox") {
                return Ok(json!([{
                    "id": "task-1",
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

            if script.contains("const sections = {") {
                return Ok(json!({
                    "overdue": [{"id": "task-2", "name": "overdue item"}],
                    "dueToday": [{"id": "task-3", "name": "today item"}],
                    "flagged": [{"id": "task-4", "name": "flagged item"}]
                }));
            }

            if script.contains("document.flattenedProjects") {
                return Ok(json!([{
                    "id": "project-1",
                    "name": "active project",
                    "status": "active",
                    "note": null,
                    "folder": null,
                    "dueDate": null,
                    "deferDate": null,
                    "completionDate": null,
                    "sequential": false,
                    "numberAvailable": 1,
                    "numberRemaining": 1,
                    "flagged": false
                }]));
            }

            Ok(Value::Null)
        })
    }
}

#[tokio::test]
async fn resources_return_json_text_with_expected_keywords() {
    let runner = MockRunner;

    let inbox = inbox_resource(&runner)
        .await
        .expect("inbox resource should render");
    assert!(inbox.contains("inbox item"));
    assert!(inbox.contains("task-1"));

    let today = today_resource(&runner)
        .await
        .expect("today resource should render");
    assert!(today.contains("overdue"));
    assert!(today.contains("overdue item"));

    let projects = projects_resource(&runner)
        .await
        .expect("projects resource should render");
    assert!(projects.contains("active project"));
    assert!(projects.contains("\"status\":\"active\""));
}
