use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Trip;

/// Read-only `trip show` use case result (CLI / future GUI).
pub struct TripShowServiceResult {
    pub trip: Trip,
}

/// Loads a trip without terminal I/O.
pub fn show_trip(conn: &Connection, trip_id: i64) -> Result<TripShowServiceResult> {
    let trip = crate::trip::get_trip(conn, trip_id)?;
    Ok(TripShowServiceResult { trip })
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
    fn service_returns_existing_trip() {
        let conn = test_db();
        let trip_id =
            crate::trip::add_trip(&conn, "Show Trip", "2026-06-01", "2026-06-02", None).unwrap();

        let result = show_trip(&conn, trip_id).unwrap();
        assert_eq!(result.trip.id, trip_id);
        assert_eq!(result.trip.name, "Show Trip");
    }

    #[test]
    fn service_preserves_not_found_error_message() {
        let conn = test_db();
        let err = show_trip(&conn, 9999).err().expect("expected error");
        assert_eq!(err.to_string(), "Trip not found: 9999");
    }
}
