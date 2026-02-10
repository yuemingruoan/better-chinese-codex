# Release Notes / 发布说明

## v1.7.1 (changes since 1.7.0)

### English

#### Collaboration & Multi-Agent Orchestration
- Added batch and registry-style collab tools:
  - `list_agents`
  - `wait_agents` (supports `any|all`, default-target behavior, timeout aggregation)
  - `close_agents`
- Extended `spawn_agent` with orchestration metadata:
  - `label`, `acceptance_criteria`, `test_commands`, `allow_nested_agents`
- Added model/control overrides in `spawn_agent`:
  - `model`, `reasoning_effort`, `reasoning_summary`
- Added permission overrides in `spawn_agent`:
  - `approval_policy`, `sandbox_mode`
- Added policy and governance controls:
  - `max_active_subagents_per_thread`
  - `max_spawn_depth`
  - `default_wait_timeout_ms`
  - `auto_close_on_parent_shutdown`
  - `allow_subagent_permission_escalation`
- Added agent registry metadata and lifecycle synchronization (`creator`, goal, acceptance criteria, test commands, status, closed flag).
- Fixed sub-agent quota slot leak after close/shutdown.
- Increased default collaboration limits:
  - `max_active_subagents_per_thread`: `8 -> 30`
  - `agents.max_threads`: `6 -> 30`

#### TUI / TUI2 and i18n
- Added collab lifecycle rendering in chat UI (spawn / interaction / wait / close begin/end).
- Added localized status labels for collab state (pending/running/completed/errored/timed out/shutdown/not found).
- Introduced SDD parallel workflow command support (`/sdd-develop-parallels`) with dedicated plan/execute/merge prompts.
- Synced related prompt and slash-command behavior across both `tui` and `tui2`.

#### Stability & Correctness
- Fixed premature turn completion when output stream only contains assistant commentary (now treated as preamble, not final answer).
- Stabilized core regression coverage around pending input and tool-flow behavior.
- Kept CLI `exec` base-instructions override authoritative in config merge flow.

#### Release Engineering
- Restored release-critical workflows:
  - `.github/workflows/release.yml`
  - `.github/workflows/build-platform-binaries.yml`
- Bumped workspace and package versions to `1.7.1`.
- Added explicit release process guidance to `AGENTS.md`.

---

### 中文

#### 协作与多 Agent 编排
- 新增批量与注册表视角的协作工具：
  - `list_agents`
  - `wait_agents`（支持 `any|all`、默认目标集合与超时聚合）
  - `close_agents`
- 扩展 `spawn_agent` 编排元数据：
  - `label`、`acceptance_criteria`、`test_commands`、`allow_nested_agents`
- 新增 `spawn_agent` 模型/思考控制参数：
  - `model`、`reasoning_effort`、`reasoning_summary`
- 新增 `spawn_agent` 权限控制参数：
  - `approval_policy`、`sandbox_mode`
- 新增治理配置与约束：
  - `max_active_subagents_per_thread`
  - `max_spawn_depth`
  - `default_wait_timeout_ms`
  - `auto_close_on_parent_shutdown`
  - `allow_subagent_permission_escalation`
- 新增子 Agent 注册信息与生命周期同步（创建者、目标、验收标准、测试命令、状态、关闭标记）。
- 修复子 Agent 关闭后配额槽位未释放问题。
- 提升默认协作并发上限：
  - `max_active_subagents_per_thread`: `8 -> 30`
  - `agents.max_threads`: `6 -> 30`

#### TUI / TUI2 与 i18n
- 在聊天界面新增协作生命周期渲染（spawn / interaction / wait / close 的开始与结束事件）。
- 新增协作状态本地化文案（初始化中、运行中、已完成、出错、超时、已关闭、未找到）。
- 引入 SDD 并行开发指令支持（`/sdd-develop-parallels`），并提供配套 plan/execute/merge 提示词。
- 在 `tui` 与 `tui2` 之间同步相关提示词与斜杠命令行为。

#### 稳定性与正确性
- 修复“仅 commentary 输出被误判为最终完成”的提前结束问题（现改为前置说明，不作为最终答复）。
- 补强 core 回归测试，稳定 pending input 与工具链路行为。
- 修复 CLI `exec` 模式下 base instructions 覆盖在配置合并中的优先级稳定性。

#### 发布工程
- 恢复发布关键工作流：
  - `.github/workflows/release.yml`
  - `.github/workflows/build-platform-binaries.yml`
- workspace 与相关包版本升级至 `1.7.1`。
- 在 `AGENTS.md` 中补充常规发布流程说明。
