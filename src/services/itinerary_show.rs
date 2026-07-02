use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::ItineraryItem;

/// Read-only `itinerary show` use case result (CLI / future GUI).
pub struct ItineraryShowServiceResult {
    pub item: ItineraryItem,
}

/// Loads an itinerary item without terminal I/O.
pub fn show_itinerary(conn: &Connection, id: i64) -> Result<ItineraryShowServiceResult> {
    let item = crate::itinerary::get_itinerary_item(conn, id)?;
    Ok(ItineraryShowServiceResult { item })
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
    fn service_returns_existing_itinerary() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Show Trip", "2026-06-01", "2026-06-02", None).unwrap();
        let id = crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            Some(1),
            Some(90),
            None,
            None,
            None,
        )
        .unwrap();

        let result = show_itinerary(&conn, id).unwrap();
        assert_eq!(result.item.id, id);
        assert_eq!(result.item.title, "首里城");
        assert_eq!(result.item.trip_id, trip_id);
    }

    #[test]
    fn service_preserves_not_found_error_message() {
        let conn = test_db();
        let err = show_itinerary(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Itinerary not found: 9999");
    }
}
