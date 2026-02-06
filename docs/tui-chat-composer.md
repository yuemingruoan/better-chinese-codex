# Chat Composer 状态机（TUI）

本文记录 `ChatComposer` 的输入状态机，以及为 Windows 终端新增的粘贴相关行为。

主要实现：

- `codex-rs/tui/src/bottom_pane/chat_composer.rs`

粘贴突发检测器：

- `codex-rs/tui/src/bottom_pane/paste_burst.rs`

## 要解决的问题是什么？

在部分终端（尤其是 Windows 上的 `crossterm`）中，_bracketed paste_ 并不总是以单一的粘贴事件上报。
相反，多行粘贴可能表现为一串快速的按键事件：

- `KeyCode::Char(..)` 表示文本
- `KeyCode::Enter` 表示换行

如果输入框将这些事件当作“正常打字”，可能导致：

- 粘贴仍在流入时意外触发 UI 切换（例如 `?`）。
- 当 `Enter` 抵达时，在粘贴途中错误提交。
- 先渲染为输入，再在足够字符到达后被“重新归类”为粘贴（产生闪烁）。

解决方案是检测类似粘贴的“突发”，并将其缓冲为单次明确的 `handle_paste(String)` 调用。

## 高层状态机

`ChatComposer` 事实上组合了两个小型状态机：

1. **UI 模式**：当前激活的弹窗（如果有）。
   - `ActivePopup::None | Command | File | Skill`
2. **粘贴突发**：用于非 bracketed paste 的瞬态检测状态。
   - 由 `PasteBurst` 实现

### 按键事件路由

`ChatComposer::handle_key_event` 会根据 `active_popup` 分派：

- 若弹窗可见，则优先由弹窗专用处理器处理按键（导航、选择、补全）。
- 否则由 `handle_key_event_without_popup` 处理更高层语义（Enter 提交、历史导航等）。
- 处理完成后调用 `sync_popups()`，确保弹窗可见性/过滤条件与最新文本与光标一致。
- 当 slash 命令名补全且用户输入空格时，`/command` token 会提升为文本元素，以便独立渲染并原子编辑。

### 历史导航（↑/↓）

上下键由 `ChatComposerHistory` 处理，并合并两种来源：

- **持久历史**（跨会话，来自 `~/.codex/history.jsonl`）：仅文本。
  它**不**携带文本元素范围或本地图片附件，因此回忆该类条目只恢复文本。
- **本地历史**（当前会话）：存储完整提交 payload，包括文本元素与本地图片路径。
  回忆本地条目会重新生成占位符与附件。

这一设计在保持磁盘历史向后兼容的同时，避免持久化附件，又能在会话内提供更丰富的回忆体验。

## 可复用的配置开关

`ChatComposer` 通过 `ChatComposerConfig` 支持功能 gating（见 `codex-rs/tui/src/bottom_pane/chat_composer.rs`）。默认配置保持现有聊天行为。

开关：

- `popups_enabled`
- `slash_commands_enabled`
- `image_paste_enabled`

关闭时的关键影响：

- `popups_enabled` 为 `false` 时，`sync_popups()` 会强制 `ActivePopup::None`。
- `slash_commands_enabled` 为 `false` 时，输入 `/...` 不再被视为命令。
- `slash_commands_enabled` 为 `false` 时，`prepare_submission_text` 不再展开自定义提示词。
- `slash_commands_enabled` 为 `false` 时，slash 语境下的粘贴突发例外被禁用。
- `image_paste_enabled` 为 `false` 时，粘贴路径不会触发图片附件。
- `ChatWidget` 会根据所选模型的 `input_modalities` 在运行时切换 `image_paste_enabled`；
  附件与提交路径也会再次校验支持度，并在不支持时发出警告而不是直接丢弃草稿。

内建 slash 命令的可用性在 `codex-rs/tui/src/bottom_pane/slash_commands.rs` 中集中管理，并被 composer 与命令弹窗复用，以保证 gating 一致。

## 提交流程（Enter/Tab）

存在多条提交路径，但核心规则一致：

### 常规提交/排队路径

`handle_submission` 会为提交与排队都调用 `prepare_submission_text`。该方法：

1. 展开所有待处理的粘贴占位符，使元素范围与最终文本对齐。
2. 去除首尾空白，并将元素范围基于裁剪后的缓冲区重定位。
3. 展开 `/prompts:` 自定义提示词：
   - 命名参数使用 key=value 解析。
   - 数字参数使用位置解析，支持 `$1..$9` 与 `$ARGUMENTS`。
     展开过程会保留文本元素，并生成最终提交 payload。
4. 修剪附件，仅保留在展开后仍存在占位符的条目。
5. 成功后清理待处理粘贴；若最终文本为空且无附件，则抑制提交。

同一准备路径也会复用于带参数的 slash 命令（例如 `/plan`、`/review`），确保提取参数时保留粘贴内容与文本元素。

### 数字自动提交路径

当 slash 弹窗开启且第一行匹配“仅数字”的自定义提示词时，Enter 会自动提交且不调用 `prepare_submission_text`。该路径仍会：

- 在解析位置参数前展开待处理粘贴。
- 使用展开后的文本元素进行提示词展开。
- 基于展开后的占位符修剪附件。
- 成功自动提交后清理待处理粘贴。

## 粘贴突发：概念与假设

突发检测器故意保持保守：仅处理“纯”字符输入（无 Ctrl/Alt 修饰）。其他输入会刷新并/或清理突发窗口，以确保快捷键保持原意。

### `PasteBurst` 的概念状态

- **Idle**：无缓冲、无待处理字符。
- **首字符待定**（仅 ASCII）：短暂保留一个快速字符，避免其先渲染再被粘贴判定移除。
- **活跃缓冲**：一旦判定为粘贴突发，将内容累积到 `String` 缓冲。
- **Enter 抑制窗口**：在突发后短时间内继续将 `Enter` 视作换行，确保多行粘贴仍被归为同一粘贴。

### ASCII 与非 ASCII（IME）输入

非 ASCII 字符常来自输入法（IME），可能以快速突发形式到达。若在这种情况下保留首字符，会让用户感觉输入被吞。

因此 composer 进行区分：

- **ASCII 路径**：允许保留首个快速字符（`PasteBurst::on_plain_char`）。
- **非 ASCII 路径**：永不保留首字符（`PasteBurst::on_plain_char_no_hold`），但仍允许突发检测。
  当该路径检测到突发时，已插入的前缀可能会从 textarea 中回收并移入粘贴缓冲。

为避免将 IME 突发误判为粘贴，非 ASCII 的回收路径会执行额外启发式
（`PasteBurst::decide_begin_buffer`），仅当回收前缀“像粘贴”（含空白或足够长）时才会回收。

### 关闭突发检测

`ChatComposer` 提供 `disable_paste_burst` 作为逃生口。

启用后：

- 新输入不会经过突发检测（不再保留首字符，也不再进行缓冲判定）。
- 按键流按正常输入处理（包括 slash 命令行为）。
- 启用该标志会先将已保留/缓冲的突发文本经由正常粘贴路径
  （`ChatComposer::handle_paste`）刷新，然后清理突发计时与 Enter 抑制窗口，避免状态泄漏到后续输入。

### Enter 处理

当粘贴突发缓冲激活时，Enter 会被视为“向突发追加 `\n`”，而不是“提交消息”。
这可防止多行粘贴在 `Enter` 事件驱动下提前提交。

在 slash 命令语境（弹窗打开或第一行以 `/` 开头）内，会禁用基于突发的 Enter 抑制，确保命令输入保持可预测的“提交/执行”语义。

## PasteBurst：事件级行为（速查）

本节说明 `ChatComposer` 如何解释 `PasteBurst` 的判定，便于审阅状态切换，而无需在脑中运行代码。

### 纯 ASCII `KeyCode::Char(c)`（无 Ctrl/Alt 修饰）

`ChatComposer::handle_input_basic` 调用 `PasteBurst::on_plain_char(c, now)` 并根据返回的 `CharDecision` 分支：

- `RetainFirstChar`：暂不把 `c` 插入 textarea。之后一个 UI tick 可能通过 `PasteBurst::flush_if_due` 将其刷新为普通输入。
- `BeginBufferFromPending`：首个 ASCII 字符已被保留/缓冲；通过 `PasteBurst::append_char_to_buffer` 追加 `c`。
- `BeginBuffer { retro_chars }`：尝试回收已插入前缀：
  - 调用 `PasteBurst::decide_begin_buffer(now, before_cursor, retro_chars)`；
  - 若返回 `Some(grab)`，删除 textarea 中 `grab.start_byte..cursor`，并将 `c` 追加到缓冲；
  - 若返回 `None`，回退为普通插入。
- `BufferAppend`：将 `c` 追加到活跃缓冲。

### 纯非 ASCII `KeyCode::Char(c)`（无 Ctrl/Alt 修饰）

`ChatComposer::handle_non_ascii_char` 使用略有不同的流程：

- 首先用 `PasteBurst::flush_before_modified_input` 刷新任何待处理的 ASCII 瞬态状态（包括单个保留字符）。
- 若突发已激活，`PasteBurst::try_append_char_if_active(c, now)` 直接追加 `c`。
- 否则调用 `PasteBurst::on_plain_char_no_hold(now)`：
  - `BufferAppend`：追加 `c` 到活跃缓冲。
  - `BeginBuffer { retro_chars }`：执行 `decide_begin_buffer(..)`，若开始缓冲则从 textarea 中删除回收前缀并追加 `c`。
  - `None`：正常将 `c` 插入 textarea。

此路径中的额外 `decide_begin_buffer` 启发式是有意为之：IME 输入可能快速突发，
因此仅当回收前缀“像粘贴”（包含空白或足够长）时才进行回收，以避免误判 IME 组合输入为粘贴。

### `KeyCode::Enter`：换行 vs 提交

存在两种“Enter 变为换行”的机制：

- **突发上下文中**（`paste_burst.is_active()`）：`append_newline_if_active(now)` 将 `\n` 追加到突发缓冲，
  让多行粘贴仍作为一次显式粘贴。
- **突发刚结束后**（Enter 抑制窗口）：
  `newline_should_insert_instead_of_submit(now)` 将 `\n` 插入 textarea 并调用
  `extend_window(now)`，让稍晚的 Enter 继续按“换行”而非“提交”处理。

在 slash 命令语境（弹窗开启或第一行以 `/` 开头）内，两者都会禁用，以保证 Enter 保持常规“提交/执行”语义。

### 非字符键 / Ctrl 修饰输入

非字符输入不应让突发状态泄漏到不相关动作中：

- 若有缓冲突发文本，调用 `clear_window_after_non_char` 前应先刷新（见“注意事项”），通常通过
  `PasteBurst::flush_before_modified_input`。
- `PasteBurst::clear_window_after_non_char` 会清除“近期突发”窗口，避免下一次按键被错误归入上次粘贴。

### 注意事项

- `PasteBurst::clear_window_after_non_char` 会清除 `last_plain_char_time`。
  若在 `buffer` 非空时调用且**未先刷新**，`flush_if_due()` 将失去超时依据，缓冲文本可能永远不会刷新。
  因此应将 `clear_window_after_non_char` 视为“刷新后丢弃分类上下文”，而不是“刷新”。
- `PasteBurst::flush_if_due` 使用严格的 `>` 比较，测试与 UI tick 应至少跨过 1ms 阈值
  （参见 `PasteBurst::recommended_flush_delay`）。

## 重要交互 / 不变量

- composer 频繁用光标位置切片 `textarea.text()`；所有切片前必须先把光标钳制到 UTF-8 字符边界。
- `sync_popups()` 必须在任何可能影响弹窗可见性/过滤的变化之后运行：插入、删除、刷新突发、应用粘贴占位符等。
- 通过 `?` 切换快捷键覆盖层时会检查 `!is_in_paste_burst()`，避免粘贴流入时切换 UI 模式。
- Mention 弹窗选择有两个 payload：可见的 `$name` 文本与隐藏的
  `mention_paths[name] -> canonical target` 关联。通用的 `set_text_content` 路径会刻意清除关联
  以处理新草稿；而恢复被阻塞/中断提交的路径必须使用保留 mention 的 setter，以便重试时保留原始目标。

## 固定行为的测试

当前 `PasteBurst` 逻辑通过 `ChatComposer` 集成测试覆盖。

- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
  - `non_ascii_burst_handles_newline`
  - `ascii_burst_treats_enter_as_newline`
  - `question_mark_does_not_toggle_during_paste_burst`
  - `burst_paste_fast_small_buffers_and_flushes_on_stop`
  - `burst_paste_fast_large_inserts_placeholder_on_flush`

本文还指出了一些额外约束（例如“先刷新再清除”），目前尚未由专门的 `PasteBurst` 单元测试完全覆盖。
