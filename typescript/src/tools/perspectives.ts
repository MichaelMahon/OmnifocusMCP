import { z } from "zod";

import { escapeForJxa, runOmniJs } from "../jxa.js";
import { errorResult, normalizeError, textResult, type Server } from "../types.js";

export function register(server: Server): void {
  server.tool(
    "list_perspectives",
    "list available perspectives including built-in and custom perspectives.",
    { limit: z.number().int().min(1).default(100) },
    async ({ limit }) => {
      try {
        const script = `
const getPerspectiveId = perspective => {
  if (perspective.id && perspective.id.primaryKey) return perspective.id.primaryKey;
  if (perspective.identifier) return String(perspective.identifier);
  if (perspective.name) return String(perspective.name);
  return "unknown";
};
const normalizePerspective = perspective => ({ id: getPerspectiveId(perspective), name: perspective.name || "" });
const collected = [];
if (typeof Perspective !== "undefined" && Perspective.BuiltIn && Perspective.BuiltIn.all) {
  Perspective.BuiltIn.all.forEach(perspective => collected.push(normalizePerspective(perspective)));
}
if (typeof Perspective !== "undefined" && Perspective.Custom && Perspective.Custom.all) {
  Perspective.Custom.all.forEach(perspective => collected.push(normalizePerspective(perspective)));
}
if (document.perspectives) {
  document.perspectives.forEach(perspective => collected.push(normalizePerspective(perspective)));
}
const unique = [];
const seen = new Set();
collected.forEach(perspective => {
  if (seen.has(perspective.id)) return;
  seen.add(perspective.id);
  unique.push(perspective);
});
return unique.slice(0, ${limit});
`.trim();
        return textResult(await runOmniJs(script));
      } catch (error: unknown) {
        return errorResult(normalizeError(error));
      }
    }
  );

  server.tool(
    "get_perspective_tasks",
    "get tasks visible in a named OmniFocus perspective (custom or built-in). switches the front document window to the perspective, reads the visible tasks, then restores the previous perspective.",
    {
      perspectiveName: z.string().min(1).describe("name of the perspective to query"),
      limit: z.number().int().min(1).default(100),
      includeMetadata: z.boolean().default(true),
    },
    async ({ perspectiveName, limit, includeMetadata }) => {
      try {
        const nameJson = escapeForJxa(perspectiveName);
        const script = `
var perspective = null;
if (typeof Perspective !== "undefined" && Perspective.Custom && Perspective.Custom.byName) {
  perspective = Perspective.Custom.byName(${nameJson});
}
if (!perspective && typeof Perspective !== "undefined" && Perspective.BuiltIn && Perspective.BuiltIn.all) {
  Perspective.BuiltIn.all.forEach(function(p) {
    if (!perspective && p.name === ${nameJson}) perspective = p;
  });
}
if (!perspective && document.perspectives) {
  document.perspectives.forEach(function(p) {
    if (!perspective && p.name === ${nameJson}) perspective = p;
  });
}
if (!perspective) {
  return { error: "Perspective not found: " + ${nameJson} };
}

var win = document.windows[0];
if (!win) {
  return { error: "No open OmniFocus document window." };
}

var previousPerspective = win.perspective;
win.perspective = perspective;

var trees = win.content.trees;
var tasks = [];
var count = 0;

for (var i = 0; i < trees.length; i++) {
  if (count >= ${limit}) break;
  var node = trees[i];
  var task = node.value;
  if (!task || typeof task.name === "undefined") continue;
  var entry = {
    id: task.id ? task.id.primaryKey : null,
    name: task.name
  };
  if (${includeMetadata}) {
    entry.dueDate = task.dueDate ? task.dueDate.toISOString() : null;
    entry.deferDate = task.deferDate ? task.deferDate.toISOString() : null;
    entry.flagged = task.flagged;
    entry.tags = task.tags ? task.tags.map(function(t) { return t.name; }) : [];
    entry.projectName = task.containingProject ? task.containingProject.name : null;
    entry.note = task.note || null;
    entry.taskStatus = (function() {
      var s = String(task.taskStatus);
      if (s.includes("Available")) return "available";
      if (s.includes("Blocked")) return "blocked";
      if (s.includes("Next")) return "next";
      if (s.includes("DueSoon")) return "due_soon";
      if (s.includes("Overdue")) return "overdue";
      if (s.includes("Completed")) return "completed";
      if (s.includes("Dropped")) return "dropped";
      return "unknown";
    })();
  }
  tasks.push(entry);
  count += 1;
}

win.perspective = previousPerspective;

return {
  perspectiveName: ${nameJson},
  count: tasks.length,
  tasks: tasks
};
`.trim();
        const result = await runOmniJs(script);
        if (
          typeof result === "object" &&
          result !== null &&
          "error" in result &&
          typeof (result as Record<string, unknown>).error === "string"
        ) {
          return errorResult((result as Record<string, unknown>).error as string);
        }
        return textResult(result);
      } catch (error: unknown) {
        return errorResult(normalizeError(error));
      }
    }
  );
}
