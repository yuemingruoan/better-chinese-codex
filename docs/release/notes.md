# Release Notes / 发布说明

## v1.7.1

### English

#### Highlights
- Fixed a sub-agent quota leak where closed agents could still consume spawn slots in long-running sessions.
- Increased default collaboration limits:
  - `max_active_subagents_per_thread`: `8 -> 30`
  - `agents.max_threads` (global guard): `6 -> 30`
- Added regression tests to verify slot release after `close_agent` / shutdown paths.
- Restored release-critical workflows:
  - `.github/workflows/release.yml`
  - `.github/workflows/build-platform-binaries.yml`

#### Upgrade Notes
- This is a patch release focused on multi-agent stability and release pipeline reliability.
- No breaking API changes are introduced.

### 中文

#### 亮点
- 修复子 Agent 配额泄漏：已关闭子 Agent 在长会话中仍可能占用 spawn 配额的问题。
- 提升默认协作并发上限：
  - `max_active_subagents_per_thread`: `8 -> 30`
  - `agents.max_threads`（全局守卫）: `6 -> 30`
- 增加回归测试，覆盖 `close_agent` / shutdown 后配额释放行为。
- 恢复发布关键工作流：
  - `.github/workflows/release.yml`
  - `.github/workflows/build-platform-binaries.yml`

#### 升级说明
- 本次为补丁版本，重点提升多 Agent 稳定性与发布链路可靠性。
- 未引入破坏性 API 变更。
