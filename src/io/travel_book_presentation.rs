//! Travel Book 向け presentation ルール（renderer 非依存）。
//!
//! Markdown / GUI / Web など複数 renderer が共有する表示判断を置く。
//! 構文（見出し級・表・箇条書き）は各 renderer 側に残す。

use chrono::{NaiveDate, NaiveDateTime, Timelike};

use crate::domain::models::{ExportNote, ItineraryCategory};

/// Daily schedule 向け itinerary カテゴリ詳細行（domain 定義の表示名）
pub(crate) fn format_travel_book_category_detail_line(category: ItineraryCategory) -> String {
    format!("- 種別: {}", category.definition().display_name)
}

/// Note entity の Travel Book 出力順（Trip → Day → Itinerary）
pub(crate) fn travel_book_note_sort_key(note: &ExportNote) -> (i32, i64, i64, String) {
    match note {
        ExportNote::Trip { title, body, .. } => {
            let label = title.as_deref().unwrap_or("").to_string();
            (0, 0, 0, label + body)
        }
        ExportNote::Day {
            day_number,
            title,
            body,
            ..
        } => {
            let label = title.as_deref().unwrap_or("").to_string();
            (1, *day_number, 0, label + body)
        }
        ExportNote::Itinerary {
            itinerary_key,
            title,
            body,
            ..
        } => {
            let label = title.as_deref().unwrap_or("").to_string();
            (
                2,
                itinerary_key.day_number,
                itinerary_key.sort_order,
                label + body,
            )
        }
    }
}

/// Note entity を Travel Book 順に並べ替える
pub(crate) fn sort_export_notes_for_travel_book(export_notes: &mut [ExportNote]) {
    export_notes.sort_by(|left, right| {
        travel_book_note_sort_key(left).cmp(&travel_book_note_sort_key(right))
    });
}

/// Provider が itinerary 見出しと冗長か（同一または相互包含なら省略）
pub(crate) fn reservation_provider_line_redundant(
    provider_name: &str,
    itinerary_title: &str,
) -> bool {
    let provider = provider_name.trim();
    let title = itinerary_title.trim();
    if provider.is_empty() {
        return true;
    }
    if provider == title {
        return true;
    }
    if title.contains(provider) || provider.contains(title) {
        return true;
    }
    false
}

fn parse_reservation_datetime(value: &str) -> Option<NaiveDateTime> {
    let trimmed = value.trim();
    NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S"))
        .ok()
}

fn parse_reservation_date(value: &str) -> Option<NaiveDate> {
    let trimmed = value.trim();
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok()
}

fn format_reservation_datetime_endpoint(value: &str) -> String {
    if let Some(dt) = parse_reservation_datetime(value) {
        return format!("{} {:02}:{:02}", dt.date(), dt.hour(), dt.minute());
    }
    if let Some(date) = parse_reservation_date(value) {
        return date.to_string();
    }
    value.trim().to_string()
}

/// Reservation の start/end を人間可読な期間文字列に整形する
pub(crate) fn format_travel_book_reservation_period(
    start_at: &Option<String>,
    end_at: &Option<String>,
) -> Option<String> {
    let start_raw = start_at
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let end_raw = end_at
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (start_raw, end_raw) {
        (Some(start), Some(end)) => {
            if let (Some(start_dt), Some(end_dt)) = (
                parse_reservation_datetime(start),
                parse_reservation_datetime(end),
            ) {
                let start_text = format!(
                    "{} {:02}:{:02}",
                    start_dt.date(),
                    start_dt.hour(),
                    start_dt.minute()
                );
                let end_text = if start_dt.date() == end_dt.date() {
                    format!("{:02}:{:02}", end_dt.hour(), end_dt.minute())
                } else {
                    format!(
                        "{} {:02}:{:02}",
                        end_dt.date(),
                        end_dt.hour(),
                        end_dt.minute()
                    )
                };
                return Some(format!("{start_text} 〜 {end_text}"));
            }
            Some(format!(
                "{} 〜 {}",
                format_reservation_datetime_endpoint(start),
                format_reservation_datetime_endpoint(end)
            ))
        }
        (Some(start), None) => Some(format_reservation_datetime_endpoint(start)),
        (None, Some(end)) => Some(format_reservation_datetime_endpoint(end)),
        (None, None) => None,
    }
}

/// Reservation 見出し行（provider 冗長時は subtitle 省略）
pub(crate) fn format_travel_book_reservation_heading(
    day_number: i64,
    itinerary_title: &str,
    provider_name: &str,
) -> String {
    let primary = format!("**Day {day_number} / {itinerary_title}**");
    if reservation_provider_line_redundant(provider_name, itinerary_title) {
        primary
    } else {
        format!("{primary} — {provider_name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::ItineraryCategory;

    #[test]
    fn test_format_travel_book_category_detail_line_uses_definition_display_name() {
        assert_eq!(
            format_travel_book_category_detail_line(ItineraryCategory::Transport),
            "- 種別: 移動"
        );
        assert_eq!(
            format_travel_book_category_detail_line(ItineraryCategory::Flight),
            "- 種別: フライト"
        );
        for category in ItineraryCategory::all() {
            let line = format_travel_book_category_detail_line(category);
            assert!(line.starts_with("- 種別: "));
            assert!(!line.contains(category.as_str()));
        }
    }

    #[test]
    fn test_reservation_provider_line_redundant() {
        assert!(reservation_provider_line_redundant(
            "NU045 NGO ⇒ OKA",
            "NU045 NGO ⇒ OKA (11:00着)"
        ));
        assert!(reservation_provider_line_redundant(
            "セントレア P1 G Parking",
            "P1 G Parking"
        ));
        assert!(!reservation_provider_line_redundant(
            "ヒルトン瀬底",
            "チェックイン"
        ));
        assert!(!reservation_provider_line_redundant(
            "Ks Rent A Car",
            "Toyota Alphard 又は同等車種"
        ));
    }

    #[test]
    fn test_format_travel_book_reservation_period_human_readable() {
        assert_eq!(
            format_travel_book_reservation_period(
                &Some("2026-04-26T16:40".to_string()),
                &Some("2026-04-29T10:00".to_string()),
            ),
            Some("2026-04-26 16:40 〜 2026-04-29 10:00".to_string())
        );
        assert_eq!(
            format_travel_book_reservation_period(
                &Some("2026-04-26T16:40".to_string()),
                &Some("2026-04-26T18:00".to_string()),
            ),
            Some("2026-04-26 16:40 〜 18:00".to_string())
        );
        assert_eq!(format_travel_book_reservation_period(&None, &None), None);
    }
}
