use std::{future::Future, pin::Pin};

use omnifocus_mcp::{
    error::OmniFocusError,
    jxa::JxaRunner,
    prompts::{daily_review, inbox_processing, project_planning, weekly_review},
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
            if script.contains("flattenedProjects") {
                return Ok(json!({
                    "id": "p1",
                    "name": "project one"
                }));
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
    assert!(daily.contains("due_soon_tasks_json"));
    assert!(daily.contains("flagged_tasks_json"));

    let weekly = weekly_review(&runner)
        .await
        .expect("weekly_review should render");
    assert!(weekly.contains("active_projects_json"));
    assert!(weekly.contains("available_tasks_json"));

    let inbox = inbox_processing(&runner)
        .await
        .expect("inbox_processing should render");
    assert!(inbox.contains("inbox_items_json"));

    let planning = project_planning(&runner, "project one")
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
