# feat: Add an i18n interface to Codex CLI — partial prototype completed

> Note: The English version was translated with GPT.

## Preface

Although the Codex app already has multilingual i18n support, I personally prefer using CLI tools. I am also a little obsessive about keeping the tools I use in the best possible state—especially when it comes to language support.

I felt the same way when using Claude Code. I previously added Windows support to a language plugin:

https://github.com/taekchef/claude-code-zh-cn/pull/11

Unfortunately, the Codex CLI localization plugins I could find were either no longer maintained or simply did not work.

So I decided to build one myself—or, more accurately, started pushing Codex to help me build one. However, as I investigated the requirement more deeply, I came to believe that adding a small i18n interface directly at the Rust layer would be the cleanest solution. It would also save a great deal of duplicated work for similarly language-obsessed users and maintainers of other locales.

If I implement everything only as a plugin, I am effectively compiling another Codex binary to replace the official one. I would then have to chase every upstream release, rebase the patch, compile it again, and ask every future user of the plugin to do the same. That sounds exhausting.

The contribution guide asks feature proposals to begin as issues, and external pull requests are invitation-only. So after going around in circles, I am here writing an issue. I realize this may receive no response and that, in the worst case, I will have to keep following upstream versions and compiling my own Codex forever—while forcing any future users of my plugin to do the same. Assuming anyone would actually use it, of course.

BTW, the Codex CLI interface is also a little plain. Improving that experience is another direction I would eventually like the plugin to explore, although it is separate from the core i18n proposal here.

> **Screenshot placeholder — main interface:** `主界面截图.png`

> **Screenshot placeholder — status page:** `status.png`

The following technical details were prepared with help from Codex.

## Current state

I have a working prototype in my fork:

https://github.com/Cec1c/codex

The fork was synchronized with upstream `main` around the Codex `0.144.5` release. The current prototype:

- adds a process-wide `Localizer` to the TUI;
- uses Mozilla Fluent for translated resources;
- keeps English source text compiled at each Rust call site as the final fallback;
- includes a built-in Simplified Chinese locale using the canonical locale code `zh-Hans`;
- accepts `zh`, `zh-CN`, and `chinese` as compatible aliases and normalizes them to `zh-Hans`;
- adds `/language`, `/language zh-Hans`, and `/language en`;
- stores the selected locale in `$CODEX_HOME/ui-language`;
- supports `CODEX_UI_LANGUAGE` as a process-only override for development and testing;
- currently requires a restart after changing the language because the global localizer is initialized once;
- includes 134 Fluent messages in the current `zh-Hans` catalog;
- provides a test-only `--i18n-self-check` command for deterministic validation.

The visible prototype currently covers, among other things:

- the startup/session card, including model, directory, permissions, and model-change hints;
- `/status`, usage information, limits, Context, and Token text;
- slash-command descriptions and the `/` command popup;
- composer placeholders such as `Write tests for @filename`;
- approval dialogs and onboarding choices;
- tips such as the `/rename` message;
- MCP startup warning prefixes;
- unknown slash-command errors;
- selected history and progress messages.

This is still a prototype rather than a finished general-purpose localization system. In particular, locale discovery, runtime catalog installation, hot language switching, and plugin-marketplace integration have not been implemented.

## Files changed

The main framework files are:

- `codex-rs/tui/src/i18n.rs` — runtime localizer, locale normalization, preference loading, Fluent lookup, and English fallback;
- `codex-rs/tui/src/i18n_tests.rs` — fallback, formatting, invalid resource, locale, and self-check tests;
- `codex-rs/tui/i18n/zh-Hans.ftl` — the current Simplified Chinese catalog;
- `codex-rs/tui/src/slash_command.rs` — `/language` command metadata and localized slash-command descriptions;
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs` — `/language` command dispatch;
- `codex-rs/tui/src/lib.rs` — process-wide localizer initialization and the test-only self-check entry;
- `codex-rs/cli/src/main.rs` — CLI forwarding for the self-check path;
- `codex-rs/tui/Cargo.toml`, `codex-rs/Cargo.toml`, and `codex-rs/Cargo.lock` — Fluent and locale dependencies.

Representative TUI call sites were also updated, including:

- session and status cards;
- the status line and footer;
- the command popup and composer;
- approval overlays;
- onboarding;
- tooltips;
- MCP startup warnings;
- common history and error surfaces.

The intention is not that an initial upstream PR must include all of these call sites. The fork uses a wider surface to demonstrate the result. A reviewable first PR could contain only the core interface, `/language`, several representative strings, and focused tests.

## Rust-layer interface

The main API deliberately keeps the original English expression at the Rust call site:

```rust
localizer.text(
    "session-card-model-label",
    None,
    || "model:".to_string(),
)
```

Parameterized text passes dynamic values through Fluent arguments while retaining the complete English fallback:

```rust
localizer.text_with_string_arg(
    "mcp-client-failed-to-start",
    "name",
    server_name,
    || format!("MCP client for `{server_name}` failed to start"),
)
```

The current `Localizer` has four important behaviors:

1. `Localizer::english()` creates a no-catalog localizer and preserves the current English UI.
2. `Localizer::from_ftl(locale, source)` parses a Fluent resource and disables localization safely if the locale or resource is invalid.
3. `Localizer::from_runtime()` resolves the locale from `CODEX_UI_LANGUAGE` or `$CODEX_HOME/ui-language`, then loads the built-in resource.
4. `text()` and `text_with_string_arg()` translate one message at a time and call the English closure whenever translation cannot be completed safely.

The per-message fallback contract is:

- no selected locale → English;
- unsupported or invalid locale → English;
- invalid Fluent resource → English;
- missing message key → English for that message;
- missing Fluent argument → English for that message;
- formatting error → English for that message;
- empty or whitespace-only translation → English for that message.

The localization layer must never display a raw message key, a partially formatted Fluent template, or an empty string.

Model names, paths, URLs, MCP server names, percentages, token values, and low-level error details remain runtime data. Only the stable user-facing shell is translated. Existing Ratatui `Span`, `Line`, and `Style` composition remains unchanged.

This design also means an English FTL file is not required: the existing Rust strings remain the source of truth and the final compatibility layer.

## What maintainers of other languages would see and maintain

For the current prototype, most translation work is isolated to one Fluent file:

```text
codex-rs/tui/i18n/<locale>.ftl
```

A message looks like this:

```ftl
session-card-model-label = 模型：
mcp-client-failed-to-start = MCP 客户端 `{ $name }` 启动失败
status-line-context-remaining = 上下文剩余 { $percent }%
```

Language maintainers should translate only stable user-facing text. They should not translate model identifiers, paths, URLs, server names, numeric values, protocol content, or detailed low-level errors.

In the current prototype, adding another built-in locale still requires a small Rust change:

1. add `codex-rs/tui/i18n/<locale>.ftl`;
2. embed or register the resource in `i18n.rs`;
3. add canonical locale and alias normalization;
4. expose the locale through `/language`;
5. add focused formatting and fallback tests.

The longer-term goal would be to replace the hard-coded locale match with a registry or provider interface. At that point, a language pack could potentially contain only metadata plus an FTL resource, making it suitable for installation from a plugin marketplace without recompiling Codex itself.

That registry/provider work has not started yet, so I do not want to present it as an implemented feature.

## Entry point

The current user-facing entry point is:

```text
/language
```

It displays the current language and the available usage. The prototype currently supports:

```text
/language zh-Hans
/language en
```

Changing the language writes the canonical locale to:

```text
$CODEX_HOME/ui-language
```

Developers can also test a locale for one process:

```text
CODEX_UI_LANGUAGE=zh-Hans codex
```

The current process must be restarted after changing the persistent language.

> **Screenshot placeholder — language entry:** `language.png`

My longer-term idea is to make language packs selectable in a plugin-marketplace-like form. That part has not been developed yet.

## What I hope to discuss

I would like to know whether maintainers consider this kind of small, fallback-safe Rust i18n boundary compatible with the direction of Codex CLI.

If the overall direction is acceptable, I can reduce the current fork into a much smaller exploratory PR containing only:

1. the minimal `Localizer` and Fluent dependencies;
2. the English per-message fallback contract;
3. `/language` and locale preference loading;
4. one example locale or only the locale-provider seam;
5. several representative TUI call sites and focused tests.

---

# feat：为 Codex CLI 添加 i18n 接口——我已完成部分原型

> 说明：英文版本使用 GPT 翻译。

## 前言

尽管 Codex App 已经完成了多语言 i18n 功能，但我实际上更倾向于使用 CLI 工具。再由于我的一部分强迫症，我希望自己的工具总是处于最好的状态，尤其是语言支持。

使用 Claude Code 的时候我也是如此，并曾经为一个语言插件添加 Windows 版本支持：

https://github.com/taekchef/claude-code-zh-cn/pull/11

遗憾的是，现在我能找到的 Codex CLI 汉化插件要么早就不维护了，要么根本不能使用。

于是我决定自己动手——准确地说，是开始 push Codex——打算写一个插件。但是随着我深入研究这个需求，我逐渐认为，直接在 Rust 层添加一个小型 i18n 接口可能才是最优雅的方案，也能让其他语言的强迫症患者和语言维护者省下大量重复工作。

只不过，如果我完全在插件中实现，这意味着我事实上是在自己编译另一个 Codex 来替换官方 Codex。以后每个上游版本发布，我都需要追着版本重新适配、编译，并强迫未来其他使用我插件的人也这么做。这实在太折磨了。

由于贡献指南要求新功能先提交 Issue，并且外部 PR 只能由维护者邀请，所以兜兜转转，我还是来写 Issue 了。尽管大概率没人理我，最不济我只能自己追着版本跑，继续编译自己的 Codex，并强迫未来其他使用我插件的人也这么干——当然，真的会有人用吗？

BTW，Codex CLI 的界面实在有点素，这也是我的插件未来想优化的另一个方向。不过它与这里讨论的核心 i18n 接口可以分开处理。

> **截图占位——主界面：** `主界面截图.png`

> **截图占位——状态页面：** `status.png`

接下来的一部分技术细节由 Codex 协助整理。

## 现状

我已经在自己的 fork 中完成了一个可以运行的原型：

https://github.com/Cec1c/codex

这个 fork 已经在 Codex `0.144.5` 发布前后同步上游 `main`。当前原型：

- 在 TUI 中加入进程级全局 `Localizer`；
- 使用 Mozilla Fluent 管理翻译资源；
- 每个 Rust 调用点仍保留编译进二进制的原始英文，作为最终回退；
- 内置一个使用规范语言代码 `zh-Hans` 的简体中文语言包；
- 兼容输入 `zh`、`zh-CN` 和 `chinese`，并统一规范化为 `zh-Hans`；
- 新增 `/language`、`/language zh-Hans` 和 `/language en`；
- 把语言偏好保存在 `$CODEX_HOME/ui-language`；
- 支持通过 `CODEX_UI_LANGUAGE` 对当前进程临时覆盖，方便开发和测试；
- 由于全局 Localizer 当前只初始化一次，修改语言后需要重启；
- 当前 `zh-Hans` 语言包包含 134 个 Fluent 消息；
- 提供测试专用的 `--i18n-self-check`，用于稳定的自动化验证。

目前已经能实际看到中文效果的范围包括：

- 启动/会话卡片中的模型、目录、权限和模型切换提示；
- `/status`、用量、限制、Context 和 Token 文本；
- 斜杠命令说明和 `/` 命令面板；
- `Write tests for @filename` 等输入占位；
- 审批窗口和登录引导；
- `/rename` 等 Tip；
- MCP 启动警告前缀；
- 输入不存在的斜杠命令后的错误；
- 部分历史记录和进度消息。

这仍然只是一个原型，并不是已经完成的通用本地化系统。尤其是 locale 自动发现、运行时安装语言包、无需重启热切换、插件商店接入等能力目前都还没有实现。

## 改动的文件

框架的主要文件包括：

- `codex-rs/tui/src/i18n.rs`：运行时 Localizer、locale 规范化、偏好读取、Fluent 查找和英文回退；
- `codex-rs/tui/src/i18n_tests.rs`：回退、参数格式化、无效资源、locale 和自检测试；
- `codex-rs/tui/i18n/zh-Hans.ftl`：当前简体中文语言包；
- `codex-rs/tui/src/slash_command.rs`：`/language` 命令元数据和斜杠命令说明翻译；
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`：`/language` 命令分发；
- `codex-rs/tui/src/lib.rs`：进程级 Localizer 初始化以及测试专用自检入口；
- `codex-rs/cli/src/main.rs`：CLI 自检路径转发；
- `codex-rs/tui/Cargo.toml`、`codex-rs/Cargo.toml` 和 `codex-rs/Cargo.lock`：Fluent 与 locale 依赖。

同时修改了一批代表性的 TUI 调用点，包括：

- 启动/会话卡片和状态卡片；
- 状态栏和底部栏；
- 命令面板和输入框；
- 审批窗口；
- 登录引导；
- Tip；
- MCP 启动警告；
- 常用历史记录和错误提示。

这并不意味着第一次上游 PR 必须包含所有调用点。当前 fork 的范围较宽，是为了展示实际效果。真正便于审阅的第一个 PR 可以只保留核心接口、`/language`、少量代表性文本和聚焦测试。

## Rust 层接口

核心 API 会在 Rust 调用点保留原始英文表达式：

```rust
localizer.text(
    "session-card-model-label",
    None,
    || "model:".to_string(),
)
```

包含运行时数据的消息通过 Fluent 参数传值，同时仍然保留完整英文回退：

```rust
localizer.text_with_string_arg(
    "mcp-client-failed-to-start",
    "name",
    server_name,
    || format!("MCP client for `{server_name}` failed to start"),
)
```

当前 `Localizer` 有四个主要行为：

1. `Localizer::english()` 创建不加载语言包的 Localizer，完整保留现有英文界面；
2. `Localizer::from_ftl(locale, source)` 解析 Fluent 资源，locale 或资源无效时安全关闭翻译；
3. `Localizer::from_runtime()` 从 `CODEX_UI_LANGUAGE` 或 `$CODEX_HOME/ui-language` 解析语言，并加载内置资源；
4. `text()` 与 `text_with_string_arg()` 逐条翻译消息，任何无法安全完成翻译的情况都会调用英文闭包。

逐条消息的回退规范为：

- 未选择语言 → 英文；
- 不支持或无效的 locale → 英文；
- Fluent 资源无效 → 英文；
- 消息键缺失 → 当前消息回退英文；
- Fluent 参数缺失 → 当前消息回退英文；
- 格式化错误 → 当前消息回退英文；
- 翻译为空或只有空白 → 当前消息回退英文。

本地化层不应向用户显示原始消息键、未完整格式化的 Fluent 模板或空字符串。

模型名、路径、URL、MCP 服务器名、百分比、Token 数量和底层错误详情仍然是运行时数据。翻译只处理稳定的用户可见外壳，并保留原有 Ratatui `Span`、`Line` 和 `Style` 组合。

这也意味着不需要单独维护英文 FTL 文件：现有 Rust 英文字符串继续作为事实来源和最终兼容层。

## 其他语言维护者会看到什么、需要维护什么

在当前原型中，大部分翻译工作集中在一个 Fluent 文件：

```text
codex-rs/tui/i18n/<locale>.ftl
```

消息格式类似：

```ftl
session-card-model-label = 模型：
mcp-client-failed-to-start = MCP 客户端 `{ $name }` 启动失败
status-line-context-remaining = 上下文剩余 { $percent }%
```

语言维护者只需要翻译稳定的用户可见文本，不应翻译模型标识、路径、URL、服务器名、数值、协议内容或详细的底层错误。

在当前原型中，新增一个内置语言仍然需要少量 Rust 接线：

1. 添加 `codex-rs/tui/i18n/<locale>.ftl`；
2. 在 `i18n.rs` 中嵌入或注册该资源；
3. 增加规范 locale 和兼容别名；
4. 通过 `/language` 暴露该语言；
5. 添加聚焦的参数格式化和英文回退测试。

更长远的目标是把硬编码的 locale 匹配替换成注册表或 provider 接口。到那时，一个语言包也许只需要包含元数据和 FTL 资源，就能通过类似插件商店的方式安装，而不需要重新编译 Codex 本体。

这部分注册表/provider 工作目前还没有开始，因此我不想把它描述成已经实现的功能。

## 入口

当前面向用户的入口是：

```text
/language
```

它会显示当前语言和使用方式。当前原型支持：

```text
/language zh-Hans
/language en
```

修改语言时，会把规范 locale 写入：

```text
$CODEX_HOME/ui-language
```

开发者也可以只为当前进程测试某种语言：

```text
CODEX_UI_LANGUAGE=zh-Hans codex
```

修改持久化语言后，当前进程需要重启。

> **截图占位——语言入口：** `language.png`

我更长远的计划是把语言包做成类似插件商店的可选择形式，这部分工作目前还没有展开。

## 我希望讨论的问题

我想了解维护者是否认为这种小型、逐条安全回退的 Rust i18n 边界符合 Codex CLI 的方向。

如果整体方向可以接受，我可以把当前 fork 缩减成一个更小的探索性 PR，只包含：

1. 最小 `Localizer` 与 Fluent 依赖；
2. 逐条消息的英文回退规范；
3. `/language` 与 locale 偏好读取；
4. 一个示例语言，或者只保留 locale provider 接口；
5. 少量代表性 TUI 调用点和聚焦测试。
