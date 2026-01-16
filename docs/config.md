# 配置

基础配置说明请参考：
https://developers.openai.com/codex/config-basic

高级配置说明请参考：
https://developers.openai.com/codex/config-advanced

完整配置参考请参考：
https://developers.openai.com/codex/config-reference

## 连接 MCP 服务器

Codex 可以连接配置在 `~/.codex/config.toml` 中的 MCP 服务器。最新 MCP 配置项请参考配置参考文档：

- https://developers.openai.com/codex/config-reference

## 通知

Codex 可以在代理完成一个回合后运行通知钩子。最新通知设置请参考配置参考文档：

- https://developers.openai.com/codex/config-reference

## 界面语言

在 `~/.codex/config.toml` 中可配置界面与提示语言：

```toml
# 可选值：en / zh-cn
language = "en"
```

当 `language` 缺失或无法识别时，默认使用英文。

## JSON Schema

`config.toml` 对应的 JSON Schema 生成在 `codex-rs/core/config.schema.json`。

## 提示（Notices）

Codex 会在 `[notice]` 表中保存部分 UI 提示的“不要再提示”标记。

通过 Ctrl+C/Ctrl+D 退出时，会使用约 1 秒的双击提示（“再次按下 ctrl + c 退出”）。
