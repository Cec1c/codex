use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Model,
    Ide,
    Permissions,
    Keymap,
    Vim,
    #[strum(serialize = "setup-default-sandbox")]
    ElevateSandbox,
    #[strum(serialize = "sandbox-add-read-dir")]
    SandboxReadRoot,
    Experimental,
    #[strum(to_string = "approve")]
    AutoReview,
    Memories,
    Skills,
    Import,
    Hooks,
    Review,
    Rename,
    New,
    Archive,
    Delete,
    Resume,
    Fork,
    App,
    Init,
    Compact,
    Plan,
    Goal,
    Agent,
    Side,
    Btw,
    Copy,
    Raw,
    Diff,
    Mention,
    Status,
    Usage,
    DebugConfig,
    Title,
    Statusline,
    Theme,
    Language,
    #[strum(to_string = "pets", serialize = "pet")]
    Pets,
    Mcp,
    Apps,
    Plugins,
    Logout,
    Quit,
    Exit,
    Feedback,
    Rollout,
    Ps,
    #[strum(to_string = "stop", serialize = "clean")]
    Stop,
    Clear,
    Personality,
    TestApproval,
    #[strum(serialize = "subagents")]
    MultiAgents,
    // Debugging commands.
    #[strum(serialize = "debug-m-drop")]
    MemoryDrop,
    #[strum(serialize = "debug-m-update")]
    MemoryUpdate,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::Feedback => "send logs to maintainers",
            SlashCommand::New => "start a new chat during a conversation",
            SlashCommand::Init => "create an AGENTS.md file with instructions for Codex",
            SlashCommand::Compact => "summarize conversation to prevent hitting the context limit",
            SlashCommand::Review => "review my current changes and find issues",
            SlashCommand::Rename => "rename the current thread",
            SlashCommand::Resume => "resume a saved chat",
            SlashCommand::Archive => "archive this session and exit",
            SlashCommand::Delete => "permanently delete this session and exit",
            SlashCommand::Clear => "clear the terminal and start a new chat",
            SlashCommand::Fork => "fork the current chat",
            SlashCommand::App => "continue this session in the Desktop app",
            SlashCommand::Quit | SlashCommand::Exit => "exit Codex",
            SlashCommand::Copy => "copy last response as markdown",
            SlashCommand::Raw => "toggle raw scrollback mode for copy-friendly terminal selection",
            SlashCommand::Diff => "show git diff (including untracked files)",
            SlashCommand::Mention => "mention a file",
            SlashCommand::Skills => "use skills to improve how Codex performs specific tasks",
            SlashCommand::Import => "import setup, this project, and recent chats from Claude Code",
            SlashCommand::Hooks => "view and manage lifecycle hooks",
            SlashCommand::Status => "show current session configuration and token usage",
            SlashCommand::Usage => "view account usage or use a usage limit reset",
            SlashCommand::DebugConfig => "show config layers and requirement sources for debugging",
            SlashCommand::Title => "configure which items appear in the terminal title",
            SlashCommand::Statusline => "configure which items appear in the status line",
            SlashCommand::Theme => "choose a syntax highlighting theme",
            SlashCommand::Language => "view or choose the display language",
            SlashCommand::Pets => "choose or hide the terminal pet",
            SlashCommand::Ps => "list background terminals",
            SlashCommand::Stop => "stop all background terminals",
            SlashCommand::MemoryDrop => "DO NOT USE",
            SlashCommand::MemoryUpdate => "DO NOT USE",
            SlashCommand::Model => "choose what model and reasoning effort to use",
            SlashCommand::Ide => {
                "include current selection, open files, and other context from your IDE"
            }
            SlashCommand::Personality => "choose a communication style for Codex",
            SlashCommand::Plan => "switch to Plan mode",
            SlashCommand::Goal => "set or view the goal for a long-running task",
            SlashCommand::Agent | SlashCommand::MultiAgents => "switch the active agent thread",
            SlashCommand::Side | SlashCommand::Btw => {
                "start a side conversation in an ephemeral fork"
            }
            SlashCommand::Permissions => "choose what Codex is allowed to do",
            SlashCommand::Keymap => "remap TUI shortcuts",
            SlashCommand::Vim => "toggle Vim mode for the composer",
            SlashCommand::ElevateSandbox => "set up elevated agent sandbox",
            SlashCommand::SandboxReadRoot => {
                "let sandbox read a directory: /sandbox-add-read-dir <absolute_path>"
            }
            SlashCommand::Experimental => "toggle experimental features",
            SlashCommand::AutoReview => "approve one retry of a recent auto-review denial",
            SlashCommand::Memories => "configure memory use and generation",
            SlashCommand::Mcp => "list configured MCP tools; use /mcp verbose for details",
            SlashCommand::Apps => "manage apps",
            SlashCommand::Plugins => "browse plugins",
            SlashCommand::Logout => "log out of Codex",
            SlashCommand::Rollout => "print the rollout file path",
            SlashCommand::TestApproval => "test approval request",
        }
    }

    pub(crate) fn description_metadata(self) -> (&'static str, &'static str) {
        match self {
            SlashCommand::Feedback => (
                "tui.slash-command.description.feedback",
                "slash-feedback-description",
            ),
            SlashCommand::New => ("tui.slash-command.description.new", "slash-new-description"),
            SlashCommand::Init => (
                "tui.slash-command.description.init",
                "slash-init-description",
            ),
            SlashCommand::Compact => (
                "tui.slash-command.description.compact",
                "slash-compact-description",
            ),
            SlashCommand::Review => (
                "tui.slash-command.description.review",
                "slash-review-description",
            ),
            SlashCommand::Rename => (
                "tui.slash-command.description.rename",
                "slash-rename-description",
            ),
            SlashCommand::Resume => (
                "tui.slash-command.description.resume",
                "slash-resume-description",
            ),
            SlashCommand::Archive => (
                "tui.slash-command.description.archive",
                "slash-archive-description",
            ),
            SlashCommand::Delete => (
                "tui.slash-command.description.delete",
                "slash-delete-description",
            ),
            SlashCommand::Clear => (
                "tui.slash-command.description.clear",
                "slash-clear-description",
            ),
            SlashCommand::Fork => (
                "tui.slash-command.description.fork",
                "slash-fork-description",
            ),
            SlashCommand::App => ("tui.slash-command.description.app", "slash-app-description"),
            SlashCommand::Quit | SlashCommand::Exit => (
                "tui.slash-command.description.exit",
                "slash-exit-description",
            ),
            SlashCommand::Copy => (
                "tui.slash-command.description.copy",
                "slash-copy-description",
            ),
            SlashCommand::Raw => ("tui.slash-command.description.raw", "slash-raw-description"),
            SlashCommand::Diff => (
                "tui.slash-command.description.diff",
                "slash-diff-description",
            ),
            SlashCommand::Mention => (
                "tui.slash-command.description.mention",
                "slash-mention-description",
            ),
            SlashCommand::Skills => (
                "tui.slash-command.description.skills",
                "slash-skills-description",
            ),
            SlashCommand::Import => (
                "tui.slash-command.description.import",
                "slash-import-description",
            ),
            SlashCommand::Hooks => (
                "tui.slash-command.description.hooks",
                "slash-hooks-description",
            ),
            SlashCommand::Status => (
                "tui.slash-command.description.status",
                "slash-status-description",
            ),
            SlashCommand::Usage => (
                "tui.slash-command.description.usage",
                "slash-usage-description",
            ),
            SlashCommand::DebugConfig => (
                "tui.slash-command.description.debug-config",
                "slash-debug-config-description",
            ),
            SlashCommand::Title => (
                "tui.slash-command.description.title",
                "slash-title-description",
            ),
            SlashCommand::Statusline => (
                "tui.slash-command.description.statusline",
                "slash-statusline-description",
            ),
            SlashCommand::Theme => (
                "tui.slash-command.description.theme",
                "slash-theme-description",
            ),
            SlashCommand::Language => (
                "tui.slash-command.description.language",
                "slash-language-description",
            ),
            SlashCommand::Pets => (
                "tui.slash-command.description.pets",
                "slash-pets-description",
            ),
            SlashCommand::Ps => ("tui.slash-command.description.ps", "slash-ps-description"),
            SlashCommand::Stop => (
                "tui.slash-command.description.stop",
                "slash-stop-description",
            ),
            SlashCommand::MemoryDrop | SlashCommand::MemoryUpdate => (
                "tui.slash-command.description.internal-debug",
                "slash-internal-debug-description",
            ),
            SlashCommand::Model => (
                "tui.slash-command.description.model",
                "slash-model-description",
            ),
            SlashCommand::Ide => ("tui.slash-command.description.ide", "slash-ide-description"),
            SlashCommand::Personality => (
                "tui.slash-command.description.personality",
                "slash-personality-description",
            ),
            SlashCommand::Plan => (
                "tui.slash-command.description.plan",
                "slash-plan-description",
            ),
            SlashCommand::Goal => (
                "tui.slash-command.description.goal",
                "slash-goal-description",
            ),
            SlashCommand::Agent | SlashCommand::MultiAgents => (
                "tui.slash-command.description.agent",
                "slash-agent-description",
            ),
            SlashCommand::Side | SlashCommand::Btw => (
                "tui.slash-command.description.side",
                "slash-side-description",
            ),
            SlashCommand::Permissions => (
                "tui.slash-command.description.permissions",
                "slash-permissions-description",
            ),
            SlashCommand::Keymap => (
                "tui.slash-command.description.keymap",
                "slash-keymap-description",
            ),
            SlashCommand::Vim => ("tui.slash-command.description.vim", "slash-vim-description"),
            SlashCommand::ElevateSandbox => (
                "tui.slash-command.description.elevate-sandbox",
                "slash-elevate-sandbox-description",
            ),
            SlashCommand::SandboxReadRoot => (
                "tui.slash-command.description.sandbox-read-root",
                "slash-sandbox-read-root-description",
            ),
            SlashCommand::Experimental => (
                "tui.slash-command.description.experimental",
                "slash-experimental-description",
            ),
            SlashCommand::AutoReview => (
                "tui.slash-command.description.approve",
                "slash-approve-description",
            ),
            SlashCommand::Memories => (
                "tui.slash-command.description.memories",
                "slash-memories-description",
            ),
            SlashCommand::Mcp => ("tui.slash-command.description.mcp", "slash-mcp-description"),
            SlashCommand::Apps => (
                "tui.slash-command.description.apps",
                "slash-apps-description",
            ),
            SlashCommand::Plugins => (
                "tui.slash-command.description.plugins",
                "slash-plugins-description",
            ),
            SlashCommand::Logout => (
                "tui.slash-command.description.logout",
                "slash-logout-description",
            ),
            SlashCommand::Rollout => (
                "tui.slash-command.description.rollout",
                "slash-rollout-description",
            ),
            SlashCommand::TestApproval => (
                "tui.slash-command.description.test-approval",
                "slash-test-approval-description",
            ),
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command supports inline args (for example `/review ...`).
    pub fn supports_inline_args(self) -> bool {
        matches!(
            self,
            SlashCommand::Review
                | SlashCommand::Rename
                | SlashCommand::Plan
                | SlashCommand::Goal
                | SlashCommand::Ide
                | SlashCommand::Keymap
                | SlashCommand::Mcp
                | SlashCommand::Raw
                | SlashCommand::Usage
                | SlashCommand::Language
                | SlashCommand::Pets
                | SlashCommand::Side
                | SlashCommand::Btw
                | SlashCommand::Resume
                | SlashCommand::SandboxReadRoot
        )
    }

    /// Whether this command remains available inside an active side conversation.
    pub fn available_in_side_conversation(self) -> bool {
        matches!(
            self,
            SlashCommand::Copy
                | SlashCommand::Raw
                | SlashCommand::Diff
                | SlashCommand::Mention
                | SlashCommand::Status
                | SlashCommand::Usage
                | SlashCommand::Language
                | SlashCommand::Ide
        )
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::New
            | SlashCommand::Archive
            | SlashCommand::Delete
            | SlashCommand::Fork
            | SlashCommand::Init
            | SlashCommand::Compact
            | SlashCommand::Keymap
            | SlashCommand::Vim
            | SlashCommand::ElevateSandbox
            | SlashCommand::SandboxReadRoot
            | SlashCommand::Experimental
            | SlashCommand::Memories
            | SlashCommand::Import
            | SlashCommand::Review
            | SlashCommand::Plan
            | SlashCommand::Clear
            | SlashCommand::Logout
            | SlashCommand::MemoryDrop
            | SlashCommand::MemoryUpdate => false,
            SlashCommand::Diff
            | SlashCommand::Resume
            | SlashCommand::Model
            | SlashCommand::Personality
            | SlashCommand::Permissions
            | SlashCommand::Copy
            | SlashCommand::Raw
            | SlashCommand::Rename
            | SlashCommand::Mention
            | SlashCommand::Skills
            | SlashCommand::Hooks
            | SlashCommand::Status
            | SlashCommand::Usage
            | SlashCommand::DebugConfig
            | SlashCommand::Ps
            | SlashCommand::Stop
            | SlashCommand::App
            | SlashCommand::Goal
            | SlashCommand::Mcp
            | SlashCommand::Apps
            | SlashCommand::Plugins
            | SlashCommand::Title
            | SlashCommand::Statusline
            | SlashCommand::Language
            | SlashCommand::AutoReview
            | SlashCommand::Feedback
            | SlashCommand::Ide
            | SlashCommand::Quit
            | SlashCommand::Exit
            | SlashCommand::Side
            | SlashCommand::Btw => true,
            SlashCommand::Rollout => true,
            SlashCommand::TestApproval => true,
            SlashCommand::Agent | SlashCommand::MultiAgents => true,
            SlashCommand::Theme | SlashCommand::Pets => false,
        }
    }

    fn is_visible(self) -> bool {
        match self {
            SlashCommand::SandboxReadRoot => cfg!(target_os = "windows"),
            SlashCommand::Copy => !cfg!(target_os = "android"),
            SlashCommand::App => cfg!(any(target_os = "macos", target_os = "windows")),
            SlashCommand::Rollout | SlashCommand::TestApproval => cfg!(debug_assertions),
            _ => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter()
        .filter(|command| command.is_visible())
        .map(|c| (c.command(), c))
        .collect()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    use super::SlashCommand;

    #[test]
    fn stop_command_is_canonical_name() {
        assert_eq!(SlashCommand::Stop.command(), "stop");
    }

    #[test]
    fn clean_alias_parses_to_stop_command() {
        assert_eq!(SlashCommand::from_str("clean"), Ok(SlashCommand::Stop));
    }

    #[test]
    fn pet_alias_parses_to_pets_command() {
        assert_eq!(SlashCommand::Pets.command(), "pets");
        assert_eq!(SlashCommand::from_str("pet"), Ok(SlashCommand::Pets));
    }

    #[test]
    fn certain_commands_are_available_during_task() {
        assert!(SlashCommand::Goal.available_during_task());
        assert!(SlashCommand::Ide.available_during_task());
        assert!(SlashCommand::Title.available_during_task());
        assert!(SlashCommand::Statusline.available_during_task());
        assert!(SlashCommand::Language.available_during_task());
        assert!(SlashCommand::Language.supports_inline_args());
        assert!(SlashCommand::Raw.available_during_task());
        assert!(SlashCommand::Raw.available_in_side_conversation());
        assert!(SlashCommand::Raw.supports_inline_args());
        assert!(SlashCommand::App.available_during_task());
    }

    #[test]
    fn auto_review_command_is_approve() {
        assert_eq!(SlashCommand::AutoReview.command(), "approve");
        assert_eq!(
            SlashCommand::from_str("approve"),
            Ok(SlashCommand::AutoReview)
        );
    }
}
