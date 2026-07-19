//! Permission and approval popup flows for `ChatWidget`.
//!
//! This module owns the generic permission pickers and confirmation surfaces;
//! Windows-specific sandbox prompting lives beside it in
//! `windows_sandbox_prompts`.

use super::permission_i18n;
use super::*;

impl ChatWidget {
    /// Open the permissions popup.
    pub(crate) fn open_approvals_popup(&mut self) {
        self.open_permissions_popup();
    }

    /// Open a popup to choose the permissions mode.
    pub(crate) fn open_permissions_popup(&mut self) {
        if self.config.explicit_permission_profile_mode {
            self.open_permission_profiles_popup();
            return;
        }

        let include_read_only = cfg!(target_os = "windows");
        let current_approval =
            AskForApproval::from(self.config.permissions.approval_policy.value());
        let current_permission_profile = self.config.permissions.permission_profile().clone();
        let guardian_approval_enabled = self.config.features.enabled(Feature::GuardianApproval);
        let current_review_policy = self.config.approvals_reviewer;
        let mut items: Vec<SelectionItem> = Vec::new();
        let presets: Vec<ApprovalPreset> = builtin_approval_presets();

        #[cfg(target_os = "windows")]
        let windows_sandbox_level = crate::windows_sandbox::level_from_config(&self.config);
        #[cfg(target_os = "windows")]
        let windows_degraded_sandbox_enabled =
            matches!(windows_sandbox_level, WindowsSandboxLevel::RestrictedToken);
        #[cfg(not(target_os = "windows"))]
        let windows_degraded_sandbox_enabled = false;

        let show_elevate_sandbox_hint =
            windows_degraded_sandbox_enabled && presets.iter().any(|preset| preset.id == "auto");

        let guardian_disabled_reason = |enabled: bool| {
            let mut next_features = self.config.features.get().clone();
            next_features.set_enabled(Feature::GuardianApproval, enabled);
            self.config
                .features
                .can_set(&next_features)
                .err()
                .map(|err| err.to_string())
        };

        for preset in presets.into_iter() {
            if !include_read_only && preset.id == "read-only" {
                continue;
            }
            let base_name = if preset.id == "auto" && windows_degraded_sandbox_enabled {
                let label = permission_i18n::ask_for_approval_label();
                let suffix =
                    permission_i18n::text("permissions-non-admin-sandbox", "non-admin sandbox");
                format!("{label} ({suffix})")
            } else if preset.id == "auto" {
                permission_i18n::ask_for_approval_label()
            } else {
                permission_i18n::preset_label(&preset)
            };
            let base_description = Some(permission_i18n::preset_description(&preset));
            let approval_disabled_reason = match self
                .config
                .permissions
                .approval_policy
                .can_set(&preset.approval)
            {
                Ok(()) => None,
                Err(err) => Some(err.to_string()),
            };
            let default_disabled_reason = approval_disabled_reason
                .clone()
                .or_else(|| guardian_disabled_reason(false));
            let default_actions = self.permission_mode_actions(
                &preset,
                base_name.clone(),
                ApprovalsReviewer::User,
                /*profile_selection*/ None,
                /*return_to_permissions*/ !include_read_only,
            );
            if preset.id == "auto" {
                items.push(SelectionItem {
                    name: base_name.clone(),
                    description: base_description.clone(),
                    is_current: current_review_policy == ApprovalsReviewer::User
                        && Self::preset_matches_current(
                            current_approval,
                            &current_permission_profile,
                            self.config.cwd.as_path(),
                            &preset,
                        ),
                    actions: default_actions,
                    dismiss_on_select: true,
                    disabled_reason: default_disabled_reason,
                    ..Default::default()
                });

                if guardian_approval_enabled {
                    let approve_for_me = permission_i18n::approve_for_me_label();
                    items.push(SelectionItem {
                        name: approve_for_me.clone(),
                        description: Some(permission_i18n::auto_review_description()),
                        is_current: current_review_policy == ApprovalsReviewer::AutoReview
                            && Self::preset_matches_current(
                                current_approval,
                                &current_permission_profile,
                                self.config.cwd.as_path(),
                                &preset,
                            ),
                        actions: self.permission_mode_actions(
                            &preset,
                            approve_for_me,
                            ApprovalsReviewer::AutoReview,
                            /*profile_selection*/ None,
                            /*return_to_permissions*/ !include_read_only,
                        ),
                        dismiss_on_select: true,
                        disabled_reason: approval_disabled_reason
                            .or_else(|| guardian_disabled_reason(true)),
                        ..Default::default()
                    });
                }
            } else {
                items.push(SelectionItem {
                    name: base_name,
                    description: base_description,
                    is_current: Self::preset_matches_current(
                        current_approval,
                        &current_permission_profile,
                        self.config.cwd.as_path(),
                        &preset,
                    ),
                    actions: default_actions,
                    dismiss_on_select: true,
                    disabled_reason: default_disabled_reason,
                    ..Default::default()
                });
            }
        }

        let footer_note = show_elevate_sandbox_hint.then(|| {
            vec![
                permission_i18n::text(
                    "permissions-non-admin-note-prefix",
                    "The non-admin sandbox protects your files and prevents network access under most circumstances. However, it carries greater risk if prompt injected. To upgrade to the default sandbox, run",
                )
                .dim(),
                " ".into(),
                "/setup-default-sandbox".cyan(),
                ".".dim(),
            ]
            .into()
        });

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(permission_i18n::text(
                "permissions-picker-title",
                "Update Model Permissions",
            )),
            footer_note,
            footer_hint: Some(standard_popup_hint_line()),
            items,
            header: Box::new(()),
            ..Default::default()
        });
    }

    pub(crate) fn open_auto_review_denials_popup(&mut self) {
        if self.review.recent_auto_review_denials.is_empty() {
            self.add_info_message(
                permission_i18n::text(
                    "auto-review-no-recent-denials",
                    "No recent auto-review denials in this thread.",
                ),
                Some(permission_i18n::text(
                    "auto-review-denials-recorded-hint",
                    "Denials are recorded after auto-review rejects an action.",
                )),
            );
            return;
        }
        let Some(thread_id) = self.thread_id() else {
            self.add_error_message(permission_i18n::text(
                "auto-review-thread-unavailable",
                "That thread is no longer available.",
            ));
            return;
        };

        let mut items = vec![SelectionItem {
            name: permission_i18n::text("auto-review-command", "Command"),
            description: Some(permission_i18n::text("auto-review-rationale", "Rationale")),
            is_disabled: true,
            search_value: Some(String::new()),
            ..Default::default()
        }];
        items.extend(
            self.review
                .recent_auto_review_denials
                .entries()
                .map(|event| {
                    let id = event.id.clone();
                    let summary = auto_review_denials::action_summary(&event.action);
                    let rationale = event
                        .rationale
                        .as_deref()
                        .map(str::to_string)
                        .unwrap_or_else(|| {
                            permission_i18n::text(
                                "auto-review-missing-rationale",
                                "Auto-review did not include a rationale.",
                            )
                        });
                    SelectionItem {
                        name: summary.clone(),
                        description: Some(rationale.clone()),
                        selected_description: Some(rationale.clone()),
                        search_value: Some(format!("{summary} {rationale}")),
                        actions: vec![Box::new(move |tx| {
                            tx.send(AppEvent::ApproveRecentAutoReviewDenial {
                                thread_id,
                                id: id.clone(),
                            });
                        })],
                        dismiss_on_select: true,
                        ..Default::default()
                    }
                }),
        );

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(permission_i18n::text(
                "auto-review-denials-title",
                "Auto-review Denials",
            )),
            subtitle: Some(permission_i18n::text(
                "auto-review-denials-subtitle",
                "Select a denied action to approve.",
            )),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            is_searchable: true,
            col_width_mode: ColumnWidthMode::AutoAllRows,
            ..Default::default()
        });
        self.request_redraw();
    }

    pub(crate) fn approve_recent_auto_review_denial(&mut self, thread_id: ThreadId, id: String) {
        let Some(event) = self.review.recent_auto_review_denials.take(&id) else {
            self.add_error_message(permission_i18n::text(
                "auto-review-denial-unavailable",
                "That auto-review denial is no longer available.",
            ));
            return;
        };

        self.app_event_tx.send(AppEvent::SubmitThreadOp {
            thread_id,
            op: AppCommand::approve_guardian_denied_action(event),
        });
        self.add_info_message(
            permission_i18n::text(
                "auto-review-approval-recorded",
                "Approval recorded for one retry of the selected auto-review denial.",
            ),
            Some(permission_i18n::text(
                "auto-review-retry-hint",
                "The model will see the approval context; the retry still goes through auto-review.",
            )),
        );
    }

    pub(super) fn approval_preset_actions(
        approval: AskForApproval,
        permission_profile: PermissionProfile,
        active_permission_profile: ActivePermissionProfile,
        label: String,
        approvals_reviewer: ApprovalsReviewer,
    ) -> Vec<SelectionAction> {
        vec![Box::new(move |tx| {
            tx.send(AppEvent::CodexOp(AppCommand::override_turn_context(
                /*cwd*/ None,
                Some(approval),
                Some(approvals_reviewer),
                Some(permission_profile.clone()),
                Some(active_permission_profile.clone()),
                /*windows_sandbox_level*/ None,
                /*model*/ None,
                /*effort*/ None,
                /*summary*/ None,
                /*service_tier*/ None,
                /*collaboration_mode*/ None,
                /*personality*/ None,
            )));
            tx.send(AppEvent::UpdateAskForApprovalPolicy(approval));
            tx.send(AppEvent::UpdateActivePermissionProfile(
                active_permission_profile.clone(),
            ));
            tx.send(AppEvent::UpdateApprovalsReviewer(approvals_reviewer));
            tx.send(AppEvent::InsertHistoryCell(Box::new(
                history_cell::new_info_event(
                    permission_i18n::text_with_arg(
                        "permissions-updated",
                        "label",
                        label.clone(),
                        || format!("Permissions updated to {label}"),
                    ),
                    /*hint*/ None,
                ),
            )));
        })]
    }

    pub(super) fn permission_profile_selection_actions(
        selection: PermissionProfileSelection,
    ) -> Vec<SelectionAction> {
        vec![Box::new(move |tx| {
            tx.send(AppEvent::SelectPermissionProfile(selection.clone()));
        })]
    }

    pub(super) fn permission_mode_actions(
        &self,
        preset: &ApprovalPreset,
        label: String,
        approvals_reviewer: ApprovalsReviewer,
        profile_selection: Option<PermissionProfileSelection>,
        return_to_permissions: bool,
    ) -> Vec<SelectionAction> {
        let apply_actions = || {
            profile_selection.clone().map_or_else(
                || {
                    Self::approval_preset_actions(
                        AskForApproval::from(preset.approval),
                        preset.permission_profile.clone(),
                        preset.active_permission_profile.clone(),
                        label.clone(),
                        approvals_reviewer,
                    )
                },
                Self::permission_profile_selection_actions,
            )
        };
        let requires_confirmation =
            approvals_reviewer == ApprovalsReviewer::User && preset.id == "full-access";
        if requires_confirmation {
            let preset = preset.clone();
            return vec![Box::new(move |tx| {
                tx.send(AppEvent::OpenFullAccessConfirmation {
                    preset: preset.clone(),
                    return_to_permissions,
                    profile_selection: profile_selection.clone(),
                });
            })];
        }
        if approvals_reviewer == ApprovalsReviewer::User && preset.id == "auto" {
            #[cfg(target_os = "windows")]
            {
                if crate::windows_sandbox::level_from_config(&self.config)
                    == WindowsSandboxLevel::Disabled
                {
                    let preset = preset.clone();
                    if crate::windows_sandbox::sandbox_setup_is_complete(
                        self.config.codex_home.as_path(),
                    ) {
                        return vec![Box::new(move |tx| {
                            tx.send(AppEvent::EnableWindowsSandboxForAgentMode {
                                preset: preset.clone(),
                                mode: WindowsSandboxEnableMode::Elevated,
                                profile_selection: profile_selection.clone(),
                            });
                        })];
                    }
                    return vec![Box::new(move |tx| {
                        tx.send(AppEvent::OpenWindowsSandboxEnablePrompt {
                            preset: preset.clone(),
                            profile_selection: profile_selection.clone(),
                        });
                    })];
                }
                if let Some((sample_paths, extra_count, failed_scan)) =
                    self.world_writable_warning_details()
                {
                    let preset = preset.clone();
                    return vec![Box::new(move |tx| {
                        tx.send(AppEvent::OpenWorldWritableWarningConfirmation {
                            preset: Some(preset.clone()),
                            profile_selection: profile_selection.clone(),
                            sample_paths: sample_paths.clone(),
                            extra_count,
                            failed_scan,
                        });
                    })];
                }
            }
        }
        apply_actions()
    }

    pub(super) fn preset_matches_current(
        current_approval: AskForApproval,
        current_permission_profile: &PermissionProfile,
        cwd: &std::path::Path,
        preset: &ApprovalPreset,
    ) -> bool {
        let preset_approval = AskForApproval::from(preset.approval);
        if current_approval != preset_approval {
            return false;
        }

        match preset.id {
            "full-access" => matches!(current_permission_profile, PermissionProfile::Disabled),
            "read-only" => {
                let file_system_policy = current_permission_profile.file_system_sandbox_policy();
                matches!(
                    current_permission_profile,
                    PermissionProfile::Managed { .. }
                ) && !file_system_policy.has_full_disk_write_access()
                    && file_system_policy
                        .get_writable_roots_with_cwd(cwd)
                        .is_empty()
                    && current_permission_profile.network_sandbox_policy()
                        == preset.permission_profile.network_sandbox_policy()
            }
            "auto" => {
                let file_system_policy = current_permission_profile.file_system_sandbox_policy();
                matches!(
                    current_permission_profile,
                    PermissionProfile::Managed { .. }
                ) && file_system_policy.can_write_path_with_cwd(cwd, cwd)
                    && !file_system_policy.has_full_disk_write_access()
                    && current_permission_profile.network_sandbox_policy()
                        == preset.permission_profile.network_sandbox_policy()
            }
            _ => current_permission_profile == &preset.permission_profile,
        }
    }

    pub(crate) fn open_full_access_confirmation(
        &mut self,
        preset: ApprovalPreset,
        return_to_permissions: bool,
        profile_selection: Option<PermissionProfileSelection>,
    ) {
        let selected_name = permission_i18n::preset_label(&preset);
        let approval = AskForApproval::from(preset.approval);
        let mut header_children: Vec<Box<dyn Renderable>> = Vec::new();
        let title_line = Line::from(permission_i18n::text(
            "permissions-full-access-confirm-title",
            "Enable full access?",
        ))
        .bold();
        let info_line = Line::from(vec![
            format!(
                "{} ",
                permission_i18n::text(
                    "permissions-full-access-confirm-info",
                    "When Codex runs with full access, it can edit any file on your computer and run commands with network, without your approval.",
                )
            )
            .into(),
            permission_i18n::text(
                "permissions-full-access-confirm-warning",
                "Exercise caution when enabling full access. This significantly increases the risk of data loss, leaks, or unexpected behavior.",
            )
            .fg(Color::Red),
        ]);
        header_children.push(Box::new(title_line));
        header_children.push(Box::new(
            Paragraph::new(vec![info_line]).wrap(Wrap { trim: false }),
        ));
        let header = ColumnRenderable::with(header_children);

        let accept_actions = profile_selection.map_or_else(
            || {
                Self::approval_preset_actions(
                    approval,
                    preset.permission_profile,
                    preset.active_permission_profile,
                    selected_name,
                    ApprovalsReviewer::User,
                )
            },
            Self::permission_profile_selection_actions,
        );

        let deny_actions: Vec<SelectionAction> = vec![Box::new(move |tx| {
            if return_to_permissions {
                tx.send(AppEvent::OpenPermissionsPopup);
            } else {
                tx.send(AppEvent::OpenApprovalsPopup);
            }
        })];

        let items = vec![
            SelectionItem {
                name: permission_i18n::text(
                    "permissions-full-access-continue",
                    "Yes, continue anyway",
                ),
                description: Some(permission_i18n::text(
                    "permissions-full-access-session-description",
                    "Apply full access for this session",
                )),
                actions: accept_actions,
                dismiss_on_select: true,
                ..Default::default()
            },
            SelectionItem {
                name: permission_i18n::text("permissions-cancel", "Cancel"),
                description: Some(permission_i18n::text(
                    "permissions-full-access-cancel-description",
                    "Go back without enabling full access",
                )),
                actions: deny_actions,
                dismiss_on_select: true,
                ..Default::default()
            },
        ];

        self.bottom_pane.show_selection_view(SelectionViewParams {
            footer_hint: Some(standard_popup_hint_line()),
            items,
            header: Box::new(header),
            ..Default::default()
        });
    }
}
