use serde_json::Value;

use crate::{
    error::{OmniFocusError, Result},
    jxa::{escape_for_jxa, JxaRunner},
};

pub async fn list_perspectives<R: JxaRunner>(runner: &R, limit: i32) -> Result<Value> {
    if limit < 1 {
        return Err(OmniFocusError::Validation(
            "limit must be greater than 0.".to_string(),
        ));
    }

    let script = format!(
        r#"const getPerspectiveId = (perspective) => {{
  if (perspective.id && perspective.id.primaryKey) return perspective.id.primaryKey;
  if (perspective.identifier) return String(perspective.identifier);
  if (perspective.name) return String(perspective.name);
  return "unknown";
}};

const normalizePerspective = (perspective) => {{
  return {{
    id: getPerspectiveId(perspective),
    name: perspective.name || ""
  }};
}};

const collected = [];

if (typeof Perspective !== "undefined" && Perspective.BuiltIn && Perspective.BuiltIn.all) {{
  Perspective.BuiltIn.all.forEach(perspective => {{
    collected.push(normalizePerspective(perspective));
  }});
}}

if (typeof Perspective !== "undefined" && Perspective.Custom && Perspective.Custom.all) {{
  Perspective.Custom.all.forEach(perspective => {{
    collected.push(normalizePerspective(perspective));
  }});
}}

if (document.perspectives) {{
  document.perspectives.forEach(perspective => {{
    collected.push(normalizePerspective(perspective));
  }});
}}

const unique = [];
const seen = new Set();
collected.forEach(perspective => {{
  if (seen.has(perspective.id)) return;
  seen.add(perspective.id);
  unique.push(perspective);
}});

return unique.slice(0, {limit});"#
    );

    runner.run_omnijs(&script).await
}

pub async fn get_perspective_tasks<R: JxaRunner>(
    runner: &R,
    name: &str,
    limit: i32,
    include_metadata: bool,
) -> Result<Value> {
    if name.trim().is_empty() {
        return Err(OmniFocusError::Validation(
            "perspectiveName must not be empty.".to_string(),
        ));
    }
    if limit < 1 {
        return Err(OmniFocusError::Validation(
            "limit must be greater than 0.".to_string(),
        ));
    }

    let name_json = escape_for_jxa(name.trim());

    // The OmniFocus scripting bridge does not populate win.content.trees
    // when accessed programmatically, so we cannot read perspective views
    // through the UI tree. Instead, we:
    //   - Handle built-in perspectives with dedicated queries
    //   - Handle custom perspectives by reading archivedFilterRules and
    //     evaluating them as filters against flattenedTasks
    let script = format!(
        r#"var perspectiveName = {name_json};

// --- helpers ---
function getTaskStatus(task) {{
  var s = String(task.taskStatus);
  if (s.indexOf("Available") >= 0) return "available";
  if (s.indexOf("Blocked") >= 0) return "blocked";
  if (s.indexOf("Next") >= 0) return "next";
  if (s.indexOf("DueSoon") >= 0) return "due_soon";
  if (s.indexOf("Overdue") >= 0) return "overdue";
  if (s.indexOf("Completed") >= 0) return "completed";
  if (s.indexOf("Dropped") >= 0) return "dropped";
  return "unknown";
}}

function isAvailable(task) {{
  var s = getTaskStatus(task);
  return s === "available" || s === "due_soon" || s === "overdue" || s === "next";
}}

function isRemaining(task) {{
  var s = getTaskStatus(task);
  return s !== "completed" && s !== "dropped";
}}

function buildEntry(task, includeMeta) {{
  var entry = {{
    id: task.id ? task.id.primaryKey : null,
    name: task.name
  }};
  if (includeMeta) {{
    entry.dueDate = task.dueDate ? task.dueDate.toISOString() : null;
    entry.deferDate = task.deferDate ? task.deferDate.toISOString() : null;
    entry.flagged = task.flagged;
    entry.tags = task.tags ? task.tags.map(function(t) {{ return t.name; }}) : [];
    entry.projectName = task.containingProject ? task.containingProject.name : null;
    entry.note = task.note || null;
    entry.taskStatus = getTaskStatus(task);
  }}
  return entry;
}}

// --- built-in perspective handlers ---
var builtinHandlers = {{
  "Inbox": function(lim, inclMeta) {{
    var tasks = [];
    inbox.forEach(function(task) {{
      if (tasks.length >= lim) return;
      if (!task.completed) tasks.push(buildEntry(task, inclMeta));
    }});
    return tasks;
  }},
  "Flagged": function(lim, inclMeta) {{
    var tasks = [];
    flattenedTasks.forEach(function(task) {{
      if (tasks.length >= lim) return;
      if (task.flagged && isAvailable(task)) {{
        tasks.push(buildEntry(task, inclMeta));
      }}
    }});
    return tasks;
  }},
  "Projects": function(lim, inclMeta) {{
    var entries = [];
    flattenedProjects.forEach(function(proj) {{
      if (entries.length >= lim) return;
      var s = String(proj.status);
      if (s.indexOf("Dropped") >= 0 || s.indexOf("Done") >= 0) return;
      var entry = {{
        id: proj.id ? proj.id.primaryKey : null,
        name: proj.name
      }};
      if (inclMeta) {{
        entry.dueDate = proj.dueDate ? proj.dueDate.toISOString() : null;
        entry.deferDate = proj.deferDate ? proj.deferDate.toISOString() : null;
        entry.flagged = proj.flagged;
        entry.tags = proj.tags ? proj.tags.map(function(t) {{ return t.name; }}) : [];
        entry.note = proj.note || null;
        entry.taskStatus = (function() {{
          var ps = String(proj.status);
          if (ps.indexOf("Active") >= 0) return "active";
          if (ps.indexOf("OnHold") >= 0) return "on_hold";
          return "unknown";
        }})();
      }}
      entries.push(entry);
    }});
    return entries;
  }},
  "Tags": function(lim, inclMeta) {{
    var entries = [];
    flattenedTags.forEach(function(tag) {{
      if (entries.length >= lim) return;
      entries.push({{
        id: tag.id ? tag.id.primaryKey : null,
        name: tag.name
      }});
    }});
    return entries;
  }},
  "Review": function(lim, inclMeta) {{
    var entries = [];
    flattenedProjects.forEach(function(proj) {{
      if (entries.length >= lim) return;
      var s = String(proj.status);
      if (s.indexOf("Dropped") >= 0 || s.indexOf("Done") >= 0) return;
      if (proj.nextReviewDate && proj.nextReviewDate <= new Date()) {{
        var entry = {{
          id: proj.id ? proj.id.primaryKey : null,
          name: proj.name
        }};
        if (inclMeta) {{
          entry.dueDate = proj.dueDate ? proj.dueDate.toISOString() : null;
          entry.nextReviewDate = proj.nextReviewDate ? proj.nextReviewDate.toISOString() : null;
          entry.flagged = proj.flagged;
          entry.tags = proj.tags ? proj.tags.map(function(t) {{ return t.name; }}) : [];
          entry.note = proj.note || null;
        }}
        entries.push(entry);
      }}
    }});
    return entries;
  }},
  "Forecast": function(lim, inclMeta) {{
    var tasks = [];
    flattenedTasks.forEach(function(task) {{
      if (tasks.length >= lim) return;
      if (task.dueDate && isRemaining(task)) {{
        tasks.push(buildEntry(task, inclMeta));
      }}
    }});
    tasks.sort(function(a, b) {{
      if (!a.dueDate) return 1;
      if (!b.dueDate) return -1;
      return a.dueDate < b.dueDate ? -1 : a.dueDate > b.dueDate ? 1 : 0;
    }});
    return tasks.slice(0, lim);
  }}
}};

// --- custom perspective rule engine ---
function taskHasAnyTag(task, tagIds) {{
  var taskTags = task.tags;
  for (var i = 0; i < taskTags.length; i++) {{
    for (var j = 0; j < tagIds.length; j++) {{
      if (taskTags[i].id.primaryKey === tagIds[j]) return true;
    }}
  }}
  return false;
}}

function taskWithinFocus(task, focusIds) {{
  var proj = task.containingProject;
  while (proj) {{
    if (proj.id) {{
      var projId = proj.id.primaryKey;
      for (var i = 0; i < focusIds.length; i++) {{
        if (projId === focusIds[i]) return true;
      }}
    }}
    var folder = proj.parentFolder;
    while (folder) {{
      if (folder.id) {{
        var folderId = folder.id.primaryKey;
        for (var i = 0; i < focusIds.length; i++) {{
          if (folderId === focusIds[i]) return true;
        }}
      }}
      folder = folder.parentFolder;
    }}
    break;
  }}
  return false;
}}

function evaluateRule(task, rule) {{
  if (rule.disabledRule) return true;

  if (rule.aggregateRules) {{
    var subResults = rule.aggregateRules.map(function(sr) {{ return evaluateRule(task, sr); }});
    if (rule.aggregateType === "any") return subResults.some(function(r) {{ return r; }});
    if (rule.aggregateType === "none") return subResults.every(function(r) {{ return !r; }});
    return subResults.every(function(r) {{ return r; }});
  }}

  if (rule.actionAvailability === "available") return isAvailable(task);
  if (rule.actionAvailability === "remaining") return isRemaining(task);
  if (rule.actionAvailability === "completed") return getTaskStatus(task) === "completed";
  if (rule.actionStatus === "flagged") return task.flagged;
  if (rule.actionStatus === "due") return task.dueDate !== null;
  if (rule.actionHasAnyOfTags) return taskHasAnyTag(task, rule.actionHasAnyOfTags);
  if (rule.actionWithinFocus) return taskWithinFocus(task, rule.actionWithinFocus);
  if (rule.actionDateIsToday && rule.actionDateField) {{
    var today = new Date();
    today.setHours(0,0,0,0);
    var tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);
    var dateVal = rule.actionDateField === "due" ? task.dueDate : task.deferDate;
    if (!dateVal) return false;
    return dateVal >= today && dateVal < tomorrow;
  }}

  return true;
}}

function evaluateCustomPerspective(perspective, lim, inclMeta) {{
  var rules = perspective.archivedFilterRules;
  var topAgg = perspective.archivedTopLevelFilterAggregation || "all";

  if (!rules || rules.length === 0) {{
    var tasks = [];
    flattenedTasks.forEach(function(task) {{
      if (tasks.length >= lim) return;
      if (isRemaining(task)) tasks.push(buildEntry(task, inclMeta));
    }});
    return tasks;
  }}

  var tasks = [];
  flattenedTasks.forEach(function(task) {{
    if (tasks.length >= lim) return;
    var results = rules.map(function(r) {{ return evaluateRule(task, r); }});
    var pass = (topAgg === "any")
      ? results.some(function(r) {{ return r; }})
      : results.every(function(r) {{ return r; }});
    if (pass) tasks.push(buildEntry(task, inclMeta));
  }});
  return tasks;
}}

// --- main logic ---
var isBuiltin = false;
var builtinName = null;
if (typeof Perspective !== "undefined" && Perspective.BuiltIn && Perspective.BuiltIn.all) {{
  Perspective.BuiltIn.all.forEach(function(p) {{
    if (p.name === perspectiveName) {{
      isBuiltin = true;
      builtinName = p.name;
    }}
  }});
}}

var customPerspective = null;
if (!isBuiltin && typeof Perspective !== "undefined" && Perspective.Custom && Perspective.Custom.byName) {{
  customPerspective = Perspective.Custom.byName(perspectiveName);
}}

if (!isBuiltin && !customPerspective) {{
  return {{ error: "Perspective not found: " + perspectiveName }};
}}

var tasks;
if (isBuiltin && builtinHandlers[builtinName]) {{
  tasks = builtinHandlers[builtinName]({limit}, {include_metadata});
}} else if (customPerspective) {{
  tasks = evaluateCustomPerspective(customPerspective, {limit}, {include_metadata});
}} else {{
  return {{ error: "Built-in perspective '" + builtinName + "' is not queryable via this tool. Use dedicated tools (get_inbox, get_forecast, search_tasks) instead." }};
}}

return {{
  perspectiveName: perspectiveName,
  count: tasks.length,
  tasks: tasks
}};"#
    );

    runner.run_omnijs(&script).await
}
