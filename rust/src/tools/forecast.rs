use serde_json::Value;

use crate::{
    error::{OmniFocusError, Result},
    jxa::JxaRunner,
    tools::task_select::TASK_TABLE_JXA,
};

pub async fn get_forecast<R: JxaRunner>(runner: &R, limit: i32) -> Result<Value> {
    if limit < 1 {
        return Err(OmniFocusError::Validation(
            "limit must be greater than 0.".to_string(),
        ));
    }

    let select_script = format!(
        r#"const now = new Date();
const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
const endOfToday = new Date(startOfToday.getTime() + (24 * 60 * 60 * 1000));
const endOfWeek = new Date(now.getTime() + (7 * 24 * 60 * 60 * 1000));

{TASK_TABLE_JXA}
const completedValues = allTasks.completed();
const flaggedValues = allTasks.flagged();
const dueDates = allTasks.dueDate();
const deferDates = allTasks.deferDate();
const overdue = [];
const dueToday = [];
const flagged = [];
const deferred = [];
const dueThisWeek = [];

const counts = {{
  overdueCount: 0,
  dueTodayCount: 0,
  flaggedCount: 0,
  deferredCount: 0,
  dueThisWeekCount: 0
}};

outlineOrder.forEach(index => {{
  if (completedValues[index]) return;
  const id = taskIds[index];
  const dueDate = dueDates[index];
  const deferDate = deferDates[index];
  if (dueDate !== null && dueDate < startOfToday) {{
    counts.overdueCount += 1;
    if (overdue.length < {limit}) overdue.push(id);
  }}
  if (dueDate !== null && dueDate >= startOfToday && dueDate < endOfToday) {{
    counts.dueTodayCount += 1;
    if (dueToday.length < {limit}) dueToday.push(id);
  }}
  if (flaggedValues[index]) {{
    counts.flaggedCount += 1;
    if (flagged.length < {limit}) flagged.push(id);
  }}
  if (deferDate !== null && deferDate > now) {{
    counts.deferredCount += 1;
    if (deferred.length < {limit}) deferred.push(id);
  }}
  if (dueDate !== null && dueDate >= endOfToday && dueDate < endOfWeek) {{
    counts.dueThisWeekCount += 1;
    if (dueThisWeek.length < {limit}) dueThisWeek.push(id);
  }}
}});

return {{ overdue, dueToday, flagged, deferred, dueThisWeek, counts }};"#
    );
    let selected = runner.run_jxa(&select_script).await?;
    let selected_json = serde_json::to_string(&selected)?;

    let script = format!(
        r#"const selected = {selected_json};
const toTaskSummary = (task) => {{
  return {{
    id: task.id.primaryKey,
    name: task.name,
    note: task.note,
    flagged: task.flagged,
    completed: task.completed,
    dueDate: task.dueDate ? task.dueDate.toISOString() : null,
    deferDate: task.deferDate ? task.deferDate.toISOString() : null,
    completionDate: task.completionDate ? task.completionDate.toISOString() : null,
    taskStatus: (() => {{
      const s = String(task.taskStatus);
      if (s.includes("Available")) return "available";
      if (s.includes("Blocked")) return "blocked";
      if (s.includes("Next")) return "next";
      if (s.includes("DueSoon")) return "due_soon";
      if (s.includes("Overdue")) return "overdue";
      if (s.includes("Completed")) return "completed";
      if (s.includes("Dropped")) return "dropped";
      return "unknown";
    }})(),
    projectName: task.containingProject ? task.containingProject.name : null,
    tags: task.tags.map(tag => tag.name),
    estimatedMinutes: task.estimatedMinutes,
    hasChildren: task.hasChildren
  }};
}};

const toSummaries = ids => (ids || [])
  .map(id => Task.byIdentifier(id))
  .filter(task => task !== null)
  .map(toTaskSummary);

return {{
  overdue: toSummaries(selected.overdue),
  dueToday: toSummaries(selected.dueToday),
  flagged: toSummaries(selected.flagged),
  deferred: toSummaries(selected.deferred),
  dueThisWeek: toSummaries(selected.dueThisWeek),
  counts: selected.counts
}};"#
    );

    runner.run_omnijs(&script).await
}
