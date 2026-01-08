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
