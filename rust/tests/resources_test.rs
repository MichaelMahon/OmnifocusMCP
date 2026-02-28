use std::{future::Future, pin::Pin};

use omnifocus_mcp::{
    jxa::JxaRunner,
    resources::{inbox_resource, projects_resource, today_resource},
};
use serde_json::{json, Value};

#[derive(Clone)]
struct ResourceRunner;

impl JxaRunner for ResourceRunner {
    fn run_omnijs<'a>(
        &'a self,
        script: &'a str,
    ) -> Pin<Box<dyn Future<Output = omnifocus_mcp::error::Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            if script.contains("const inboxTasks = inbox.tasks") {
                return Ok(json!([{
                    "id": "t-inbox-1",
                    "name": "inbox task",
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
                    "overdue": [{"id": "t-overdue-1", "name": "overdue item"}],
                    "dueToday": [{"id": "t-today-1", "name": "today item"}],
                    "flagged": [{"id": "t-flagged-1", "name": "flagged item"}]
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
                    "numberRemaining": 2,
                    "flagged": false
                }]));
            }

            Ok(Value::Null)
        })
    }
}

#[tokio::test]
async fn resources_render_expected_content_keywords() {
    let runner = ResourceRunner;

    let inbox = inbox_resource(&runner).await.expect("inbox resource should render");
    assert!(inbox.contains("inbox task"));
    assert!(inbox.contains("t-inbox-1"));

    let today = today_resource(&runner).await.expect("today resource should render");
    assert!(today.contains("overdue item"));
    assert!(today.contains("flagged item"));

    let projects = projects_resource(&runner)
        .await
        .expect("projects resource should render");
    assert!(projects.contains("active project"));
    assert!(projects.contains("project-1"));
}
use std::{future::Future, pin::Pin};

use omnifocus_mcp::{
    jxa::JxaRunner,
    resources::{inbox_resource, projects_resource, today_resource},
};
use serde_json::{Value, json};

#[derive(Clone)]
struct MockRunner;

impl JxaRunner for MockRunner {
    fn run_omnijs<'a>(
        &'a self,
        script: &'a str,
    ) -> Pin<Box<dyn Future<Output = omnifocus_mcp::error::Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            if script.contains("const tasks = inbox") {
                return Ok(json!([
                    {
                        "id": "t1",
                        "name": "inbox task",
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
                    }
                ]));
            }

            if script.contains("const sections = {") {
                return Ok(json!({
                    "overdue": [],
                    "dueToday": [],
                    "flagged": []
                }));
            }

            if script.contains("flattenedProjects") {
                return Ok(json!([
                    {
                        "id": "p1",
                        "name": "project one"
                    }
                ]));
            }

            Ok(json!([]))
        })
    }
}

#[tokio::test]
async fn resources_return_expected_keywords() {
    let runner = MockRunner;

    let inbox = inbox_resource(&runner).await.expect("inbox resource should render");
    assert!(inbox.contains("inbox task"));

    let today = today_resource(&runner).await.expect("today resource should render");
    assert!(today.contains("overdue"));
    assert!(today.contains("dueToday"));

    let projects = projects_resource(&runner)
        .await
        .expect("projects resource should render");
    assert!(projects.contains("project one"));
}
