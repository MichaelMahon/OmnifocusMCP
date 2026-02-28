use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskResult {
    pub id: String,
    pub name: String,
    pub note: Option<String>,
    pub flagged: bool,
    pub completed: bool,
    pub project: Option<String>,
    pub due_date: Option<String>,
    pub defer_date: Option<String>,
    pub completion_date: Option<String>,
    pub tags: Vec<String>,
    pub estimated_minutes: Option<i32>,
    pub in_inbox: bool,
    pub has_children: bool,
    pub sequential: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectResult {
    pub id: String,
    pub name: String,
    pub status: String,
    pub note: Option<String>,
    pub folder: Option<String>,
    pub due_date: Option<String>,
    pub defer_date: Option<String>,
    pub completion_date: Option<String>,
    pub sequential: bool,
    pub number_available: Option<i32>,
    pub number_remaining: Option<i32>,
    pub flagged: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagResult {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub available_task_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FolderResult {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForecastDay {
    pub date: String,
    pub task_count: i32,
    pub tasks: Vec<TaskResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PerspectiveResult {
    pub id: String,
    pub name: String,
}
