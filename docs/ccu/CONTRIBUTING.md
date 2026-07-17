# CCU fork contribution guide / CCU fork 贡献指南

## 中文

此 fork 只承载必须进入 Codex Rust/TUI 的通用机制：

- 外部 Fluent/FTL 语言包发现、校验和逐条英文回退；
- `/language` 选择器；
- 主题驱动的欢迎页与状态栏扩展点；
- 编译期 CCU 版本号和自动跟随上游稳定 Release 的构建流程。

翻译内容、主题内容、安装器和更新管理属于 `codex-cli-ultra` 仓库。提交前运行：

```text
cargo fmt --all -- --check
cargo test -p codex-tui i18n::tests --locked
cargo test -p codex-tui ccu_theme --locked
cargo test -p codex-tui status_line --locked
cargo test -p codex-tui footer::tests --locked
```

## English

This fork carries only reusable mechanisms that must live in Codex Rust/TUI:

- external Fluent/FTL pack discovery, validation, and per-message English fallback;
- the `/language` picker;
- theme-driven welcome and status-line extension points;
- compile-time CCU versioning and automated stable-upstream release builds.

Translation content, theme content, installation, and update management belong in `codex-cli-ultra`.
