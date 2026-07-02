use anyhow::Result;
use rusqlite::Connection;

use crate::domain::models::{Note, NoteOwnerType};

/// Read-only `note list` use case result (CLI / future GUI).
pub struct NoteListServiceResult {
    pub owner_type: NoteOwnerType,
    pub owner_id: i64,
    pub notes: Vec<Note>,
}

/// Resolves the note owner and lists its notes without terminal I/O.
pub fn list_notes(
    conn: &Connection,
    trip: Option<i64>,
    day: Option<i64>,
    itinerary: Option<i64>,
) -> Result<NoteListServiceResult> {
    let owner = crate::note::resolve_note_owner_for_list(conn, trip, day, itinerary)?;
    let owner_type = owner.owner_type();
    let owner_id = owner.owner_id();
    let notes = crate::note::list_notes_for_owner(conn, owner_type, owner_id)?;
    Ok(NoteListServiceResult {
        owner_type,
        owner_id,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::open_db_at;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    fn seed_trip_with_day_and_itinerary(conn: &Connection) -> (i64, i64, i64) {
        let trip_id =
            crate::trip::add_trip(conn, "Note Trip", "2026-06-01", "2026-06-02", None).unwrap();
        let day = crate::day::find_day_id_by_trip_and_day_number(conn, trip_id, 1).unwrap();
        let itinerary_id = crate::itinerary::add_itinerary_item(
            conn,
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
        (trip_id, day, itinerary_id)
    }

    #[test]
    fn service_returns_notes_for_trip_target() {
        let conn = test_db();
        let (trip_id, _, _) = seed_trip_with_day_and_itinerary(&conn);
        crate::note::add_note(
            &conn,
            crate::note::resolve_note_owner_for_list(&conn, Some(trip_id), None, None).unwrap(),
            Some("t"),
            "trip note",
        )
        .unwrap();

        let result = list_notes(&conn, Some(trip_id), None, None).unwrap();
        assert_eq!(result.owner_type, NoteOwnerType::Trip);
        assert_eq!(result.owner_id, trip_id);
        assert_eq!(result.notes.len(), 1);
        assert_eq!(result.notes[0].body, "trip note");
    }

    #[test]
    fn service_returns_notes_for_day_target() {
        let conn = test_db();
        let (trip_id, day_id, _) = seed_trip_with_day_and_itinerary(&conn);
        crate::note::add_note(
            &conn,
            crate::note::resolve_note_owner_for_list(&conn, Some(trip_id), Some(1), None).unwrap(),
            None,
            "day note",
        )
        .unwrap();

        let result = list_notes(&conn, Some(trip_id), Some(1), None).unwrap();
        assert_eq!(result.owner_type, NoteOwnerType::Day);
        assert_eq!(result.owner_id, day_id);
        assert_eq!(result.notes.len(), 1);
        assert_eq!(result.notes[0].body, "day note");
    }

    #[test]
    fn service_returns_notes_for_itinerary_target() {
        let conn = test_db();
        let (_, _, itinerary_id) = seed_trip_with_day_and_itinerary(&conn);
        crate::note::add_note(
            &conn,
            crate::note::resolve_note_owner_for_list(&conn, None, None, Some(itinerary_id))
                .unwrap(),
            None,
            "itinerary note",
        )
        .unwrap();

        let result = list_notes(&conn, None, None, Some(itinerary_id)).unwrap();
        assert_eq!(result.owner_type, NoteOwnerType::Itinerary);
        assert_eq!(result.owner_id, itinerary_id);
        assert_eq!(result.notes.len(), 1);
        assert_eq!(result.notes[0].body, "itinerary note");
    }

    #[test]
    fn service_preserves_sort_order_then_id_ordering() {
        let conn = test_db();
        let (trip_id, _, _) = seed_trip_with_day_and_itinerary(&conn);
        let owner =
            crate::note::resolve_note_owner_for_list(&conn, Some(trip_id), None, None).unwrap();
        let first = crate::note::add_note(&conn, owner, None, "first").unwrap();
        let second = crate::note::add_note(&conn, owner, None, "second").unwrap();

        let result = list_notes(&conn, Some(trip_id), None, None).unwrap();
        assert_eq!(result.notes.len(), 2);
        // Same sort_order (0) -> ordered by id ascending.
        assert_eq!(result.notes[0].id, first);
        assert_eq!(result.notes[1].id, second);
    }

    #[test]
    fn service_returns_empty_for_target_without_notes() {
        let conn = test_db();
        let (trip_id, _, _) = seed_trip_with_day_and_itinerary(&conn);
        let result = list_notes(&conn, Some(trip_id), None, None).unwrap();
        assert!(result.notes.is_empty());
    }

    #[test]
    fn service_preserves_owner_resolution_error() {
        let conn = test_db();
        let err = list_notes(&conn, None, None, None)
            .err()
            .expect("expected error");
        assert!(err.to_string().contains("owner を指定してください"));
    }
}
