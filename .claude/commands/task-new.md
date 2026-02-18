Create a new task in the backlog.

Task title: $ARGUMENTS

Steps:
1. Use Glob to check if `backlog/` exists. If not, create it with Bash (`mkdir -p backlog`).
2. Use Glob("backlog/TASK-*/") to list existing task folders. Find the highest N and use N+1 (start at 1 if none exist).
3. If $ARGUMENTS is empty, use AskUserQuestion to ask for a task title before continuing.

4. Use AskUserQuestion to gather the following information before creating any files. Ask all questions at once (up to 4 per call), then ask the remainder in a follow-up call if needed:
   - What problem does this task solve? (the pain point or gap)
   - What does "done" look like? (the observable outcome when complete)
   - What are the acceptance criteria? (specific, testable conditions — list them)
   - What is explicitly out of scope? (what won't be addressed)

   If the user's answers are vague, ask one targeted follow-up with AskUserQuestion to sharpen the detail before proceeding.

5. Create `backlog/TASK-{N}/spec.md` using the Write tool, filling in the template with the user's answers:

---
id: TASK-{N}
title: {title}
status: backlog
created: {today's date in YYYY-MM-DD}
---

# TASK-{N}: {title}

## Problem
{user's answer to "what problem does this solve?"}

## Goal
{user's answer to "what does done look like?"}

## Acceptance Criteria
{user's acceptance criteria as a checked checklist}

## Out of Scope
{user's out-of-scope items}

## Notes
...

6. Display the created spec.md to the user and confirm it accurately reflects their intent. If anything is off, edit the file to correct it.
