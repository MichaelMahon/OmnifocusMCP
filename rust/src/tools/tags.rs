use serde_json::Value;

use crate::{
    error::{OmniFocusError, Result},
    jxa::{escape_for_jxa, JxaRunner},
};

pub async fn list_tags<R: JxaRunner>(runner: &R, limit: i32) -> Result<Value> {
    if limit < 1 {
        return Err(OmniFocusError::Validation(
            "limit must be greater than 0.".to_string(),
        ));
    }

    let script = format!(
        r#"const tagCounts = new Map();
document.flattenedTasks.forEach(task => {{
  if (task.completed) return;
  task.tags.forEach(tag => {{
    const tagId = tag.id.primaryKey;
    const current = tagCounts.get(tagId) || 0;
    tagCounts.set(tagId, current + 1);
  }});
}});

const normalizeTagStatus = (tag) => {{
  const rawStatus = String(tag.status || "").toLowerCase().trim();
  if (rawStatus === "") return "active";
  return rawStatus.replace(/\s+/g, "_");
}};

const tags = document.flattenedTags.slice(0, {limit});
return tags.map(tag => {{
  return {{
    id: tag.id.primaryKey,
    name: tag.name,
    parent: tag.parent ? tag.parent.name : null,
    availableTaskCount: tagCounts.get(tag.id.primaryKey) || 0,
    status: normalizeTagStatus(tag)
  }};
}});"#
    );

    runner.run_omnijs(&script).await
}

pub async fn create_tag<R: JxaRunner>(
    runner: &R,
    name: &str,
    parent: Option<&str>,
) -> Result<Value> {
    if name.trim().is_empty() {
        return Err(OmniFocusError::Validation(
            "name must not be empty.".to_string(),
        ));
    }
    if let Some(parent_name) = parent {
        if parent_name.trim().is_empty() {
            return Err(OmniFocusError::Validation(
                "parent must not be empty when provided.".to_string(),
            ));
        }
    }

    let tag_name = escape_for_jxa(name.trim());
    let parent_name = parent
        .map(|value| escape_for_jxa(value.trim()))
        .unwrap_or_else(|| "null".to_string());

    let script = format!(
        r#"const tagName = {tag_name};
const parentName = {parent_name};

const tag = (() => {{
  if (parentName === null) return new Tag(tagName);
  const parentTag = document.flattenedTags.byName(parentName);
  if (!parentTag) {{
    throw new Error(`Tag not found: ${{parentName}}`);
  }}
  return new Tag(tagName, parentTag.ending);
}})();

return {{
  id: tag.id.primaryKey
}};"#
    );

    runner.run_omnijs(&script).await
}
