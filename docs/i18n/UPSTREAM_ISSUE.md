# Draft upstream issue: Add an incremental i18n boundary to the Codex TUI

> Draft only. This file is intended to be reviewed before opening an issue in `openai/codex`.

## Suggested title

**Proposal: an incremental, fallback-safe i18n framework for the Codex TUI**

## English

### Summary

Would the maintainers be open to an incremental internationalization boundary for the Codex TUI?

I have implemented a working prototype in a fork, synchronized with upstream `main` at the time Codex `0.144.5` was published. It uses Mozilla Fluent, keeps English at the original Rust call sites, and lets each message fall back independently. The intention is not to translate the whole TUI in one large change. It is to introduce a small interface that allows individual user-visible strings and languages to be added safely over time.

### Motivation

Codex already has many stable user-facing strings in session cards, status views, slash-command descriptions, composer placeholders, approval dialogs, onboarding, and errors. Directly replacing these strings with catalog lookups would make failures harder to reason about and would create a large all-or-nothing migration.

The prototype instead treats localization as an optional presentation layer:

- English remains compiled into the binary at every localized call site.
- Missing or invalid translations affect only that message.
- Model prompts, protocol messages, command execution, and other non-UI behavior are unchanged.
- Adoption can start with a few representative surfaces and expand incrementally.

### Proposed interface

The central API is intentionally small:

```rust
localizer.text(
    "session-card-model-label",
    None,
    || "model:".to_string(),
)
```

Messages with runtime values use Fluent arguments while retaining the complete English fallback:

```rust
localizer.text_with_string_arg(
    "mcp-client-failed-to-start",
    "name",
    server_name,
    || format!("MCP client for `{server_name}` failed to start"),
)
```

This provides the following contract:

1. The Rust call site remains the source of truth for English behavior.
2. Translation resources contain only localized alternatives.
3. Dynamic values such as model names, paths, URLs, MCP server names, percentages, token counts, and low-level error details remain program data.
4. Existing Ratatui `Span`, `Line`, and `Style` composition remains intact; only stable text fragments cross the localization boundary.
5. A missing key, missing argument, invalid locale, Fluent parse/format error, or empty translation falls back to the English closure for that one message.

The localization layer must never render a raw key, a partially formatted template, or an empty string.

### Language selection

The prototype adds:

- `/language` to show the active locale and usage;
- `/language zh-Hans` and `/language en` to select a locale;
- `$CODEX_HOME/ui-language` as the persistent preference;
- `CODEX_UI_LANGUAGE` as an optional process-only override for development and testing.

English remains the default. The aliases `zh`, `zh-CN`, and `chinese` are accepted and normalized to the canonical locale `zh-Hans`. A restart is currently required because the process-wide localizer is initialized once.

### Current prototype and visible result

The fork currently contains one built-in `zh-Hans.ftl` resource with 134 Fluent message entries. The implemented demo coverage includes:

- the startup/session card (`model`, `directory`, `permissions`, and the model-change hint);
- the `/status` card and usage/limits messages;
- status-line and footer Context/Token text;
- slash-command descriptions and the `/` command popup;
- `/language` status, selection, validation, and restart messages;
- composer placeholders such as `Write tests for @filename`;
- approval overlays;
- onboarding and sign-in choices;
- tips such as the `/rename` hint;
- MCP startup warning prefixes while preserving server names and detailed transport errors;
- unknown slash-command errors;
- selected history and progress text.

There is also a test-only `--i18n-self-check` entry point that emits representative localized messages as JSON and includes an explicit missing-key fallback probe. This flag is for deterministic automated validation, not intended as a user-facing command.

### Suggested upstream scope

The fork is intentionally broad enough to demonstrate the UX. A first upstream PR could be much smaller:

1. the minimal `Localizer` and Fluent dependencies;
2. compiled English per-message fallback behavior;
3. locale preference loading and a small `/language` surface;
4. one built-in example locale, or only the resource-loading seam if maintainers prefer;
5. a few representative call sites and focused fallback tests.

The fork's installer, updater, compatibility policy, and release workflow are not part of this proposal.

### Questions for maintainers

1. Is this fallback-at-the-call-site model compatible with the project's preferred direction?
2. Would maintainers prefer the initial contribution to include `/language`, or begin only with the localization boundary and tests?
3. Should locale resources be compiled into the TUI crate initially, or loaded through a different project-owned mechanism?
4. If the direction is acceptable, would a small exploratory PR be useful?

I can prepare a reduced PR that keeps the review surface intentionally small.

---

## 中文

### 概要

想请教维护者是否愿意考虑为 Codex TUI 引入一层可渐进接入的国际化边界。

我已经在 fork 中完成了一个可运行原型，并在 Codex `0.144.5` 发布时同步了上游 `main`。它使用 Mozilla Fluent，同时把英文保留在原始 Rust 调用点，并且让每条消息都能独立回退。这个方案并不打算通过一次大改动翻译整个 TUI，而是先提供一个小而稳定的接口，让用户可见文本和新语言能够逐步、安全地加入。

### 动机

Codex 的启动卡片、状态页面、斜杠命令说明、输入占位、审批窗口、登录引导和错误提示中已经存在许多稳定的用户可见文本。如果直接把这些字符串批量替换成语言包查找，会让故障更难定位，也会形成一次性迁移的压力。

这个原型把本地化视为一个可选的展示层：

- 每个已本地化调用点仍把英文编译进二进制；
- 缺失或错误的翻译只影响当前一条消息；
- 模型提示词、协议消息、命令执行和其他非 UI 行为不变；
- 可以从少量代表性界面开始，再逐步扩大范围。

### 建议的接口规范

核心 API 保持很小：

```rust
localizer.text(
    "session-card-model-label",
    None,
    || "model:".to_string(),
)
```

包含运行时数据的消息通过 Fluent 参数传值，同时保留完整英文回退：

```rust
localizer.text_with_string_arg(
    "mcp-client-failed-to-start",
    "name",
    server_name,
    || format!("MCP client for `{server_name}` failed to start"),
)
```

这套接口遵循以下约定：

1. Rust 调用点仍是英文行为的事实来源；
2. 翻译资源只保存其他语言的替代文本；
3. 模型名、路径、URL、MCP 服务器名、百分比、Token 数量和底层错误详情仍作为程序数据传入；
4. 保留现有 Ratatui 的 `Span`、`Line` 和 `Style` 组合，只让稳定文本片段经过本地化边界；
5. 缺键、缺少参数、locale 无效、Fluent 解析或格式化失败、翻译为空时，只让当前消息回退到英文闭包。

本地化层不应把原始键名、未完整格式化的模板或空字符串显示给用户。

### 语言选择

原型增加了：

- `/language`：查看当前语言和使用方式；
- `/language zh-Hans` 与 `/language en`：选择语言；
- `$CODEX_HOME/ui-language`：持久化语言偏好；
- `CODEX_UI_LANGUAGE`：开发和测试时可选的单进程覆盖。

默认语言仍是英文。兼容输入 `zh`、`zh-CN` 和 `chinese`，保存时会统一规范化为 `zh-Hans`。当前进程级 Localizer 只初始化一次，所以切换后需要重启 Codex。

### 当前原型与实际效果

fork 目前内置一个 `zh-Hans.ftl`，包含 134 个 Fluent 消息条目。已经能演示的中文范围包括：

- 启动/会话卡片中的 `model`、`directory`、`permissions` 和模型切换提示；
- `/status` 卡片以及用量、限制相关说明；
- 状态栏和底部栏中的 Context/Token 文本；
- 斜杠命令说明和 `/` 命令面板；
- `/language` 的状态、选择、校验和重启提示；
- `Write tests for @filename` 等输入占位；
- 审批窗口；
- 登录引导和登录选项；
- `/rename` 等 Tip；
- MCP 启动警告前缀，同时保留服务器名和底层传输错误；
- 不存在的斜杠命令错误；
- 部分历史记录和进度文本。

原型还提供测试专用的 `--i18n-self-check`，用 JSON 输出代表性翻译，并显式检查缺键时的英文回退。它用于稳定的自动化验证，不打算作为面向普通用户的命令。

### 建议的上游首个改动范围

fork 为了展示效果，覆盖范围有意做得较宽。真正提交给上游的第一个 PR 可以缩小为：

1. 最小 `Localizer` 与 Fluent 依赖；
2. 每条消息保留编译期英文回退；
3. locale 偏好读取和一个很小的 `/language` 入口；
4. 一个内置示例语言，或者按维护者意见只提供资源加载边界；
5. 少量代表性调用点和聚焦的回退测试。

fork 中的安装器、更新器、兼容策略和发布流程不属于本提案。

### 希望向维护者确认的问题

1. 这种“在调用点保留英文回退”的模型是否符合项目方向？
2. 首次贡献是否应包含 `/language`，还是只提交本地化边界与测试？
3. 初期语言资源适合直接编译进 TUI crate，还是应通过其他项目统一的机制加载？
4. 如果方向可以接受，是否欢迎一个刻意缩小范围的探索性 PR？

我可以据此准备一个审阅面尽量小的 PR。
