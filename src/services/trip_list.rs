use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::Trip;

/// Read-only `trip list` use case result (CLI / future GUI).
pub struct TripListServiceResult {
    pub trips: Vec<Trip>,
}

/// Lists trips without terminal I/O.
pub fn list_trips(conn: &Connection) -> Result<TripListServiceResult> {
    let trips = crate::trip::list_trips(conn)?;
    Ok(TripListServiceResult { trips })
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
    fn service_returns_empty_list_for_empty_db() {
        let conn = test_db();
        let result = list_trips(&conn).unwrap();
        assert!(result.trips.is_empty());
    }

    #[test]
    fn service_preserves_ordering_by_id() {
        let conn = test_db();
        let id1 = crate::trip::add_trip(&conn, "Trip A", "2026-06-01", "2026-06-02", None).unwrap();
        let id2 = crate::trip::add_trip(&conn, "Trip B", "2026-06-03", "2026-06-04", None).unwrap();

        let result = list_trips(&conn).unwrap();
        assert_eq!(result.trips.len(), 2);
        assert_eq!(result.trips[0].id, id1);
        assert_eq!(result.trips[1].id, id2);
    }
}
