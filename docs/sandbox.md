## 沙箱与审批

Codex 的权限由 **沙箱模式**（无需人工监督即可执行的范围）与 **审批策略**（何时需要你确认）共同决定。本页介绍可选项、它们如何协同，以及各平台的沙箱行为。

### 审批策略

Codex 会以保守策略启动。除非你明确把某个工作目录标记为可信，CLI 会默认 **read-only**：Codex 可以阅读文件并回答问题，但每次编辑或运行命令都需要审批。

当你把工作目录标记为可信（例如在首次引导的提示中或通过 `/approvals` → “信任此目录”），Codex 会把默认预设升级为 **Agent**，允许在工作区内写入。只有在需要离开工作区或在沙箱外重试操作时才会打断你。注意，工作区包括当前工作目录以及 `/tmp` 等临时目录。可通过 `/status` 查看精确的可写根目录。

如果即使在可信仓库也想保持最严格的限制，可在 `/approvals` 选择器中切回 Read Only。若确实需要完全无人值守的自动化，可使用 `Full Access`，但要谨慎——它会同时跳过沙箱与审批。

### 可以完全关闭审批吗？

可以。使用 `--ask-for-approval never` 即可禁用所有审批提示。该选项可以搭配任意 `--sandbox` 模式，因此你仍能决定 Codex 的自治级别，它会在你设定的约束下尽力完成任务。

### 常见的沙箱 + 审批组合

| 目标                               | 参数                                                                                        | 效果                                                                                                                       |
| ---------------------------------- | ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| 安全的只读浏览                     | `--sandbox read-only --ask-for-approval on-request`                                         | Codex 可读文件并回答问题；编辑、运行命令或联网前需要审批。                                                                  |
| 只读的非交互（CI）                 | `--sandbox read-only --ask-for-approval never`                                              | 纯读取，不会升级审批。                                                                                                     |
| 允许修改仓库，如有风险再询问       | `--sandbox workspace-write --ask-for-approval on-request`                                   | 可在工作区读取、编辑并运行命令；离开工作区或联网时需审批。                                                                  |
| Auto（预设，可信仓库）             | `--full-auto`（等价于 `--sandbox workspace-write` + `--ask-for-approval on-request`）       | Codex 在沙箱内写入时不会提示，只有必须离开沙箱时才请求升级。                                                                |
| YOLO（不推荐）                     | `--dangerously-bypass-approvals-and-sandbox`（别名 `--yolo`）                               | 无沙箱、无提示。                                                                                                           |

> 注意：在 `workspace-write` 中，网络默认关闭，除非在配置中显式开启（`[sandbox_workspace_write].network_access = true`）。

#### 在 `config.toml` 中微调

```toml
# 审批模式
approval_policy = "untrusted"
sandbox_mode    = "read-only"

# 全自动模式
approval_policy = "on-request"
sandbox_mode    = "workspace-write"

# 可选：在 workspace-write 中允许联网
[sandbox_workspace_write]
network_access = true
```

你也可以把设置保存为 **profiles**：

```toml
[profiles.full_auto]
approval_policy = "on-request"
sandbox_mode    = "workspace-write"

[profiles.readonly_quiet]
approval_policy = "never"
sandbox_mode    = "read-only"
```

### 各平台的沙箱机制

Codex 依据操作系统选择实现方式：

#### macOS 12+

使用 **Apple Seatbelt**。Codex 会调用 `sandbox-exec` 并传入与 `--sandbox` 对应的 profile，从操作系统层面限制文件系统与网络访问。

#### Linux

结合 **Landlock** 与 **seccomp** API，以提供类似保障。需要内核支持；较旧的内核可能缺乏必要功能。

在容器化的 Linux 环境（例如 Docker）中，如果宿主或容器配置未开放 Landlock/seccomp，沙箱可能失效。这种情况下，可先在容器层面保证隔离，再在容器内以 `--sandbox danger-full-access`（或 `--dangerously-bypass-approvals-and-sandbox`）运行 Codex。

#### Windows

Windows 的沙箱支持仍属实验性质。原理如下：

- 在由 AppContainer profile 派生而来的受限 token 中执行命令。
- 通过向该 profile 附加 capability SID，仅授予显式请求的文件系统权限。
- 通过覆盖代理相关环境变量及插入占位可执行文件，阻断常见工具的出站网络访问。

主要限制：若目录本身对 Everyone SID 具有写权限（如世界可写文件夹），则无法阻止在该目录写入、删除或创建文件。更多讨论与限制见 [Windows Sandbox Security Details](./windows_sandbox_security.md)。

## 体验 Codex 沙箱

可使用以下 CLI 辅助命令测试沙箱中命令的行为：

```
# macOS
codex sandbox macos [--full-auto] [COMMAND]...

# Linux
codex sandbox linux [--full-auto] [COMMAND]...

# 旧版别名
codex debug seatbelt [--full-auto] [COMMAND]...
codex debug landlock [--full-auto] [COMMAND]...
```
