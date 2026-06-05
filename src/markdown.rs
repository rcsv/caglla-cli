use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::models::{ChecklistItem, ItineraryItem, Trip};

/// Markdown 出力用に日程一覧を取得する（day → sort_order → id 順）
pub(crate) fn list_itinerary_items_for_markdown(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<ItineraryItem>> {
    crate::trip::get_trip(conn, trip_id)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, trip_id, day, title, note, start_time, sort_order,
                    duration_minutes, travel_minutes, location, category, created_at, updated_at
             FROM itinerary_items
             WHERE trip_id = ?1
             ORDER BY day, sort_order, id",
        )
        .context("日程一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id], crate::itinerary::row_to_itinerary_item)
        .context("日程一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("日程一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// 旅行の日付範囲を Markdown 用の1行テキストに整形する
pub(crate) fn format_trip_date_range(trip: &Trip) -> Option<String> {
    match (&trip.start_date, &trip.end_date) {
        (Some(start), Some(end)) => Some(format!("{start} 〜 {end}")),
        (Some(start), None) => Some(start.clone()),
        (None, Some(end)) => Some(end.clone()),
        (None, None) => None,
    }
}

/// 1件の日程を Markdown 形式に整形する
pub(crate) fn format_itinerary_item_markdown(item: &ItineraryItem) -> String {
    let mut lines = Vec::new();
    let heading = match &item.start_time {
        Some(time) => format!("### {time} {}", item.title),
        None => format!("### {}", item.title),
    };
    lines.push(heading);

    let mut detail_lines = Vec::new();
    if let Some(location) = &item.location {
        detail_lines.push(format!("- 場所: {location}"));
    }
    if let Some(duration) = item.duration_minutes {
        detail_lines.push(format!("- 所要時間: {duration}分"));
    }
    if let Some(travel) = item.travel_minutes {
        detail_lines.push(format!("- 移動時間: {travel}分"));
    }
    if let Some(note) = &item.note {
        detail_lines.push(format!("- メモ: {note}"));
    }

    if let Some(category) = item.category {
        lines.push(String::new());
        lines.push(format!("Category: {}", category.as_str()));
    }
    if !detail_lines.is_empty() {
        if item.category.is_some() {
            lines.push(String::new());
        }
        lines.extend(detail_lines);
    }

    lines.join("\n")
}

/// チェックリスト一覧を Markdown 形式に整形する（項目がなければ None）
pub(crate) fn format_checklist_markdown(items: &[ChecklistItem]) -> Option<String> {
    if items.is_empty() {
        return None;
    }

    let mut lines = vec!["## Checklist".to_string(), String::new()];
    for item in items {
        let mark = if item.is_done { 'x' } else { ' ' };
        lines.push(format!("- [{mark}] {}", item.title));
    }
    Some(format!("\n{}\n", lines.join("\n")))
}

/// 旅行と日程一覧から Markdown 文字列を組み立てる
pub(crate) fn format_trip_markdown(
    trip: &Trip,
    items: &[ItineraryItem],
    checklist: &[ChecklistItem],
) -> String {
    let mut output = format!("# {}\n", trip.name);
    if let Some(dates) = format_trip_date_range(trip) {
        output.push('\n');
        output.push_str(&dates);
        output.push('\n');
    }

    let mut current_day: Option<i64> = None;
    for item in items {
        if current_day != Some(item.day) {
            output.push_str(&format!("\n## Day {}\n\n", item.day));
            current_day = Some(item.day);
        } else {
            output.push('\n');
        }
        output.push_str(&format_itinerary_item_markdown(item));
    }

    if let Some(checklist_md) = format_checklist_markdown(checklist) {
        output.push_str(&checklist_md);
    }

    output
}

/// 旅行しおりを Markdown 文字列として組み立てる
pub(crate) fn build_trip_markdown(conn: &Connection, trip_id: i64) -> Result<String> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    let items = list_itinerary_items_for_markdown(conn, trip_id)?;
    let checklist = crate::checklist::list_checklist_items(conn, trip_id)?;
    Ok(format_trip_markdown(&trip, &items, &checklist))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checklist::{add_checklist_item, set_checklist_done};
    use crate::db::open_db_at;
    use crate::itinerary::add_itinerary_item;
    use crate::models::ItineraryCategory;
    use crate::trip::add_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_export_md_day_and_sort_order() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "並び順テスト", None, None).unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目・後",
            None,
            Some("14:00"),
            Some(2),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目・先",
            None,
            Some("09:00"),
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "1日目",
            None,
            Some("10:00"),
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let md = build_trip_markdown(&conn, trip_id).unwrap();
        let day1_pos = md.find("## Day 1").unwrap();
        let day2_pos = md.find("## Day 2").unwrap();
        let first_item_pos = md.find("### 10:00 1日目").unwrap();
        let second_day_first_pos = md.find("### 09:00 2日目・先").unwrap();
        let second_day_second_pos = md.find("### 14:00 2日目・後").unwrap();

        assert!(day1_pos < day2_pos);
        assert!(day1_pos < first_item_pos);
        assert!(second_day_first_pos < second_day_second_pos);
    }

    #[test]
    fn test_export_md_includes_category() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "ハワイ旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Hilton Hawaiian Village",
            None,
            None,
            None,
            None,
            None,
            Some("Waikiki"),
            Some(ItineraryCategory::Hotel),
        )
        .unwrap();

        let md = build_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("### Hilton Hawaiian Village"));
        assert!(md.contains("Category: hotel"));
        assert!(md.contains("- 場所: Waikiki"));
        let category_pos = md.find("Category: hotel").unwrap();
        let location_pos = md.find("- 場所: Waikiki").unwrap();
        assert!(category_pos < location_pos);
    }

    #[test]
    fn test_export_md_includes_checklist() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        let charger_id = add_checklist_item(&conn, trip_id, "充電器").unwrap();
        set_checklist_done(&conn, charger_id, true).unwrap();

        let md = build_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("## Checklist"));
        assert!(md.contains("- [ ] パスポート"));
        assert!(md.contains("- [x] 充電器"));
        assert!(md.find("## Checklist").unwrap() > md.find("# 沖縄旅行").unwrap());

        // 一覧表示と同じ並び（未完了が先）
        let passport_pos = md.find("- [ ] パスポート").unwrap();
        let charger_pos = md.find("- [x] 充電器").unwrap();
        assert!(passport_pos < charger_pos);
    }

    #[test]
    fn test_export_md_no_checklist_section() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        let md = build_trip_markdown(&conn, trip_id).unwrap();
        assert!(!md.contains("## Checklist"));
    }
    #[test]
    fn test_export_md_omits_category_when_unset() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let md = build_trip_markdown(&conn, trip_id).unwrap();
        assert!(!md.contains("Category:"));
    }

    #[test]
    fn test_export_md_optional_fields_omitted() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "ミニマル旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "散歩",
            None,
            None,
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let md = build_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("### 散歩"));
        assert!(!md.contains("- 場所:"));
        assert!(!md.contains("- 所要時間:"));
        assert!(!md.contains("- 移動時間:"));
        assert!(!md.contains("- メモ:"));
    }

    #[test]
    fn test_export_md_start_time_with_and_without() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "テスト旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "朝食",
            None,
            Some("08:00"),
            Some(1),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "自由時間",
            None,
            None,
            Some(2),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let md = build_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("### 08:00 朝食"));
        assert!(md.contains("### 自由時間"));
        assert!(!md.contains("### 自由時間 自由時間"));
    }

    #[test]
    fn test_export_md_with_itinerary() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", Some("2026-04-26"), Some("2026-04-29")).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "那覇空港",
            Some("レンタカー受け取り"),
            Some("09:00"),
            Some(1),
            Some(60),
            Some(30),
            Some("那覇空港"),
            None,
        )
        .unwrap();

        let md = build_trip_markdown(&conn, trip_id).unwrap();
        assert!(md.contains("# 沖縄旅行"));
        assert!(md.contains("2026-04-26 〜 2026-04-29"));
        assert!(md.contains("## Day 1"));
        assert!(md.contains("### 09:00 那覇空港"));
        assert!(md.contains("- 場所: 那覇空港"));
        assert!(md.contains("- 所要時間: 60分"));
        assert!(md.contains("- 移動時間: 30分"));
        assert!(md.contains("- メモ: レンタカー受け取り"));
    }
}
