use anyhow::Result;
use rusqlite::Connection;

use crate::cli::{Command, DbAction};
use crate::storage::db;

pub fn run_before_open_db(command: &Command) -> Result<bool> {
    match command {
        Command::Db {
            action: DbAction::Path,
        } => {
            db::run_db_path()?;
            Ok(true)
        }
        Command::Db {
            action: DbAction::Status { json },
        } => {
            db::run_db_status(*json)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn run_after_open(conn: &Connection, action: DbAction) -> Result<()> {
    match action {
        DbAction::Path | DbAction::Status { .. } => unreachable!(),
        DbAction::Reset => {
            db::reset_db(conn)?;
            println!("【開発用】データベースを初期化しました");
            println!("  - checklist_items / itinerary_items / trips の全データを削除");
            println!("  - AUTOINCREMENT の採番をリセット");
            Ok(())
        }
    }
}
