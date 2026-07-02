use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use rusqlite::{params, Connection};
use serde::Serialize;

use crate::domain::models::{ExportReceiptDayRef, ExportReceiptV7, Receipt};
use crate::money::{format_amount_display, parse_amount_for_currency, validate_currency_code};

pub(crate) const RECEIPT_STATUS_UNREVIEWED: &str = "unreviewed";
pub(crate) const RECEIPT_STATUS_IGNORED: &str = "ignored";

const RECEIPT_SELECT_SQL: &str = "
    SELECT id, trip_id, day_id, trashed_at, amount, currency, occurred_date, memo, status, created_at, updated_at
    FROM receipts";

pub(crate) fn migrate_receipts(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS receipts (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            trip_id             INTEGER NOT NULL,
            day_id              INTEGER NULL,
            trashed_at          TEXT NULL,
            amount              INTEGER NULL,
            currency            TEXT NULL,
            occurred_date       TEXT NULL,
            memo                TEXT NULL,
            status              TEXT NOT NULL,
            created_at          TEXT NOT NULL,
            updated_at          TEXT NOT NULL
        )",
        [],
    )
    .context("receipts テーブルの作成に失敗しました")?;
    crate::storage::db::add_column_if_not_exists(conn, "receipts", "trashed_at", "TEXT NULL")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_receipts_trip ON receipts(trip_id)",
        [],
    )
    .context("idx_receipts_trip の作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_receipts_day ON receipts(day_id)",
        [],
    )
    .context("idx_receipts_day の作成に失敗しました")?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_receipts_trashed ON receipts(trashed_at)",
        [],
    )
    .context("idx_receipts_trashed の作成に失敗しました")?;

    // v3.7+: ignored は Trash として扱う（既存データを normalize）
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts
         SET trashed_at = ?1
         WHERE status = 'ignored' AND trashed_at IS NULL",
        params![&now],
    )
    .context("ignored receipts の trashed_at 正規化に失敗しました")?;
    Ok(())
}

fn receipts_table_has_column(conn: &Connection, column: &str) -> Result<bool> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(receipts)")
        .context("receipts テーブル情報の取得に失敗しました")?;
    let exists = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .any(|name| name == column);
    Ok(exists)
}

/// 旧 receipts スキーマ（itinerary_id / linked_expense_id）から単純化スキーマへ移行する
pub(crate) fn migrate_receipts_simplified_schema(conn: &Connection) -> Result<()> {
    if !receipts_table_has_column(conn, "itinerary_id")? {
        return Ok(());
    }

    crate::storage::db::with_transaction(conn, "receipts schema simplify", |tx| {
        tx.execute(
            "CREATE TABLE receipts_simplified (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                trip_id             INTEGER NOT NULL,
                day_id              INTEGER NULL,
                trashed_at          TEXT NULL,
                amount              INTEGER NULL,
                currency            TEXT NULL,
                occurred_date       TEXT NULL,
                memo                TEXT NULL,
                status              TEXT NOT NULL,
                created_at          TEXT NOT NULL,
                updated_at          TEXT NOT NULL
            )",
            [],
        )
        .context("receipts_simplified テーブルの作成に失敗しました")?;
        tx.execute(
            "INSERT INTO receipts_simplified
             (id, trip_id, day_id, trashed_at, amount, currency, occurred_date, memo, status, created_at, updated_at)
             SELECT id, trip_id, day_id,
                    CASE WHEN status = 'ignored' THEN updated_at ELSE NULL END,
                    amount, currency, occurred_date, memo,
                    CASE WHEN status = 'ignored' THEN 'ignored' ELSE 'unreviewed' END,
                    created_at, updated_at
             FROM receipts",
            [],
        )
        .context("receipts の移行に失敗しました")?;
        tx.execute("DROP TABLE receipts", [])
            .context("旧 receipts テーブルの削除に失敗しました")?;
        tx.execute("ALTER TABLE receipts_simplified RENAME TO receipts", [])
            .context("receipts テーブルのリネームに失敗しました")?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_receipts_trip ON receipts(trip_id)",
            [],
        )
        .context("idx_receipts_trip の作成に失敗しました")?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_receipts_day ON receipts(day_id)",
            [],
        )
        .context("idx_receipts_day の作成に失敗しました")?;
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_receipts_trashed ON receipts(trashed_at)",
            [],
        )
        .context("idx_receipts_trashed の作成に失敗しました")?;
        Ok(())
    })
}

pub(crate) fn validate_receipt_status(status: &str) -> Result<()> {
    match status {
        RECEIPT_STATUS_UNREVIEWED | RECEIPT_STATUS_IGNORED => Ok(()),
        _ => anyhow::bail!("invalid receipt status: {status}"),
    }
}

fn normalize_import_receipt_status(status: &str) -> Result<String> {
    match status {
        RECEIPT_STATUS_UNREVIEWED | RECEIPT_STATUS_IGNORED => Ok(status.to_string()),
        "linked" | "converted" => Ok(RECEIPT_STATUS_UNREVIEWED.to_string()),
        _ => validate_receipt_status(status).map(|_| status.to_string()),
    }
}

fn row_to_receipt(row: &rusqlite::Row) -> rusqlite::Result<Receipt> {
    Ok(Receipt {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        day_id: row.get(2)?,
        trashed_at: row.get(3)?,
        amount: row.get(4)?,
        currency: row.get(5)?,
        occurred_date: row.get(6)?,
        memo: row.get(7)?,
        status: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn normalize_optional_memo(memo: Option<&str>) -> Result<Option<String>> {
    match memo {
        None => Ok(None),
        Some(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
    }
}

pub(crate) fn validate_receipt_amount_currency_pair(
    amount: Option<i64>,
    currency: &Option<String>,
) -> Result<()> {
    match (amount, currency.as_deref()) {
        (Some(_), None) => anyhow::bail!("currency is required when amount is set"),
        (None, Some(c)) if !c.trim().is_empty() => {
            anyhow::bail!("amount is required when currency is set")
        }
        _ => Ok(()),
    }
}

fn validate_receipt_has_content(memo: &Option<String>, amount: Option<i64>) -> Result<()> {
    if memo.is_some() || amount.is_some() {
        Ok(())
    } else {
        anyhow::bail!("receipt requires memo and/or amount with currency")
    }
}

pub(crate) struct AddReceiptParams<'a> {
    pub trip_id: i64,
    pub day_number: Option<i64>,
    pub amount_input: Option<&'a str>,
    pub currency_input: Option<&'a str>,
    pub occurred_date: Option<&'a str>,
    pub memo: Option<&'a str>,
}

pub(crate) fn add_receipt(conn: &Connection, params: AddReceiptParams<'_>) -> Result<i64> {
    crate::trip::get_trip(conn, params.trip_id)?;

    let memo = normalize_optional_memo(params.memo)?;
    let (amount, currency) = match (params.amount_input, params.currency_input) {
        (Some(amount_str), Some(currency_str)) => {
            let currency = validate_currency_code(currency_str)?;
            let amount = parse_amount_for_currency(amount_str, &currency)?;
            (Some(amount), Some(currency))
        }
        (None, None) => (None, None),
        _ => anyhow::bail!("amount and currency must be provided together"),
    };
    validate_receipt_amount_currency_pair(amount, &currency)?;
    validate_receipt_has_content(&memo, amount)?;

    if let Some(date) = params.occurred_date {
        crate::expense::validate_expense_date(date)?;
    }

    let day_id = if let Some(day_number) = params.day_number {
        Some(crate::day::find_day_id_by_trip_and_day_number(
            conn,
            params.trip_id,
            day_number,
        )?)
    } else {
        None
    };

    let status = RECEIPT_STATUS_UNREVIEWED;
    let now = crate::storage::db::now_string();
    conn.execute(
        "INSERT INTO receipts
         (trip_id, day_id, trashed_at, amount, currency, occurred_date, memo, status, created_at, updated_at)
         VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            params.trip_id,
            day_id,
            amount,
            currency,
            params.occurred_date,
            memo,
            status,
            &now,
            &now,
        ],
    )
    .context("Receipt の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_receipts_for_trip(
    conn: &Connection,
    trip_id: i64,
    status_filter: Option<&str>,
    include_trashed: bool,
    trashed_only: bool,
) -> Result<Vec<Receipt>> {
    crate::trip::get_trip(conn, trip_id)?;
    if let Some(status) = status_filter {
        validate_receipt_status(status)?;
    }

    let mut where_clauses = vec!["trip_id = ?1".to_string()];
    if trashed_only {
        where_clauses.push("trashed_at IS NOT NULL".to_string());
    } else if !include_trashed {
        where_clauses.push("trashed_at IS NULL".to_string());
    }
    if status_filter.is_some() {
        where_clauses.push("status = ?2".to_string());
    }
    let where_sql = where_clauses.join(" AND ");
    let sql = format!("{RECEIPT_SELECT_SQL} WHERE {where_sql} ORDER BY id ASC");

    let mut stmt = conn
        .prepare(&sql)
        .context("Receipt 一覧取得の準備に失敗しました")?;

    let rows = if let Some(status) = status_filter {
        stmt.query_map(params![trip_id, status], row_to_receipt)
    } else {
        stmt.query_map(params![trip_id], row_to_receipt)
    }
    .context("Receipt 一覧取得に失敗しました")?
    .collect::<std::result::Result<Vec<_>, _>>()
    .context("Receipt 一覧の読み込みに失敗しました")?;
    Ok(rows)
}

pub(crate) fn get_receipt(conn: &Connection, id: i64) -> Result<Receipt> {
    crate::storage::db::map_query_row(
        conn.query_row(
            &format!("{RECEIPT_SELECT_SQL} WHERE id = ?1"),
            params![id],
            row_to_receipt,
        ),
        || anyhow::anyhow!("receipt not found: {id}"),
    )
}

pub(crate) struct UpdateReceiptParams<'a> {
    pub day_number: Option<i64>,
    pub amount_input: Option<&'a str>,
    pub currency_input: Option<&'a str>,
    pub occurred_date: Option<Option<&'a str>>,
    pub memo: Option<Option<&'a str>>,
    pub clear_day: bool,
    pub clear_amount_currency: bool,
}

pub(crate) fn update_receipt(
    conn: &Connection,
    id: i64,
    params: UpdateReceiptParams<'_>,
) -> Result<()> {
    let existing = get_receipt(conn, id)?;
    let mut day_id = existing.day_id;
    let mut amount = existing.amount;
    let mut currency = existing.currency;
    let mut occurred_date = existing.occurred_date.clone();
    let mut memo = existing.memo.clone();

    if params.clear_day {
        day_id = None;
    }
    if let Some(day_number) = params.day_number {
        day_id = Some(crate::day::find_day_id_by_trip_and_day_number(
            conn,
            existing.trip_id,
            day_number,
        )?);
    }

    if params.clear_amount_currency {
        amount = None;
        currency = None;
    } else if params.amount_input.is_some() || params.currency_input.is_some() {
        let amount_str = params
            .amount_input
            .ok_or_else(|| anyhow::anyhow!("amount is required when updating currency"))?;
        let currency_str = params
            .currency_input
            .ok_or_else(|| anyhow::anyhow!("currency is required when updating amount"))?;
        let parsed_currency = validate_currency_code(currency_str)?;
        amount = Some(parse_amount_for_currency(amount_str, &parsed_currency)?);
        currency = Some(parsed_currency);
    }

    if let Some(date_opt) = params.occurred_date {
        match date_opt {
            Some(date) => {
                crate::expense::validate_expense_date(date)?;
                occurred_date = Some(date.to_string());
            }
            None => occurred_date = None,
        }
    }

    if let Some(memo_opt) = params.memo {
        memo = normalize_optional_memo(memo_opt)?;
    }

    validate_receipt_amount_currency_pair(amount, &currency)?;
    validate_receipt_has_content(&memo, amount)?;

    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET day_id = ?1, amount = ?2, currency = ?3,
         occurred_date = ?4, memo = ?5, updated_at = ?6
         WHERE id = ?7",
        params![day_id, amount, currency, occurred_date, memo, &now, id,],
    )
    .context("Receipt の更新に失敗しました")?;
    Ok(())
}

pub(crate) fn ignore_receipt(conn: &Connection, id: i64, memo: Option<&str>) -> Result<()> {
    // v3.7+: `ignore` は deprecated alias として `trash` 相当に扱う
    let existing = get_receipt(conn, id)?;
    let new_memo = if let Some(text) = memo {
        normalize_optional_memo(Some(text))?
    } else {
        existing.memo
    };
    trash_receipt_with_memo(conn, id, new_memo.as_deref())
}

pub(crate) fn trash_receipt(conn: &Connection, id: i64) -> Result<()> {
    trash_receipt_with_memo(conn, id, None)
}

fn trash_receipt_with_memo(conn: &Connection, id: i64, memo: Option<&str>) -> Result<()> {
    get_receipt(conn, id)?;
    let new_memo = memo.map(|m| m.to_string());
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts
         SET status = ?1, trashed_at = ?2, memo = COALESCE(?3, memo), updated_at = ?4
         WHERE id = ?5",
        params![RECEIPT_STATUS_IGNORED, &now, new_memo, &now, id],
    )
    .context("Receipt の trash に失敗しました")?;
    Ok(())
}

pub(crate) fn restore_receipt(conn: &Connection, id: i64) -> Result<()> {
    get_receipt(conn, id)?;
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts
         SET status = ?1, trashed_at = NULL, updated_at = ?2
         WHERE id = ?3",
        params![RECEIPT_STATUS_UNREVIEWED, &now, id],
    )
    .context("Receipt の restore に失敗しました")?;
    Ok(())
}

pub(crate) fn assign_receipt_to_itinerary(
    conn: &Connection,
    receipt_id: i64,
    itinerary_id: i64,
    amount_input: Option<&str>,
    currency_input: Option<&str>,
    memo_input: Option<&str>,
) -> Result<i64> {
    // NOTE: Receipt は Actual ではない。assign により Expense を作ることでのみ Actual に入る。
    let receipt = get_receipt(conn, receipt_id)?;
    if receipt.trashed_at.is_some() {
        anyhow::bail!("cannot assign trashed receipt");
    }

    // Receipt 側の amount/currency が無い場合は assign で補完できる。
    let (amount_str, currency_str) = match (
        receipt.amount,
        receipt.currency.as_deref(),
        amount_input,
        currency_input,
    ) {
        (Some(amount), Some(currency), None, None) => (amount.to_string(), currency.to_string()),
        (_, _, Some(a), Some(c)) => (a.to_string(), c.to_string()),
        (Some(_), Some(_), Some(_), None) | (Some(_), Some(_), None, Some(_)) => {
            anyhow::bail!("amount and currency must be provided together")
        }
        (None, None, None, None) => anyhow::bail!("amount and currency are required for assign"),
        (None, None, Some(_), None) | (None, None, None, Some(_)) => {
            anyhow::bail!("amount and currency must be provided together")
        }
        (Some(_), None, _, _) | (None, Some(_), _, _) => {
            anyhow::bail!("receipt amount/currency pair is invalid")
        }
    };

    // Expense title: memo を優先。無ければ amount/currency から自動生成。
    let merged_memo = memo_input
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| receipt.memo.clone());
    let parsed_currency = validate_currency_code(&currency_str)?;
    let parsed_amount = parse_amount_for_currency(&amount_str, &parsed_currency)?;
    let generated_title = format!(
        "Receipt {}",
        format_amount_display(parsed_amount, &parsed_currency)
    );
    let title = merged_memo.as_deref().unwrap_or(generated_title.as_str());

    // occurred_date は Expense の expense_date として引き継ぐ（存在する場合）。
    let expense_date = receipt.occurred_date.as_deref();

    let shared = crate::expense::ExpenseSharedOptions {
        paid_by_participant_id: None,
        beneficiary_participant_ids: None,
        clear_paid_by: false,
        clear_beneficiaries: false,
    };

    let tx = conn
        .unchecked_transaction()
        .context("receipt assign: トランザクション開始に失敗しました")?;
    let expense_id = crate::expense::add_expense(
        &tx,
        itinerary_id,
        &amount_str,
        &currency_str,
        Some(title),
        None,
        None,
        expense_date,
        &shared,
    )
    .context("receipt assign: Expense 作成に失敗しました")?;
    tx.execute("DELETE FROM receipts WHERE id = ?1", params![receipt_id])
        .context("receipt assign: Receipt の削除に失敗しました")?;
    tx.commit()
        .context("receipt assign: トランザクション確定に失敗しました")?;
    Ok(expense_id)
}

pub(crate) fn delete_receipt(conn: &Connection, id: i64) -> Result<()> {
    get_receipt(conn, id)?;
    conn.execute("DELETE FROM receipts WHERE id = ?1", params![id])
        .context("Receipt の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_receipts_for_trip(conn: &Connection, trip_id: i64) -> Result<()> {
    conn.execute("DELETE FROM receipts WHERE trip_id = ?1", params![trip_id])
        .context("Receipt の Trip 削除に失敗しました")?;
    Ok(())
}

pub(crate) fn nullify_receipts_for_day(conn: &Connection, day_id: i64) -> Result<()> {
    let now = crate::storage::db::now_string();
    conn.execute(
        "UPDATE receipts SET day_id = NULL, updated_at = ?1 WHERE day_id = ?2",
        params![&now, day_id],
    )
    .context("Receipt day_id のクリアに失敗しました")?;
    Ok(())
}

fn format_db_timestamp_for_export(ts: &str) -> String {
    if DateTime::parse_from_rfc3339(ts).is_ok() {
        return ts.to_string();
    }
    if let Ok(naive) = NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S") {
        if let Some(local) = Local.from_local_datetime(&naive).single() {
            return local.to_rfc3339();
        }
    }
    ts.to_string()
}

fn day_number_for_receipt(conn: &Connection, receipt: &Receipt) -> Result<Option<i64>> {
    if let Some(day_id) = receipt.day_id {
        let day_number: i64 = conn.query_row(
            "SELECT day_number FROM days WHERE id = ?1",
            params![day_id],
            |row| row.get(0),
        )?;
        return Ok(Some(day_number));
    }
    Ok(None)
}

pub(crate) fn build_export_receipt_v7(
    conn: &Connection,
    receipt: &Receipt,
) -> Result<ExportReceiptV7> {
    validate_receipt_status(&receipt.status)?;

    let day_ref = if let Some(day_id) = receipt.day_id {
        let day_number: i64 = conn.query_row(
            "SELECT day_number FROM days WHERE id = ?1",
            params![day_id],
            |row| row.get(0),
        )?;
        Some(ExportReceiptDayRef { day_number })
    } else {
        None
    };

    Ok(ExportReceiptV7 {
        day_ref,
        trashed_at: receipt
            .trashed_at
            .as_deref()
            .map(format_db_timestamp_for_export),
        amount: receipt.amount,
        currency: receipt.currency.clone(),
        occurred_date: receipt.occurred_date.clone(),
        memo: receipt.memo.clone(),
        status: receipt.status.clone(),
    })
}

pub(crate) fn build_export_receipts_for_trip(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<ExportReceiptV7>> {
    // export では trashed Receipt も含める（restore 可能性を保つ）
    list_receipts_for_trip(conn, trip_id, None, true, false)?
        .iter()
        .map(|r| build_export_receipt_v7(conn, r))
        .collect()
}

pub(crate) fn import_receipt_v7(
    conn: &Connection,
    trip_id: i64,
    export: &ExportReceiptV7,
) -> Result<i64> {
    let status = normalize_import_receipt_status(&export.status)?;
    validate_receipt_amount_currency_pair(export.amount, &export.currency)?;
    if let Some(date) = export.occurred_date.as_deref() {
        crate::expense::validate_expense_date(date)?;
    }
    let memo = export.memo.clone();
    validate_receipt_has_content(&memo, export.amount)?;

    let day_id = if let Some(day_ref) = &export.day_ref {
        Some(crate::day::find_day_id_by_trip_and_day_number(
            conn,
            trip_id,
            day_ref.day_number,
        )?)
    } else {
        None
    };

    let now = crate::storage::db::now_string();
    let trashed_at = if let Some(value) = export.trashed_at.as_deref() {
        Some(value.to_string())
    } else if status == RECEIPT_STATUS_IGNORED {
        Some(now.clone())
    } else {
        None
    };
    conn.execute(
        "INSERT INTO receipts
         (trip_id, day_id, trashed_at, amount, currency, occurred_date, memo, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            trip_id,
            day_id,
            trashed_at,
            export.amount,
            export.currency,
            export.occurred_date,
            memo,
            status,
            &now,
            &now,
        ],
    )
    .context("Receipt の import に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn collect_export_receipt_validation_errors(
    receipts: &[ExportReceiptV7],
    effective_schema: i32,
) -> Vec<String> {
    use crate::domain::models::{TRIP_EXPORT_SCHEMA_VERSION, TRIP_EXPORT_SCHEMA_VERSION_V7};

    if effective_schema < TRIP_EXPORT_SCHEMA_VERSION_V7 {
        return Vec::new();
    }

    let mut errors = Vec::new();
    for (index, receipt) in receipts.iter().enumerate() {
        let prefix = format!("receipts[{index}]");
        if let Err(error) = validate_receipt_status(&receipt.status) {
            errors.push(format!("{prefix}.status: {error}"));
        }
        if let Err(error) = validate_receipt_amount_currency_pair(receipt.amount, &receipt.currency)
        {
            errors.push(format!("{prefix}: {error}"));
        }
        if receipt.amount.is_none() && receipt.memo.is_none() {
            errors.push(format!("{prefix}: memo and/or amount required"));
        }
        if let Some(currency) = receipt.currency.as_deref() {
            if let Err(error) = validate_currency_code(currency) {
                errors.push(format!("{prefix}.currency: {error}"));
            }
        }
        if let Some(date) = receipt.occurred_date.as_deref() {
            if let Err(error) = crate::expense::validate_expense_date(date) {
                errors.push(format!("{prefix}.occurred_date: {error}"));
            }
        }
        if effective_schema >= TRIP_EXPORT_SCHEMA_VERSION {
            if let Some(ts) = receipt.trashed_at.as_deref() {
                if chrono::DateTime::parse_from_rfc3339(ts).is_err() {
                    errors.push(format!("{prefix}.trashed_at: invalid RFC3339 timestamp"));
                }
            }
        } else if receipt.trashed_at.is_some() {
            errors.push(format!("{prefix}.trashed_at: unsupported before schema v8"));
        }
    }
    errors
}

#[derive(Serialize)]
pub(crate) struct ReceiptJson {
    pub id: i64,
    pub trip_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_number: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trashed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurred_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

pub(crate) fn receipt_to_json(conn: &Connection, receipt: &Receipt) -> Result<ReceiptJson> {
    Ok(ReceiptJson {
        id: receipt.id,
        trip_id: receipt.trip_id,
        day_id: receipt.day_id,
        day_number: day_number_for_receipt(conn, receipt)?,
        trashed_at: receipt.trashed_at.clone(),
        amount: receipt.amount,
        currency: receipt.currency.clone(),
        occurred_date: receipt.occurred_date.clone(),
        memo: receipt.memo.clone(),
        status: receipt.status.clone(),
        created_at: receipt.created_at.clone(),
        updated_at: receipt.updated_at.clone(),
    })
}

#[derive(Clone, Serialize)]
pub(crate) struct PendingReceiptSummaryJson {
    pub active_count: usize,
    pub without_amount_count: usize,
    pub totals_by_currency: std::collections::BTreeMap<String, i64>,
}

#[derive(Serialize)]
pub(crate) struct ReceiptListJson {
    pub trip_id: i64,
    pub summary: PendingReceiptSummaryJson,
    pub receipts: Vec<ReceiptJson>,
}

pub(crate) fn compute_pending_receipt_summary(
    conn: &Connection,
    trip_id: i64,
) -> Result<PendingReceiptSummaryJson> {
    let receipts = list_receipts_for_trip(conn, trip_id, None, false, false)?;
    let active_count = receipts.len();
    let without_amount_count = receipts.iter().filter(|r| r.amount.is_none()).count();
    let mut totals_by_currency: std::collections::BTreeMap<String, i64> =
        std::collections::BTreeMap::new();
    for r in &receipts {
        if let (Some(amount), Some(currency)) = (r.amount, r.currency.as_deref()) {
            *totals_by_currency.entry(currency.to_string()).or_default() += amount;
        }
    }
    Ok(PendingReceiptSummaryJson {
        active_count,
        without_amount_count,
        totals_by_currency,
    })
}

/// Trip の active な Receipt 件数（`trashed_at IS NULL`）を返す
pub(crate) fn count_active_receipts_for_trip(conn: &Connection, trip_id: i64) -> Result<usize> {
    crate::trip::get_trip(conn, trip_id)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM receipts WHERE trip_id = ?1 AND trashed_at IS NULL",
        params![trip_id],
        |row| row.get(0),
    )?;
    Ok(count.max(0) as usize)
}

/// Receipt の状態矛盾を検出して id を返す
///
/// - `status = ignored` かつ `trashed_at IS NULL`
/// - `status = unreviewed` かつ `trashed_at IS NOT NULL`
pub(crate) fn find_inconsistent_receipts_for_trip(
    conn: &Connection,
    trip_id: i64,
) -> Result<Vec<i64>> {
    crate::trip::get_trip(conn, trip_id)?;
    let mut stmt = conn.prepare(
        "SELECT id
         FROM receipts
         WHERE trip_id = ?1
           AND (
             (status = ?2 AND trashed_at IS NULL)
             OR
             (status = ?3 AND trashed_at IS NOT NULL)
           )
         ORDER BY id",
    )?;
    let rows = stmt.query_map(
        params![trip_id, RECEIPT_STATUS_IGNORED, RECEIPT_STATUS_UNREVIEWED],
        |row| row.get::<_, i64>(0),
    )?;
    let mut ids = Vec::new();
    for id in rows {
        ids.push(id?);
    }
    Ok(ids)
}

pub(crate) fn print_pending_receipt_summary(summary: &PendingReceiptSummaryJson) {
    println!("Pending Receipts:");
    println!("  Count: {}", summary.active_count);
    println!("  Without amount: {}", summary.without_amount_count);
    if summary.totals_by_currency.is_empty() {
        println!("  Totals: -");
    } else {
        println!("  Totals:");
        for (currency, amount) in &summary.totals_by_currency {
            println!("    - {}", format_amount_display(*amount, currency));
        }
    }
    println!();
}

fn format_amount_optional(amount: Option<i64>, currency: &Option<String>) -> String {
    match (amount, currency.as_deref()) {
        (Some(value), Some(cur)) => format_amount_display(value, cur),
        _ => "-".to_string(),
    }
}

pub(crate) fn print_receipt_list(conn: &Connection, receipts: &[Receipt]) -> Result<()> {
    if receipts.is_empty() {
        println!("Receipt はまだ登録されていません。");
        return Ok(());
    }
    println!(
        "{:<4} {:<10} {:<6} {:<16} {:<12} {:<6} Memo",
        "ID", "Status", "Day", "Amount", "Date", "Trash"
    );
    println!("{}", "-".repeat(72));
    for receipt in receipts {
        let day_number = day_number_for_receipt(conn, receipt)?
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string());
        let amount = format_amount_optional(receipt.amount, &receipt.currency);
        let date = receipt.occurred_date.as_deref().unwrap_or("-");
        let trash = if receipt.trashed_at.is_some() {
            "yes"
        } else {
            "-"
        };
        let memo = receipt.memo.as_deref().unwrap_or("-");
        println!(
            "{:<4} {:<10} {:<6} {:<16} {:<12} {:<6} {}",
            receipt.id, receipt.status, day_number, amount, date, trash, memo,
        );
    }
    println!();
    println!("合計: {} 件", receipts.len());
    Ok(())
}

pub(crate) fn print_receipt_detail(conn: &Connection, receipt: &Receipt) -> Result<()> {
    let day_number = day_number_for_receipt(conn, receipt)?;
    println!("ID              : {}", receipt.id);
    println!("Trip ID         : {}", receipt.trip_id);
    println!(
        "Day             : {}",
        day_number
            .map(|d| d.to_string())
            .unwrap_or_else(|| "-".to_string())
    );
    println!(
        "Amount          : {}",
        format_amount_optional(receipt.amount, &receipt.currency)
    );
    println!(
        "Currency        : {}",
        receipt.currency.as_deref().unwrap_or("-")
    );
    println!(
        "Occurred date   : {}",
        receipt.occurred_date.as_deref().unwrap_or("-")
    );
    println!(
        "Trashed at      : {}",
        receipt.trashed_at.as_deref().unwrap_or("-")
    );
    println!(
        "Memo            : {}",
        receipt.memo.as_deref().unwrap_or("-")
    );
    println!("Status          : {}", receipt.status);
    println!("Created at      : {}", receipt.created_at);
    println!("Updated at      : {}", receipt.updated_at);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::{init_db, open_db_at};
    use std::path::PathBuf;

    fn memory_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    fn temp_conn() -> (Connection, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "caglla-receipt-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("caglla.db");
        let conn = open_db_at(path.to_str().unwrap()).unwrap();
        (conn, dir)
    }

    fn setup_trip(conn: &Connection) -> i64 {
        crate::trip::add_trip(conn, "Receipt Trip", "2026-04-26", "2026-04-29", None).unwrap()
    }

    #[test]
    fn test_add_list_show_update_delete_receipt() {
        let conn = memory_conn();
        let trip_id = setup_trip(&conn);
        let id = add_receipt(
            &conn,
            AddReceiptParams {
                trip_id,
                day_number: Some(1),
                amount_input: Some("1700"),
                currency_input: Some("JPY"),
                occurred_date: Some("2026-04-26"),
                memo: Some("これなんだっけ？"),
            },
        )
        .unwrap();
        let receipts = list_receipts_for_trip(&conn, trip_id, None, true, false).unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].status, RECEIPT_STATUS_UNREVIEWED);

        update_receipt(
            &conn,
            id,
            UpdateReceiptParams {
                day_number: None,
                amount_input: None,
                currency_input: None,
                occurred_date: None,
                memo: Some(Some("おかんのお土産っぽい")),
                clear_day: false,
                clear_amount_currency: false,
            },
        )
        .unwrap();
        let updated = get_receipt(&conn, id).unwrap();
        assert_eq!(updated.memo.as_deref(), Some("おかんのお土産っぽい"));

        ignore_receipt(&conn, id, Some("旅行費用ではない")).unwrap();
        let ignored = get_receipt(&conn, id).unwrap();
        assert_eq!(ignored.status, RECEIPT_STATUS_IGNORED);
        assert_eq!(ignored.amount, Some(1700));

        delete_receipt(&conn, id).unwrap();
        assert!(get_receipt(&conn, id).is_err());
    }

    #[test]
    fn test_invalid_receipt_status_rejected() {
        assert!(validate_receipt_status("linked").is_err());
        assert!(validate_receipt_status("converted").is_err());
        assert!(validate_receipt_status("bogus").is_err());
    }

    #[test]
    fn test_import_normalizes_legacy_linked_status() {
        let conn = memory_conn();
        let trip_id = setup_trip(&conn);
        let export = ExportReceiptV7 {
            day_ref: None,
            amount: Some(100),
            currency: Some("JPY".to_string()),
            occurred_date: None,
            memo: Some("legacy".to_string()),
            status: "linked".to_string(),
            trashed_at: None,
        };
        import_receipt_v7(&conn, trip_id, &export).unwrap();
        let receipt = list_receipts_for_trip(&conn, trip_id, None, true, false).unwrap()[0].clone();
        assert_eq!(receipt.status, RECEIPT_STATUS_UNREVIEWED);
    }

    #[test]
    fn test_export_import_receipt_roundtrip() {
        let (conn, _dir) = temp_conn();
        let trip_id = setup_trip(&conn);
        add_receipt(
            &conn,
            AddReceiptParams {
                trip_id,
                day_number: Some(1),
                amount_input: Some("1700"),
                currency_input: Some("JPY"),
                occurred_date: None,
                memo: Some("memo"),
            },
        )
        .unwrap();

        let exports = build_export_receipts_for_trip(&conn, trip_id).unwrap();
        assert_eq!(exports.len(), 1);
        assert_eq!(exports[0].status, RECEIPT_STATUS_UNREVIEWED);
        assert!(exports[0].day_ref.is_some());

        crate::storage::db::reset_db(&conn).unwrap();
        let new_trip = setup_trip(&conn);
        import_receipt_v7(&conn, new_trip, &exports[0]).unwrap();
        let imported = list_receipts_for_trip(&conn, new_trip, None, true, false).unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].memo.as_deref(), Some("memo"));
    }

    #[test]
    fn test_nullify_on_day_delete() {
        let conn = memory_conn();
        let trip_id = setup_trip(&conn);
        let day_id = crate::day::find_day_id_by_trip_and_day_number(&conn, trip_id, 1).unwrap();
        let receipt_id = add_receipt(
            &conn,
            AddReceiptParams {
                trip_id,
                day_number: Some(1),
                amount_input: None,
                currency_input: None,
                occurred_date: None,
                memo: Some("keep"),
            },
        )
        .unwrap();
        nullify_receipts_for_day(&conn, day_id).unwrap();
        let receipt = get_receipt(&conn, receipt_id).unwrap();
        assert!(receipt.day_id.is_none());
        assert_eq!(receipt.memo.as_deref(), Some("keep"));
    }

    #[test]
    fn test_format_amount_optional_negative_decimal_currency_display() {
        let display = format_amount_optional(Some(-50), &Some("USD".to_string()));
        assert_eq!(display, "-0.50 USD");
        assert!(!display.contains("-0.500"));
    }

    #[test]
    fn test_migrate_receipts_simplified_schema_from_legacy() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE receipts (
                id INTEGER PRIMARY KEY,
                trip_id INTEGER NOT NULL,
                day_id INTEGER NULL,
                itinerary_id INTEGER NULL,
                linked_expense_id INTEGER NULL,
                amount INTEGER NULL,
                currency TEXT NULL,
                occurred_date TEXT NULL,
                memo TEXT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO receipts
             (id, trip_id, day_id, itinerary_id, linked_expense_id, amount, currency,
              occurred_date, memo, status, created_at, updated_at)
             VALUES (1, 1, NULL, 2, 3, 100, 'JPY', NULL, 'x', 'linked', 't', 't')",
            [],
        )
        .unwrap();
        migrate_receipts_simplified_schema(&conn).unwrap();
        assert!(!receipts_table_has_column(&conn, "itinerary_id").unwrap());
        let receipt = conn
            .query_row("SELECT status FROM receipts WHERE id = 1", [], |row| {
                row.get::<_, String>(0)
            })
            .unwrap();
        assert_eq!(receipt, RECEIPT_STATUS_UNREVIEWED);
    }
}
