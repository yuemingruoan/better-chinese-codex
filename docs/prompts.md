## 自定义提示词

自定义提示词可以把常用指令封装成可复用的 slash 命令，无需重复输入或复制粘贴。每个提示词都是一个 Markdown 文件，当你运行对应命令时，Codex 会立即把文件内容插入对话。

### 存放位置

- **路径**：请把提示词放在 `$CODEX_HOME/prompts/`（默认为 `~/.codex/prompts/`）。如需使用其他目录，设置 `CODEX_HOME` 环境变量即可。
- **文件类型**：Codex 只加载 `.md` 文件，其它类型会被忽略。普通文件与指向 Markdown 的符号链接均受支持。
- **命名**：去掉 `.md` 扩展名后的文件名就是提示词名称。比如 `review.md` 会注册成 `review`。
- **刷新方式**：会话启动时会加载提示词。添加或编辑文件后，请重新启动 Codex（或开启新会话）。
- **命名冲突**：若文件名与内置命令（如 `init`）冲突，该提示词不会出现在 slash 弹窗，但仍可通过 `/prompts:<name>` 手动调用。

### 文件结构

- **正文**：运行提示词时，文件内容会按原样发送（在占位符替换完成后）。
- **Frontmatter（可选）**：可在文件顶部添加 YAML 风格元数据，帮助 slash 弹窗展示信息。

  ```markdown
  ---
  description: Request a concise git diff review
  argument-hint: FILE=<path> [FOCUS=<section>]
  ---
  ```

  - `description` 会显示在弹窗条目下方。
  - `argument-hint`（或 `argument_hint`）可用于描述期望的参数，目前 UI 暂未使用该元数据，但保留写法方便未来扩展。

### 占位符与参数

- **数字占位符**：`$1`–`$9` 会替换为命令后输入的前九个位置参数。`$ARGUMENTS` 会把所有位置参数以单个空格拼接。若要输出字面意义的 `$`，请使用 `$$`（Codex 会保持 `$$` 不变）。
- **命名占位符**：诸如 `$FILE`、`$TICKET_ID` 等标记会根据你提供的 `KEY=value` 对进行替换。键名区分大小写，请在命令中使用相同的大写名称（如 `FILE=...`）。
- **带空格的参数**：若值包含空格，请用双引号包裹，例如 `TICKET_TITLE="Fix logging"`。
- **调用语法**：通过 `/prompts:<name> ...` 运行提示词。在 slash 弹窗中输入 `prompts:` 或直接输入提示词名称，即可看到 `/prompts:<name>` 的建议项。
- **错误处理**：若提示词包含命名占位符，Codex 要求全部提供。若缺失或格式错误，会提示校验失败。

### 如何执行提示词

1. 启动一个新的 Codex 会话（确保提示词列表为最新）。
2. 在输入框中按 `/` 打开 slash 弹窗。
3. 输入 `prompts:`（或直接输入提示词名称）并用 ↑/↓ 选择。
4. 填写所需参数，按 Enter，Codex 就会发送展开后的内容。

### 示例

#### 示例 1：基本的命名参数

**文件**：`~/.codex/prompts/ticket.md`

```markdown
---
description: Generate a commit message for a ticket
argument-hint: TICKET_ID=<id> TICKET_TITLE=<title>
---

Please write a concise commit message for ticket $TICKET_ID: $TICKET_TITLE
```

**使用方式**：

```
/prompts:ticket TICKET_ID=JIRA-1234 TICKET_TITLE="Fix login bug"
```

**展开后发送给 Codex 的内容**：

```
Please write a concise commit message for ticket JIRA-1234: Fix login bug
```

**说明**：`TICKET_ID` 与 `TICKET_TITLE` 都是必填项，缺失会触发校验错误。包含空格的值必须用双引号包裹。

#### 示例 2：混合位置与命名参数

**文件**：`~/.codex/prompts/review.md`

```markdown
---
description: Review code in a specific file with focus area
argument-hint: FILE=<path> [FOCUS=<section>]
---

Review the code in $FILE. Pay special attention to $FOCUS.
```

**使用方式**：

```
/prompts:review FILE=src/auth.js FOCUS="error handling"
```

**展开内容**：

```
Review the code in src/auth.js. Pay special attention to error handling.

```
