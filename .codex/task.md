# /model 弹窗中英文重复显示修复任务规划（TUI/TUI2）

## 1) 标题与目标
- 修复 `/lang` 设为简体中文时 `/model` 弹窗标题/说明中英文重复显示的问题（TUI/TUI2），确保仅显示当前语言版本且不影响警告行展示。

## 2) 交付物
- 修复后的 TUI/TUI2 代码变更（`codex-rs/tui`、`codex-rs/tui2`）。
- 更新后的相关快照文件（若 UI 输出变更）。
- 测试与格式化记录（`cargo test -p codex-tui`、`cargo test -p codex-tui2`、`cargo insta`、`just fmt`、`just fix -p codex-tui`、`just fix -p codex-tui2`）。

## 3) 范围 / 非范围
范围：
- 定位并修复 `codex-rs/tui` 与 `codex-rs/tui2` 的模型选择弹窗重复渲染问题。
- 保持现有文案与中英文翻译，确保仅展示当前语言。
- 维持模型选择警告行（OPENAI_BASE_URL）行为不变。
- 更新相关快照并确保测试通过。

非范围：
- 不改动模型列表内容、排序或选择逻辑。
- 不新增/修改与本问题无关的文案或功能。
- 不引入与本问题无关的 UI 结构调整。
- 不变更配置、协议或后端模型逻辑。

## 4) 工作项清单
| ID | 内容 | 完成状态 | 负责人 | 实施要点 | 验证方式 |
| --- | --- | --- | --- | --- | --- |
| T1 | 现状定位与方案确认 | [x] | AI | 阅读 `codex-rs/tui/src/chatwidget.rs`、`codex-rs/tui/src/bottom_pane/list_selection_view.rs` 与 `codex-rs/tui2/src/chatwidget.rs`、`codex-rs/tui2/src/bottom_pane/list_selection_view.rs`，确认 header 与 title/subtitle 叠加导致重复显示的路径；梳理 `/model` 与“全部模型”两处入口的差异。 | 记录重复显示成因与拟定修复方式（标题/副标题只渲染一次）。 |
| T2 | 修复模型选择弹窗重复显示 | [ ] | AI | 在 `tui` 与 `tui2` 的 `open_model_popup_with_presets`、`open_all_models_popup` 中避免同时传 `header` 与 `title/subtitle`；统一通过 `model_menu_header`（或等价方案）渲染标题/副标题与 warning；确保 `tr(...)` 用于中英文；用 `apply_patch` 修改。 | 运行后仅显示当前语言标题/说明；warning 仍出现且位置合理。 |
| T3 | 更新快照与单测 | [ ] | AI | 运行 `cargo test -p codex-tui` 与 `cargo test -p codex-tui2` 生成快照；用 `cargo insta pending-snapshots -p codex-tui`/`-p codex-tui2` 审核；必要时分别 `cargo insta accept -p codex-tui`/`-p codex-tui2`；检查 `.snap.new` 仅反映标题重复修复。 | 两个 crate 测试通过；快照无待处理或已接受。 |
| T4 | 格式化与静态检查 | [ ] | AI | 在 `codex-rs` 下运行 `just fmt`；完成后运行 `just fix -p codex-tui` 与 `just fix -p codex-tui2`；若有新增告警，按需修复。 | 命令返回成功；无新增未处理告警。 |

## 5) 里程碑与顺序
- M1 现状确认与修复方案：T1
- M2 代码修复：T2
- M3 验证与快照：T3
- M4 格式化与检查：T4

## 6) 风险与缓解
- 风险：修复后 warning 行位置变化影响可读性。  
  缓解：保持 `model_menu_header` 内标题/副标题/警告的顺序，确认渲染布局。
- 风险：仅修复单一 UI（tui 或 tui2），遗漏另一套实现。  
  缓解：两套实现同步修改与验证。
- 风险：仅修复 `/model` 入口，遗漏“全部模型”弹窗。  
  缓解：同时覆盖 `open_model_popup_with_presets` 与 `open_all_models_popup`。
- 风险：快照更新误含其他 UI 变更。  
  缓解：审查 `.snap.new`，确保仅标题重复相关变更。
- 风险：语言切换边界未覆盖。  
  缓解：手动验证 `Language::En` 与 `Language::ZhCn` 的输出。

## 7) 验收与测试
- `/lang` 设为简体中文时，`/model` 弹窗仅显示中文标题/说明，不再夹带英文重复（TUI/TUI2）。
- 英文模式下标题/说明只出现一次。
- OPENAI_BASE_URL 警告仍可显示（如适用）。
- `cargo test -p codex-tui` 与 `cargo test -p codex-tui2` 通过，快照已更新或无变更。

## 8) 回滚与清理
- 若需回滚：`git revert <commit>` 还原本次修复。
- 若中途放弃：`git restore --source=HEAD --worktree --staged <file>` 清理未提交变更。
- 清理分支：删除本次修复分支（如 `fix/model-menu-i18n`）。

## 9) 工具与命令
- 修改文件：优先用 `apply_patch` 覆盖 `.codex/task.md` 与后续代码改动。
- 分支管理：`git switch develop-main` → `git switch -c fix/model-menu-i18n`。
- 格式化：`just fmt`（在 `codex-rs` 目录）。
- 静态检查：`just fix -p codex-tui`、`just fix -p codex-tui2`（在 `codex-rs` 目录）。
- 测试/快照：`cargo test -p codex-tui`、`cargo test -p codex-tui2`、`cargo insta pending-snapshots -p codex-tui`、`cargo insta pending-snapshots -p codex-tui2`、`cargo insta accept -p codex-tui`、`cargo insta accept -p codex-tui2`。
- 进度同步：每完成一个任务更新 `.codex/task.md` 勾选。

## 10) 测试计划
- T1/T2：无自动化测试；以代码审查与 UI 输出逻辑为准。
- T3：`cargo test -p codex-tui` 与 `cargo test -p codex-tui2` 生成快照；`cargo insta pending-snapshots -p codex-tui`/`-p codex-tui2` 校验变更；必要时对应 `cargo insta accept` 接受。
- T4：`just fmt` 与 `just fix -p codex-tui` 完成且无新增问题。
- 预期标准：测试通过；快照仅反映标题/说明重复修复；无额外警告或失败。

## 11) 汇报清单
- 计划确认要点与修复方案（标题/副标题只渲染一次）。
- 当前分支名。
- 已完成任务/剩余任务状态。
- 测试与快照处理结果。
- 是否存在阻塞或待决事项。
