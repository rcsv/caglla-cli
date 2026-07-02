use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::ItineraryItem;

/// Read-only `itinerary list` use case result (CLI / future GUI).
pub struct ItineraryListServiceResult {
    pub items: Vec<ItineraryItem>,
}

/// Lists itinerary items for a trip without terminal I/O.
pub fn list_itineraries(conn: &Connection, trip_id: i64) -> Result<ItineraryListServiceResult> {
    let items = crate::itinerary::list_itinerary_items(conn, trip_id)?;
    Ok(ItineraryListServiceResult { items })
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
    fn service_preserves_trip_not_found_error_message() {
        let conn = test_db();
        let err = list_itineraries(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }

    #[test]
    fn service_preserves_sequence_first_ordering() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Ordering Trip", "2026-06-01", "2026-06-02", None)
                .unwrap();

        // Day 1: order 2000 then 1000
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "Second",
            None,
            None,
            Some(2000),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            1,
            "First",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        // Day 2: order 1000
        crate::itinerary::add_itinerary_item(
            &conn,
            trip_id,
            2,
            "Day2",
            None,
            None,
            Some(1000),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let result = list_itineraries(&conn, trip_id).unwrap();
        assert_eq!(result.items.len(), 3);
        assert_eq!(result.items[0].day, 1);
        assert_eq!(result.items[0].title, "First");
        assert_eq!(result.items[1].day, 1);
        assert_eq!(result.items[1].title, "Second");
        assert_eq!(result.items[2].day, 2);
        assert_eq!(result.items[2].title, "Day2");
    }
}
