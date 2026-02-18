Groom a backlog task: research the codebase and write a detailed implementation plan.

Arguments: $ARGUMENTS

Steps:
1. Determine which task to groom:
   - If $ARGUMENTS is provided (e.g. "TASK-1"), use that task.
   - If empty, use Glob("backlog/TASK-*/spec.md") to list tasks. Read each spec.md and show only tasks where status=backlog AND no plan.md exists yet. Use AskUserQuestion to ask the user which task to groom.

2. Read the task's spec.md to understand the requirements.

3. Use AskUserQuestion to clarify intent before exploring the codebase. Ask all questions at once (up to 4 per call):
   - Are there any specific files, components, or modules you know should be involved?
   - Are there any implementation constraints or approaches you want to avoid?
   - Are there any open questions or unknowns that should influence the plan?
   - Is there a preferred order or priority among the acceptance criteria?

   If answers reveal ambiguity in the spec itself (e.g. conflicting criteria, unclear scope), surface that explicitly and use AskUserQuestion to resolve it before continuing.

4. Based on the spec and the user's answers, explore relevant parts of the codebase:
   - Find existing patterns, utilities, and files that will need changes
   - Identify where new code should live, following existing conventions
   - Note any constraints or dependencies

5. Write `backlog/{TASK-ID}/plan.md` with:
   - Context: why this change is needed
   - Approach: the recommended strategy, including any alternatives considered
   - 3-5 Stages with goals, affected file paths, and concrete steps
   - Verification: how to test end-to-end

6. Display the plan to the user and use AskUserQuestion to confirm it looks right before finalizing:
   - Does the approach match their expectations?
   - Are the stages in the right order?
   - Is anything missing or over-engineered?

   Incorporate any feedback, then confirm the task is ready for `/task-start`.
