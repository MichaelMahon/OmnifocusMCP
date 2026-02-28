use std::sync::{Arc, Mutex};

use omnifocus_mcp::{
    error::OmniFocusError,
    jxa::{escape_for_jxa, JxaRunner},
    tools::{
        projects::{complete_project, create_project},
        tags::create_tag,
        tasks::{
            complete_task, create_task, create_tasks_batch, delete_task, move_task, update_task,
            CreateTaskInput,
        },
    },
};
use serde_json::{json, Value};

#[derive(Clone)]
struct MockRunner {
    payload: Value,
}

impl JxaRunner for MockRunner {
    async fn run_omnijs(&self, _script: &str) -> omnifocus_mcp::error::Result<Value> {
        Ok(self.payload.clone())
    }
}

#[derive(Clone)]
struct RecordingRunner {
    payload: Value,
    scripts: Arc<Mutex<Vec<String>>>,
    error_message: Option<String>,
}

impl JxaRunner for RecordingRunner {
    async fn run_omnijs(&self, script: &str) -> omnifocus_mcp::error::Result<Value> {
        self.scripts
            .lock()
            .expect("scripts lock should succeed")
            .push(script.to_string());
        if let Some(message) = &self.error_message {
            return Err(OmniFocusError::OmniFocus(message.clone()));
        }
        Ok(self.payload.clone())
    }
}

#[tokio::test]
async fn write_task_tools_happy_path() {
    let runner = MockRunner {
        payload: json!({"id": "t1", "name": "task"}),
    };

    let created = create_task(
        &runner,
        "task",
        Some("project"),
        Some("note"),
        Some("2026-03-01T12:00:00Z"),
        Some("2026-02-29T12:00:00Z"),
        Some(true),
        Some(vec!["home".to_string()]),
        Some(30),
    )
    .await
    .expect("create_task should succeed");
    assert_eq!(created["id"], "t1");

    let created_batch = create_tasks_batch(
        &runner,
        vec![CreateTaskInput {
            name: "batch task".to_string(),
            project: Some("project".to_string()),
            note: Some("note".to_string()),
            due_date: None,
            defer_date: None,
            flagged: Some(false),
            tags: Some(vec!["home".to_string()]),
            estimated_minutes: Some(15),
        }],
    )
    .await
    .expect("create_tasks_batch should succeed");
    assert_eq!(created_batch["id"], "t1");

    let completed = complete_task(&runner, "t1")
        .await
        .expect("complete_task should succeed");
    assert_eq!(completed["id"], "t1");

    let updated = update_task(
        &runner,
        "t1",
        Some("updated"),
        Some("updated note"),
        None,
        None,
        Some(false),
        Some(vec!["work".to_string()]),
        Some(10),
    )
    .await
    .expect("update_task should succeed");
    assert_eq!(updated["id"], "t1");

    let deleted = delete_task(&runner, "t1")
        .await
        .expect("delete_task should succeed");
    assert_eq!(deleted["id"], "t1");

    let moved = move_task(&runner, "t1", Some("project"))
        .await
        .expect("move_task should succeed");
    assert_eq!(moved["id"], "t1");
}

#[tokio::test]
async fn write_project_and_tag_tools_happy_path() {
    let runner = MockRunner {
        payload: json!({"id": "p1", "name": "entity"}),
    };

    let created_project = create_project(
        &runner,
        "project",
        Some("folder"),
        Some("note"),
        Some("2026-03-01T00:00:00Z"),
        Some("2026-02-29T00:00:00Z"),
        Some(true),
    )
    .await
    .expect("create_project should succeed");
    assert_eq!(created_project["id"], "p1");

    let completed_project = complete_project(&runner, "p1")
        .await
        .expect("complete_project should succeed");
    assert_eq!(completed_project["id"], "p1");

    let created_tag = create_tag(&runner, "home", Some("parent"))
        .await
        .expect("create_tag should succeed");
    assert_eq!(created_tag["id"], "p1");
}

#[tokio::test]
async fn validation_errors_for_write_tools() {
    let runner = MockRunner { payload: json!({}) };

    assert!(matches!(
        create_task(&runner, "   ", None, None, None, None, None, None, None).await,
        Err(OmniFocusError::Validation(_))
    ));
    assert!(matches!(
        create_task(
            &runner,
            "name",
            Some("   "),
            None,
            None,
            None,
            None,
            None,
            None
        )
        .await,
        Err(OmniFocusError::Validation(_))
    ));
    assert!(matches!(
        create_project(&runner, "   ", None, None, None, None, None).await,
        Err(OmniFocusError::Validation(_))
    ));
    assert!(matches!(
        create_project(&runner, "name", Some("   "), None, None, None, None).await,
        Err(OmniFocusError::Validation(_))
    ));
    assert!(matches!(
        create_tag(&runner, "   ", None).await,
        Err(OmniFocusError::Validation(_))
    ));
    assert!(matches!(
        create_tag(&runner, "name", Some("   ")).await,
        Err(OmniFocusError::Validation(_))
    ));
    assert!(matches!(
        move_task(&runner, "task-id", Some("   ")).await,
        Err(OmniFocusError::Validation(_))
    ));
}

#[tokio::test]
async fn jxa_error_propagates_from_write_tools() {
    let runner = RecordingRunner {
        payload: json!({}),
        scripts: Arc::new(Mutex::new(Vec::new())),
        error_message: Some("Task not found: missing".to_string()),
    };

    let result = delete_task(&runner, "missing").await;
    assert!(matches!(result, Err(OmniFocusError::OmniFocus(_))));
    assert_eq!(
        result.err().map(|e| e.to_string()),
        Some("Task not found: missing".to_string())
    );
}

#[tokio::test]
async fn create_task_script_contains_expected_escaped_values() {
    let scripts = Arc::new(Mutex::new(Vec::new()));
    let runner = RecordingRunner {
        payload: json!({"id": "t1"}),
        scripts: Arc::clone(&scripts),
        error_message: None,
    };

    let name = "he said \"hello\"";
    let project = "proj\\root";
    let note = "line1\nline2";
    let due_date = "2026-03-01T12:00:00Z";

    let result = create_task(
        &runner,
        name,
        Some(project),
        Some(note),
        Some(due_date),
        None,
        Some(true),
        Some(vec!["tag-a".to_string(), "tag-b".to_string()]),
        Some(25),
    )
    .await;
    assert!(result.is_ok());

    let captured = scripts
        .lock()
        .expect("scripts lock should succeed")
        .last()
        .cloned()
        .expect("one script should be captured");

    assert!(captured.contains(&format!("const taskName = {};", escape_for_jxa(name))));
    assert!(captured.contains(&format!("const projectName = {};", escape_for_jxa(project))));
    assert!(captured.contains(&format!("const noteValue = {};", escape_for_jxa(note))));
    assert!(captured.contains(&format!(
        "const dueDateValue = {};",
        escape_for_jxa(due_date)
    )));
}
