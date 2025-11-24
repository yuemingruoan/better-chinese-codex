# Change: 为 CLI 增加 /sdd-develop 开发流程

## Why
当前 CLI 缺少基于 SDD 的标准化开发流程，无法快速生成任务计划、分支隔离实施并引导用户选择后续操作。

## What Changes
- 新增 `/sdd-develop` 斜杠命令，要求在 Git 仓库中运行。
- 通过预置 prompt 先生成 task.md，再根据用户确认结果进入开发/修改/放弃流程。
- 在确认后指导 AI 创建独立分支编写代码，并提供合并、继续修改、放弃三种后续选项。

## Impact
- Affected specs: cli-workflows
- Affected code: codex-rs/tui slash command处理、prompt占位文件
