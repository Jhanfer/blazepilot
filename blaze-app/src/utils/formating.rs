use chrono::{DateTime, Local, TimeDelta};
use std::time::{Duration, UNIX_EPOCH};

use crate::core::bootstrap::configs::config_manager::with_configs;

pub fn format_size(size: u64) -> String {
    const KB: u64 = 1000;
    const MB: u64 = KB * 1000;
    const GB: u64 = MB * 1000;
    const TB: u64 = GB * 1000;
    const PB: u64 = TB * 1000;
    const EB: u64 = PB * 1000;

    match size {
        0 => "0 B".to_string(),
        s if s < KB => format!("{} B", s),
        s if s < MB => format!("{:.1} KB", s as f64 / KB as f64),
        s if s < GB => format!("{:.1} MB", s as f64 / MB as f64),
        s if s < TB => format!("{:.1} GB", s as f64 / GB as f64),
        s if s < PB => format!("{:.1} TB", s as f64 / TB as f64),
        s if s < EB => format!("{:.1} PB", s as f64 / PB as f64),
        s => format!("{:.1} EB", s as f64 / EB as f64),
    }
}

// Asumiendo que tienes acceso a tu struct I18n
pub fn format_date(seconds: u64) -> Box<str> {
    let i18n = with_configs(|c| c.get_i18n());
    let d = UNIX_EPOCH + Duration::from_secs(seconds);
    let Ok(elapsed) = d.elapsed() else {
        return "—".into();
    };

    let secs = elapsed.as_secs();
    let query = |val: u64| val.to_string();

    match secs {
        0..=59 => i18n.t("modified_date_formating.now"),
        60..=3599 => i18n.t_args(
            "modified_date_formating.min",
            &[("query", &query(secs / 60))],
        ),
        3600..=86399 => i18n.t_args(
            "modified_date_formating.hour",
            &[("query", &query(secs / 3600))],
        ),
        86400..=604799 => i18n.t_args(
            "modified_date_formating.day",
            &[("query", &query(secs / 86400))],
        ),
        604800..=2591999 => i18n.t_args(
            "modified_date_formating.week",
            &[("query", &query(secs / 604800))],
        ),
        2592000..=31535999 => i18n.t_args(
            "modified_date_formating.month",
            &[("query", &query(secs / 2592000))],
        ),
        _ => i18n.t_args(
            "modified_date_formating.year",
            &[("query", &query(secs / 31536000))],
        ),
    }
}

pub fn _format_date_deprecate(seconds: u64) -> String {
    if seconds == 0 {
        return "---".to_string();
    }

    let d = UNIX_EPOCH + Duration::from_secs(seconds);

    let modified_date: DateTime<Local> = d.into();
    let now: DateTime<Local> = Local::now();

    let diff: TimeDelta = now - modified_date;

    let min: i32 = diff.num_minutes() as i32;
    let hours: i32 = diff.num_hours() as i32;
    let days: i32 = diff.num_days() as i32;
    let weeks: i32 = days / 7;
    let months: i32 = weeks / 4;
    let years: i32 = months / 12;

    if min < 60 {
        return format!("Hace {:?}min", min).to_string();
    } else if hours < 24 {
        return format!("Hace {:?}h", hours).to_string();
    } else if days < 7 {
        return format!("Hace {:?} dia/s", days).to_string();
    } else if weeks < 4 {
        return format!("Hace {:?} semana/s", weeks).to_string();
    } else if months < 12 {
        return format!("Hace {:?} mes/ses", months).to_string();
    } else if years > 1 {
        return format!("Hace {:?} año/s", years).to_string();
    }
    "desconocido".to_string()
}

fn _format_time(seconds: u64) -> String {
    if seconds == 0 {
        return "---".to_string();
    }

    let d = UNIX_EPOCH + Duration::from_secs(seconds);
    let datetime: DateTime<Local> = d.into();
    datetime.format("%d/%m/%Y %H:%M").to_string()
}
