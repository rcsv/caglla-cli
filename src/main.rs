use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection};

/// SQLite データベースのファイル名
const DB_FILE: &str = "caglla.db";

/// trips テーブルの1行分のデータ
struct Trip {
    id: i64,
    name: String,
    start_date: Option<String>,
    end_date: Option<String>,
    created_at: String,
    updated_at: String,
}

// ---------------------------------------------------------------------------
// CLI 定義（clap derive）
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "caglla", about = "Caglla.Travel CLI - 旅行管理ツール")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 旅行 (Trip) の管理
    Trip {
        #[command(subcommand)]
        action: TripAction,
    },
}

#[derive(Subcommand)]
enum TripAction {
    /// 新しい旅行を追加
    Add {
        /// 旅行名（必須）
        name: String,
        /// 開始日 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,
        /// 終了日 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },
    /// 旅行一覧を表示
    List,
    /// 旅行の詳細を表示
    Show {
        /// 旅行 ID
        id: i64,
    },
    /// 旅行を更新
    Update {
        /// 旅行 ID
        id: i64,
        /// 新しい旅行名
        #[arg(long)]
        name: Option<String>,
        /// 新しい開始日 (YYYY-MM-DD)
        #[arg(long)]
        start: Option<String>,
        /// 新しい終了日 (YYYY-MM-DD)
        #[arg(long)]
        end: Option<String>,
    },
    /// 旅行を削除
    Delete {
        /// 旅行 ID
        id: i64,
    },
}

// ---------------------------------------------------------------------------
// データベース操作
// ---------------------------------------------------------------------------

/// 指定パスの DB に接続し、trips テーブルがなければ作成する
fn open_db_at(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("データベース '{path}' を開けませんでした"))?;
    init_db(&conn)?;
    Ok(conn)
}

/// 本番 DB (caglla.db) に接続する
fn open_db() -> Result<Connection> {
    open_db_at(DB_FILE)
}

/// trips テーブルを作成する（既に存在する場合は何もしない）
fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS trips (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            start_date  TEXT,
            end_date    TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        )",
        [],
    )
    .context("trips テーブルの作成に失敗しました")?;
    Ok(())
}

/// 現在時刻を文字列で返す（created_at / updated_at 用）
fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 新しい旅行を追加する
fn add_trip(conn: &Connection, name: &str, start: Option<&str>, end: Option<&str>) -> Result<i64> {
    let now = now_string();
    conn.execute(
        "INSERT INTO trips (name, start_date, end_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![name, start, end, &now, &now],
    )
    .context("旅行の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

/// すべての旅行を取得する
fn list_trips(conn: &Connection) -> Result<Vec<Trip>> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, start_date, end_date, created_at, updated_at
             FROM trips
             ORDER BY id",
        )
        .context("一覧取得の準備に失敗しました")?;

    let trips = stmt
        .query_map([], row_to_trip)
        .context("一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("一覧の読み込みに失敗しました")?;

    Ok(trips)
}

/// ID を指定して1件の旅行を取得する
fn get_trip(conn: &Connection, id: i64) -> Result<Trip> {
    conn.query_row(
        "SELECT id, name, start_date, end_date, created_at, updated_at
         FROM trips
         WHERE id = ?1",
        params![id],
        row_to_trip,
    )
    .with_context(|| format!("ID {id} の旅行が見つかりません"))
}

/// 旅行を更新する（指定されたフィールドのみ上書き）
fn update_trip(
    conn: &Connection,
    id: i64,
    name: Option<&str>,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<()> {
    if name.is_none() && start.is_none() && end.is_none() {
        anyhow::bail!("更新する項目を1つ以上指定してください (--name, --start, --end)");
    }

    // 既存データを読み込み、指定された項目だけ上書きする
    let mut trip = get_trip(conn, id)?;
    if let Some(n) = name {
        trip.name = n.to_string();
    }
    if let Some(s) = start {
        trip.start_date = Some(s.to_string());
    }
    if let Some(e) = end {
        trip.end_date = Some(e.to_string());
    }

    let now = now_string();
    conn.execute(
        "UPDATE trips
         SET name = ?1, start_date = ?2, end_date = ?3, updated_at = ?4
         WHERE id = ?5",
        params![trip.name, trip.start_date, trip.end_date, &now, id],
    )
    .context("旅行の更新に失敗しました")?;
    Ok(())
}

/// 旅行を削除する
fn delete_trip(conn: &Connection, id: i64) -> Result<()> {
    // 存在確認（見つからなければエラー）
    get_trip(conn, id)?;
    conn.execute("DELETE FROM trips WHERE id = ?1", params![id])
        .context("旅行の削除に失敗しました")?;
    Ok(())
}

/// rusqlite の行データを Trip 構造体に変換する
fn row_to_trip(row: &rusqlite::Row) -> rusqlite::Result<Trip> {
    Ok(Trip {
        id: row.get(0)?,
        name: row.get(1)?,
        start_date: row.get(2)?,
        end_date: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

// ---------------------------------------------------------------------------
// 表示用ヘルパー
// ---------------------------------------------------------------------------

/// 日付を表示用に整形する（未設定なら "-"）
fn fmt_date(date: &Option<String>) -> &str {
    date.as_deref().unwrap_or("-")
}

/// 旅行一覧を表形式で表示する
fn print_trip_list(trips: &[Trip]) {
    if trips.is_empty() {
        println!("旅行はまだ登録されていません。");
        return;
    }

    println!(
        "{:<6} {:<20} {:<12} {:<12}",
        "ID", "名前", "開始日", "終了日"
    );
    println!("{}", "-".repeat(52));
    for trip in trips {
        println!(
            "{:<6} {:<20} {:<12} {:<12}",
            trip.id,
            trip.name,
            fmt_date(&trip.start_date),
            fmt_date(&trip.end_date),
        );
    }
    println!();
    println!("合計: {} 件", trips.len());
}

/// 旅行の詳細を表示する
fn print_trip_detail(trip: &Trip) {
    println!("ID        : {}", trip.id);
    println!("名前      : {}", trip.name);
    println!("開始日    : {}", fmt_date(&trip.start_date));
    println!("終了日    : {}", fmt_date(&trip.end_date));
    println!("作成日時  : {}", trip.created_at);
    println!("更新日時  : {}", trip.updated_at);
}

// ---------------------------------------------------------------------------
// エントリポイント
// ---------------------------------------------------------------------------

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = open_db()?;

    match cli.command {
        Command::Trip { action } => match action {
            TripAction::Add { name, start, end } => {
                let id = add_trip(&conn, &name, start.as_deref(), end.as_deref())?;
                println!("旅行を追加しました (ID: {id})");
                println!("  名前   : {name}");
                println!("  開始日 : {}", fmt_date(&start));
                println!("  終了日 : {}", fmt_date(&end));
            }
            TripAction::List => {
                let trips = list_trips(&conn)?;
                print_trip_list(&trips);
            }
            TripAction::Show { id } => {
                let trip = get_trip(&conn, id)?;
                print_trip_detail(&trip);
            }
            TripAction::Update {
                id,
                name,
                start,
                end,
            } => {
                update_trip(&conn, id, name.as_deref(), start.as_deref(), end.as_deref())?;
                println!("旅行を更新しました (ID: {id})");
                let trip = get_trip(&conn, id)?;
                print_trip_detail(&trip);
            }
            TripAction::Delete { id } => {
                let trip = get_trip(&conn, id)?;
                delete_trip(&conn, id)?;
                println!("旅行を削除しました (ID: {id})");
                println!("  名前: {}", trip.name);
            }
        },
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// ユニットテスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用のインメモリ DB を作成する（本番 caglla.db は使わない）
    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
    }

    /// add → list → get → update → delete の一連の CRUD を確認する
    #[test]
    fn test_crud_flow() {
        let conn = test_db();

        // add
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        assert_eq!(id, 1);

        // list
        let trips = list_trips(&conn).unwrap();
        assert_eq!(trips.len(), 1);
        assert_eq!(trips[0].name, "沖縄旅行");

        // get
        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "沖縄旅行");

        // update
        update_trip(&conn, id, Some("沖縄・瀬底旅行"), None, None).unwrap();
        let updated = get_trip(&conn, id).unwrap();
        assert_eq!(updated.name, "沖縄・瀬底旅行");

        // delete
        delete_trip(&conn, id).unwrap();
        assert!(list_trips(&conn).unwrap().is_empty());
        assert!(get_trip(&conn, id).is_err());
    }

    #[test]
    fn test_init_db_creates_trips_table() {
        let conn = Connection::open(":memory:").unwrap();
        init_db(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'trips'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_add_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", Some("2025-06-01"), Some("2025-06-05")).unwrap();
        assert_eq!(id, 1);

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "沖縄旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-06-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-06-05"));
        assert!(!trip.created_at.is_empty());
        assert!(!trip.updated_at.is_empty());
    }

    #[test]
    fn test_add_trip_without_dates() {
        let conn = test_db();
        let id = add_trip(&conn, "大阪旅行", None, None).unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "大阪旅行");
        assert!(trip.start_date.is_none());
        assert!(trip.end_date.is_none());
    }

    #[test]
    fn test_list_trips() {
        let conn = test_db();
        add_trip(&conn, "沖縄旅行", None, None).unwrap();
        add_trip(&conn, "京都旅行", Some("2025-07-01"), None).unwrap();

        let trips = list_trips(&conn).unwrap();
        assert_eq!(trips.len(), 2);
        assert_eq!(trips[0].name, "沖縄旅行");
        assert_eq!(trips[1].name, "京都旅行");
    }

    #[test]
    fn test_list_trips_empty() {
        let conn = test_db();
        let trips = list_trips(&conn).unwrap();
        assert!(trips.is_empty());
    }

    #[test]
    fn test_get_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "北海道旅行", Some("2025-08-01"), Some("2025-08-10")).unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.id, id);
        assert_eq!(trip.name, "北海道旅行");
    }

    #[test]
    fn test_get_trip_not_found() {
        let conn = test_db();
        let result = get_trip(&conn, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_trip_name() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        update_trip(&conn, id, Some("沖縄・瀬底旅行"), None, None).unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "沖縄・瀬底旅行");
        assert!(trip.updated_at >= trip.created_at);
    }

    #[test]
    fn test_update_trip_dates() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        update_trip(&conn, id, None, Some("2025-06-01"), Some("2025-06-07")).unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.start_date.as_deref(), Some("2025-06-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-06-07"));
    }

    #[test]
    fn test_update_trip_no_fields_fails() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        let result = update_trip(&conn, id, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        delete_trip(&conn, id).unwrap();

        let trips = list_trips(&conn).unwrap();
        assert!(trips.is_empty());
        assert!(get_trip(&conn, id).is_err());
    }

    #[test]
    fn test_delete_trip_not_found() {
        let conn = test_db();
        let result = delete_trip(&conn, 999);
        assert!(result.is_err());
    }
}
