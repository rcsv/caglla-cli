use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::ChecklistItem;

/// Read-only `checklist list` use case result (CLI / future GUI).
pub struct ChecklistListServiceResult {
    pub items: Vec<ChecklistItem>,
}

/// Lists checklist items for a trip without terminal I/O.
pub fn list_checklist(conn: &Connection, trip_id: i64) -> Result<ChecklistListServiceResult> {
    let items = crate::checklist::list_checklist_items(conn, trip_id)?;
    Ok(ChecklistListServiceResult { items })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn service_returns_items_for_existing_trip() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Checklist Trip", "2026-06-01", "2026-06-02", None)
                .unwrap();
        crate::checklist::add_checklist_item(&conn, trip_id, "パスポート").unwrap();

        let result = list_checklist(&conn, trip_id).unwrap();
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].title, "パスポート");
        assert_eq!(result.items[0].trip_id, trip_id);
    }

    #[test]
    fn service_preserves_ordering_and_checked_state() {
        let conn = test_db();
        let trip_id = crate::trip::add_test_trip(&conn, "Sorted Trip").unwrap();

        let passport_id =
            crate::checklist::add_checklist_item(&conn, trip_id, "パスポート").unwrap();
        let etc_id = crate::checklist::add_checklist_item(&conn, trip_id, "ETCカード").unwrap();
        let charger_id = crate::checklist::add_checklist_item(&conn, trip_id, "充電器").unwrap();

        crate::checklist::update_checklist_item(&conn, passport_id, None, Some(1)).unwrap();
        crate::checklist::update_checklist_item(&conn, etc_id, None, Some(3)).unwrap();
        crate::checklist::update_checklist_item(&conn, charger_id, None, Some(2)).unwrap();
        crate::checklist::set_checklist_done(&conn, charger_id, true).unwrap();

        let result = list_checklist(&conn, trip_id).unwrap();
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].title, "パスポート");
        assert_eq!(result.items[1].title, "ETCカード");
        assert_eq!(result.items[2].title, "充電器");
        assert!(!result.items[0].is_done);
        assert!(!result.items[1].is_done);
        assert!(result.items[2].is_done);
    }

    #[test]
    fn service_returns_empty_list_for_trip_without_items() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Empty Trip", "2026-06-01", "2026-06-02", None).unwrap();

        let result = list_checklist(&conn, trip_id).unwrap();
        assert!(result.items.is_empty());
    }

    #[test]
    fn service_preserves_trip_not_found_error_message() {
        let conn = test_db();
        let err = list_checklist(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }
}
