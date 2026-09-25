//! Status filters and viewport state survive refreshes of the shared task projection.

mod hints;
mod input;
mod navigation;
mod render;
mod rows;

use super::*;

// Counts and filtering use the same status groups.
pub(super) static TASK_FILTERS: std::sync::LazyLock<[(&str, Option<AgentsOverviewGroup>); 5]> =
    std::sync::LazyLock::new(|| {
        [
            (crate::i18n::tr!("ui-all", "All"), None),
            (
                crate::i18n::tr!("agents-needs-you", "Needs you"),
                Some(AgentsOverviewGroup::NeedsYou),
            ),
            (
                crate::i18n::tr!("agents-working", "Working"),
                Some(AgentsOverviewGroup::Working),
            ),
            (
                crate::i18n::tr!("agents-ready", "Ready"),
                Some(AgentsOverviewGroup::Ready),
            ),
            (
                crate::i18n::tr!("agents-inactive", "Inactive"),
                Some(AgentsOverviewGroup::Finished),
            ),
        ]
    });

impl AgentsOverviewView {
    pub(in crate::app::agents_overview_view) fn reconcile_command_center_selection(&mut self) {
        let visible = self.visible_indices();
        if !visible.contains(&self.selected) {
            self.selected = visible.first().copied().unwrap_or(usize::MAX);
        }
    }
}
