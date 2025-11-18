# Codex CLI 中文指南

## 项目简介
Codex CLI 是一套跨平台的开发辅助工具链，内含命令行界面、TUI UI、文件搜索、MCP 客户端等多个子组件。本仓库 `codex-rs` 提供 Rust 实现版本，适用于自动化执行、交互式开发以及与 AI 协作的场景。

## 目录约定
- `.codex/AGENTS.md`：AI 协作者在当前项目中的行为记录与说明
- `.codex/checkpoint.md`：Codex自动维护的开发日志，用于记录·每一步操作的结果、待办事项与风险提示。
- `.codex/PROMPT.md`:执行init时额外给予AI的提示词，包含想要实现的需求，注意事项，甚至项目结构

## 环境准备
1. 安装 Git ≥ 2.23（建议）。
2. 系统要求：macOS 12+/Linux（Ubuntu 20.04+/Debian 10+）/Windows 11（需 WSL2）。
3. 内存至少 4 GB（推荐 8 GB）。
4. 若需参与开发，可酌情安装 Rust 工具链与 `just`、`rg` 等辅助工具；仅使用发布版二进制时无需这些依赖。

## 安装 / 更新指南

### 1. 获取最新版本
- 打开本仓库的[发布页面](https://github.com/yuemingruoan/better-chinese-codex/releases)，Release 标题通常为 `v1.3.0`、`v1.2.1` 这类语义化版本号。
- 在最新 Release 的 **Assets** 区域中，根据平台下载对应压缩包或可执行文件（Windows、macOS、Linux 皆会提供，各文件名已在说明中标注）。
- 解压得到的二进制文件后：
  - Windows 重命名为 `codex.exe`；
  - macOS / Linux 重命名为 `codex`。
- 后续按平台指南替换旧版本即可完成升级。

### 2. Windows（搭配 npm 全局安装）
```powershell
npm install -g @openai/codex-cli
```
- npm 全局目录通常位于 `%APPDATA%\npm`。
- 将下载得到的 `codex.exe` 放入 `%APPDATA%\npm` 目录，并替换与 `codex.cmd` 同级的旧版本（务必保留 `.cmd` 启动脚本）。

### 3. macOS / Linux
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
4. 使用从发布页下载的最新 `codex` 可执行文件替换目标路径中的旧版本，并保持原有权限位即可完成升级。
