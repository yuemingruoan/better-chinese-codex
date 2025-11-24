## ADDED Requirements
### Requirement: SDD 开发流程命令
CLI SHALL 提供 `/sdd-develop` 斜杠命令，引导基于 SDD 的计划生成、分支化开发与合并/放弃流程。

#### Scenario: 未在 Git 仓库时给出阻止提示
- **WHEN** 用户运行 `/sdd-develop` 且当前目录不是 Git 仓库
- **THEN** CLI SHALL 提示需在 Git 仓库内使用，并不触发后续流程

#### Scenario: 生成任务计划并等待用户确认
- **WHEN** 用户在 Git 仓库中运行 `/sdd-develop <需求描述>`
- **THEN** CLI SHALL 将需求描述填入预置 prompt，指导 AI 生成 `task.md`
- **AND** CLI SHALL 提示用户选择“同意计划，继续开发”或“不同意计划，修改计划”

#### Scenario: 计划被拒绝时重新生成
- **WHEN** 用户选择“不同意计划，修改计划”
- **THEN** CLI SHALL 请求 AI 依据用户反馈更新 `task.md`，并再次提供同意/不同意选项

#### Scenario: 确认计划后分支化开发
- **WHEN** 用户同意计划
- **THEN** CLI SHALL 发送二次预置 prompt，要求 AI 创建独立分支并按 `task.md` 实施改动
- **AND** 完成后 CLI SHALL 提供“使用 Pull Request 合并分支”“继续修改”“放弃修改（删除分支）”三项选项

#### Scenario: 合并或放弃时使用对应提示词
- **WHEN** 用户在完成开发后选择“使用 Pull Request 合并分支”
- **THEN** CLI SHALL 发送合并用 prompt 指导 AI 完成 PR 合并相关步骤并回到正常交互
- **WHEN** 用户选择“放弃修改（删除分支）”
- **THEN** CLI SHALL 发送放弃用 prompt 指导 AI 删除分支并回到正常交互
