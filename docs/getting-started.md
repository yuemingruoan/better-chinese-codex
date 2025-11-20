## 快速上手

在找特定内容？可直接跳转：

- [技巧与快捷操作](#tips--shortcuts)：热键、会话恢复、提示词
- [非交互运行](./exec.md)：使用 `codex exec` 自动化
- 想深入自定义？前往 [`advanced.md`](./advanced.md)

### CLI 用法

| 命令                | 目的                         | 示例                               |
| ------------------ | ---------------------------- | ---------------------------------- |
| `codex`            | 交互式 TUI                   | `codex`
| `codex "..."`      | 在交互式 TUI 启动时指定提示词 | `codex "修复 lint 错误"`
| `codex exec "..."` | 非交互“自动化”模式           | `codex exec "解释 utils.ts"`

常用标志：`--model/-m`、`--ask-for-approval/-a`。

### 恢复交互会话

- 执行 `codex resume` 打开会话选择器
- 恢复最近一次会话：`codex resume --last`
- 根据 ID 恢复：`codex resume <SESSION_ID>`（可通过 /status 或 `~/.codex/sessions/` 查看）
- 选择器会显示会话记录的原始工作目录、若可用还会展示当时的 Git 分支

示例：

```shell
# 打开最近会话列表
codex resume

# 恢复最新的会话
codex resume --last

# 根据 ID 恢复
codex resume 7f9f9a2e-1b3c-4c7a-9b0e-123456789abc
```

### 直接用提示词启动

可以直接携带提示词运行 Codex CLI：

```shell
codex "向我解释这个代码库"
```

### 示例提示词

以下是可以复制粘贴的示例，把引号中的内容替换成你自己的任务即可。

| ✨  | 输入                                                                              | Codex 的行为                                                        |
| --- | --------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| 1   | `codex "把 Dashboard 组件重构为 React Hooks"`                                         | 重写 class 组件、运行 `npm test` 并展示 diff。                        |
| 2   | `codex "生成新增 users 表的 SQL 迁移"`                                                | 推断 ORM、创建迁移文件并在沙箱数据库中运行。                          |
| 3   | `codex "为 utils/date.ts 编写单元测试"`                                               | 生成测试、执行并迭代直至通过。                                       |
| 4   | `codex "用 git mv 批量把 *.jpeg 重命名为 *.jpg"`                                      | 安全重命名文件并更新引用。                                           |
| 5   | `codex "解释这个正则：^(?=.*[A-Z]).{8,}$"`                                           | 输出逐步的人类可读解释。                                             |
| 6   | `codex "仔细 Review 这个仓库，给出 3 个最有价值且范围清晰的 PR"`                       | 针对当前代码库提出高影响力的 PR 建议。                               |
| 7   | `codex "检查漏洞并生成安全审计报告"`                                                   | 发现并解释安全问题。                                                 |

想复用自定义指令？可以通过 [自定义提示词](./prompts.md) 创建 slash 命令。

### 通过 AGENTS.md 扩展记忆

你可以借助 `AGENTS.md` 为 Codex 提供额外说明。Codex 会按以下顺序查找并自上而下合并：

1. `~/.codex/AGENTS.md`：个人全局指导
2. 从仓库根目录到当前工作目录（含）之间的每个目录。每一层优先寻找 `AGENTS.override.md`，若不存在则使用 `AGENTS.md`。当你需要在该目录下替换继承的指令时，请使用 override 版本。

关于 AGENTS.md 的更多使用方式，请参阅 [官方 AGENTS.md 文档](https://agents.md/)。

### Tips & shortcuts

#### 使用 `@` 搜索文件

输入 `@` 会在工作区根目录触发模糊文件名搜索。可用上下方向键选择结果，用 Tab 或 Enter 将 `@` 替换为选定路径。按 Esc 可取消搜索。

#### Esc–Esc 编辑上一条消息

当输入框为空时，按一次 Esc 进入“回溯”模式。再次按 Esc 将打开历史消息预览，并高亮最近的用户消息；重复按 Esc 可向更早的消息移动。按 Enter 确认后，Codex 会从该节点分叉对话、裁剪可见历史，并把所选消息填入输入框，方便你修改后重新提交。

在预览中，底部会显示 `Esc edit prev` 提示，表示当前处于可编辑状态。

#### `--cd`/`-C`

有时不便提前 `cd` 到希望的“工作根目录”。`codex` 提供 `--cd` 选项，你可以指定任意文件夹。可在新会话开始时查看 TUI 顶部显示的 **workdir**，确认 `--cd` 生效。

#### `--add-dir`

需要在一次运行中跨多个项目工作？多次传入 `--add-dir` 可把额外目录暴露为可写根路径，同时保持主工作目录不变。例如：

```shell
codex --cd apps/frontend --add-dir ../backend --add-dir ../shared
```

此后 Codex 可以在列出的每个目录中查看与编辑文件，而无需离开主工作区。

#### Shell 补全

生成 shell 补全脚本：

```shell
codex completion bash
codex completion zsh
codex completion fish
```

#### 图片输入

可直接把图片粘贴到输入框（Ctrl+V / Cmd+V）以附加到提示中。通过 CLI 也可以用 `-i/--image`（逗号分隔）附加图片：

```bash
codex -i screenshot.png "帮我解释这个错误"
codex --image img1.png,img2.jpg "总结这些图表"
```
