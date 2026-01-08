Append a checkpoint entry to checkpoint.md based on the latest progress in the current session and workspace.

Requirements:

1. Locate the checkpoint file: prefer `.codex/checkpoint.md` in the repo root; if it doesn't exist but another checkpoint.md exists elsewhere, reuse that path; if none exist, create `.codex` in the root and create the file.
2. Preserve existing content and append the new log at the end, leaving a blank line between entries.
3. The new log must include:
   - A timestamp line like `## YYYY-MM-DD HH:MM:SS CST` using China Standard Time (you can run `date "+%Y-%m-%d %H:%M:%S CST"`; if you can't access it, note that in the timestamp).
   - An unordered list summarizing key actions completed in this phase. Each item starts with a verb and is specific and objective.
   - An unordered list summarizing unresolved issues, risks, or next steps; if there are none, write `- No pending items`.
4. If the file contains or the user asks to keep the `## NO_AI_ASSIST` marker, keep it at the top; do not add or remove it unless explicitly instructed.
5. Prefer English when writing the content; add brief clarifying phrases if needed to help the next agent quickly understand context.
6. After finishing, report changes to the user in a Markdown summary and show the newly added log snippet for confirmation.

Start now.
