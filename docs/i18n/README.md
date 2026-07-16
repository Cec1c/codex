# Experimental TUI Internationalization Framework

> This document describes an experimental i18n implementation maintained in the `Cec1c/codex` fork. It is not an official OpenAI localization feature.

[中文说明](#中文说明)

## Overview

This fork adds a small Fluent-based localization boundary to the Codex TUI. The goal is to make localization optional, safe, and incrementally adoptable without replacing the existing English strings or changing Codex's model prompts.

The current prototype includes:

- a process-wide `Localizer` backed by Mozilla Fluent;
- compiled English call-site fallbacks;
- a built-in Simplified Chinese resource;
- `/language`, `/language zh-CN`, and `/language en`;
- localized session cards, status surfaces, slash-command descriptions, composer placeholders, approvals, onboarding, MCP startup prefixes, and common errors;
- a hidden self-check used by automated tests.

## Build and run

Build the fork like a normal Codex checkout:

```shell
cd codex-rs
cargo build -p codex-cli --locked
```

Run the resulting binary directly:

```shell
# Windows
.\target\debug\codex.exe

# macOS or Linux
./target/debug/codex
```

No Ultra launcher or external FTL path is required.

English is the default. To enable Simplified Chinese, run this inside the TUI:

```text
/language zh-CN
```

Restart Codex after changing the language. To return to English:

```text
/language en
```

The preference is stored in `$CODEX_HOME/ui-language`. For a temporary one-process override, developers can set `CODEX_UI_LANGUAGE=zh-CN` before starting Codex.

## Architecture

```text
/language or CODEX_UI_LANGUAGE
             |
             v
      process-wide Localizer
             |
             +-- built-in zh-CN.ftl
             |
             +-- Fluent arguments
             |
             +-- compiled English closure fallback
             v
          TUI spans
```

Each localized call site keeps its original English expression:

```rust
localizer.text(
    "session-card-model-label",
    None,
    || "model:".to_string(),
)
```

Parameterized text keeps dynamic values outside the translation:

```rust
localizer.text_with_string_arg(
    "mcp-client-failed-to-start",
    "name",
    server_name,
    || format!("MCP client for `{server_name}` failed to start"),
)
```

Model names, paths, URLs, server names, percentages, token values, and low-level error details remain program data. Only the stable user-facing shell is translated.

## Fallback contract

Codex continues in English when:

- no supported language is selected;
- the locale is invalid;
- a message key is missing;
- a Fluent argument is missing;
- formatting reports an error;
- the translated result is empty or whitespace-only.

The localization layer must never return a raw message key, a partially formatted template, or an empty string to the UI.

## Resource layout

```text
codex-rs/tui/src/i18n.rs
codex-rs/tui/src/i18n_tests.rs
codex-rs/tui/i18n/zh-CN.ftl
```

English remains compiled at each call site, so an English FTL file is not required for safe operation. Additional built-in or external language packs can be added later without changing the fallback contract.

## Test-only interface

The following flag is intended for automated validation, not normal use:

```shell
codex --i18n-self-check
```

It returns JSON containing representative localized messages and an explicit missing-key fallback probe.

## Demo media placeholders

Keep the capture instructions below after adding the media. Place files under [`docs/i18n/assets`](./assets/README.md) using the specified names.

### 1. Startup card and composer

> **Expected file:** `assets/01-startup-card.gif`  
> Capture the Chinese startup card (`model`, `directory`, and `permissions`), a localized tip, and at least one Chinese composer placeholder. Avoid exposing private paths, account details, API keys, or thread content.

<!-- Media slot: 01-startup-card.gif. Add the image here after the file is available. -->

### 2. `/language` workflow

> **Expected file:** `assets/02-language-command.gif`  
> Capture `/language`, `/language zh-CN`, the restart notice, and a restarted Chinese session. A second short sequence may show `/language en` returning the UI to English.

<!-- Media slot: 02-language-command.gif. Add the image here after the file is available. -->

### 3. Status and error surfaces

> **Expected file:** `assets/03-status-and-errors.gif`  
> Capture `/status`, localized Context/Token text, the usage URL sentence, and an unknown command such as `/sdsd`. If an MCP startup warning naturally appears, include it only after checking that the details contain no sensitive information.

<!-- Media slot: 03-status-and-errors.gif. Add the image here after the file is available. -->

### 4. Slash-command popup

> **Expected file:** `assets/04-slash-command-popup.png`  
> Capture the `/` command popup with several translated command descriptions and the localized empty-result state if practical.

<!-- Media slot: 04-slash-command-popup.png. Add the image here after the file is available. -->

### 5. English fallback

> **Expected file:** `assets/05-english-fallback.gif`  
> Capture a normal English launch and, if useful, the test-only self-check output showing that the runtime remains functional when localization is inactive. Do not make the hidden self-check the primary product demo.

<!-- Media slot: 05-english-fallback.gif. Add the image here after the file is available. -->

## Scope and upstream direction

This fork is deliberately broader than a first upstream contribution. A maintainer-facing PR should be reduced to:

1. the minimal `Localizer` and Fluent dependencies;
2. the language preference surface;
3. compiled English fallback behavior;
4. a few representative TUI call sites;
5. focused tests.

Installer, updater, compatibility, and release-management code from Codex CLI Ultra are not part of this fork implementation.

---

# 中文说明

> 本文档描述 `Cec1c/codex` fork 中维护的实验性 i18n 实现，并非 OpenAI 官方本地化功能。

## 概述

这个 fork 在 Codex TUI 内加入了一层很薄的 Fluent 本地化边界。目标是让多语言能力保持可选、安全并且可以逐步接入，而不是批量替换原有英文，更不会翻译模型提示词。

当前原型包含：

- 进程级全局 `Localizer`；
- 编译进调用点的英文回退；
- 内置简体中文 FTL；
- `/language`、`/language zh-CN` 和 `/language en`；
- 启动卡片、状态栏、斜杠命令、输入占位、审批、登录引导、MCP 启动前缀和常用错误提示；
- 供自动化测试使用的隐藏自检入口。

## 编译与运行

像普通 Codex 源码一样编译：

```shell
cd codex-rs
cargo build -p codex-cli --locked
```

直接运行生成的二进制，不需要 Ultra 启动器或外部 FTL 路径。

界面默认使用英文。在 TUI 中输入以下命令启用简体中文：

```text
/language zh-CN
```

重启 Codex 后生效。恢复英文：

```text
/language en
```

语言偏好保存在 `$CODEX_HOME/ui-language`。开发者也可以在启动前设置 `CODEX_UI_LANGUAGE=zh-CN`，只覆盖当前进程。

## 核心设计

每个翻译调用点都保留原始英文闭包。语言未启用、键缺失、参数错误、格式化失败或翻译为空时，只回退当前消息的英文，不会让 Codex 启动失败。

模型名、路径、URL、服务器名、百分比、Token 数量和底层错误正文继续作为动态数据传递。翻译只处理稳定的用户可见外壳，并保留原有 Ratatui Span 和样式。

简体中文资源位于：

```text
codex-rs/tui/i18n/zh-CN.ftl
```

## 媒体文件说明

请按照英文部分的文件名和录制要求准备素材，并放入 `docs/i18n/assets`。添加媒体后保留对应的录制说明，方便后续审阅者了解每张图片或 GIF 展示的功能及安全边界。

建议素材包括：

1. 中文启动卡片与输入占位；
2. `/language` 切换流程；
3. `/status`、Context/Token 和未知命令错误；
4. 中文斜杠命令面板；
5. 英文默认与安全回退。

## 上游化边界

当前 fork 演示范围较大。真正面向维护者的首个 PR 应缩减为最小 Localizer、语言偏好、英文回退、少量代表性调用点和聚焦测试，不应包含 Ultra 的安装器、更新器、兼容层或发布系统。
