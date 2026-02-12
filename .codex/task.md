# 上游同步任务：rust-v0.99.0

## 1) 标题与目标
在 `develop-main` 基础上合并 OpenAI 官方仓库 `rust-v0.98.0..rust-v0.99.0` 全量更新，并在功能冲突点由你逐项裁决后完成可验证集成（代码可编译、关键测试通过、同步策略不被破坏）。

## 2) 交付物
- 合并后的代码与配置变更（含 `codex-rs`、`codex-cli`、`sdk`、`shell-tool-mcp` 等受影响目录）。
- 按策略更新后的文档与资产（README 保持 fork 版本，其他 docs 与上游对齐并中文化）。
- 冲突裁决清单与处理记录（包含“冲突点-候选方案-你的裁决-落地结果”）。
- 测试与验证记录（格式化、分模块测试、必要时全量测试、快照处理结果）。
- 更新后的 `.codex/task.md` 勾选状态与 `.codex/checkpoint.md` 阶段日志。

## 3) 范围 / 非范围
### 范围
- 合并 `rust-v0.98.0..rust-v0.99.0` 的上游更新，并按仓库策略处理冲突。
- 保留 fork 特性：中文提示词、README、保留工作流、fork 版本号策略。
- 新增/变更的人机交互文本完成中英文 i18n 对齐。
- 对受影响模块执行测试验证，优先先补测试再实现（TDD）。
- 输出可审核的冲突决策点，涉及功能语义冲突时先由你裁决再继续。

### 非范围
- 不直接在 `main` 分支开发或提交。
- 不发布正式 Release（打 tag、触发 release workflow）除非你后续明确要求。
- 不引入与本次上游同步无关的重构或功能扩展。
- 不恢复被策略明确禁用的上游自动化工作流。
- 不在未获你确认前执行 `cargo test --all-features`。

## 4) 工作项清单
| ID | 内容 | 完成状态 | 负责人 | 实施要点 | 验证方式 |
|---|---|---|---|---|---|
| T1 | 基线与分支准备 | [x] | Codex | 1) `git switch develop-main`；2) 新建工作分支（建议 `sync/upstream-rust-v0.99`）；3) 检查工作区状态并记录基线。 | `git branch --show-current` 为工作分支；`git status --short --branch` 无异常未跟踪风险。 |
| T2 | 上游源与版本窗口确认 | [x] | Codex | 1) 配置并校验 OpenAI 官方 `upstream` 远端；2) 拉取 tags；3) 确认 `rust-v0.98.0` 与 `rust-v0.99.0` 都可解析。 | `git remote -v` 出现 `upstream`；`git rev-parse rust-v0.99.0` 成功；`git log --oneline rust-v0.98.0..rust-v0.99.0` 有输出。 |
| T3 | 差异盘点与测试先行计划 | [x] | Codex | 1) 生成变更文件清单并按模块分类（core/tui/tui2/docs/workflows/assets）；2) 标记高风险冲突点；3) 先定义需补充/调整的回归测试项（先测后改）。 | 输出差异盘点清单；每个高风险点绑定至少一个验证命令或测试用例。 |
| T4 | 机械合并与策略过滤 | [x] | Codex | 1) 执行上游合并（merge/cherry-pick 按实际冲突量选择）；2) 按策略过滤：保留中文提示词、保留 README、仅保留允许工作流；3) 文件修改优先 `apply_patch`。 | `git status` 仅剩待处理冲突或已合并变更；`.github/workflows` 仅包含允许文件；README 未被覆盖。 |
| T5 | 功能冲突裁决关卡 | [ ] | 你（用户）+ Codex | 1) Codex 输出“冲突点+候选方案+影响范围+推荐”；2) 你逐项裁决；3) 未裁决项不进入最终提交。 | 每个功能冲突项都有明确“你的决定”；无“语义未定”条目残留。 |
| T6 | 按裁决落地与 i18n 收口 | [ ] | Codex | 1) 落地冲突处理代码；2) 新增/变更交互文本同步 en/zh；3) 文档（README 除外）与上游一致并完成中文化。 | `cargo test -p codex-core i18n::tests::catalogs_share_keys` 通过；`rg` 检查新增 key 在中英文均存在；文档变更可审阅。 |
| T7 | 规范化与分模块验证 | [ ] | Codex | 1) Rust 变更后执行 `just fmt`；2) 大改动按 crate 执行 `just fix -p <project>`；3) 逐模块跑测试（`cargo test -p <crate>`）；4) 快照按 `cargo insta pending-snapshots/show/accept` 流程处理。 | 格式化与 lint 命令成功；受影响 crate 测试通过；无遗留 `.snap.new`（或有明确说明待确认）。 |
| T8 | 全量回归（条件触发） | [ ] | Codex + 你（确认） | 若变更涉及 `common/core/protocol`：先征求你同意，再执行 `cargo test --all-features`；失败则分组定位并回填任务状态。 | 得到你的“允许执行”确认；全量测试通过，或失败项有明确归因与修复/豁免决策。 |
| T9 | 发布素材与收尾同步 | [ ] | Codex | 1) 更新 `docs/release/notes.md`（中英文）记录本轮同步；2) 复核版本号策略（保持 fork 版本，不对齐上游）；3) 清点交付物与剩余风险。 | 发布说明可读且双语一致；版本策略检查通过；交付清单完整。 |
| T10 | 任务状态与阶段记录 | [ ] | Codex | 1) 按实际完成进度更新本文件勾选；2) 追加 `.codex/checkpoint.md` 阶段日志；3) 汇报分支、进展、测试、阻塞。 | `task.md` 勾选与实际一致；checkpoint 追加成功且结构符合规范。 |

### T3 盘点结果（已完成）
- 版本窗口：`rust-v0.98.0..rust-v0.99.0`，上游改动文件 `498` 个。
- 顶层目录分布：`codex-rs 477`、`patches 5`、`.github 5`、`codex-cli 3`、`docs 1`。
- `codex-rs` 主要改动：`core 169`、`app-server-protocol 83`、`tui 70`、`app-server 32`、`network-proxy 15`。
- 与 fork 自身改动重叠文件：`216` 个（高冲突概率），重点集中在 `codex-rs/core`、`codex-rs/tui`、`codex-rs/app-server`。

#### 高风险冲突点与测试先行映射
| 风险点 | 影响范围 | 先行/回归验证命令 | 通过信号 |
|---|---|---|---|
| 工作流策略冲突（上游变更了多条 workflow） | `.github/workflows/*` | `ls .github/workflows` | 仅保留 `build-platform-binaries.yml` 与 `release.yml` |
| README/文档策略冲突 | `README.md`、`docs/**`、`codex-rs/**/README.md` | `git diff -- README.md docs/` | 根 README 不被覆盖；docs 按策略对齐且中文化 |
| fork 特性与上游 core/tui 同时改动 | `codex-rs/core/**`、`codex-rs/tui/**`、`codex-rs/tui2/**` | `cargo test -p codex-core`、`cargo test -p codex-tui`、`cargo test -p codex-tui2` | 关键测试无新增失败，快照差异可解释 |
| 协议层变更引发 API 兼容风险 | `codex-rs/app-server-protocol/**`、`codex-rs/app-server/**` | `cargo test -p codex-app-server-protocol`、`cargo test -p codex-app-server` | 协议测试通过，schema/README 与实现一致 |
| i18n 覆盖遗漏（新增交互文本） | `codex-rs/core/i18n/*.toml`、TUI 文案 | `cargo test -p codex-core i18n::tests::catalogs_share_keys`、`rg -n \"<新 key>\" codex-rs/core/i18n` | en/zh key 成对存在，i18n 测试通过 |

## 5) 里程碑与顺序
- 里程碑 M1：基线与上游窗口确认（T1 -> T2）。
- 里程碑 M2：差异盘点与合并落地准备（T3 -> T4）。
- 里程碑 M3：冲突决策与实现收口（T5 -> T6）。
- 里程碑 M4：验证、发布素材与交付闭环（T7 -> T8 -> T9 -> T10）。

## 6) 风险与缓解
- 上游标签或远端配置异常导致无法准确对齐版本窗口；缓解：先做 tag 可解析性检查，失败时立即停在 T2 并汇报。
- 功能冲突点过多导致合并停滞；缓解：将冲突按“高/中/低影响”分批提交你裁决，优先处理阻塞路径。
- 快照与 i18n 大量漂移造成评审成本高；缓解：先缩小影响范围，逐模块验证并附带变更说明。
- 全量测试耗时长或受环境依赖影响；缓解：先跑 crate 级测试，按失败类别拆分，`--all-features` 在你确认后执行。
- 不慎引入上游禁用工作流或覆盖 README；缓解：在 T4/T9 增加自动检查（路径白名单 + README diff 审核）。

## 7) 验收与测试
- 合并窗口正确：变更来源可追溯到 `rust-v0.98.0..rust-v0.99.0`，无越界提交。
- 策略符合 AGENTS：README 未被上游覆盖；`.github/workflows` 仅保留 `build-platform-binaries.yml` 与 `release.yml`。
- i18n 合格：新增/变更交互文本 en/zh 同步，`catalogs_share_keys` 通过。
- 测试合格：受影响 crate 测试通过；涉及 `common/core/protocol` 时完成（或明确记录）全量测试。
- 产出完整：`docs/release/notes.md`（中英文）更新，`task.md`/`checkpoint.md` 同步到最新状态。

## 8) 回滚与清理
- 合并后发现严重回归：在工作分支使用 `git revert -m 1 <merge_commit>` 回滚对应 merge commit，保留审计轨迹。
- 若仅局部策略误合并：用补丁方式修正（优先 `apply_patch`），避免破坏其他已验证改动。
- 阶段失败需中止：保留冲突清单与失败日志，回退到最近可用 checkpoint 后重新切分任务。
- 收尾清理：删除临时分析文件、处理残留 `.snap.new`、清理不再需要的临时分支（合并后执行）。

## 9) 工具与命令
| 工具/命令 | 何时使用 | 产出 | 注意事项 |
|---|---|---|---|
| `apply_patch` | 代码/文档/配置修改阶段 | 最小化、可审阅补丁 | 优先用于文本修改，避免大段口述；保持改动聚焦。 |
| shell（`git`/`rg`/`just`/`cargo`/`cargo insta`） | 基线检查、合并、验证阶段 | 命令日志与通过信号 | 每条命令注明目的与成功信号；测试遵循“先局部后全量”。 |
| `git switch` / `git branch` | 分支准备与清理 | 隔离开发分支 | 避免在 `main` 直接提交；默认基于 `develop-main`。 |
| `.codex/task.md` 勾选更新 | 每完成一步后 | 最新任务状态 | 状态必须与实际执行一致，不跳步。 |
| `.codex/checkpoint.md` 追加 | 关键阶段完成后 | 可追溯阶段日志 | 采用 `/checkpoint` 规范，记录完成项与待办/风险。 |
| 定期汇报 | 里程碑结束、冲突待裁决、测试完成后 | 用户可决策信息 | 至少同步：计划确认、当前分支、完成/剩余任务、测试结果、阻塞项。 |

建议命令清单（执行阶段按需使用）：
- 基线：`git status --short --branch`、`git branch --show-current`
- 上游：`git remote -v`、`git fetch upstream --tags`、`git log --oneline rust-v0.98.0..rust-v0.99.0`
- 合并：`git merge --no-ff rust-v0.99.0`（或按冲突量改为分批 cherry-pick）
- 格式化/测试：`just fmt`、`just fix -p <project>`、`cargo test -p <crate>`、`cargo insta pending-snapshots -p codex-tui`
- 全量（需你确认）：`cargo test --all-features`

## 10) 测试计划
| 任务/模块 | 测试类型 | 命令 | 预期通过标准与日志信号 |
|---|---|---|---|
| T3 差异盘点阶段 | 基线回归 | `cargo test -p codex-core <filter>`（按影响点） | 基线可复现；若已有失败需先记录为“既有问题”。 |
| T6 冲突落地（core/common/protocol） | 单测+集成 | `cargo test -p codex-core`、`cargo test -p codex-common`、`cargo test -p codex-protocol` | 新旧行为符合裁决；无新增失败。 |
| T6/T7 UI 相关（tui/tui2） | 单测+快照 | `cargo test -p codex-tui`、`cargo test -p codex-tui2`、`cargo insta pending-snapshots -p codex-tui` | 无未确认快照；快照变更有业务解释。 |
| T7 i18n | 配置一致性测试 | `cargo test -p codex-core i18n::tests::catalogs_share_keys` | 中英文 key 完整一致，测试通过。 |
| T8 全量回归（条件触发） | 工作区全量 | `cargo test --all-features` | 在你确认后执行；全部通过或失败项有清晰归因与处置记录。 |
| T9 发布说明与策略检查 | 手测+静态检查 | `rg --files .github/workflows`、`git diff -- README.md docs/release/notes.md` | 工作流白名单满足策略；README 未被上游覆盖；发布说明双语完整。 |

## 11) 汇报清单
- 计划确认要点：版本窗口、分支策略、冲突裁决机制、测试门禁。
- 当前分支名称：开始执行、每次切换、合并完成后都同步一次。
- 已完成任务/剩余任务：按 T1~T10 勾选结果汇报，附下一步动作。
- 测试与验证结果：命令、通过/失败信号、失败归因与处理建议。
- 阻塞或待决事项：需要你裁决的冲突项、是否允许 `cargo test --all-features`、发布节奏相关决策。
