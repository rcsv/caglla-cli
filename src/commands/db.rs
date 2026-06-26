use anyhow::Result;
use rusqlite::Connection;

use crate::cli::{Command, DbAction};
use crate::config::ResolvedDbPath;
use crate::storage::db;

pub fn run_before_open_db(command: &Command, resolved: &ResolvedDbPath) -> Result<bool> {
    match command {
        Command::Db {
            action: DbAction::Path,
        } => {
            db::run_db_path(resolved)?;
            Ok(true)
        }
        Command::Db {
            action: DbAction::Status { json },
        } => {
            db::run_db_status(resolved, *json)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub fn run_after_open(
    conn: &Connection,
    action: DbAction,
    resolved: &ResolvedDbPath,
) -> Result<()> {
    match action {
        DbAction::Path | DbAction::Status { .. } => unreachable!(),
        DbAction::Reset => {
            db::reset_db(conn)?;
            println!("【開発用】データベースを初期化しました");
            println!("  DB: {}", resolved.path.display());
            Ok(())
        }
    }
}
