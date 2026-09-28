# CCU maintenance cadence

CCU updates are opt-in. There is no scheduled patch replay, resolver launch,
multi-platform build, fork release, or downstream channel synchronization.

`CCU weekly maintenance reminder` runs on Mondays at 02:17 UTC (10:17
Asia/Shanghai). It reads the latest upstream/fork releases, the manager's stable
channel, unresolved conflicts for unpublished releases, and the latest workflow
results in both repositories. It uses read-only permissions and does not change
issues, branches, tags, or releases.

If work is pending and no reminder has failed during the previous seven days,
the weekly workflow deliberately reports a failure with a maintenance summary.
This reuses GitHub Actions failure-email notifications; receipt still depends on
the user's GitHub notification settings. A red weekly reminder means maintenance
is pending, not that an upgrade was attempted. Successful checks are silent for
users who subscribe only to failed workflows.

Repeated runs are suppressed, including reruns and attempts within seven days of
a previous failed reminder. If the notification history cannot be read, the check
skips sending rather than risking another notification. Use the manual dispatch's
default `dry_run: true` to preview the report without requesting a failure email.
GitHub runner/checkout outages and manually triggered CI retain their native
notification behavior; this policy controls automatic maintenance reminders.

When an update is wanted:

1. Manually run `CCU i18n release` in `Cec1c/codex`, selecting the upstream tag or
   a previously prepared release branch as needed. Explicit release attempts
   retain the existing conflict-resolution handoff and real failure reporting.
2. After the fork release succeeds, manually run `Sync fork release channel` in
   `Cec1c/codex-cli-ultra`. It validates the fork release and publishes the manager
   through the existing deterministic release flow.

Normal push/PR CI and manual release failures remain visible immediately. No
global GitHub email preferences are changed.
