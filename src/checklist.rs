use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::models::ChecklistItem;

/// 新しいチェックリスト項目を追加する
pub(crate) fn add_checklist_item(conn: &Connection, trip_id: i64, title: &str) -> Result<i64> {
    crate::trip::get_trip(conn, trip_id)?;
    let now = crate::db::now_string();
    conn.execute(
        "INSERT INTO checklist_items
         (trip_id, title, is_done, sort_order, created_at, updated_at)
         VALUES (?1, ?2, 0, 0, ?3, ?4)",
        params![trip_id, title, &now, &now],
    )
    .context("チェックリスト項目の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

/// 旅行に紐づくチェックリスト一覧を取得する
pub(crate) fn list_checklist_items(conn: &Connection, trip_id: i64) -> Result<Vec<ChecklistItem>> {
    crate::trip::get_trip(conn, trip_id)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, trip_id, title, is_done, sort_order, created_at, updated_at
             FROM checklist_items
             WHERE trip_id = ?1
             ORDER BY is_done ASC, sort_order ASC, id ASC",
        )
        .context("チェックリスト一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id], row_to_checklist_item)
        .context("チェックリスト一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("チェックリスト一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// ID を指定して1件のチェックリスト項目を取得する
pub(crate) fn get_checklist_item(conn: &Connection, id: i64) -> Result<ChecklistItem> {
    conn.query_row(
        "SELECT id, trip_id, title, is_done, sort_order, created_at, updated_at
         FROM checklist_items
         WHERE id = ?1",
        params![id],
        row_to_checklist_item,
    )
    .with_context(|| format!("ID {id} のチェックリスト項目が見つかりません"))
}

/// チェックリスト項目を更新する（指定されたフィールドのみ上書き）
pub(crate) fn update_checklist_item(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    sort_order: Option<i64>,
) -> Result<()> {
    if title.is_none() && sort_order.is_none() {
        anyhow::bail!("更新する項目を1つ以上指定してください (--title, --sort-order)");
    }

    let mut item = get_checklist_item(conn, id)?;
    if let Some(t) = title {
        item.title = t.to_string();
    }
    if let Some(order) = sort_order {
        item.sort_order = order;
    }

    let now = crate::db::now_string();
    conn.execute(
        "UPDATE checklist_items
         SET title = ?1, sort_order = ?2, updated_at = ?3
         WHERE id = ?4",
        params![item.title, item.sort_order, &now, id],
    )
    .context("チェックリスト項目の更新に失敗しました")?;
    Ok(())
}

/// チェックリスト項目の完了状態を変更する
pub(crate) fn set_checklist_done(conn: &Connection, id: i64, is_done: bool) -> Result<()> {
    get_checklist_item(conn, id)?;
    let now = crate::db::now_string();
    let done_value = i64::from(is_done);
    conn.execute(
        "UPDATE checklist_items SET is_done = ?1, updated_at = ?2 WHERE id = ?3",
        params![done_value, &now, id],
    )
    .context("チェックリスト項目の状態変更に失敗しました")?;
    Ok(())
}

/// チェックリスト項目を削除する
pub(crate) fn delete_checklist_item(conn: &Connection, id: i64) -> Result<()> {
    get_checklist_item(conn, id)?;
    conn.execute("DELETE FROM checklist_items WHERE id = ?1", params![id])
        .context("チェックリスト項目の削除に失敗しました")?;
    Ok(())
}

/// rusqlite の行データを ChecklistItem 構造体に変換する
fn row_to_checklist_item(row: &rusqlite::Row) -> rusqlite::Result<ChecklistItem> {
    let is_done: i64 = row.get(3)?;
    Ok(ChecklistItem {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        title: row.get(2)?,
        is_done: is_done != 0,
        sort_order: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
/// チェック状態を表示用に整形する
fn fmt_checklist_mark(is_done: bool) -> char {
    if is_done {
        'x'
    } else {
        ' '
    }
}

/// チェックリスト一覧を表示する
pub(crate) fn print_checklist_list(items: &[ChecklistItem]) {
    if items.is_empty() {
        println!("チェックリストはまだ登録されていません。");
        return;
    }

    for item in items {
        let mark = fmt_checklist_mark(item.is_done);
        println!("[{mark}] {}. {}", item.id, item.title);
    }
}

/// チェックリスト項目の詳細を表示する
pub(crate) fn print_checklist_detail(item: &ChecklistItem) {
    let mark = fmt_checklist_mark(item.is_done);
    println!("ID        : {}", item.id);
    println!("旅行 ID   : {}", item.trip_id);
    println!("状態      : [{mark}]");
    println!("並び順    : {}", item.sort_order);
    println!("タイトル  : {}", item.title);
    println!("作成日時  : {}", item.created_at);
    println!("更新日時  : {}", item.updated_at);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_db_at;
    use crate::trip::add_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn test_add_checklist_item() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        assert_eq!(id, 1);
        let item = get_checklist_item(&conn, id).unwrap();
        assert_eq!(item.trip_id, trip_id);
        assert_eq!(item.title, "パスポート");
        assert!(!item.is_done);
        assert_eq!(item.sort_order, 0);
    }

    #[test]
    fn test_check_and_uncheck_checklist_item() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        set_checklist_done(&conn, id, true).unwrap();
        let checked = get_checklist_item(&conn, id).unwrap();
        assert!(checked.is_done);

        set_checklist_done(&conn, id, false).unwrap();
        let unchecked = get_checklist_item(&conn, id).unwrap();
        assert!(!unchecked.is_done);
    }

    #[test]
    fn test_delete_checklist_item() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        delete_checklist_item(&conn, id).unwrap();
        assert!(get_checklist_item(&conn, id).is_err());
        assert!(list_checklist_items(&conn, trip_id).unwrap().is_empty());
    }

    #[test]
    fn test_list_checklist_items_by_trip() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let other_trip_id = add_trip(&conn, "京都旅行", None, None).unwrap();

        add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        add_checklist_item(&conn, trip_id, "充電器").unwrap();
        add_checklist_item(&conn, other_trip_id, "雨具").unwrap();

        let items = list_checklist_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "パスポート");
        assert_eq!(items[1].title, "充電器");
    }

    #[test]
    fn test_list_checklist_items_sorted() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        let passport_id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        let etc_id = add_checklist_item(&conn, trip_id, "ETCカード").unwrap();
        let charger_id = add_checklist_item(&conn, trip_id, "充電器").unwrap();

        update_checklist_item(&conn, passport_id, None, Some(1)).unwrap();
        update_checklist_item(&conn, etc_id, None, Some(3)).unwrap();
        update_checklist_item(&conn, charger_id, None, Some(2)).unwrap();
        set_checklist_done(&conn, charger_id, true).unwrap();

        let items = list_checklist_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "パスポート");
        assert_eq!(items[1].title, "ETCカード");
        assert_eq!(items[2].title, "充電器");
        assert!(!items[0].is_done);
        assert!(!items[1].is_done);
        assert!(items[2].is_done);
    }

    #[test]
    fn test_update_checklist_item() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id = add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        update_checklist_item(&conn, id, Some("旅券"), Some(5)).unwrap();
        let item = get_checklist_item(&conn, id).unwrap();
        assert_eq!(item.title, "旅券");
        assert_eq!(item.sort_order, 5);
    }
}
