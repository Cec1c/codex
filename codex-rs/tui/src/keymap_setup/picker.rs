//! Shortcut picker construction for `/keymap`.

use codex_config::types::TuiKeymap;
use ratatui::style::Styled;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use unicode_width::UnicodeWidthStr;

use crate::app_event::AppEvent;
use crate::bottom_pane::ColumnWidthMode;
use crate::bottom_pane::SelectionItem;
use crate::bottom_pane::SelectionRowDisplay;
use crate::bottom_pane::SelectionTab;
use crate::bottom_pane::SelectionViewParams;
use crate::keymap::RuntimeKeymap;
use crate::render::renderable::ColumnRenderable;
use crate::render::renderable::Renderable;
use crate::style::accent_style;

use super::actions::KEYMAP_ACTIONS;
use super::actions::KeymapActionFilter;
use super::actions::action_description;
use super::actions::action_label;
use super::actions::bindings_for_action;
use super::actions::context_label;
use super::actions::format_binding_summary;
use super::has_custom_binding;

fn keymap_text(key: &str, english: &'static str) -> String {
    crate::i18n::global().text(key, None, || english.to_string())
}

fn keymap_count_text(key: &str, count: usize, english: impl FnOnce() -> String) -> String {
    crate::i18n::global().text_with_string_arg(key, "count", count.to_string(), english)
}

fn keymap_tab_text(tab_id: &str, field: &str, english: &'static str) -> String {
    crate::i18n::global().text(&format!("keymap-tab-{tab_id}-{field}"), None, || {
        english.to_string()
    })
}

pub(crate) const KEYMAP_PICKER_VIEW_ID: &str = "keymap-picker";
pub(super) const KEYMAP_ALL_TAB_ID: &str = "all-shortcuts";
pub(super) const KEYMAP_COMMON_TAB_ID: &str = "common-shortcuts";
pub(super) const KEYMAP_CUSTOM_TAB_ID: &str = "custom-shortcuts";
pub(super) const KEYMAP_UNBOUND_TAB_ID: &str = "unbound-shortcuts";
pub(super) const KEYMAP_DEBUG_TAB_ID: &str = "debug-shortcuts";
const KEYMAP_CONTEXT_LABEL_WIDTH: usize = 12;
const KEYMAP_ROW_PREFIX_WIDTH: usize = KEYMAP_CONTEXT_LABEL_WIDTH + 3;

#[derive(Clone, Debug)]
struct KeymapActionRow {
    context: &'static str,
    context_label: String,
    action: &'static str,
    label: String,
    description: String,
    binding_summary: String,
    custom_binding: bool,
}

impl KeymapActionRow {
    fn is_unbound(&self) -> bool {
        self.binding_summary == "unbound"
    }
}

struct KeymapContextTab {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    contexts: &'static [&'static str],
}

const KEYMAP_COMMON_ACTIONS: &[(&str, &str)] = &[
    ("composer", "submit"),
    ("chat", "interrupt_turn"),
    ("editor", "insert_newline"),
    ("composer", "queue"),
    ("global", "toggle_fast_mode"),
    ("global", "open_external_editor"),
    ("global", "copy"),
    ("global", "toggle_vim_mode"),
    ("editor", "delete_backward_word"),
    ("editor", "delete_forward_word"),
    ("editor", "move_word_left"),
    ("editor", "move_word_right"),
    ("global", "open_transcript"),
    ("pager", "close"),
    ("pager", "page_up"),
    ("pager", "page_down"),
    ("approval", "open_fullscreen"),
    ("approval", "approve"),
    ("approval", "approve_for_session"),
    ("approval", "decline"),
    ("approval", "cancel"),
];

const KEYMAP_CONTEXT_TABS: &[KeymapContextTab] = &[
    KeymapContextTab {
        id: "app-shortcuts",
        label: "App",
        description: "Global and chat-level shortcuts.",
        contexts: &["global", "chat"],
    },
    KeymapContextTab {
        id: "composer-shortcuts",
        label: "Composer",
        description: "Composer submission and queue shortcuts.",
        contexts: &["composer"],
    },
    KeymapContextTab {
        id: "editor-shortcuts",
        label: "Editor",
        description: "Inline editor movement and editing shortcuts.",
        contexts: &["editor"],
    },
    KeymapContextTab {
        id: "vim-shortcuts",
        label: "Vim",
        description: "Vim normal-mode and operator shortcuts.",
        contexts: &["vim_normal", "vim_operator", "vim_text_object"],
    },
    KeymapContextTab {
        id: "navigation-shortcuts",
        label: "Navigation",
        description: "Pager and selection-list navigation shortcuts.",
        contexts: &["pager", "list"],
    },
    KeymapContextTab {
        id: "approval-shortcuts",
        label: "Approval",
        description: "Approval prompt shortcuts.",
        contexts: &["approval"],
    },
];

#[cfg(test)]
pub(crate) fn build_keymap_picker_params(
    runtime_keymap: &RuntimeKeymap,
    keymap_config: &TuiKeymap,
) -> SelectionViewParams {
    build_keymap_picker_params_with_filter(
        runtime_keymap,
        keymap_config,
        KeymapActionFilter::default(),
    )
}

pub(crate) fn build_keymap_picker_params_with_filter(
    runtime_keymap: &RuntimeKeymap,
    keymap_config: &TuiKeymap,
    action_filter: KeymapActionFilter,
) -> SelectionViewParams {
    build_keymap_picker_params_for_action(
        runtime_keymap,
        keymap_config,
        action_filter,
        /*selected_action*/ None,
    )
}

#[cfg(test)]
pub(crate) fn build_keymap_picker_params_for_selected_action(
    runtime_keymap: &RuntimeKeymap,
    keymap_config: &TuiKeymap,
    context: &str,
    action: &str,
) -> SelectionViewParams {
    build_keymap_picker_params_for_selected_action_with_filter(
        runtime_keymap,
        keymap_config,
        KeymapActionFilter::default(),
        context,
        action,
    )
}

pub(crate) fn build_keymap_picker_params_for_selected_action_with_filter(
    runtime_keymap: &RuntimeKeymap,
    keymap_config: &TuiKeymap,
    action_filter: KeymapActionFilter,
    context: &str,
    action: &str,
) -> SelectionViewParams {
    build_keymap_picker_params_for_action(
        runtime_keymap,
        keymap_config,
        action_filter,
        Some((context, action)),
    )
}

fn build_keymap_picker_params_for_action(
    runtime_keymap: &RuntimeKeymap,
    keymap_config: &TuiKeymap,
    action_filter: KeymapActionFilter,
    selected_action: Option<(&str, &str)>,
) -> SelectionViewParams {
    let rows = build_keymap_rows(runtime_keymap, keymap_config, action_filter);
    let total = rows.len();
    let custom_count = rows.iter().filter(|row| row.custom_binding).count();
    let unbound_count = rows.iter().filter(|row| row.is_unbound()).count();
    let initial_selected_idx = selected_action.and_then(|(context, action)| {
        rows.iter()
            .position(|row| row.context == context && row.action == action)
    });
    let name_column_width = rows
        .iter()
        .map(|row| KEYMAP_ROW_PREFIX_WIDTH + UnicodeWidthStr::width(row.label.as_str()))
        .max();

    let mut tabs = Vec::new();
    tabs.push(SelectionTab {
        id: KEYMAP_ALL_TAB_ID.to_string(),
        label: keymap_text("keymap-tab-all-label", "All"),
        header: keymap_header(
            keymap_text("keymap-tab-all-description", "All configurable shortcuts."),
            {
                let mut args = fluent_bundle::FluentArgs::new();
                args.set("total", total.to_string());
                args.set("custom", custom_count.to_string());
                args.set("unbound", unbound_count.to_string());
                crate::i18n::global().text("keymap-summary", Some(&args), || {
                    format!("{total} actions, {custom_count} customized, {unbound_count} unbound.")
                })
            },
        ),
        items: keymap_selection_items(
            rows.iter(),
            keymap_text("keymap-empty-all", "No shortcuts available"),
            keymap_text(
                "keymap-empty-all-description",
                "No configurable shortcuts are available.",
            ),
        ),
    });

    let common_rows = keymap_common_rows(&rows);
    let common_count = common_rows.len();
    tabs.push(SelectionTab {
        id: KEYMAP_COMMON_TAB_ID.to_string(),
        label: keymap_text("keymap-tab-common-label", "Common"),
        header: keymap_header(
            keymap_text(
                "keymap-tab-common-description",
                "Frequently customized shortcuts.",
            ),
            action_count_line(common_count),
        ),
        items: keymap_selection_items(
            common_rows,
            keymap_text("keymap-empty-common", "No common shortcuts"),
            keymap_text(
                "keymap-empty-common-description",
                "No common shortcut actions are available.",
            ),
        ),
    });

    let custom_rows = rows
        .iter()
        .filter(|row| row.custom_binding)
        .collect::<Vec<_>>();
    tabs.push(SelectionTab {
        id: KEYMAP_CUSTOM_TAB_ID.to_string(),
        label: keymap_count_text("keymap-tab-customized-label", custom_count, || {
            format!("Customized ({custom_count})")
        }),
        header: keymap_header(
            keymap_text(
                "keymap-tab-customized-description",
                "Root-level shortcut overrides.",
            ),
            action_count_line(custom_count),
        ),
        items: keymap_selection_items(
            custom_rows,
            keymap_text("keymap-empty-customized", "No customized shortcuts"),
            keymap_text(
                "keymap-empty-customized-description",
                "No root-level keymap overrides have been configured.",
            ),
        ),
    });

    let unbound_rows = rows
        .iter()
        .filter(|row| row.is_unbound())
        .collect::<Vec<_>>();
    tabs.push(SelectionTab {
        id: KEYMAP_UNBOUND_TAB_ID.to_string(),
        label: keymap_count_text("keymap-tab-unbound-label", unbound_count, || {
            format!("Unbound ({unbound_count})")
        }),
        header: keymap_header(
            keymap_text(
                "keymap-tab-unbound-description",
                "Actions without an active shortcut.",
            ),
            action_count_line(unbound_count),
        ),
        items: keymap_selection_items(
            unbound_rows,
            keymap_text("keymap-empty-unbound", "No unbound shortcuts"),
            keymap_text(
                "keymap-empty-unbound-description",
                "Every configurable action currently has a shortcut.",
            ),
        ),
    });

    for tab in KEYMAP_CONTEXT_TABS {
        let tab_rows = rows
            .iter()
            .filter(|row| tab.contexts.contains(&row.context))
            .collect::<Vec<_>>();
        let count = tab_rows.len();
        tabs.push(SelectionTab {
            id: tab.id.to_string(),
            label: keymap_tab_text(tab.id, "label", tab.label),
            header: keymap_header(
                keymap_tab_text(tab.id, "description", tab.description),
                action_count_line(count),
            ),
            items: keymap_selection_items(
                tab_rows,
                keymap_text("keymap-empty-group", "No shortcuts in this group"),
                keymap_text(
                    "keymap-empty-group-description",
                    "No configurable actions are available in this group.",
                ),
            ),
        });
    }
    tabs.push(keymap_debug_tab());

    SelectionViewParams {
        view_id: Some(KEYMAP_PICKER_VIEW_ID),
        header: Box::new(()),
        footer_hint: Some(keymap_picker_hint_line()),
        tab_footer_hints: vec![(KEYMAP_DEBUG_TAB_ID.to_string(), keymap_debug_hint_line())],
        tabs,
        initial_tab_id: Some(KEYMAP_ALL_TAB_ID.to_string()),
        is_searchable: true,
        search_placeholder: Some(keymap_text(
            "keymap-search-placeholder",
            "Type to search shortcuts",
        )),
        col_width_mode: ColumnWidthMode::AutoAllRows,
        row_display: SelectionRowDisplay::SingleLine,
        name_column_width,
        initial_selected_idx,
        ..Default::default()
    }
}

fn keymap_debug_tab() -> SelectionTab {
    SelectionTab {
        id: KEYMAP_DEBUG_TAB_ID.to_string(),
        label: keymap_text("keymap-tab-debug-label", "Debug"),
        header: keymap_header(
            keymap_text(
                "keymap-tab-debug-description",
                "Inspect keypresses from your terminal.",
            ),
            keymap_text(
                "keymap-tab-debug-summary",
                "See the key Codex detects and any shortcuts assigned to it.",
            ),
        ),
        items: vec![SelectionItem {
            name: keymap_text("keymap-inspect-keypresses", "Inspect keypresses"),
            description: Some(keymap_text(
                "keymap-inspect-keypresses-description",
                "Press Enter to start. Then press any key to inspect it; Ctrl+C exits.",
            )),
            selected_description: Some(keymap_text(
                "keymap-inspect-keypresses-selected",
                "Open a live inspector that shows the detected key, config key, and matching actions.",
            )),
            actions: vec![Box::new(|tx| {
                tx.send(AppEvent::OpenKeymapDebug);
            })],
            search_value: Some("debug inspect keypress key terminal detected actions".to_string()),
            ..Default::default()
        }],
    }
}

fn build_keymap_rows(
    runtime_keymap: &RuntimeKeymap,
    keymap_config: &TuiKeymap,
    action_filter: KeymapActionFilter,
) -> Vec<KeymapActionRow> {
    KEYMAP_ACTIONS
        .iter()
        .copied()
        .filter(|descriptor| descriptor.is_visible(action_filter))
        .map(|descriptor| {
            let bindings =
                bindings_for_action(runtime_keymap, descriptor.context, descriptor.action)
                    .unwrap_or(&[]);
            KeymapActionRow {
                context: descriptor.context,
                context_label: context_label(descriptor.context, descriptor.context_label),
                action: descriptor.action,
                label: action_label(descriptor.action),
                description: action_description(
                    descriptor.context,
                    descriptor.action,
                    descriptor.description,
                ),
                binding_summary: format_binding_summary(bindings),
                custom_binding: has_custom_binding(
                    keymap_config,
                    descriptor.context,
                    descriptor.action,
                )
                .unwrap_or(false),
            }
        })
        .collect()
}

fn keymap_common_rows(rows: &[KeymapActionRow]) -> Vec<&KeymapActionRow> {
    KEYMAP_COMMON_ACTIONS
        .iter()
        .filter_map(|(context, action)| {
            rows.iter()
                .find(|row| row.context == *context && row.action == *action)
        })
        .collect()
}

fn keymap_selection_items<'a>(
    rows: impl IntoIterator<Item = &'a KeymapActionRow>,
    empty_name: String,
    empty_description: String,
) -> Vec<SelectionItem> {
    let items = rows
        .into_iter()
        .map(keymap_selection_item)
        .collect::<Vec<_>>();
    if items.is_empty() {
        return vec![SelectionItem {
            name: empty_name,
            description: Some(empty_description),
            is_disabled: true,
            ..Default::default()
        }];
    }

    items
}

fn keymap_selection_item(row: &KeymapActionRow) -> SelectionItem {
    let context = row.context.to_string();
    let action = row.action.to_string();
    let source = if row.custom_binding {
        "Custom"
    } else {
        "Default"
    };
    let search_value = format!(
        "{} {} {} {} {} {}",
        row.context_label, row.action, row.label, row.description, row.binding_summary, source
    );

    SelectionItem {
        name: row.label.clone(),
        name_prefix_spans: keymap_row_prefix(row),
        description: Some(if row.is_unbound() {
            keymap_text("keymap-unbound", "unbound")
        } else {
            row.binding_summary.clone()
        }),
        actions: vec![Box::new(move |tx| {
            tx.send(AppEvent::OpenKeymapActionMenu {
                context: context.clone(),
                action: action.clone(),
            });
        })],
        search_value: Some(search_value),
        ..Default::default()
    }
}

fn keymap_row_prefix(row: &KeymapActionRow) -> Vec<Span<'static>> {
    let indicator = if row.custom_binding {
        "*".set_style(accent_style())
    } else if row.is_unbound() {
        "-".dim()
    } else {
        " ".into()
    };

    vec![
        format!(
            "{:<width$} ",
            row.context_label,
            width = KEYMAP_CONTEXT_LABEL_WIDTH
        )
        .dim(),
        indicator,
        " ".dim(),
    ]
}

fn keymap_header(description: String, summary: String) -> Box<dyn Renderable> {
    let mut header = ColumnRenderable::new();
    header.push(Line::from(keymap_text("keymap-title", "Keymap").bold()));
    header.push(Line::from(description.dim()));
    header.push(Line::from(summary.dim()));
    Box::new(header)
}

fn action_count_line(count: usize) -> String {
    match count {
        1 => keymap_text("keymap-one-action", "1 action."),
        _ => keymap_count_text("keymap-action-count", count, || format!("{count} actions.")),
    }
}

fn keymap_picker_hint_line() -> Line<'static> {
    let style = accent_style();
    Line::from(vec![
        "left/right".set_style(style),
        " ".dim(),
        keymap_text("keymap-hint-group", "group").dim(),
        " · ".dim(),
        "enter".set_style(style),
        " ".dim(),
        keymap_text("keymap-hint-edit", "edit shortcut").dim(),
        " · ".dim(),
        "*".set_style(style),
        " ".dim(),
        keymap_text("keymap-hint-custom", "custom").dim(),
        " · ".dim(),
        "-".set_style(style),
        " ".dim(),
        keymap_text("keymap-hint-unbound", "unbound").dim(),
        " · ".dim(),
        "esc".set_style(style),
        " ".dim(),
        keymap_text("keymap-hint-close", "close").dim(),
    ])
}

fn keymap_debug_hint_line() -> Line<'static> {
    let style = accent_style();
    Line::from(vec![
        "enter".set_style(style),
        " ".dim(),
        keymap_text("keymap-hint-start-inspector", "start inspector").dim(),
        " · ".dim(),
        "esc".set_style(style),
        " ".dim(),
        keymap_text("keymap-hint-close", "close").dim(),
    ])
}
