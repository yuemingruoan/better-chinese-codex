# Codex CLI 中文指南

## 项目简介
Codex CLI 是一套跨平台的开发辅助工具链，内含命令行界面、TUI UI、文件搜索、MCP 客户端等多个子组件。本仓库 `codex-rs` 提供 Rust 实现版本，适用于自动化执行、交互式开发以及与 AI 协作的场景。

## 目录约定
- `.codex/AGENTS.md`：AI 协作者在当前项目中的行为记录与说明（自动生成，会覆盖同名文件）。
- `.codex/checkpoint.md`：人工维护的开发日志，用于串联每一步 AI 或人工操作的结果、待办事项与风险提示。
- `codex-rs/`：Rust 工作空间，包含 `codex-core`、`codex-tui` 等 crate。
- `codex-cli/`：DotSlash 脚本及辅助工具，确保团队成员使用统一版本的可执行程序。
- 对于先前使用Claude Code

## 环境准备
1. 安装 Git ≥ 2.23（建议）。
2. 系统要求：macOS 12+/Linux（Ubuntu 20.04+/Debian 10+）/Windows 11（需 WSL2）。
3. 内存至少 4 GB（推荐 8 GB）。
4. 若需参与开发，可酌情安装 Rust 工具链与 `just`、`rg` 等辅助工具；仅使用发布版二进制时无需这些依赖。

## 安装指南

在每次更新前，请先从 Release 页面下载对应平台的最新 `codex` 可执行文件，将其改名为`codex`(macOS/Limux)或`codex.exe`(Windows)然后按下述步骤替换。

### Windows（全局固定使用 npm 安装）
```powershell
npm install -g @openai/codex-cli
```
- npm 全局目录通常位于 `%APPDATA%\npm`。
- 将 Release 中的 `codex.exe` 替换 `%APPDATA%\npm\codex.cmd` 同级目录中的 `codex`（注意保留 `.cmd` 启动脚本）。

### macOS / Linux
1. 先安装 CLI（可选择 npm、Homebrew 或其他渠道）：
   ```bash
   npm install -g @openai/codex-cli
   # 或
   brew install openai/codex/codex-cli
   ```
2. 查找当前 `codex` 可执行文件位置：
   ```bash
   which codex
   ```
3. 若输出是符号链接（Homebrew 与部分 npm 发行版常见），请使用 `readlink` 获取真实路径：
   ```bash
   realpath "$(which codex)"            # GNU 系统
   readlink -f "$(which codex)"         # Linux
   readlink "$(which codex)"            # macOS 需逐级解析
   ```
4. 将 Release 中的 `codex` 二进制替换目标文件（保留权限位），完成升级。
