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

/// itinerary_items テーブルの1行分のデータ
struct ItineraryItem {
    id: i64,
    trip_id: i64,
    day: i64,
    title: String,
    note: Option<String>,
    start_time: Option<String>,
    sort_order: i64,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
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
    /// 日程 (Itinerary) の管理
    Itinerary {
        #[command(subcommand)]
        action: ItineraryAction,
    },
    /// データベース操作（開発用）
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
}

#[derive(Subcommand)]
enum DbAction {
    /// 【開発用】全データを削除して DB を初期状態に戻す（本番運用では使わない）
    Reset,
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

#[derive(Subcommand)]
enum ItineraryAction {
    /// 日程を追加
    Add {
        /// 旅行 ID
        trip_id: i64,
        /// 何日目か
        #[arg(long)]
        day: i64,
        /// タイトル（必須）
        title: String,
        /// メモ
        #[arg(long)]
        note: Option<String>,
        /// 開始時刻 (HH:MM)
        #[arg(long)]
        time: Option<String>,
        /// 並び順（小さいほど先）
        #[arg(long)]
        order: Option<i64>,
        /// 所要時間（分）
        #[arg(long)]
        duration: Option<i64>,
        /// 次の予定までの移動時間（分）
        #[arg(long)]
        travel: Option<i64>,
    },
    /// 旅行の日程一覧を表示
    List {
        /// 旅行 ID
        trip_id: i64,
    },
    /// 旅行のタイムラインを表示
    Timeline {
        /// 旅行 ID
        trip_id: i64,
    },
    /// 日程の詳細を表示
    Show {
        /// 日程 ID
        id: i64,
    },
    /// 日程を更新
    Update {
        /// 日程 ID
        id: i64,
        /// 何日目か
        #[arg(long)]
        day: Option<i64>,
        /// タイトル
        #[arg(long)]
        title: Option<String>,
        /// メモ
        #[arg(long)]
        note: Option<String>,
        /// 開始時刻 (HH:MM)
        #[arg(long)]
        time: Option<String>,
        /// 並び順（小さいほど先）
        #[arg(long)]
        order: Option<i64>,
        /// 所要時間（分）
        #[arg(long)]
        duration: Option<i64>,
        /// 次の予定までの移動時間（分）
        #[arg(long)]
        travel: Option<i64>,
    },
    /// 日程を削除
    Delete {
        /// 日程 ID
        id: i64,
    },
}

// ---------------------------------------------------------------------------
// データベース操作
// ---------------------------------------------------------------------------

/// 指定パスの DB に接続し、テーブルがなければ作成する
fn open_db_at(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("データベース '{path}' を開けませんでした"))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .context("外部キー制約の有効化に失敗しました")?;
    init_db(&conn)?;
    Ok(conn)
}

/// 本番 DB (caglla.db) に接続する
fn open_db() -> Result<Connection> {
    open_db_at(DB_FILE)
}

/// テーブルを作成する（既に存在する場合は何もしない）
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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS itinerary_items (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            trip_id     INTEGER NOT NULL,
            day         INTEGER NOT NULL,
            title       TEXT NOT NULL,
            note        TEXT,
            start_time  TEXT,
            sort_order  INTEGER NOT NULL DEFAULT 0,
            duration_minutes INTEGER,
            travel_minutes INTEGER,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL,
            FOREIGN KEY(trip_id) REFERENCES trips(id) ON DELETE CASCADE
        )",
        [],
    )
    .context("itinerary_items テーブルの作成に失敗しました")?;
    migrate_itinerary_items(conn)?;
    Ok(())
}

/// 列がなければ ALTER TABLE で追加する（既にある場合は何もしない）
fn add_column_if_not_exists(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<()> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .with_context(|| format!("{table} テーブル情報の取得に失敗しました"))?;

    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .with_context(|| format!("{table} テーブル情報の読み込みに失敗しました"))?
        .any(|name| name.map(|n| n == column).unwrap_or(false));

    if !exists {
        let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        conn.execute(&sql, [])
            .with_context(|| format!("{table}.{column} 列の追加に失敗しました"))?;
    }
    Ok(())
}

/// 既存 DB 向け: itinerary_items に不足している列を追加する
fn migrate_itinerary_items(conn: &Connection) -> Result<()> {
    add_column_if_not_exists(conn, "itinerary_items", "start_time", "TEXT")?;
    add_column_if_not_exists(
        conn,
        "itinerary_items",
        "sort_order",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_not_exists(conn, "itinerary_items", "duration_minutes", "INTEGER")?;
    add_column_if_not_exists(conn, "itinerary_items", "travel_minutes", "INTEGER")?;
    Ok(())
}

/// 【開発用】全テーブルのデータを削除し、AUTOINCREMENT をリセットする
///
/// - itinerary_items → trips の順で削除する（外部キー参照を考慮）
/// - テーブル定義は残す
/// - 本番運用では使わないこと
fn reset_db(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM itinerary_items", [])
        .context("itinerary_items の全削除に失敗しました")?;
    conn.execute("DELETE FROM trips", [])
        .context("trips の全削除に失敗しました")?;
    conn.execute(
        "DELETE FROM sqlite_sequence WHERE name IN ('itinerary_items', 'trips')",
        [],
    )
    .context("AUTOINCREMENT のリセットに失敗しました")?;
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

/// 新しい日程を追加する
#[allow(clippy::too_many_arguments)]
fn add_itinerary_item(
    conn: &Connection,
    trip_id: i64,
    day: i64,
    title: &str,
    note: Option<&str>,
    start_time: Option<&str>,
    sort_order: Option<i64>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
) -> Result<i64> {
    get_trip(conn, trip_id)?;
    if let Some(t) = start_time {
        parse_time_hhmm(t)?;
    }
    let now = now_string();
    let sort_order = sort_order.unwrap_or(0);
    conn.execute(
        "INSERT INTO itinerary_items
         (trip_id, day, title, note, start_time, sort_order, duration_minutes, travel_minutes,
          created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            trip_id,
            day,
            title,
            note,
            start_time,
            sort_order,
            duration_minutes,
            travel_minutes,
            &now,
            &now
        ],
    )
    .context("日程の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

/// 旅行に紐づく日程一覧を取得する
fn list_itinerary_items(conn: &Connection, trip_id: i64) -> Result<Vec<ItineraryItem>> {
    get_trip(conn, trip_id)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, trip_id, day, title, note, start_time, sort_order,
                    duration_minutes, travel_minutes, created_at, updated_at
             FROM itinerary_items
             WHERE trip_id = ?1
             ORDER BY day, start_time IS NULL, start_time, sort_order, id",
        )
        .context("日程一覧取得の準備に失敗しました")?;

    let items = stmt
        .query_map(params![trip_id], row_to_itinerary_item)
        .context("日程一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("日程一覧の読み込みに失敗しました")?;

    Ok(items)
}

/// ID を指定して1件の日程を取得する
fn get_itinerary_item(conn: &Connection, id: i64) -> Result<ItineraryItem> {
    conn.query_row(
        "SELECT id, trip_id, day, title, note, start_time, sort_order,
                duration_minutes, travel_minutes, created_at, updated_at
         FROM itinerary_items
         WHERE id = ?1",
        params![id],
        row_to_itinerary_item,
    )
    .with_context(|| format!("ID {id} の日程が見つかりません"))
}

/// 日程を更新する（指定されたフィールドのみ上書き）
#[allow(clippy::too_many_arguments)]
fn update_itinerary_item(
    conn: &Connection,
    id: i64,
    day: Option<i64>,
    title: Option<&str>,
    note: Option<Option<&str>>,
    start_time: Option<Option<&str>>,
    sort_order: Option<i64>,
    duration_minutes: Option<i64>,
    travel_minutes: Option<i64>,
) -> Result<()> {
    if day.is_none()
        && title.is_none()
        && note.is_none()
        && start_time.is_none()
        && sort_order.is_none()
        && duration_minutes.is_none()
        && travel_minutes.is_none()
    {
        anyhow::bail!(
            "更新する項目を1つ以上指定してください \
             (--day, --title, --note, --time, --order, --duration, --travel)"
        );
    }

    let mut item = get_itinerary_item(conn, id)?;
    if let Some(d) = day {
        item.day = d;
    }
    if let Some(t) = title {
        item.title = t.to_string();
    }
    if let Some(n) = note {
        item.note = n.map(str::to_string);
    }
    if let Some(t) = start_time {
        if let Some(time_str) = t {
            parse_time_hhmm(time_str)?;
        }
        item.start_time = t.map(str::to_string);
    }
    if let Some(o) = sort_order {
        item.sort_order = o;
    }
    if let Some(d) = duration_minutes {
        item.duration_minutes = Some(d);
    }
    if let Some(t) = travel_minutes {
        item.travel_minutes = Some(t);
    }

    let now = now_string();
    conn.execute(
        "UPDATE itinerary_items
         SET day = ?1, title = ?2, note = ?3, start_time = ?4, sort_order = ?5,
             duration_minutes = ?6, travel_minutes = ?7, updated_at = ?8
         WHERE id = ?9",
        params![
            item.day,
            item.title,
            item.note,
            item.start_time,
            item.sort_order,
            item.duration_minutes,
            item.travel_minutes,
            &now,
            id
        ],
    )
    .context("日程の更新に失敗しました")?;
    Ok(())
}

/// 日程を削除する
fn delete_itinerary_item(conn: &Connection, id: i64) -> Result<()> {
    get_itinerary_item(conn, id)?;
    conn.execute("DELETE FROM itinerary_items WHERE id = ?1", params![id])
        .context("日程の削除に失敗しました")?;
    Ok(())
}

/// rusqlite の行データを ItineraryItem 構造体に変換する
fn row_to_itinerary_item(row: &rusqlite::Row) -> rusqlite::Result<ItineraryItem> {
    Ok(ItineraryItem {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        day: row.get(2)?,
        title: row.get(3)?,
        note: row.get(4)?,
        start_time: row.get(5)?,
        sort_order: row.get(6)?,
        duration_minutes: row.get(7)?,
        travel_minutes: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

// ---------------------------------------------------------------------------
// 表示用ヘルパー
// ---------------------------------------------------------------------------

/// 日付を表示用に整形する（未設定なら "-"）
fn fmt_date(date: &Option<String>) -> &str {
    date.as_deref().unwrap_or("-")
}

/// テキストを表示用に整形する（未設定なら "-"）
fn fmt_text(text: &Option<String>) -> &str {
    text.as_deref().unwrap_or("-")
}

/// 分数を表示用に整形する（未設定なら "-"）
fn fmt_minutes(minutes: Option<i64>) -> String {
    match minutes {
        Some(m) => format!("{m}分"),
        None => "-".to_string(),
    }
}

/// HH:MM 形式を検証し、(時, 分) を返す
fn parse_time_hhmm(time: &str) -> Result<(i32, i32)> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 || parts[0].len() != 2 || parts[1].len() != 2 {
        anyhow::bail!("時刻は HH:MM 形式で指定してください: {time}");
    }
    let hour: i32 = parts[0]
        .parse()
        .with_context(|| format!("不正な時刻です: {time}"))?;
    let minute: i32 = parts[1]
        .parse()
        .with_context(|| format!("不正な時刻です: {time}"))?;
    if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) {
        anyhow::bail!("不正な時刻です: {time}");
    }
    Ok((hour, minute))
}

/// HH:MM に分数を加算した時刻を返す（日をまたぐ計算はしない）
fn add_minutes_to_time(time: &str, minutes: i64) -> Result<String> {
    let (hour, minute) = parse_time_hhmm(time)?;
    let total = hour * 60 + minute + minutes as i32;
    if total < 0 {
        anyhow::bail!("時刻の計算結果が不正です");
    }
    let new_hour = total / 60;
    let new_minute = total % 60;
    if new_hour >= 24 {
        anyhow::bail!("終了予定時刻が24時を超えました（日跨ぎには未対応です）");
    }
    Ok(format!("{new_hour:02}:{new_minute:02}"))
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

/// 日程一覧を表形式で表示する
fn print_itinerary_list(items: &[ItineraryItem]) {
    if items.is_empty() {
        println!("日程はまだ登録されていません。");
        return;
    }

    println!(
        "{:<6} {:<6} {:<8} {:<16} {:<8} {:<8} {:<12}",
        "ID", "日目", "時刻", "タイトル", "所要", "移動", "メモ"
    );
    println!("{}", "-".repeat(70));
    for item in items {
        println!(
            "{:<6} {:<6} {:<8} {:<16} {:<8} {:<8} {:<12}",
            item.id,
            item.day,
            fmt_text(&item.start_time),
            item.title,
            fmt_minutes(item.duration_minutes),
            fmt_minutes(item.travel_minutes),
            fmt_text(&item.note),
        );
    }
    println!();
    println!("合計: {} 件", items.len());
}

/// 日程の詳細を表示する
fn print_itinerary_detail(item: &ItineraryItem) {
    println!("ID        : {}", item.id);
    println!("旅行 ID   : {}", item.trip_id);
    println!("日目      : {}", item.day);
    println!("時刻      : {}", fmt_text(&item.start_time));
    println!("並び順    : {}", item.sort_order);
    println!("所要時間  : {}", fmt_minutes(item.duration_minutes));
    println!("移動時間  : {}", fmt_minutes(item.travel_minutes));
    println!("タイトル  : {}", item.title);
    println!("メモ      : {}", fmt_text(&item.note));
    println!("作成日時  : {}", item.created_at);
    println!("更新日時  : {}", item.updated_at);
}

/// 旅行のタイムラインを表示する
fn print_itinerary_timeline(items: &[ItineraryItem]) {
    if items.is_empty() {
        println!("日程はまだ登録されていません。");
        return;
    }

    let mut current_day: Option<i64> = None;
    for (index, item) in items.iter().enumerate() {
        if current_day != Some(item.day) {
            if current_day.is_some() {
                println!();
            }
            println!("Day {}", item.day);
            println!();
            current_day = Some(item.day);
        }

        match &item.start_time {
            Some(time) => {
                println!("{time} {}", item.title);
                if let Some(duration) = item.duration_minutes {
                    println!("  所要時間: {duration}分");
                    if let Ok(end_time) = add_minutes_to_time(time, duration) {
                        println!("  終了予定: {end_time}");
                    }
                }
            }
            None => {
                println!("時刻: 未定");
                println!("{}", item.title);
                if let Some(duration) = item.duration_minutes {
                    println!("  所要時間: {duration}分");
                }
            }
        }

        // 次の予定への移動時間を表示（同じ日の次の予定がある場合）
        if let Some(travel) = item.travel_minutes {
            let has_next_same_day = items
                .get(index + 1)
                .is_some_and(|next| next.day == item.day);
            if has_next_same_day {
                println!();
                println!("  ↓ 移動 {travel}分");
                println!();
            }
        } else if items
            .get(index + 1)
            .is_some_and(|next| next.day == item.day)
        {
            println!();
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let conn = open_db()?;

    match cli.command {
        Command::Db { action } => match action {
            DbAction::Reset => {
                reset_db(&conn)?;
                println!("【開発用】データベースを初期化しました");
                println!("  - itinerary_items / trips の全データを削除");
                println!("  - AUTOINCREMENT の採番をリセット");
            }
        },
        Command::Itinerary { action } => match action {
            ItineraryAction::Add {
                trip_id,
                day,
                title,
                note,
                time,
                order,
                duration,
                travel,
            } => {
                let id = add_itinerary_item(
                    &conn,
                    trip_id,
                    day,
                    &title,
                    note.as_deref(),
                    time.as_deref(),
                    order,
                    duration,
                    travel,
                )?;
                println!("日程を追加しました (ID: {id})");
                println!("  旅行 ID : {trip_id}");
                println!("  日目    : {day}");
                println!("  時刻    : {}", fmt_text(&time));
                println!("  並び順  : {}", order.unwrap_or(0));
                println!("  所要時間: {}", fmt_minutes(duration));
                println!("  移動時間: {}", fmt_minutes(travel));
                println!("  タイトル: {title}");
                println!("  メモ    : {}", fmt_text(&note));
            }
            ItineraryAction::List { trip_id } => {
                let items = list_itinerary_items(&conn, trip_id)?;
                println!("旅行 ID {trip_id} の日程:");
                print_itinerary_list(&items);
            }
            ItineraryAction::Timeline { trip_id } => {
                let items = list_itinerary_items(&conn, trip_id)?;
                let trip = get_trip(&conn, trip_id)?;
                println!("{} のタイムライン:", trip.name);
                println!();
                print_itinerary_timeline(&items);
            }
            ItineraryAction::Show { id } => {
                let item = get_itinerary_item(&conn, id)?;
                print_itinerary_detail(&item);
            }
            ItineraryAction::Update {
                id,
                day,
                title,
                note,
                time,
                order,
                duration,
                travel,
            } => {
                let note_update = note.as_ref().map(|n| Some(n.as_str()));
                let time_update = time.as_ref().map(|t| Some(t.as_str()));
                update_itinerary_item(
                    &conn,
                    id,
                    day,
                    title.as_deref(),
                    note_update,
                    time_update,
                    order,
                    duration,
                    travel,
                )?;
                println!("日程を更新しました (ID: {id})");
                let item = get_itinerary_item(&conn, id)?;
                print_itinerary_detail(&item);
            }
            ItineraryAction::Delete { id } => {
                let item = get_itinerary_item(&conn, id)?;
                delete_itinerary_item(&conn, id)?;
                println!("日程を削除しました (ID: {id})");
                println!("  タイトル: {}", item.title);
            }
        },
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
// ユニットテスト（:memory: のみ使用、caglla.db は使わない）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用のインメモリ DB を作成する
    fn test_db() -> Connection {
        open_db_at(":memory:").expect("インメモリ DB の作成に失敗")
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
    }

    #[test]
    fn test_list_trips() {
        let conn = test_db();
        add_trip(&conn, "沖縄旅行", Some("2025-06-01"), Some("2025-06-05")).unwrap();
        add_trip(&conn, "京都旅行", Some("2025-07-01"), Some("2025-07-03")).unwrap();

        let trips = list_trips(&conn).unwrap();
        assert_eq!(trips.len(), 2);
        assert_eq!(trips[0].name, "沖縄旅行");
        assert_eq!(trips[1].name, "京都旅行");
    }

    #[test]
    fn test_get_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "北海道旅行", Some("2025-08-01"), Some("2025-08-10")).unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.id, id);
        assert_eq!(trip.name, "北海道旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-08-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-08-10"));
    }

    #[test]
    fn test_update_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        update_trip(
            &conn,
            id,
            Some("沖縄・瀬底旅行"),
            Some("2025-06-01"),
            Some("2025-06-07"),
        )
        .unwrap();

        let trip = get_trip(&conn, id).unwrap();
        assert_eq!(trip.name, "沖縄・瀬底旅行");
        assert_eq!(trip.start_date.as_deref(), Some("2025-06-01"));
        assert_eq!(trip.end_date.as_deref(), Some("2025-06-07"));
    }

    #[test]
    fn test_delete_trip() {
        let conn = test_db();
        let id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        delete_trip(&conn, id).unwrap();

        assert!(list_trips(&conn).unwrap().is_empty());
        assert!(get_trip(&conn, id).is_err());
    }

    #[test]
    fn test_init_db_creates_itinerary_items_table() {
        let conn = Connection::open(":memory:").unwrap();
        init_db(&conn).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'itinerary_items'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_add_itinerary_item() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            Some("午前"),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(id, 1);

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.trip_id, trip_id);
        assert_eq!(item.day, 1);
        assert_eq!(item.title, "首里城");
        assert_eq!(item.note.as_deref(), Some("午前"));
        assert_eq!(item.sort_order, 0);
    }

    #[test]
    fn test_add_itinerary_item_with_start_time() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.start_time.as_deref(), Some("09:00"));
    }

    #[test]
    fn test_add_itinerary_item_without_start_time() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテルチェックイン",
            None,
            None,
            Some(99),
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert!(item.start_time.is_none());
        assert_eq!(item.sort_order, 99);
    }

    #[test]
    fn test_list_itinerary_items() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        add_itinerary_item(&conn, trip_id, 1, "首里城", None, None, None, None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "美ら海水族館",
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "首里城");
        assert_eq!(items[1].title, "美ら海水族館");
    }

    #[test]
    fn test_list_itinerary_items_sorted_by_day_and_time() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        // 登録順をバラバラにしても、一覧は day → 時刻順になること
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "昼食",
            None,
            Some("12:30"),
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "ホテル",
            None,
            None,
            Some(99),
            None,
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目",
            None,
            Some("10:00"),
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].title, "首里城");
        assert_eq!(items[1].title, "昼食");
        assert_eq!(items[2].title, "ホテル");
        assert_eq!(items[3].title, "2日目");
    }

    #[test]
    fn test_get_itinerary_item() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            Some("午前"),
            Some("09:00"),
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.id, id);
        assert_eq!(item.day, 1);
        assert_eq!(item.title, "首里城");
    }

    #[test]
    fn test_update_itinerary_item() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id =
            add_itinerary_item(&conn, trip_id, 1, "首里城", None, None, None, None, None).unwrap();

        update_itinerary_item(
            &conn,
            id,
            Some(2),
            Some("美ら海水族館"),
            Some(Some("終日")),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.day, 2);
        assert_eq!(item.title, "美ら海水族館");
        assert_eq!(item.note.as_deref(), Some("終日"));
    }

    #[test]
    fn test_update_itinerary_item_start_time_and_sort_order() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id =
            add_itinerary_item(&conn, trip_id, 1, "首里城", None, None, None, None, None).unwrap();

        update_itinerary_item(
            &conn,
            id,
            None,
            None,
            None,
            Some(Some("09:30")),
            Some(5),
            None,
            None,
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.start_time.as_deref(), Some("09:30"));
        assert_eq!(item.sort_order, 5);
    }

    #[test]
    fn test_migrate_itinerary_items_adds_columns() {
        // 旧スキーマの DB に対して migrate が列を追加できること
        let conn = Connection::open(":memory:").unwrap();
        conn.execute(
            "CREATE TABLE itinerary_items (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                trip_id     INTEGER NOT NULL,
                day         INTEGER NOT NULL,
                title       TEXT NOT NULL,
                note        TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        migrate_itinerary_items(&conn).unwrap();
        migrate_itinerary_items(&conn).unwrap(); // 2回実行してもエラーにならない

        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(itinerary_items)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(columns.contains(&"start_time".to_string()));
        assert!(columns.contains(&"sort_order".to_string()));
        assert!(columns.contains(&"duration_minutes".to_string()));
        assert!(columns.contains(&"travel_minutes".to_string()));
    }

    #[test]
    fn test_delete_itinerary_item() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id =
            add_itinerary_item(&conn, trip_id, 1, "首里城", None, None, None, None, None).unwrap();

        delete_itinerary_item(&conn, id).unwrap();

        assert!(list_itinerary_items(&conn, trip_id).unwrap().is_empty());
        assert!(get_itinerary_item(&conn, id).is_err());
    }

    #[test]
    fn test_reset_db() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            None,
            None,
        )
        .unwrap();

        reset_db(&conn).unwrap();

        assert!(list_trips(&conn).unwrap().is_empty());

        // AUTOINCREMENT がリセットされ、次の ID は 1 から再開する
        let new_trip_id = add_trip(&conn, "新規旅行", None, None).unwrap();
        assert_eq!(new_trip_id, 1);

        let new_item_id = add_itinerary_item(
            &conn,
            new_trip_id,
            1,
            "テスト",
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(new_item_id, 1);
    }

    #[test]
    fn test_add_itinerary_item_with_duration_and_travel() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        let id = add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
        )
        .unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.duration_minutes, Some(90));
        assert_eq!(item.travel_minutes, Some(20));
    }

    #[test]
    fn test_add_minutes_to_time() {
        assert_eq!(add_minutes_to_time("09:00", 90).unwrap(), "10:30");
        assert_eq!(add_minutes_to_time("12:30", 30).unwrap(), "13:00");
        assert!(parse_time_hhmm("25:00").is_err());
        assert!(parse_time_hhmm("9:00").is_err());
        assert!(add_minutes_to_time("23:00", 120).is_err());
    }

    #[test]
    fn test_timeline_items_sorted_by_day_and_time() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();

        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "国際通り",
            None,
            Some("10:50"),
            None,
            Some(60),
            None,
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            1,
            "首里城",
            None,
            Some("09:00"),
            None,
            Some(90),
            Some(20),
        )
        .unwrap();
        add_itinerary_item(
            &conn,
            trip_id,
            2,
            "2日目",
            None,
            Some("10:00"),
            None,
            None,
            None,
        )
        .unwrap();

        let items = list_itinerary_items(&conn, trip_id).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "首里城");
        assert_eq!(items[1].title, "国際通り");
        assert_eq!(items[2].title, "2日目");
        assert_eq!(items[0].day, 1);
        assert_eq!(items[0].start_time.as_deref(), Some("09:00"));
    }

    #[test]
    fn test_update_itinerary_item_duration_and_travel() {
        let conn = test_db();
        let trip_id = add_trip(&conn, "沖縄旅行", None, None).unwrap();
        let id =
            add_itinerary_item(&conn, trip_id, 1, "首里城", None, None, None, None, None).unwrap();

        update_itinerary_item(&conn, id, None, None, None, None, None, Some(90), Some(20)).unwrap();

        let item = get_itinerary_item(&conn, id).unwrap();
        assert_eq!(item.duration_minutes, Some(90));
        assert_eq!(item.travel_minutes, Some(20));
    }
}
