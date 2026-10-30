//! Two-step task reads. Touching `flattenedTasks` in OmniJS materializes every task (about 8 s
//! for 1,094 tasks on a 2012 Mac mini), while a plain JXA bulk read such as
//! `defaultDocument.flattenedTasks.dueDate()` returns one array per Apple Event in about 0.1 s.
//! So tools first select task ids with bulk JXA reads, then build their results in OmniJS for
//! just those ids with `Task.byIdentifier`, keeping the existing result code unchanged.

use serde_json::Value;

/// JXA: `doc`, `allTasks`, `taskIds`, and `outlineOrder`, the indices of `taskIds` in the order
/// OmniJS `flattenedTasks` uses: inbox tasks first, then each project's tasks in
/// `flattenedProjects` order. JXA's own order differs, and unsorted results keep the OmniJS order.
pub const TASK_TABLE_JXA: &str = r#"const doc = Application('OmniFocus').defaultDocument;
const allTasks = doc.flattenedTasks;
const taskIds = allTasks.id();
const outlineOrder = (() => {
  const inInbox = allTasks.inInbox();
  const projectIds = allTasks.containingProject.id();
  const projectOrder = doc.flattenedProjects.id();
  const projectRank = {};
  projectOrder.forEach((id, index) => { projectRank[id] = index; });
  const rank = index => {
    if (inInbox[index]) return -1;
    const projectId = projectIds[index];
    return projectId !== null && projectRank[projectId] !== undefined ? projectRank[projectId] : projectOrder.length;
  };
  return taskIds.map((_, index) => index).sort((a, b) => rank(a) - rank(b) || a - b);
})();"#;

/// JXA filter and sort shared by list_tasks and search_tasks. Expects the tool's header
/// constants (filters, parsed dates, `now`, `soon`, `sortBy`, `sortOrder`, `queryFilter`,
/// `includeCompletedForDateFilter`) and `TASK_TABLE_JXA`; leaves the ordered indices in `sorted`.
/// Only the arrays a request needs are read.
const FILTER_AND_SORT_JXA: &str = r#"const readIf = (needed, read) => (needed ? read() : null);
const completedValues = allTasks.completed();
const flaggedValues = allTasks.flagged();
const dueDates = allTasks.dueDate();
const deferDates = allTasks.deferDate();
const names = readIf(queryFilter !== null || sortBy === "name", () => allTasks.name());
const notes = readIf(queryFilter !== null, () => allTasks.note());
const projectNames = readIf(projectFilter !== null || sortBy === "project", () => allTasks.containingProject.name());
const projectStatuses = readIf(statusFilter === "on_hold", () => allTasks.containingProject.status());
const tagNameLists = readIf(tagNames !== null && tagNames.length > 0, () => allTasks.tags.name());
const completionDates = readIf(
  completedBefore !== null || completedAfter !== null || sortBy === "completionDate",
  () => allTasks.completionDate()
);
const addedDates = readIf(
  addedBefore !== null || addedAfter !== null || sortBy === "addedDate" || sortBy === "added",
  () => allTasks.creationDate()
);
const modifiedDates = readIf(
  changedBefore !== null || changedAfter !== null || sortBy === "changedDate" || sortBy === "modified",
  () => allTasks.modificationDate()
);
// Older OmniFocus versions have no planned date; then planned filters are skipped, as before.
const plannedDates = readIf(
  plannedBefore !== null || plannedAfter !== null || sortBy === "plannedDate" || sortBy === "planned",
  () => {
    try {
      return allTasks.plannedDate();
    } catch (e) {
      return null;
    }
  }
);
const estimatedMinutesValues = readIf(
  maxEstimatedMinutes !== null || sortBy === "estimatedMinutes",
  () => allTasks.estimatedMinutes()
);

const matches = index => {
  if (queryFilter !== null) {
    const name = (names[index] || "").toLowerCase();
    const note = (notes[index] || "").toLowerCase();
    if (!(name.includes(queryFilter) || note.includes(queryFilter))) return false;
  }
  if (projectFilter !== null && projectNames[index] !== projectFilter) return false;
  if (tagNameLists !== null) {
    const taskTags = tagNameLists[index];
    const tagMatches = tagFilterMode === "all"
      ? tagNames.every(tn => taskTags.includes(tn))
      : taskTags.some(tn => tagNames.includes(tn));
    if (!tagMatches) return false;
  }
  if (flaggedFilter !== null && flaggedValues[index] !== flaggedFilter) return false;

  const completed = completedValues[index];
  const dueDate = dueDates[index];
  const deferDate = deferDates[index];
  let statusMatches = false;
  if (statusFilter === "all") {
    statusMatches = true;
  } else if (statusFilter === "completed") {
    statusMatches = completed;
  } else if (completed) {
    statusMatches = includeCompletedForDateFilter;
  } else if (statusFilter === "available") {
    statusMatches = true;
  } else if (statusFilter === "overdue") {
    statusMatches = dueDate !== null && dueDate < now;
  } else if (statusFilter === "due_soon") {
    statusMatches = dueDate !== null && dueDate >= now && dueDate <= soon;
  } else if (statusFilter === "on_hold") {
    const projectStatus = projectStatuses[index] === null ? "" : String(projectStatuses[index]).toLowerCase();
    statusMatches = projectStatus.includes("onhold") || projectStatus.includes("on hold") || projectStatus.includes("on_hold");
  }
  if (!statusMatches) return false;

  if (dueBefore !== null && !(dueDate !== null && dueDate < dueBefore)) return false;
  if (dueAfter !== null && !(dueDate !== null && dueDate > dueAfter)) return false;
  if (deferBefore !== null && !(deferDate !== null && deferDate < deferBefore)) return false;
  if (deferAfter !== null && !(deferDate !== null && deferDate > deferAfter)) return false;
  if (completedBefore !== null && !(completionDates[index] !== null && completionDates[index] < completedBefore)) return false;
  if (completedAfter !== null && !(completionDates[index] !== null && completionDates[index] > completedAfter)) return false;
  if (addedBefore !== null && !(addedDates[index] !== null && addedDates[index] <= addedBefore)) return false;
  if (addedAfter !== null && !(addedDates[index] !== null && addedDates[index] >= addedAfter)) return false;
  if (changedBefore !== null && !(modifiedDates[index] !== null && modifiedDates[index] <= changedBefore)) return false;
  if (changedAfter !== null && !(modifiedDates[index] !== null && modifiedDates[index] >= changedAfter)) return false;
  if (plannedDates !== null) {
    const plannedDate = plannedDates[index];
    if (plannedBefore !== null && !(plannedDate !== null && plannedDate < plannedBefore)) return false;
    if (plannedAfter !== null && !(plannedDate !== null && plannedDate > plannedAfter)) return false;
  }
  if (maxEstimatedMinutes !== null && !(estimatedMinutesValues[index] !== null && estimatedMinutesValues[index] <= maxEstimatedMinutes)) return false;
  return true;
};

const compareValues = (aValue, bValue, isString = false) => {
  if (aValue === null && bValue === null) return 0;
  if (aValue === null) return 1;
  if (bValue === null) return -1;
  let left = aValue;
  let right = bValue;
  if (isString) {
    left = String(aValue).toLowerCase();
    right = String(bValue).toLowerCase();
  }
  if (left < right) return sortOrder === "asc" ? -1 : 1;
  if (left > right) return sortOrder === "asc" ? 1 : -1;
  return 0;
};

const sortValue = index => {
  if (sortBy === "dueDate") return dueDates[index];
  if (sortBy === "deferDate") return deferDates[index];
  if (sortBy === "name") return names[index];
  if (sortBy === "completionDate") return completionDates[index];
  if (sortBy === "estimatedMinutes") return estimatedMinutesValues[index];
  if (sortBy === "project") return projectNames[index];
  if (sortBy === "flagged") return flaggedValues[index];
  if (sortBy === "addedDate" || sortBy === "added") return addedDates[index];
  if (sortBy === "changedDate" || sortBy === "modified") return modifiedDates[index];
  if (sortBy === "plannedDate" || sortBy === "planned") return plannedDates === null ? null : plannedDates[index];
  return null;
};
const sortIsString = sortBy === "name" || sortBy === "project";

const filtered = outlineOrder.filter(matches);
const sorted = sortBy === null
  ? filtered
  : filtered.slice().sort((a, b) => compareValues(sortValue(a), sortValue(b), sortIsString));"#;

/// OmniJS: `getPlannedDate(task)`, guarded for OmniFocus versions without planned dates.
/// Probes one of the selected `tasks`, never `flattenedTasks`.
const PLANNED_DATE_OMNIJS: &str = r#"const supportsPlannedDate = (() => {
  try {
    const sampleTask = tasks[0];
    if (!sampleTask) return true;
    void sampleTask.plannedDate;
    return true;
  } catch (e) {
    return false;
  }
})();
const getPlannedDate = (task) => {
  if (!supportsPlannedDate) return null;
  try {
    const value = task.plannedDate;
    return value === undefined ? null : value;
  } catch (e) {
    return null;
  }
};"#;

/// OmniJS result shape for list_tasks and search_tasks.
pub const TASK_RESULT_OMNIJS: &str = r#"return tasks.map(task => {
  const tags = task.tags.map(taskTag => taskTag.name);
  const plannedDate = getPlannedDate(task);
  return {
    id: task.id.primaryKey,
    name: task.name,
    note: task.note,
    flagged: task.flagged,
    dueDate: task.dueDate ? task.dueDate.toISOString() : null,
    addedDate: task.added ? task.added.toISOString() : null,
    changedDate: task.modified ? task.modified.toISOString() : null,
    deferDate: task.deferDate ? task.deferDate.toISOString() : null,
    completed: task.completed,
    completionDate: task.completionDate ? task.completionDate.toISOString() : null,
    plannedDate: plannedDate ? plannedDate.toISOString() : null,
    projectName: task.containingProject ? task.containingProject.name : null,
    tags: tags,
    estimatedMinutes: task.estimatedMinutes,
    hasChildren: task.hasChildren,
    taskStatus: (() => {
      const s = String(task.taskStatus);
      if (s.includes("Available")) return "available";
      if (s.includes("Blocked")) return "blocked";
      if (s.includes("Next")) return "next";
      if (s.includes("DueSoon")) return "due_soon";
      if (s.includes("Overdue")) return "overdue";
      if (s.includes("Completed")) return "completed";
      if (s.includes("Dropped")) return "dropped";
      return "unknown";
    })()
  };
});"#;

/// JXA that returns up to `limit` matching task ids, in result order, for a list or search header.
pub fn select_task_ids_script(header: &str, limit: i32) -> String {
    format!(
        "{header}\n{TASK_TABLE_JXA}\n{FILTER_AND_SORT_JXA}\nreturn sorted.slice(0, {limit}).map(index => taskIds[index]);"
    )
}

/// OmniJS that resolves the ids a JXA step selected into `tasks` (in that order, skipping any
/// deleted in between) and then runs `body`.
pub fn hydrate_tasks_script(selected_ids: &Value, body: &str) -> String {
    let ids = serde_json::to_string(selected_ids).unwrap_or_else(|_| "[]".to_string());
    format!(
        "const selectedIds = {ids};\nconst tasks = selectedIds.map(id => Task.byIdentifier(id)).filter(task => task !== null);\n{PLANNED_DATE_OMNIJS}\n{body}"
    )
}
