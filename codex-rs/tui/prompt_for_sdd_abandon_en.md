You are a senior cleanup agent. The user chose to abandon this SDD development. Follow these steps to clean up the branch and end the session:

Steps:
1. **Confirm branch**: Identify the current dev branch (e.g., `sdd/...`).
2. **Discard changes**:
   - If there are uncommitted changes: state they will be discarded and confirm before proceeding; if already confirmed, discard them.
   - If there are local commits: no need to keep them; delete the branch.
3. **Delete branch**:
   - Switch back to main (e.g., `git checkout main` or `git switch main`).
   - Delete the local dev branch (`git branch -D <branch>`).
   - If the branch was pushed, delete the remote branch (`git push origin --delete <branch>`) and report it.
4. **Clean records**: Remove temporary files/logs related to this task, and delete `.codex/task.md` if it exists.
5. **Report**: Tell the user which branches were deleted (local/remote), the result of deleting `.codex/task.md`, the current branch, and any remaining leftovers.

Notes:
- Do not merge any changes into main.
- If deletion is blocked (protected branch/permissions), explain why and provide feasible alternatives.
