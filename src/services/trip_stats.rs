use anyhow::Result;
use rusqlite::Connection;

use crate::analysis::statistics::{compute_trip_stats, TripStats};

/// Read-only `trip stats` use case result (CLI / future GUI).
pub struct TripStatsServiceResult {
    pub stats: TripStats,
}

/// Aggregates trip statistics without terminal I/O.
pub fn get_trip_stats(conn: &Connection, trip_id: i64) -> Result<TripStatsServiceResult> {
    let stats = compute_trip_stats(conn, trip_id)?;
    Ok(TripStatsServiceResult { stats })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use crate::trip::add_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    #[test]
    fn service_returns_same_days_as_compute_trip_stats() {
        let conn = test_db();
        let trip_id = add_trip(
            &conn,
            "Service Stats Trip",
            "2026-04-26",
            "2026-04-29",
            None,
        )
        .unwrap();
        crate::itinerary::add_itinerary_item(
            &conn, trip_id, 1, "Aquarium", None, None, None, None, None, None, None,
        )
        .unwrap();

        let service = get_trip_stats(&conn, trip_id).unwrap();
        let direct = compute_trip_stats(&conn, trip_id).unwrap();

        assert_eq!(service.stats.days, direct.days);
        assert_eq!(service.stats.days, 4);
        assert_eq!(service.stats.itinerary_count, 1);
    }
}
