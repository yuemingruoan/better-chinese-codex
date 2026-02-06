use crate::i18n::tr;
use crate::i18n::tr_list;
use codex_core::features::FEATURES;
use codex_core::features::Feature;
use codex_protocol::account::PlanType;
use codex_protocol::config_types::Language;
use lazy_static::lazy_static;
use rand::Rng;

const ANNOUNCEMENT_TIP_URL: &str =
    "https://raw.githubusercontent.com/openai/codex/main/announcement_tip.toml";

lazy_static! {
    static ref TOOLTIPS_EN: Vec<String> = tr_list(Language::En, "tooltips.items").to_vec();
    static ref TOOLTIPS_ZH: Vec<String> = tr_list(Language::ZhCn, "tooltips.items").to_vec();
    static ref ALL_TOOLTIPS_EN: Vec<String> = {
        let mut tips = Vec::new();
        tips.extend(TOOLTIPS_EN.iter().cloned());
        tips.extend(experimental_tooltips(Language::En));
        tips
    };
    static ref ALL_TOOLTIPS_ZH: Vec<String> = {
        let mut tips = Vec::new();
        tips.extend(TOOLTIPS_ZH.iter().cloned());
        tips.extend(experimental_tooltips(Language::ZhCn));
        tips
    };
}

fn experimental_tooltips(language: Language) -> Vec<String> {
    FEATURES
        .iter()
        .filter_map(|spec| {
            spec.stage
                .experimental_announcement()
                .and_then(|_| experimental_tooltip(spec.id, language))
        })
        .collect()
}

fn experimental_tooltip(feature: Feature, language: Language) -> Option<String> {
    let key = match feature {
        Feature::ShellSnapshot => "tooltips.experimental.shell_snapshot",
        Feature::Collab => "tooltips.experimental.collab",
        Feature::Apps => "tooltips.experimental.apps",
        _ => return None,
    };
    Some(tr(language, key).to_string())
}

/// Pick a random tooltip to show to the user when starting Codex.
pub(crate) fn get_tooltip(plan: Option<PlanType>, language: Language) -> Option<String> {
    let mut rng = rand::rng();

    // Leave small chance for a random tooltip to be shown.
    if rng.random_ratio(8, 10) {
        let promo_key = match plan {
            Some(PlanType::Plus)
            | Some(PlanType::Business)
            | Some(PlanType::Team)
            | Some(PlanType::Enterprise)
            | Some(PlanType::Pro) => "tooltips.promo.paid",
            Some(PlanType::Go) | Some(PlanType::Free) => "tooltips.promo.free_go",
            _ => "tooltips.promo.other",
        };
        return Some(tr(language, promo_key).to_string());
    }

    if let Some(announcement) = announcement::fetch_announcement_tip() {
        return Some(announcement);
    }

    pick_tooltip(&mut rng, language).map(str::to_string)
}

fn pick_tooltip<R: Rng + ?Sized>(rng: &mut R, language: Language) -> Option<&'static str> {
    let tooltips = match language {
        Language::ZhCn => &*ALL_TOOLTIPS_ZH,
        Language::En => &*ALL_TOOLTIPS_EN,
    };
    if tooltips.is_empty() {
        None
    } else {
        tooltips
            .get(rng.random_range(0..tooltips.len()))
            .map(String::as_str)
    }
}

pub(crate) mod announcement {
    use crate::tooltips::ANNOUNCEMENT_TIP_URL;
    use crate::version::CODEX_CLI_VERSION;
    use chrono::NaiveDate;
    use chrono::Utc;
    use regex_lite::Regex;
    use serde::Deserialize;
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Duration;

    static ANNOUNCEMENT_TIP: OnceLock<Option<String>> = OnceLock::new();

    /// Prewarm the cache of the announcement tip.
    pub(crate) fn prewarm() {
        let _ = thread::spawn(|| ANNOUNCEMENT_TIP.get_or_init(init_announcement_tip_in_thread));
    }

    /// Fetch the announcement tip, return None if the prewarm is not done yet.
    pub(crate) fn fetch_announcement_tip() -> Option<String> {
        ANNOUNCEMENT_TIP
            .get()
            .cloned()
            .flatten()
            .and_then(|raw| parse_announcement_tip_toml(&raw))
    }

    #[derive(Debug, Deserialize)]
    struct AnnouncementTipRaw {
        content: String,
        from_date: Option<String>,
        to_date: Option<String>,
        version_regex: Option<String>,
        target_app: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct AnnouncementTipDocument {
        announcements: Vec<AnnouncementTipRaw>,
    }

    #[derive(Debug)]
    struct AnnouncementTip {
        content: String,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
        version_regex: Option<Regex>,
        target_app: String,
    }

    fn init_announcement_tip_in_thread() -> Option<String> {
        thread::spawn(blocking_init_announcement_tip)
            .join()
            .ok()
            .flatten()
    }

    fn blocking_init_announcement_tip() -> Option<String> {
        // Avoid system proxy detection to prevent macOS system-configuration panics (#8912).
        let client = reqwest::blocking::Client::builder()
            .no_proxy()
            .build()
            .ok()?;
        let response = client
            .get(ANNOUNCEMENT_TIP_URL)
            .timeout(Duration::from_millis(2000))
            .send()
            .ok()?;
        response.error_for_status().ok()?.text().ok()
    }

    pub(crate) fn parse_announcement_tip_toml(text: &str) -> Option<String> {
        let announcements = toml::from_str::<AnnouncementTipDocument>(text)
            .map(|doc| doc.announcements)
            .or_else(|_| toml::from_str::<Vec<AnnouncementTipRaw>>(text))
            .ok()?;

        let mut latest_match = None;
        let today = Utc::now().date_naive();
        for raw in announcements {
            let Some(tip) = AnnouncementTip::from_raw(raw) else {
                continue;
            };
            if tip.version_matches(CODEX_CLI_VERSION)
                && tip.date_matches(today)
                && tip.target_app == "cli"
            {
                latest_match = Some(tip.content);
            }
        }
        latest_match
    }

    impl AnnouncementTip {
        fn from_raw(raw: AnnouncementTipRaw) -> Option<Self> {
            let content = raw.content.trim();
            if content.is_empty() {
                return None;
            }

            let from_date = match raw.from_date {
                Some(date) => Some(NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?),
                None => None,
            };
            let to_date = match raw.to_date {
                Some(date) => Some(NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?),
                None => None,
            };
            let version_regex = match raw.version_regex {
                Some(pattern) => Some(Regex::new(&pattern).ok()?),
                None => None,
            };

            Some(Self {
                content: content.to_string(),
                from_date,
                to_date,
                version_regex,
                target_app: raw.target_app.unwrap_or("cli".to_string()).to_lowercase(),
            })
        }

        fn version_matches(&self, version: &str) -> bool {
            self.version_regex
                .as_ref()
                .is_none_or(|regex| regex.is_match(version))
        }

        fn date_matches(&self, today: NaiveDate) -> bool {
            if let Some(from) = self.from_date
                && today < from
            {
                return false;
            }
            if let Some(to) = self.to_date
                && today >= to
            {
                return false;
            }
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tooltips::announcement::parse_announcement_tip_toml;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn random_tooltip_returns_some_tip_when_available() {
        let mut rng = StdRng::seed_from_u64(42);
        assert!(pick_tooltip(&mut rng, Language::En).is_some());
    }

    #[test]
    fn random_tooltip_is_reproducible_with_seed() {
        let expected = {
            let mut rng = StdRng::seed_from_u64(7);
            pick_tooltip(&mut rng, Language::En)
        };

        let mut rng = StdRng::seed_from_u64(7);
        assert_eq!(expected, pick_tooltip(&mut rng, Language::En));
    }

    #[test]
    fn announcement_tip_toml_picks_last_matching() {
        let toml = r#"
[[announcements]]
content = "first"
from_date = "2000-01-01"

[[announcements]]
content = "latest match"
version_regex = ".*"
target_app = "cli"

[[announcements]]
content = "should not match"
to_date = "2000-01-01"
        "#;

        assert_eq!(
            Some("latest match".to_string()),
            parse_announcement_tip_toml(toml)
        );

        let toml = r#"
[[announcements]]
content = "first"
from_date = "2000-01-01"
target_app = "cli"

[[announcements]]
content = "latest match"
version_regex = ".*"

[[announcements]]
content = "should not match"
to_date = "2000-01-01"
        "#;

        assert_eq!(
            Some("latest match".to_string()),
            parse_announcement_tip_toml(toml)
        );
    }

    #[test]
    fn announcement_tip_toml_picks_no_match() {
        let toml = r#"
[[announcements]]
content = "first"
from_date = "2000-01-01"
to_date = "2000-01-05"

[[announcements]]
content = "latest match"
version_regex = "invalid_version_name"

[[announcements]]
content = "should not match either "
target_app = "vsce"
        "#;

        assert_eq!(None, parse_announcement_tip_toml(toml));
    }

    #[test]
    fn announcement_tip_toml_bad_deserialization() {
        let toml = r#"
[[announcements]]
content = 123
from_date = "2000-01-01"
        "#;

        assert_eq!(None, parse_announcement_tip_toml(toml));
    }

    #[test]
    fn announcement_tip_toml_parse_comments() {
        let toml = r#"
# Example announcement tips for Codex TUI.
# Each [[announcements]] entry is evaluated in order; the last matching one is shown.
# Dates are UTC, formatted as YYYY-MM-DD. The from_date is inclusive and the to_date is exclusive.
# version_regex matches against the CLI version (env!("CARGO_PKG_VERSION")); omit to apply to all versions.
# target_app specify which app should display the announcement (cli, vsce, ...).

[[announcements]]
content = "Welcome to Codex! Check out the new onboarding flow."
from_date = "2024-10-01"
to_date = "2024-10-15"
target_app = "cli"
version_regex = "^0\\.0\\.0$"

[[announcements]]
content = "This is a test announcement"
        "#;

        assert_eq!(
            Some("This is a test announcement".to_string()),
            parse_announcement_tip_toml(toml)
        );
    }
}
