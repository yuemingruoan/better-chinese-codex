# 退出与关闭流程（TUI）

本文说明 Rust TUI（`codex-rs/tui`）中退出、关闭与中断的工作方式。
面向 Codex 开发者与 Codex 本身，用于推理未来退出/关闭相关改动。

本文替代此前分散的历史与设计笔记。简要历史见下文；完整细节请参考 PR #8936。

## 术语

- **Exit（退出）**：结束 UI 事件循环并终止进程。
- **Shutdown（关闭）**：请求优雅关闭 agent/core（`Op::Shutdown`），并等待 `ShutdownComplete` 以便执行清理。
- **Interrupt（中断）**：取消正在执行的操作（`Op::Interrupt`）。

## 事件模型（AppEvent）

退出通过一个带显式模式的事件进行协调：

- `AppEvent::Exit(ExitMode::ShutdownFirst)`
  - 用户主动退出时优先使用，确保清理逻辑执行。
- `AppEvent::Exit(ExitMode::Immediate)`
  - 立即退出的逃生口。跳过关闭流程，可能丢弃在途工作（例如任务、rollout flush、子进程清理）。

`App` 是协调者：它提交 `Op::Shutdown`，并且只有在收到 `ExitMode::Immediate` 时才退出 UI 循环（通常发生在 `ShutdownComplete` 之后）。

## 用户触发的退出流程

### Ctrl+C

UI 层的优先级顺序：

1. 活跃的 modal/view 先尝试处理（`BottomPane::on_ctrl_c`）。
   - 如果 modal 处理了，退出流程停止。
   - 当 modal/popup 处理 Ctrl+C 时，会清除退出快捷键状态，避免关闭 modal 后意外触发下一次 Ctrl+C 退出。
2. 若用户已在 1 秒窗口内“武装” Ctrl+C，再次按下会立刻触发“先关闭再退出”。
3. 否则，`ChatWidget` 会武装 Ctrl+C，并显示退出提示（`ctrl + c again to quit`）持续 1 秒。
4. 如果存在可取消工作（streaming/tools/review），`ChatWidget` 会提交 `Op::Interrupt`。

### Ctrl+D

- 仅在输入框为空且无 modal 时参与退出。
  - 第一次按下时，显示退出提示（同 Ctrl+C）并启动 1 秒计时。
  - 在提示可见时再次按下，请求“先关闭再退出”。
- 如果有任何 modal/popup 打开，按键事件会路由给视图，Ctrl+D 不会尝试退出。

### Slash 命令

- `/quit`、`/exit`、`/logout` 无提示直接请求“先关闭再退出”，
  因为 slash 命令更难误触，且更明确表达退出意图。

### /new

- 使用“关闭但不退出”（抑制 `ShutdownComplete`），以便应用开启新会话而不终止进程。

## Shutdown 完成与抑制

`ShutdownComplete` 是 core 清理完成的信号。UI 将其视作退出边界：

- `ChatWidget` 在收到 `ShutdownComplete` 时请求 `Exit(Immediate)`。
- 当关闭仅用于清理步骤时（例如 `/new`），`App` 可抑制一次 `ShutdownComplete`。

## 边界情况与不变量

- **Review 模式**视为可取消工作。Ctrl+C 应优先中断 review，而不是退出。
- **Modal 打开**时，除非 modal 明确拒绝处理 Ctrl+C，否则 Ctrl+C/Ctrl+D 不应退出。
- **Immediate exit** 不是正常用户路径；它是关闭完成后的兜底或紧急退出。请谨慎使用，因为它跳过清理。

## 测试期望

至少覆盖以下场景：

- 工作中按 Ctrl+C：应中断，不退出。
- 空闲且输入为空时按 Ctrl+C：显示退出提示；再次按下触发“先关闭再退出”。
- 有 modal 打开时按 Ctrl+D：不退出。
- `/quit` / `/exit` / `/logout`：无提示退出，但仍为“先关闭再退出”。
  - 空闲且输入为空时按 Ctrl+D：显示退出提示；再次按下触发“先关闭再退出”。

## 历史（高层）

Codex 过去在不同退出手势中混用“立即退出”和“先关闭再退出”，
主要源于状态追踪的增量变更与回归。本文反映当前统一的“先关闭再退出”方案。
详细历史与动机请参考 PR #8936。
