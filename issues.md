# feat: Add an i18n interface to Codex CLI; I have completed part of the prototype

## Preface:

> The English version was translated with GPT.

Although the Codex app has already implemented multilingual i18n, I actually prefer using CLI tools. Also, because of some of my obsessive-compulsive tendencies, I want my tools to always be in the best possible state, especially when it comes to language.

I was the same when using Claude Code, and I added Windows support to a language plugin:

https://github.com/taekchef/claude-code-zh-cn/pull/11

Unfortunately, the plugins I can find now are either no longer maintained or simply do not work.

So I started doing it myself (`push Codex`) and planned to write a plugin. However, as I studied this requirement more deeply, adding an i18n interface directly at the Rust layer may be the most elegant solution, and it could also save a lot of work for obsessive users of other languages.

However, if I implement it myself as a plugin, this means I am effectively compiling another Codex to replace the original Codex. I would also have to chase every version. This is too painful, and it would force future users of my plugin to do the same.

Because the contribution guide does not allow me to submit a PR directly, after going around in circles I still came here to write an issue. Although there is a high probability that nobody will pay attention to me, in the worst case I can only keep chasing versions and compiling Codex myself, while forcing future users of my plugin to do the same. (Will anyone really use it?)

BTW, the Codex CLI interface is really a little plain. This is also a direction that I want my plugin to improve.

-主界面截图.png

-status.png

I may let Codex explain some of the following technical details directly:

### Current status

The current prototype has been implemented in my Codex fork and synchronized with upstream `main` around the Codex `0.144.5` release.

The prototype currently includes:

- a process-wide `Localizer` in the TUI;
- Mozilla Fluent as the translation resource format;
- a built-in `zh-Hans` Simplified Chinese resource containing 134 messages;
- English text retained at every Rust call site as the final fallback;
- `/language`, `/language zh-Hans`, and `/language en`;
- compatible aliases `zh`, `zh-CN`, and `chinese`, normalized to `zh-Hans`;
- persistent language selection through `$CODEX_HOME/ui-language`;
- a temporary process override through `CODEX_UI_LANGUAGE`;
- a test-only `--i18n-self-check` entry for automated validation.

The currently translated visible text includes the startup card, `/status`, status-line Context and Token text, slash-command descriptions, the `/` command list, composer placeholders, approval dialogs, onboarding, Tip messages, MCP startup warning prefixes, unknown-command errors, and some history/progress messages.

This is still a prototype. Runtime installation of external language packs, automatic locale discovery, hot language switching, and plugin-store integration have not been implemented yet.

### Changed files

The main framework files are:

- `codex-rs/tui/src/i18n.rs`: `Localizer`, locale normalization, preference loading, Fluent formatting, and English fallback;
- `codex-rs/tui/src/i18n_tests.rs`: tests for missing keys, missing arguments, invalid resources, empty translations, locale handling, and self-check output;
- `codex-rs/tui/i18n/zh-Hans.ftl`: the current Simplified Chinese language resource;
- `codex-rs/tui/src/slash_command.rs`: `/language` command metadata and localized command descriptions;
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`: `/language` command handling;
- `codex-rs/tui/src/lib.rs`: global localizer initialization and the self-check entry;
- `codex-rs/cli/src/main.rs`: forwarding for the test-only self-check;
- `codex-rs/tui/Cargo.toml`, `codex-rs/Cargo.toml`, and `codex-rs/Cargo.lock`: Fluent and locale dependencies.

The prototype also changes the TUI call sites that display the startup/session card, status card, status line, footer, command popup, composer, approval dialogs, onboarding, tooltips, MCP startup warnings, history text, and common errors.

### Rust-layer interface

The basic interface retains the original English text at the call site:

```rust
localizer.text(
    "session-card-model-label",
    None,
    || "model:".to_string(),
)
```

Text containing dynamic values uses Fluent arguments while retaining a complete English fallback:

```rust
localizer.text_with_string_arg(
    "mcp-client-failed-to-start",
    "name",
    server_name,
    || format!("MCP client for `{server_name}` failed to start"),
)
```

The current interface consists mainly of:

- `Localizer::english()`: use the original English interface without loading a language resource;
- `Localizer::from_ftl(locale, source)`: create a localizer from a Fluent resource;
- `Localizer::from_runtime()`: read `CODEX_UI_LANGUAGE` or `$CODEX_HOME/ui-language`;
- `text()`: format a static or multi-argument message;
- `text_with_string_arg()`: format a common single-string-argument message;
- `global()`: return the process-wide localizer.

If the locale is invalid, the resource cannot be parsed, a key or argument is missing, formatting fails, or the translated text is empty, only that message falls back to the original English closure. The UI does not display a raw key, an incomplete template, or an empty string.

Model names, paths, URLs, server names, percentages, Token values, and detailed low-level errors remain runtime data. Only stable user-visible text is translated, and the existing Ratatui `Span`, `Line`, and `Style` structure is retained.

### What maintainers of other languages will see and need to maintain

Most translation content is placed in one Fluent resource:

```text
codex-rs/tui/i18n/<locale>.ftl
```

For example:

```ftl
session-card-model-label = 模型：
mcp-client-failed-to-start = MCP 客户端 `{ $name }` 启动失败
status-line-context-remaining = 上下文剩余 { $percent }%
```

Language maintainers translate stable user-visible text and keep dynamic arguments such as `{ $name }` and `{ $percent }`. They do not need to translate model identifiers, paths, URLs, server names, numeric values, protocol content, or detailed underlying errors.

In the current prototype, adding another built-in language still requires:

1. adding `codex-rs/tui/i18n/<locale>.ftl`;
2. registering the resource in `i18n.rs`;
3. adding the canonical locale and any compatible aliases;
4. exposing the language through `/language`;
5. adding formatting and fallback tests.

If a resource registry/provider interface is added later, language maintainers may eventually need to maintain only metadata and the FTL resource. This part has not been implemented yet.

### Entry

The current entry is:

```text
/language
```

It shows the current language and the usage. The current prototype supports:

```text
/language zh-Hans
/language en
```

The selected language is written to:

```text
$CODEX_HOME/ui-language
```

Changing the persistent language currently requires restarting Codex. Developers can also use `CODEX_UI_LANGUAGE=zh-Hans` to test Chinese for one process.

-language.png

(My plan is to make it selectable in a plugin-store-like form. This part of the work has not started yet.)

You can see the code changes in my fork:

https://github.com/Cec1c/codex

---

# feat:为Codex CLI添加i18n接口，我已完成部分原型

## 前言：

> 英文版本使用GPT翻译

尽管codex app已经完成了多语言i18n功能，但我实际上更倾向使用cli工具，再由于我的一部分强迫症，我希望我的工具总是最好的状态，尤其是语言

使用claude code 的时候我就如此，并为一个语言插件添加了windows版本的支持https://github.com/taekchef/claude-code-zh-cn/pull/11

遗憾现在我能找到的插件要么早就不维护了要么根本不能用

于是我自己动手（push codex）打算写一个插件，但是随着我深入研究这个需求，直接rust层添加一个i18n接口可能是最优雅的方案，也能让其他语言的强迫症患者省下很多工作

只不过如果我自己在插件实现，这意味着我事实上是自己又编译一个codex来替换原来的codex，还要每个版本都追着跑，这太折磨了

由于贡献指南不允许我直接提交PR，所以兜兜转转还是来写issues了，尽管大概率没人理我，最不济我只能自己追着版本跑自己编译的codex了并强迫未来其他用我插件的人也这么干（真的有人会用吗？）

btw，codex cli的界面实在有点素，这也是我插件想优化的方向

-主界面截图.png

-status.png

接下来一部分技术细节我可能直接让codex给我说明了：

### 现状

当前原型已经实现在我的 Codex fork 中，并在 Codex `0.144.5` 发布前后同步了上游 `main`。

原型目前包含：

- TUI 内的进程级全局 `Localizer`；
- 使用 Mozilla Fluent 作为翻译资源格式；
- 内置包含 134 条消息的 `zh-Hans` 简体中文资源；
- 每个 Rust 调用点都保留原始英文，作为最终回退；
- `/language`、`/language zh-Hans` 和 `/language en`；
- 兼容输入 `zh`、`zh-CN` 和 `chinese`，并统一规范化为 `zh-Hans`；
- 通过 `$CODEX_HOME/ui-language` 持久化语言选择；
- 通过 `CODEX_UI_LANGUAGE` 临时覆盖当前进程；
- 用于自动化验证的测试专用入口 `--i18n-self-check`。

目前实际翻译的可见文本包括启动卡片、`/status`、状态栏中的 Context 和 Token、斜杠命令说明、`/` 命令列表、输入占位、审批窗口、登录引导、Tip、MCP 启动警告前缀、未知命令错误，以及部分历史和进度消息。

这仍然只是一个原型。运行时安装外部语言包、自动发现 locale、无需重启热切换和插件商店接入都还没有实现。

### 改动的文件

框架主要涉及：

- `codex-rs/tui/src/i18n.rs`：`Localizer`、locale 规范化、偏好读取、Fluent 格式化和英文回退；
- `codex-rs/tui/src/i18n_tests.rs`：缺键、缺参数、无效资源、空翻译、locale 和自检输出测试；
- `codex-rs/tui/i18n/zh-Hans.ftl`：当前简体中文语言资源；
- `codex-rs/tui/src/slash_command.rs`：`/language` 命令元数据和命令说明翻译；
- `codex-rs/tui/src/chatwidget/slash_dispatch.rs`：`/language` 命令处理；
- `codex-rs/tui/src/lib.rs`：全局 Localizer 初始化和自检入口；
- `codex-rs/cli/src/main.rs`：测试专用自检入口转发；
- `codex-rs/tui/Cargo.toml`、`codex-rs/Cargo.toml` 和 `codex-rs/Cargo.lock`：Fluent 与 locale 依赖。

原型还修改了显示启动/会话卡片、状态卡片、状态栏、底部栏、命令面板、输入框、审批窗口、登录引导、Tip、MCP 启动警告、历史文本和常见错误的 TUI 调用点。

### rust层接口

基础接口会在调用点保留原始英文：

```rust
localizer.text(
    "session-card-model-label",
    None,
    || "model:".to_string(),
)
```

包含动态值的文本使用 Fluent 参数，同时保留完整英文回退：

```rust
localizer.text_with_string_arg(
    "mcp-client-failed-to-start",
    "name",
    server_name,
    || format!("MCP client for `{server_name}` failed to start"),
)
```

当前接口主要包括：

- `Localizer::english()`：不加载语言资源，使用原始英文界面；
- `Localizer::from_ftl(locale, source)`：从 Fluent 资源创建 Localizer；
- `Localizer::from_runtime()`：读取 `CODEX_UI_LANGUAGE` 或 `$CODEX_HOME/ui-language`；
- `text()`：格式化静态或多参数消息；
- `text_with_string_arg()`：格式化常见的单字符串参数消息；
- `global()`：返回进程级全局 Localizer。

locale 无效、资源无法解析、键或参数缺失、格式化失败，或者翻译为空时，只让当前消息回退到原始英文闭包，不会向界面显示原始键名、不完整模板或空字符串。

模型名、路径、URL、服务器名、百分比、Token 数量和底层详细错误继续作为运行时数据。翻译只处理稳定的用户可见文本，并保留原有 Ratatui `Span`、`Line` 和 `Style` 结构。

### 其他语言维护者会看到的，会需要维护的

大部分翻译内容都放在一个 Fluent 资源中：

```text
codex-rs/tui/i18n/<locale>.ftl
```

例如：

```ftl
session-card-model-label = 模型：
mcp-client-failed-to-start = MCP 客户端 `{ $name }` 启动失败
status-line-context-remaining = 上下文剩余 { $percent }%
```

语言维护者翻译稳定的用户可见文本，并保留 `{ $name }`、`{ $percent }` 等动态参数。模型标识、路径、URL、服务器名、数值、协议内容和底层详细错误不需要翻译。

在当前原型中，新增另一个内置语言仍然需要：

1. 添加 `codex-rs/tui/i18n/<locale>.ftl`；
2. 在 `i18n.rs` 注册资源；
3. 添加规范 locale 和兼容别名；
4. 通过 `/language` 暴露该语言；
5. 添加格式化和回退测试。

如果以后加入资源注册表/provider 接口，语言维护者最终可能只需要维护元数据和 FTL 资源。这部分目前还没有实现。

### 入口

当前入口是：

```text
/language
```

它会显示当前语言和使用方式。当前原型支持：

```text
/language zh-Hans
/language en
```

选择的语言会写入：

```text
$CODEX_HOME/ui-language
```

修改持久化语言后，目前需要重启 Codex。开发者也可以使用 `CODEX_UI_LANGUAGE=zh-Hans` 只为当前进程测试中文。

-language.png

（我计划是能做成插件商店那种可选择的形式，这部分工作还没展开）

你可以在我的fork里看到这次的代码改动https://github.com/Cec1c/codex
