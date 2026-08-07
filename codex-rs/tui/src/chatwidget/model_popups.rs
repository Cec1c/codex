//! Model, collaboration, and reasoning popups for `ChatWidget`.
//!
//! These surfaces are tightly related because changing one often redirects
//! into another, especially while Plan mode is active.

use super::*;
use fluent_bundle::FluentArgs;

const ULTRA_REASONING_CONCURRENCY_WARNING_THRESHOLD: usize = 8;

fn model_popup_text(key: &str, english: &'static str) -> String {
    crate::i18n::global().text(key, None, || english.to_string())
}

fn model_popup_text_with_arg<F>(
    key: &str,
    arg_name: &str,
    arg_value: impl Into<String>,
    english: F,
) -> String
where
    F: FnOnce() -> String,
{
    crate::i18n::global().text_with_string_arg(key, arg_name, arg_value, english)
}

fn localize_model_description(description: &str) -> String {
    let localized = match description {
        "Automatic approval review model for Codex." => Some((
            "model-description-automatic-approval-review",
            "Automatic approval review model for Codex.",
        )),
        "Balanced agentic coding model for everyday work." => Some((
            "model-description-balanced-agentic",
            "Balanced agentic coding model for everyday work.",
        )),
        "Extra high reasoning depth for complex problems" => Some((
            "reasoning-description-extra-high-depth",
            "Extra high reasoning depth for complex problems",
        )),
        "Extra high reasoning for complex problems" => Some((
            "reasoning-description-extra-high",
            "Extra high reasoning for complex problems",
        )),
        "Fast and affordable agentic coding model." => Some((
            "model-description-fast-affordable-agentic",
            "Fast and affordable agentic coding model.",
        )),
        "Fast responses with lighter reasoning" => Some((
            "reasoning-description-fast-lighter",
            "Fast responses with lighter reasoning",
        )),
        "Frontier model for complex coding, research, and real-world work." => Some((
            "model-description-frontier-complex-work",
            "Frontier model for complex coding, research, and real-world work.",
        )),
        "Greater reasoning depth for complex problems" => Some((
            "reasoning-description-greater-depth",
            "Greater reasoning depth for complex problems",
        )),
        "Latest frontier agentic coding model." => Some((
            "model-description-latest-frontier-agentic",
            "Latest frontier agentic coding model.",
        )),
        "Maximizes reasoning depth for complex or ambiguous problems" => Some((
            "reasoning-description-max-complex-ambiguous",
            "Maximizes reasoning depth for complex or ambiguous problems",
        )),
        "Maximum reasoning depth for the hardest problems" => Some((
            "reasoning-description-maximum-hardest",
            "Maximum reasoning depth for the hardest problems",
        )),
        "Maximum reasoning with automatic task delegation" => Some((
            "reasoning-description-maximum-delegation",
            "Maximum reasoning with automatic task delegation",
        )),
        "Optimized for professional work and long-running agents." => Some((
            "model-description-professional-long-running",
            "Optimized for professional work and long-running agents.",
        )),
        "Provides a solid balance of reasoning depth and latency for general-purpose tasks" => {
            Some((
                "reasoning-description-balanced-general",
                "Provides a solid balance of reasoning depth and latency for general-purpose tasks",
            ))
        }
        "Balances speed and reasoning depth for everyday tasks" => Some((
            "reasoning-description-balanced-everyday",
            "Balances speed and reasoning depth for everyday tasks",
        )),
        "Balances speed with some reasoning; useful for straightforward queries and short explanations" => {
            Some((
                "reasoning-description-balanced-straightforward",
                "Balances speed with some reasoning; useful for straightforward queries and short explanations",
            ))
        }
        "Small, fast, and cost-efficient model for simpler coding tasks." => Some((
            "model-description-small-fast-efficient",
            "Small, fast, and cost-efficient model for simpler coding tasks.",
        )),
        "Strong model for everyday coding." => Some((
            "model-description-strong-everyday",
            "Strong model for everyday coding.",
        )),
        _ => None,
    };
    localized.map_or_else(
        || description.to_string(),
        |(key, english)| model_popup_text(key, english),
    )
}

impl ChatWidget {
    /// Open a popup to choose a quick auto model. Selecting "All models"
    /// opens the full picker with every available preset.
    pub(crate) fn open_model_popup(&mut self) {
        if !self.is_session_configured() {
            self.add_info_message(
                model_popup_text(
                    "model-picker-startup-disabled",
                    "Model selection is disabled until startup completes.",
                ),
                /*hint*/ None,
            );
            return;
        }

        let presets: Vec<ModelPreset> = match self.model_catalog.try_list_models() {
            Ok(models) => models,
            Err(_) => {
                self.add_info_message(
                    model_popup_text(
                        "model-picker-updating",
                        "Models are being updated; please try /model again in a moment.",
                    ),
                    /*hint*/ None,
                );
                return;
            }
        };
        self.open_model_popup_with_presets(presets);
    }

    fn model_menu_header(&self, title: &str, subtitle: &str) -> Box<dyn Renderable> {
        let title = title.to_string();
        let subtitle = subtitle.to_string();
        let mut header = ColumnRenderable::new();
        header.push(Line::from(title.bold()));
        header.push(Line::from(subtitle.dim()));
        if let Some(warning) = self.model_menu_warning_line() {
            header.push(warning);
        }
        Box::new(header)
    }

    fn model_menu_warning_line(&self) -> Option<Line<'static>> {
        let base_url = self.custom_openai_base_url()?;
        let warning = model_popup_text_with_arg(
            "model-picker-custom-base-url-warning",
            "base_url",
            base_url.clone(),
            || {
                format!(
                    "Warning: OpenAI base URL is overridden to {base_url}. Selecting models may not be supported or work properly."
                )
            },
        );
        Some(Line::from(warning.red()))
    }

    fn custom_openai_base_url(&self) -> Option<String> {
        if !self.config.model_provider.is_openai() {
            return None;
        }

        let base_url = self.config.model_provider.base_url.as_ref()?;
        let trimmed = base_url.trim();
        if trimmed.is_empty() {
            return None;
        }

        let normalized = trimmed.trim_end_matches('/');
        if normalized == DEFAULT_OPENAI_BASE_URL {
            return None;
        }

        Some(trimmed.to_string())
    }

    pub(crate) fn open_model_popup_with_presets(&mut self, presets: Vec<ModelPreset>) {
        let presets: Vec<ModelPreset> = presets
            .into_iter()
            .filter(|preset| preset.show_in_picker)
            .collect();

        let current_model = self.current_model();
        let current_label = presets
            .iter()
            .find(|preset| preset.model.as_str() == current_model)
            .map(|preset| preset.model.to_string())
            .unwrap_or_else(|| self.model_display_name().to_string());

        let (mut auto_presets, other_presets): (Vec<ModelPreset>, Vec<ModelPreset>) = presets
            .into_iter()
            .partition(|preset| Self::is_auto_model(&preset.model));

        if auto_presets.is_empty() {
            self.open_all_models_popup(other_presets);
            return;
        }

        auto_presets.sort_by_key(|preset| Self::auto_model_order(&preset.model));
        let mut items: Vec<SelectionItem> = auto_presets
            .into_iter()
            .map(|preset| {
                let description = (!preset.description.is_empty())
                    .then(|| localize_model_description(&preset.description));
                let model = preset.model.clone();
                let requires_advanced_selection =
                    Self::is_advanced_reasoning_effort(&preset.default_reasoning_effort)
                        || preset
                            .supported_reasoning_efforts
                            .iter()
                            .any(|option| Self::is_advanced_reasoning_effort(&option.effort));
                let actions: Vec<SelectionAction> = if requires_advanced_selection {
                    let preset_for_action = preset.clone();
                    vec![Box::new(move |tx| {
                        tx.send(AppEvent::OpenReasoningPopup {
                            model: preset_for_action.clone(),
                        });
                    })]
                } else {
                    let should_prompt_plan_mode_scope = self
                        .should_prompt_plan_mode_reasoning_scope(
                            model.as_str(),
                            Some(preset.default_reasoning_effort.clone()),
                        );
                    self.model_selection_actions(
                        model.clone(),
                        Some(preset.default_reasoning_effort.clone()),
                        should_prompt_plan_mode_scope,
                    )
                };
                SelectionItem {
                    name: model.clone(),
                    description,
                    is_current: model.as_str() == current_model,
                    is_default: preset.is_default,
                    actions,
                    dismiss_on_select: !requires_advanced_selection,
                    dismiss_parent_on_child_accept: requires_advanced_selection,
                    ..Default::default()
                }
            })
            .collect();

        if !other_presets.is_empty() {
            let all_models = other_presets;
            let actions: Vec<SelectionAction> = vec![Box::new(move |tx| {
                tx.send(AppEvent::OpenAllModelsPopup {
                    models: all_models.clone(),
                });
            })];

            let is_current = !items.iter().any(|item| item.is_current);
            let description = Some(model_popup_text_with_arg(
                "model-picker-all-models-description",
                "current_model",
                current_label.clone(),
                || {
                    format!(
                        "Choose a specific model and reasoning level (current: {current_label})"
                    )
                },
            ));

            items.push(SelectionItem {
                name: model_popup_text("model-picker-all-models", "All models"),
                description,
                is_current,
                actions,
                dismiss_on_select: true,
                ..Default::default()
            });
        }

        let header = self.model_menu_header(
            &model_popup_text("model-picker-title", "Select Model"),
            &model_popup_text(
                "model-picker-subtitle",
                "Pick a quick auto mode or browse all models.",
            ),
        );
        self.bottom_pane.show_selection_view(SelectionViewParams {
            footer_hint: Some(standard_popup_hint_line()),
            items,
            header,
            ..Default::default()
        });
    }

    fn is_auto_model(model: &str) -> bool {
        model.starts_with("codex-auto-")
    }

    fn auto_model_order(model: &str) -> usize {
        match model {
            "codex-auto-fast" => 0,
            "codex-auto-balanced" => 1,
            "codex-auto-thorough" => 2,
            _ => 3,
        }
    }

    pub(crate) fn open_all_models_popup(&mut self, presets: Vec<ModelPreset>) {
        if presets.is_empty() {
            self.add_info_message(
                model_popup_text(
                    "model-picker-no-additional-models",
                    "No additional models are available right now.",
                ),
                /*hint*/ None,
            );
            return;
        }

        let mut items: Vec<SelectionItem> = Vec::new();
        for preset in presets.into_iter() {
            let description = (!preset.description.is_empty())
                .then(|| localize_model_description(&preset.description));
            let is_current = preset.model.as_str() == self.current_model();
            let single_supported_effort = preset.supported_reasoning_efforts.len() == 1;
            let preset_for_action = preset.clone();
            let actions: Vec<SelectionAction> = vec![Box::new(move |tx| {
                let preset_for_event = preset_for_action.clone();
                tx.send(AppEvent::OpenReasoningPopup {
                    model: preset_for_event,
                });
            })];
            items.push(SelectionItem {
                name: preset.model.clone(),
                description,
                is_current,
                is_default: preset.is_default,
                actions,
                dismiss_on_select: single_supported_effort,
                dismiss_parent_on_child_accept: !single_supported_effort,
                ..Default::default()
            });
        }

        let header = self.model_menu_header(
            &model_popup_text("model-picker-all-models-title", "Select Model and Effort"),
            &model_popup_text(
                "model-picker-all-models-subtitle",
                "Access legacy models by running codex -m <model_name> or in your config.toml",
            ),
        );
        self.bottom_pane.show_selection_view(SelectionViewParams {
            footer_hint: Some(self.bottom_pane.standard_popup_hint_line()),
            items,
            header,
            ..Default::default()
        });
    }

    fn model_selection_actions(
        &self,
        model_for_action: String,
        effort_for_action: Option<ReasoningEffortConfig>,
        should_prompt_plan_mode_scope: bool,
    ) -> Vec<SelectionAction> {
        let warning = effort_for_action
            .as_ref()
            .and_then(|effort| self.ultra_reasoning_concurrency_warning(effort));
        vec![Box::new(move |tx| {
            if effort_for_action == Some(ReasoningEffortConfig::Ultra) {
                tx.send(AppEvent::ApplyAdvancedReasoning {
                    model: model_for_action.clone(),
                    effort: ReasoningEffortConfig::Ultra,
                });
            } else if should_prompt_plan_mode_scope {
                tx.send(AppEvent::OpenPlanReasoningScopePrompt {
                    model: model_for_action.clone(),
                    effort: effort_for_action.clone(),
                });
            } else {
                tx.send(AppEvent::UpdateModel(model_for_action.clone()));
                tx.send(AppEvent::UpdateReasoningEffort(effort_for_action.clone()));
                tx.send(AppEvent::PersistModelSelection {
                    model: model_for_action.clone(),
                    effort: effort_for_action.clone(),
                });
            }
            if let Some(warning) = warning.clone() {
                tx.send(AppEvent::InsertHistoryCell(Box::new(
                    history_cell::new_warning_event(warning),
                )));
            }
        })]
    }

    fn should_prompt_plan_mode_reasoning_scope(
        &self,
        selected_model: &str,
        selected_effort: Option<ReasoningEffortConfig>,
    ) -> bool {
        if !self.collaboration_modes_enabled()
            || self.active_mode_kind() != ModeKind::Plan
            || selected_model != self.current_model()
        {
            return false;
        }

        // Prompt whenever the selection is not a true no-op for both:
        // 1) the active Plan-mode effective reasoning, and
        // 2) the stored global defaults that would be updated by the fallback path.
        selected_effort != self.effective_reasoning_effort()
            || selected_model != self.current_collaboration_mode.model()
            || selected_effort != self.current_collaboration_mode.reasoning_effort()
    }

    pub(crate) fn open_plan_reasoning_scope_prompt(
        &mut self,
        model: String,
        effort: Option<ReasoningEffortConfig>,
    ) {
        let reasoning_phrase = match effort.as_ref() {
            Some(ReasoningEffortConfig::None) => {
                model_popup_text("model-picker-no-reasoning", "no reasoning")
            }
            Some(selected_effort) => {
                let effort_label = Self::reasoning_effort_sentence_label(selected_effort);
                model_popup_text_with_arg(
                    "model-picker-reasoning-phrase",
                    "effort",
                    effort_label.clone(),
                    || format!("{effort_label} reasoning"),
                )
            }
            None => model_popup_text("model-picker-selected-reasoning", "the selected reasoning"),
        };
        let plan_only_description = model_popup_text_with_arg(
            "model-picker-plan-only-description",
            "reasoning",
            reasoning_phrase.clone(),
            || format!("Always use {reasoning_phrase} in Plan mode."),
        );
        let plan_reasoning_source = if let Some(plan_override) =
            self.config.plan_mode_reasoning_effort.as_ref()
        {
            let effort_label = Self::reasoning_effort_sentence_label(plan_override);
            model_popup_text_with_arg(
                "model-picker-plan-source-user-override",
                "effort",
                effort_label.clone(),
                || format!("user-chosen Plan override ({effort_label})"),
            )
        } else if let Some(plan_mask) = collaboration_modes::plan_mask(self.model_catalog.as_ref())
        {
            match plan_mask
                .reasoning_effort
                .as_ref()
                .and_then(|effort| effort.as_ref())
            {
                Some(plan_effort) => {
                    let effort_label = Self::reasoning_effort_sentence_label(plan_effort);
                    model_popup_text_with_arg(
                        "model-picker-plan-source-built-in-effort",
                        "effort",
                        effort_label.clone(),
                        || format!("built-in Plan default ({effort_label})"),
                    )
                }
                None => model_popup_text(
                    "model-picker-plan-source-built-in-no-reasoning",
                    "built-in Plan default (no reasoning)",
                ),
            }
        } else {
            model_popup_text("model-picker-plan-source-built-in", "built-in Plan default")
        };
        let all_modes_description = model_popup_text_with_arg(
            "model-picker-all-modes-description",
            "source",
            plan_reasoning_source.clone(),
            || {
                format!(
                    "Set the global default reasoning level and the Plan mode override. This replaces the current {plan_reasoning_source}."
                )
            },
        );
        let subtitle = model_popup_text_with_arg(
            "model-picker-plan-scope-subtitle",
            "reasoning",
            reasoning_phrase.clone(),
            || format!("Choose where to apply {reasoning_phrase}."),
        );
        let scope_title = model_popup_text(
            "model-picker-plan-scope-title",
            PLAN_MODE_REASONING_SCOPE_TITLE,
        );
        let plan_only_name = model_popup_text(
            "model-picker-plan-scope-plan-only",
            PLAN_MODE_REASONING_SCOPE_PLAN_ONLY,
        );
        let all_modes_name = model_popup_text(
            "model-picker-plan-scope-all-modes",
            PLAN_MODE_REASONING_SCOPE_ALL_MODES,
        );
        let warning = effort
            .as_ref()
            .and_then(|effort| self.ultra_reasoning_concurrency_warning(effort));

        let plan_only_actions: Vec<SelectionAction> = vec![Box::new({
            let model = model.clone();
            let effort = effort.clone();
            let warning = warning.clone();
            move |tx| {
                tx.send(AppEvent::UpdateModel(model.clone()));
                tx.send(AppEvent::UpdatePlanModeReasoningEffort(effort.clone()));
                tx.send(AppEvent::PersistPlanModeReasoningEffort(effort.clone()));
                if let Some(warning) = warning.clone() {
                    tx.send(AppEvent::InsertHistoryCell(Box::new(
                        history_cell::new_warning_event(warning),
                    )));
                }
            }
        })];
        let all_modes_actions: Vec<SelectionAction> = vec![Box::new(move |tx| {
            tx.send(AppEvent::UpdateModel(model.clone()));
            tx.send(AppEvent::UpdateReasoningEffort(effort.clone()));
            tx.send(AppEvent::UpdatePlanModeReasoningEffort(effort.clone()));
            tx.send(AppEvent::PersistPlanModeReasoningEffort(effort.clone()));
            tx.send(AppEvent::PersistModelSelection {
                model: model.clone(),
                effort: effort.clone(),
            });
            if let Some(warning) = warning.clone() {
                tx.send(AppEvent::InsertHistoryCell(Box::new(
                    history_cell::new_warning_event(warning),
                )));
            }
        })];

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(scope_title.clone()),
            subtitle: Some(subtitle),
            footer_hint: Some(standard_popup_hint_line()),
            items: vec![
                SelectionItem {
                    name: plan_only_name,
                    description: Some(plan_only_description),
                    actions: plan_only_actions,
                    dismiss_on_select: true,
                    ..Default::default()
                },
                SelectionItem {
                    name: all_modes_name,
                    description: Some(all_modes_description),
                    actions: all_modes_actions,
                    dismiss_on_select: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        self.notify(Notification::PlanModePrompt { title: scope_title });
    }

    /// Open a popup to choose the standard reasoning effort for the given model.
    ///
    /// Max and Ultra require an explicit second step so expensive efforts cannot
    /// be selected accidentally while moving through the normal effort scale.
    pub(crate) fn open_reasoning_popup(&mut self, preset: ModelPreset) {
        let default_effort = preset.default_reasoning_effort.clone();
        let supported = &preset.supported_reasoning_efforts;
        let in_plan_mode =
            self.collaboration_modes_enabled() && self.active_mode_kind() == ModeKind::Plan;

        let warn_effort = if supported
            .iter()
            .any(|option| option.effort == ReasoningEffortConfig::XHigh)
        {
            Some(ReasoningEffortConfig::XHigh)
        } else if supported
            .iter()
            .any(|option| option.effort == ReasoningEffortConfig::High)
        {
            Some(ReasoningEffortConfig::High)
        } else {
            None
        };
        let warning_text = warn_effort.as_ref().map(|effort| {
            let effort_label = Self::reasoning_effort_label(effort);
            model_popup_text_with_arg(
                "model-picker-plus-limit-warning",
                "effort",
                effort_label.clone(),
                || {
                    format!(
                        "⚠ {effort_label} reasoning effort can quickly consume Plus plan rate limits."
                    )
                },
            )
        });
        let warn_for_model = preset.model.starts_with("gpt-5.1-codex")
            || preset.model.starts_with("gpt-5.1-codex-max")
            || preset.model.starts_with("gpt-5.2");

        let mut all_choices: Vec<ReasoningEffortConfig> = supported
            .iter()
            .map(|option| option.effort.clone())
            .collect();
        if all_choices.is_empty() {
            all_choices.push(default_effort.clone());
        }
        let (choices, advanced_choices): (Vec<_>, Vec<_>) = all_choices
            .into_iter()
            .partition(|effort| !Self::is_advanced_reasoning_effort(effort));

        if choices.len() == 1 && advanced_choices.is_empty() {
            let selected_effort = choices.first().cloned();
            let selected_model = preset.model;
            if self
                .should_prompt_plan_mode_reasoning_scope(&selected_model, selected_effort.clone())
            {
                self.app_event_tx
                    .send(AppEvent::OpenPlanReasoningScopePrompt {
                        model: selected_model,
                        effort: selected_effort,
                    });
            } else {
                self.apply_model_and_effort(selected_model, selected_effort);
            }
            return;
        }

        let default_choice = choices
            .contains(&default_effort)
            .then(|| default_effort.clone());

        let model_slug = preset.model.to_string();
        let is_current_model = self.current_model() == preset.model.as_str();
        let highlight_choice = if is_current_model {
            if in_plan_mode {
                self.config
                    .plan_mode_reasoning_effort
                    .clone()
                    .or_else(|| self.effective_reasoning_effort())
            } else {
                self.effective_reasoning_effort()
            }
        } else {
            default_choice.clone().or_else(|| choices.first().cloned())
        };
        let selection_choice = highlight_choice.clone().or_else(|| default_choice.clone());
        let initial_selected_idx = choices
            .iter()
            .position(|choice| Some(choice) == selection_choice.as_ref());
        let mut items: Vec<SelectionItem> = Vec::new();
        for choice in choices.iter() {
            let effort = choice.clone();
            let mut effort_label = Self::reasoning_effort_label(&effort);
            if Some(choice) == default_choice.as_ref() {
                let default_label =
                    crate::i18n::global()
                        .text("selection-marker-default", None, || "default".to_string());
                effort_label.push_str(&format!(" ({default_label})"));
            }

            let description = supported
                .iter()
                .find(|option| option.effort == effort)
                .map(|option| localize_model_description(&option.description))
                .filter(|text| !text.is_empty());

            let show_warning = warn_for_model && warn_effort.as_ref() == Some(&effort);
            let selected_description = if show_warning {
                warning_text.as_ref().map(|warning_message| {
                    description.as_ref().map_or_else(
                        || warning_message.clone(),
                        |d| format!("{d}\n{warning_message}"),
                    )
                })
            } else {
                None
            };

            let choice_effort = Some(effort);
            let should_prompt_plan_mode_scope = self.should_prompt_plan_mode_reasoning_scope(
                model_slug.as_str(),
                choice_effort.clone(),
            );
            let actions = self.model_selection_actions(
                model_slug.clone(),
                choice_effort,
                should_prompt_plan_mode_scope,
            );

            items.push(SelectionItem {
                name: effort_label,
                description,
                selected_description,
                is_current: is_current_model && Some(choice) == highlight_choice.as_ref(),
                actions,
                dismiss_on_select: true,
                ..Default::default()
            });
        }

        if !advanced_choices.is_empty() {
            let advanced_label = advanced_choices
                .iter()
                .map(Self::reasoning_effort_label)
                .collect::<Vec<_>>()
                .join(&format!(
                    " {} ",
                    model_popup_text("model-picker-effort-join", "and")
                ));
            let description = if advanced_choices.len() == 1 {
                model_popup_text_with_arg(
                    "model-picker-more-reasoning-description-one",
                    "efforts",
                    advanced_label.clone(),
                    || format!("{advanced_label} consumes usage limits faster"),
                )
            } else {
                model_popup_text_with_arg(
                    "model-picker-more-reasoning-description-many",
                    "efforts",
                    advanced_label.clone(),
                    || format!("{advanced_label} consume usage limits faster"),
                )
            };
            let preset_for_action = preset;
            let actions: Vec<SelectionAction> = vec![Box::new(move |tx| {
                tx.send(AppEvent::OpenAdvancedReasoningPopup {
                    model: preset_for_action.clone(),
                });
            })];
            items.push(SelectionItem {
                name: model_popup_text("model-picker-more-reasoning", "More reasoning…"),
                description: Some(description),
                is_current: is_current_model
                    && highlight_choice
                        .as_ref()
                        .is_some_and(Self::is_advanced_reasoning_effort),
                actions,
                dismiss_parent_on_child_accept: true,
                ..Default::default()
            });
        }

        let mut header = ColumnRenderable::new();
        let title = model_popup_text_with_arg(
            "model-picker-reasoning-title",
            "model",
            model_slug.clone(),
            || format!("Select Reasoning Level for {model_slug}"),
        );
        header.push(Line::from(title.bold()));

        self.bottom_pane.show_selection_view(SelectionViewParams {
            header: Box::new(header),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            initial_selected_idx,
            ..Default::default()
        });
    }

    /// Open the explicit Max/Ultra effort picker for the given model.
    pub(crate) fn open_advanced_reasoning_popup(&mut self, preset: ModelPreset) {
        let mut choices = preset
            .supported_reasoning_efforts
            .iter()
            .map(|option| option.effort.clone())
            .filter(Self::is_advanced_reasoning_effort)
            .collect::<Vec<_>>();
        if choices.is_empty()
            && Self::is_advanced_reasoning_effort(&preset.default_reasoning_effort)
        {
            choices.push(preset.default_reasoning_effort.clone());
        }
        choices.sort_by_key(|effort| matches!(effort, ReasoningEffortConfig::Ultra));
        if choices.is_empty() {
            return;
        }

        let model_slug = preset.model.to_string();
        let is_current_model = self.current_model() == preset.model.as_str();
        let highlight_choice = is_current_model
            .then(|| self.effective_reasoning_effort())
            .flatten();
        let mut items = Vec::new();
        for effort in choices {
            let description = match &effort {
                ReasoningEffortConfig::Max => model_popup_text(
                    "model-picker-advanced-max-description",
                    "For difficult problems when quality matters more than speed · higher usage",
                ),
                ReasoningEffortConfig::Ultra => model_popup_text(
                    "model-picker-advanced-ultra-description",
                    "For demanding work using multiple agents · highest usage",
                ),
                _ => unreachable!("advanced choices are limited to Max and Ultra"),
            };
            let should_prompt_plan_mode_scope = self
                .should_prompt_plan_mode_reasoning_scope(model_slug.as_str(), Some(effort.clone()));
            let actions = self.model_selection_actions(
                model_slug.clone(),
                Some(effort.clone()),
                should_prompt_plan_mode_scope,
            );

            items.push(SelectionItem {
                name: Self::reasoning_effort_label(&effort),
                description: Some(description),
                is_current: is_current_model && Some(&effort) == highlight_choice.as_ref(),
                actions,
                dismiss_on_select: true,
                ..Default::default()
            });
        }

        let mut header = ColumnRenderable::new();
        header.push(Line::from(
            model_popup_text("model-picker-advanced-title", "Advanced Reasoning").bold(),
        ));
        header.push(Line::from(
            model_popup_text(
                "model-picker-advanced-subtitle",
                "⚠ Consumes usage limits faster",
            )
            .cyan(),
        ));
        self.bottom_pane.show_selection_view(SelectionViewParams {
            header: Box::new(header),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            ..Default::default()
        });
    }

    pub(super) fn is_advanced_reasoning_effort(effort: &ReasoningEffortConfig) -> bool {
        matches!(
            effort,
            ReasoningEffortConfig::Max | ReasoningEffortConfig::Ultra
        )
    }

    pub(super) fn reasoning_effort_label(effort: &ReasoningEffortConfig) -> String {
        match effort {
            ReasoningEffortConfig::None => model_popup_text("reasoning-effort-none", "None"),
            ReasoningEffortConfig::Minimal => {
                model_popup_text("reasoning-effort-minimal", "Minimal")
            }
            ReasoningEffortConfig::Low => model_popup_text("reasoning-effort-low", "Low"),
            ReasoningEffortConfig::Medium => model_popup_text("reasoning-effort-medium", "Medium"),
            ReasoningEffortConfig::High => model_popup_text("reasoning-effort-high", "High"),
            ReasoningEffortConfig::XHigh => {
                model_popup_text("reasoning-effort-extra-high", "Extra high")
            }
            ReasoningEffortConfig::Max => model_popup_text("reasoning-effort-max", "Max"),
            ReasoningEffortConfig::Ultra => model_popup_text("reasoning-effort-ultra", "Ultra"),
            ReasoningEffortConfig::Custom(value) => value.clone(),
        }
    }

    pub(super) fn reasoning_effort_sentence_label(effort: &ReasoningEffortConfig) -> String {
        match effort {
            ReasoningEffortConfig::Custom(value) => value.clone(),
            effort => Self::reasoning_effort_label(effort).to_lowercase(),
        }
    }

    pub(super) fn ultra_reasoning_concurrency_warning(
        &self,
        effort: &ReasoningEffortConfig,
    ) -> Option<String> {
        if effort != &ReasoningEffortConfig::Ultra {
            return None;
        }

        let max_threads = self
            .config
            .multi_agent_v2
            .max_concurrent_threads_per_session;
        if max_threads < ULTRA_REASONING_CONCURRENCY_WARNING_THRESHOLD {
            return None;
        }

        let max_subagents = max_threads.saturating_sub(1);
        let mut args = FluentArgs::new();
        args.set("max_threads", max_threads);
        args.set("max_subagents", max_subagents);
        Some(crate::i18n::global().text(
            "model-picker-ultra-concurrency-warning",
            Some(&args),
            || {
                format!(
                    "Ultra reasoning may proactively use multiple agents. This session is configured for \
                     {max_threads} concurrent threads with up to {max_subagents} subagents which can \
                     increase usage quickly. Consider setting \
                     features.multi_agent_v2.max_concurrent_threads_per_session below 8."
                )
            },
        ))
    }

    pub(super) fn apply_model_and_effort_without_persist(
        &self,
        model: String,
        effort: Option<ReasoningEffortConfig>,
    ) {
        let warning = effort
            .as_ref()
            .and_then(|effort| self.ultra_reasoning_concurrency_warning(effort));
        self.app_event_tx.send(AppEvent::UpdateModel(model));
        self.app_event_tx
            .send(AppEvent::UpdateReasoningEffort(effort));
        if let Some(warning) = warning {
            self.app_event_tx.send(AppEvent::InsertHistoryCell(Box::new(
                history_cell::new_warning_event(warning),
            )));
        }
    }

    fn apply_model_and_effort(&self, model: String, effort: Option<ReasoningEffortConfig>) {
        self.apply_model_and_effort_without_persist(model.clone(), effort.clone());
        self.app_event_tx
            .send(AppEvent::PersistModelSelection { model, effort });
    }
}
