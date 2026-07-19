//! Review preset selection and custom review prompt surfaces.

use super::*;

fn review_popup_text(key: &str, english: &'static str) -> String {
    crate::i18n::global().text(key, None, || english.to_string())
}

impl ChatWidget {
    pub(crate) fn open_review_popup(&mut self) {
        let mut items: Vec<SelectionItem> = Vec::new();

        items.push(SelectionItem {
            name: review_popup_text("review-picker-base-branch", "Review against a base branch"),
            description: Some(review_popup_text("review-picker-pr-style", "(PR Style)")),
            actions: vec![Box::new({
                let cwd = self.config.cwd.to_path_buf();
                move |tx| {
                    tx.send(AppEvent::OpenReviewBranchPicker(cwd.clone()));
                }
            })],
            dismiss_on_select: false,
            dismiss_parent_on_child_accept: true,
            ..Default::default()
        });

        items.push(SelectionItem {
            name: review_popup_text("review-picker-uncommitted", "Review uncommitted changes"),
            actions: vec![Box::new(move |tx: &AppEventSender| {
                tx.review(ReviewTarget::UncommittedChanges);
            })],
            dismiss_on_select: true,
            ..Default::default()
        });

        items.push(SelectionItem {
            name: review_popup_text("review-picker-commit", "Review a commit"),
            actions: vec![Box::new({
                let cwd = self.config.cwd.to_path_buf();
                move |tx| {
                    tx.send(AppEvent::OpenReviewCommitPicker(cwd.clone()));
                }
            })],
            dismiss_on_select: false,
            dismiss_parent_on_child_accept: true,
            ..Default::default()
        });

        items.push(SelectionItem {
            name: review_popup_text(
                "review-picker-custom-instructions",
                "Custom review instructions",
            ),
            actions: vec![Box::new(move |tx| {
                tx.send(AppEvent::OpenReviewCustomPrompt);
            })],
            dismiss_on_select: false,
            dismiss_parent_on_child_accept: true,
            ..Default::default()
        });

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(review_popup_text(
                "review-picker-title",
                "Select a review preset",
            )),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            ..Default::default()
        });
    }

    pub(crate) async fn show_review_branch_picker(&mut self, cwd: &Path) {
        let branches = local_git_branches(cwd).await;
        let current_branch = current_branch_name(cwd)
            .await
            .unwrap_or_else(|| review_popup_text("review-picker-detached-head", "(detached HEAD)"));
        let mut items: Vec<SelectionItem> = Vec::with_capacity(branches.len());

        for option in branches {
            let branch = option.clone();
            items.push(SelectionItem {
                name: format!("{current_branch} -> {branch}"),
                actions: vec![Box::new(move |tx3: &AppEventSender| {
                    tx3.review(ReviewTarget::BaseBranch {
                        branch: branch.clone(),
                    });
                })],
                dismiss_on_select: true,
                search_value: Some(option),
                ..Default::default()
            });
        }

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(review_popup_text(
                "review-picker-base-branch-title",
                "Select a base branch",
            )),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            is_searchable: true,
            search_placeholder: Some(review_popup_text(
                "search-placeholder-branches",
                "Type to search branches",
            )),
            ..Default::default()
        });
    }

    pub(crate) async fn show_review_commit_picker(&mut self, cwd: &Path) {
        let commits = recent_commits(cwd, /*limit*/ 100).await;

        let mut items: Vec<SelectionItem> = Vec::with_capacity(commits.len());
        for entry in commits {
            let subject = entry.subject.clone();
            let sha = entry.sha.clone();
            let search_val = format!("{subject} {sha}");

            items.push(SelectionItem {
                name: subject.clone(),
                actions: vec![Box::new(move |tx3: &AppEventSender| {
                    tx3.review(ReviewTarget::Commit {
                        sha: sha.clone(),
                        title: Some(subject.clone()),
                    });
                })],
                dismiss_on_select: true,
                search_value: Some(search_val),
                ..Default::default()
            });
        }

        self.bottom_pane.show_selection_view(SelectionViewParams {
            title: Some(review_popup_text(
                "review-picker-commit-title",
                "Select a commit to review",
            )),
            footer_hint: Some(standard_popup_hint_line()),
            items,
            is_searchable: true,
            search_placeholder: Some(review_popup_text(
                "search-placeholder-commits",
                "Type to search commits",
            )),
            ..Default::default()
        });
    }

    pub(crate) fn show_review_custom_prompt(&mut self) {
        let tx = self.app_event_tx.clone();
        let view = CustomPromptView::new(
            review_popup_text(
                "review-picker-custom-instructions",
                "Custom review instructions",
            ),
            review_popup_text(
                "review-picker-custom-placeholder",
                "Type instructions and press Enter",
            ),
            /*initial_text*/ String::new(),
            /*context_label*/ None,
            Box::new(move |prompt: String| {
                let trimmed = prompt.trim().to_string();
                if trimmed.is_empty() {
                    return;
                }
                tx.review(ReviewTarget::Custom {
                    instructions: trimmed,
                });
            }),
        );
        self.bottom_pane.show_view(Box::new(view));
    }
}

#[cfg(test)]
pub(crate) fn show_review_commit_picker_with_entries(
    chat: &mut ChatWidget,
    entries: Vec<CommitLogEntry>,
) {
    let mut items: Vec<SelectionItem> = Vec::with_capacity(entries.len());
    for entry in entries {
        let subject = entry.subject.clone();
        let sha = entry.sha.clone();
        let search_val = format!("{subject} {sha}");

        items.push(SelectionItem {
            name: subject.clone(),
            actions: vec![Box::new(move |tx3: &AppEventSender| {
                tx3.review(ReviewTarget::Commit {
                    sha: sha.clone(),
                    title: Some(subject.clone()),
                });
            })],
            dismiss_on_select: true,
            search_value: Some(search_val),
            ..Default::default()
        });
    }

    chat.bottom_pane.show_selection_view(SelectionViewParams {
        title: Some(review_popup_text(
            "review-picker-commit-title",
            "Select a commit to review",
        )),
        footer_hint: Some(standard_popup_hint_line()),
        items,
        is_searchable: true,
        search_placeholder: Some(review_popup_text(
            "search-placeholder-commits",
            "Type to search commits",
        )),
        ..Default::default()
    });
}
