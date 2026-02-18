Start implementing a groomed backlog task.

Arguments: $ARGUMENTS

Steps:
1. Determine which task to start:
   - If $ARGUMENTS is provided, verify it has a plan.md. If not, stop and tell the user to run /task-groom first.
   - If empty, use Glob("backlog/TASK-*/spec.md") to list tasks. Show only tasks where status=backlog AND plan.md exists. Ask the user which to start.

2. Update the task's spec.md: change `status: backlog` to `status: in-progress` using the Edit tool.

3. Read and display both spec.md and plan.md to load the full context.

4. Begin implementing according to plan.md. Work through stages sequentially:
   - Complete each stage before moving to the next
   - Commit working code incrementally after each stage
   - Update Acceptance Criteria checkboxes in spec.md as they are satisfied

5. When all acceptance criteria are checked off, update spec.md status to `done`.
