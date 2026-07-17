use super::*;

impl ChatWidget {
    pub(crate) fn open_language_picker(&mut self) {
        let active_locale = crate::i18n::active_locale();
        let codex_home = self.config.codex_home.clone();
        let language_root = crate::i18n::language_pack_root(&codex_home);
        let localizer = crate::i18n::global();
        let mut initial_selected_idx = active_locale.eq_ignore_ascii_case("en").then_some(0);

        let english_name =
            localizer.text("language-picker-english", None, || "English".to_string());
        let english_description =
            localizer.text("language-picker-english-description", None, || {
                "Built into Codex and always available.".to_string()
            });
        let english_home = codex_home.clone();
        let mut items = vec![SelectionItem {
            name: english_name,
            description: Some(english_description),
            is_current: active_locale.eq_ignore_ascii_case("en"),
            dismiss_on_select: true,
            search_value: Some("en English".to_string()),
            actions: vec![Box::new(move |tx| {
                let cell = match crate::i18n::save_language_preference(&english_home, "en") {
                    Ok(message) => history_cell::new_info_event(message, /*hint*/ None),
                    Err(message) => history_cell::new_error_event(message),
                };
                tx.send(AppEvent::InsertHistoryCell(Box::new(cell)));
            })],
            ..Default::default()
        }];

        match crate::i18n::discover_language_packs(&language_root) {
            Ok(candidates) if candidates.is_empty() => {
                items.push(SelectionItem {
                    name: "No external language packs installed".to_string(),
                    disabled_reason: Some("Use CCU to install language packs.".to_string()),
                    ..Default::default()
                });
            }
            Ok(candidates) => {
                for candidate in candidates {
                    let is_available = candidate.is_available();
                    let is_current = candidate
                        .is_available()
                        .then_some(candidate.locale.as_str())
                        .is_some_and(|locale| locale.eq_ignore_ascii_case(&active_locale));
                    if is_current {
                        initial_selected_idx = Some(items.len());
                    }
                    let description = candidate
                        .id
                        .as_deref()
                        .map(|id| format!("{} · {id}", candidate.locale))
                        .or_else(|| Some(format!("Locale {}", candidate.locale)));
                    let search_value = Some(format!(
                        "{} {} {}",
                        candidate.locale,
                        candidate.display_name,
                        candidate.id.as_deref().unwrap_or_default()
                    ));
                    let actions: Vec<SelectionAction> = if is_available {
                        let selected_home = codex_home.clone();
                        let selected_locale = candidate.locale.clone();
                        vec![Box::new(move |tx| {
                            let cell = match crate::i18n::save_language_preference(
                                &selected_home,
                                &selected_locale,
                            ) {
                                Ok(message) => {
                                    history_cell::new_info_event(message, /*hint*/ None)
                                }
                                Err(message) => history_cell::new_error_event(message),
                            };
                            tx.send(AppEvent::InsertHistoryCell(Box::new(cell)));
                        })]
                    } else {
                        Vec::new()
                    };
                    items.push(SelectionItem {
                        name: candidate.display_name,
                        description,
                        is_current,
                        actions,
                        dismiss_on_select: is_available,
                        search_value,
                        disabled_reason: candidate.disabled_reason,
                        ..Default::default()
                    });
                }
            }
            Err(error) => {
                items.push(SelectionItem {
                    name: "Language packs unavailable".to_string(),
                    disabled_reason: Some(error),
                    ..Default::default()
                });
            }
        }

        let title = localizer.text("language-picker-title", None, || {
            "Select Language".to_string()
        });
        let subtitle = localizer.text("language-picker-subtitle", None, || {
            "Language packs are managed by CCU. Restart Codex after selection.".to_string()
        });
        let mut header = ColumnRenderable::new();
        header.push(Line::from(title.bold()));
        header.push(Line::from(subtitle.dim()));

        self.bottom_pane.show_selection_view(SelectionViewParams {
            header: Box::new(header),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            initial_selected_idx,
            is_searchable: true,
            search_placeholder: Some("Type to search languages".to_string()),
            ..Default::default()
        });
    }
}
