use crate::clock_format::ClockFormat;
use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use codex_app_server_protocol::RateLimitResetCreditStatus;
use codex_app_server_protocol::RateLimitResetCreditsSummary;
fn reset_text(key: &str, english: &'static str) -> String {
    crate::i18n::global().text(key, None, || english.to_string())
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct ResetCreditOption {
    pub(super) credit_id: Option<String>,
    pub(super) name: String,
    pub(super) detail: Option<String>,
    pub(super) description: String,
}

pub(super) fn reset_credit_options(
    summary: &RateLimitResetCreditsSummary,
    clock_format: ClockFormat,
) -> Vec<ResetCreditOption> {
    let available_count = summary.available_count.max(0);
    let detail_limit = usize::try_from(available_count).unwrap_or(usize::MAX);
    let mut available_credits = summary
        .credits
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter(|credit| credit.status == RateLimitResetCreditStatus::Available)
        .collect::<Vec<_>>();
    available_credits.sort_by_key(|credit| credit.expires_at.unwrap_or(i64::MAX));

    let mut options = available_credits
        .into_iter()
        .take(detail_limit)
        .map(|credit| {
            let expiration = match credit.expires_at {
                Some(expires_at) => DateTime::<Utc>::from_timestamp(expires_at, 0)
                    .map(|expires_at| {
                        let expires_at = expires_at.with_timezone(&Local);
                        format!(
                            "Expires {} on {}",
                            expires_at.format(clock_format.time_format()),
                            expires_at.format("%-d %b %Y")
                        )
                    })
                    .unwrap_or_else(|| {
                        reset_text(
                            "usage-reset-expiration-unavailable",
                            "Expiration unavailable",
                        )
                    }),
                None => reset_text("usage-reset-does-not-expire", "Does not expire"),
            };
            let reset_title = credit
                .title
                .as_deref()
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| reset_text("usage-reset-scope-full", "Full reset"));
            let reset_description = credit
                .description
                .as_deref()
                .map(str::trim)
                .filter(|description| !description.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| {
                    reset_text(
                        "usage-reset-description-full",
                        "Reset your current usage limits",
                    )
                });
            ResetCreditOption {
                credit_id: Some(credit.id.clone()),
                name: reset_title,
                detail: Some(expiration),
                description: reset_description,
            }
        })
        .collect::<Vec<_>>();

    if options.is_empty() {
        options.push(ResetCreditOption {
            credit_id: None,
            name: reset_text("usage-reset-scope-full", "Full reset"),
            detail: None,
            description: reset_text(
                "usage-reset-description-full",
                "Reset your current usage limits",
            ),
        });
    }

    options
}
