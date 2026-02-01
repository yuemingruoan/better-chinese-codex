# Release Notes / 发布说明

## English

### Highlights
- Added the `batches_read_file` tool for structured multi-file reads (glob support, limits, and default enablement).
- Improved SDD workflows: baseline branch now tracks the current branch, and SDD Git actions dispatch correctly with new tests.
- TUI/TUI2 fixes: removed duplicate language rendering in `/model`, improved clipboard image paste via data URLs, and refreshed related snapshots.
- Added `/clean` to clear `.codex` clipboard image cache and fixed built-in slash command ordering.

### Tooling
- Added a manual Release workflow that reads notes from `docs/release/notes.md`.

## 中文

### 亮点
- 新增 `batches_read_file` 工具：支持通配符、多文件结构化读取、限制与默认启用。
- 优化 SDD 流程：基线分支改为当前分支，修复 SDD Git 动作调度并新增测试。
- TUI/TUI2 修复：`/model` 弹窗中英文重复显示修复、剪贴板图片改为 data URL 附件链路、更新相关快照。
- 新增 `/clean` 清理 `.codex` 剪贴板图片缓存，并修复内建 slash 命令补全排序。

### 工具链
- 新增手动触发的 Release 工作流，Release Notes 统一从 `docs/release/notes.md` 读取。
