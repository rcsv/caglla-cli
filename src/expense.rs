use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::models::{Expense, ExportExpenseV3};

const EXPENSE_SELECT_SQL: &str = "
    SELECT id, itinerary_id, title, amount, currency, paid_by_name, expense_date, note,
           sort_order, created_at, updated_at
    FROM expenses";

/// 通貨コードの形式を検証し、正規化した 3 文字コードを返す。
/// v1.x: 大文字化 + 英字 3 文字チェックのみ（未知コードは許可）。
pub(crate) fn validate_currency_code(code: &str) -> Result<String> {
    let trimmed = code.trim();
    if trimmed.len() != 3 {
        anyhow::bail!("currency は 3 文字である必要があります");
    }
    let normalized: String = trimmed.chars().map(|c| c.to_ascii_uppercase()).collect();
    if !normalized.chars().all(|c| c.is_ascii_alphabetic()) {
        anyhow::bail!("currency は英字 3 文字である必要があります");
    }
    Ok(normalized)
}

/// 通貨の最小通貨単位における小数桁数（ISO 4217 の主要通貨のみ。未知は 2）。
fn currency_minor_unit_digits(currency: &str) -> u32 {
    match currency {
        "BIF" | "CLP" | "DJF" | "GNF" | "ISK" | "JPY" | "KMF" | "KRW" | "PYG" | "RWF" | "UGX"
        | "UYI" | "VND" | "VUV" | "XAF" | "XOF" | "XPF" => 0,
        _ => 2,
    }
}

/// CLI 入力 amount を最小通貨単位の整数へ変換する（浮動小数点を使わない）。
pub(crate) fn parse_amount_for_currency(input: &str, currency: &str) -> Result<i64> {
    let currency = validate_currency_code(currency)?;
    let decimals = currency_minor_unit_digits(&currency);
    let s = input.trim();
    if s.is_empty() {
        anyhow::bail!("amount は必須です");
    }

    let (whole_part, frac_part) = match s.split_once('.') {
        Some((whole, frac)) => {
            if frac.contains('.') {
                anyhow::bail!("amount の形式が不正です");
            }
            (whole, Some(frac))
        }
        None => (s, None),
    };

    if !whole_part.is_empty() && !whole_part.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!("amount は数値である必要があります");
    }
    if let Some(frac) = frac_part {
        if !frac.chars().all(|c| c.is_ascii_digit()) {
            anyhow::bail!("amount は数値である必要があります");
        }
        if frac.len() > decimals as usize {
            anyhow::bail!("amount の小数桁が多すぎます（{currency} は {decimals} 桁まで）");
        }
    }

    let whole: i64 = if whole_part.is_empty() {
        0
    } else {
        whole_part
            .parse()
            .map_err(|_| anyhow::anyhow!("amount の形式が不正です"))?
    };

    let frac_val: i64 = match frac_part {
        None => 0,
        Some(frac) => {
            let padded = format!("{frac:0<width$}", width = decimals as usize);
            padded
                .parse()
                .map_err(|_| anyhow::anyhow!("amount の形式が不正です"))?
        }
    };

    let multiplier = 10_i64.pow(decimals);
    whole
        .checked_mul(multiplier)
        .and_then(|v| v.checked_add(frac_val))
        .ok_or_else(|| anyhow::anyhow!("amount が大きすぎます"))
}

fn validate_expense_date(value: &str) -> Result<()> {
    if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() {
        anyhow::bail!("expense_date は YYYY-MM-DD 形式である必要があります");
    }
    Ok(())
}

pub(crate) fn validate_expense_date_opt(value: &Option<String>) -> Result<()> {
    if let Some(v) = value.as_deref() {
        validate_expense_date(v)?;
    }
    Ok(())
}

fn format_integer_with_commas(value: i64) -> String {
    let negative = value < 0;
    let digits = value.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if negative {
        format!("-{out}")
    } else {
        out
    }
}

/// 金額の数値部分を表示用に整形する（通貨コードは含めない）
pub(crate) fn format_amount_value(amount: i64, currency: &str) -> String {
    let decimals = currency_minor_unit_digits(currency);
    if decimals == 0 {
        return format_integer_with_commas(amount);
    }
    let divisor = 10_i64.pow(decimals);
    let whole = amount / divisor;
    let frac = amount % divisor;
    format!(
        "{}.{:0width$}",
        format_integer_with_commas(whole),
        frac,
        width = decimals as usize
    )
}

pub(crate) fn format_amount_display(amount: i64, currency: &str) -> String {
    format!("{} {}", format_amount_value(amount, currency), currency)
}

/// Markdown export 用の Expense 1行
pub(crate) fn format_expense_markdown_line(exp: &Expense) -> String {
    let amount = format_amount_display(exp.amount, &exp.currency);
    match exp.title.as_deref() {
        Some(title) if !title.trim().is_empty() => format!("- {title}: {amount}"),
        _ => format!("- {amount}"),
    }
}

pub(crate) fn fmt_optional_text(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("-")
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ExpenseListJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itinerary_id: Option<i64>,
    pub expenses: Vec<Expense>,
}

pub(crate) fn resolve_expense_list_target(
    trip: Option<i64>,
    itinerary: Option<i64>,
) -> Result<ExpenseListTarget> {
    match (trip, itinerary) {
        (Some(trip_id), None) => Ok(ExpenseListTarget::Trip(trip_id)),
        (None, Some(itinerary_id)) => Ok(ExpenseListTarget::Itinerary(itinerary_id)),
        (Some(_), Some(_)) => {
            anyhow::bail!("--trip と --itinerary は同時に指定できません");
        }
        (None, None) => {
            anyhow::bail!("--trip または --itinerary のいずれかを指定してください");
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpenseListTarget {
    Trip(i64),
    Itinerary(i64),
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_expense(
    conn: &Connection,
    itinerary_id: i64,
    amount_input: &str,
    currency_input: &str,
    title: Option<&str>,
    note: Option<&str>,
    paid_by_name: Option<&str>,
    expense_date: Option<&str>,
) -> Result<i64> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    let currency = validate_currency_code(currency_input)?;
    let amount = parse_amount_for_currency(amount_input, &currency)?;
    if let Some(date) = expense_date {
        validate_expense_date(date)?;
    }

    let now = crate::db::now_string();
    conn.execute(
        "INSERT INTO expenses
         (itinerary_id, title, amount, currency, paid_by_name, expense_date, note,
          sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
        params![
            itinerary_id,
            title,
            amount,
            currency,
            paid_by_name,
            expense_date,
            note,
            &now,
            &now,
        ],
    )
    .context("Expense の追加に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

/// export schema v3 の Expense を import する（amount は最小通貨単位整数）
pub(crate) fn import_expense_v3(
    conn: &Connection,
    itinerary_id: i64,
    export: &ExportExpenseV3,
) -> Result<i64> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    let currency = validate_currency_code(&export.currency)?;
    validate_expense_date_opt(&export.expense_date)?;

    let now = crate::db::now_string();
    conn.execute(
        "INSERT INTO expenses
         (itinerary_id, title, amount, currency, paid_by_name, expense_date, note,
          sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            itinerary_id,
            export.title,
            export.amount,
            currency,
            export.paid_by_name,
            export.expense_date,
            export.note,
            export.sort_order,
            &now,
            &now,
        ],
    )
    .context("Expense の import に失敗しました")?;
    Ok(conn.last_insert_rowid())
}

pub(crate) fn list_expenses_for_itinerary(
    conn: &Connection,
    itinerary_id: i64,
) -> Result<Vec<Expense>> {
    crate::itinerary::get_itinerary_item(conn, itinerary_id)?;
    list_expenses_where(conn, "itinerary_id = ?1", params![itinerary_id])
}

pub(crate) fn list_expenses_for_trip(conn: &Connection, trip_id: i64) -> Result<Vec<Expense>> {
    crate::trip::get_trip(conn, trip_id)?;
    list_expenses_where(
        conn,
        "itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)",
        params![trip_id],
    )
}

fn list_expenses_where<P: rusqlite::Params>(
    conn: &Connection,
    where_clause: &str,
    params: P,
) -> Result<Vec<Expense>> {
    let sql = format!(
        "{EXPENSE_SELECT_SQL}
         WHERE {where_clause}
         ORDER BY itinerary_id ASC, sort_order ASC, id ASC"
    );
    let mut stmt = conn
        .prepare(&sql)
        .context("Expense 一覧取得の準備に失敗しました")?;
    let expenses = stmt
        .query_map(params, row_to_expense)
        .context("Expense 一覧取得に失敗しました")?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Expense 一覧の読み込みに失敗しました")?;
    Ok(expenses)
}

pub(crate) fn get_expense(conn: &Connection, id: i64) -> Result<Expense> {
    crate::db::map_query_row(
        conn.query_row(
            &format!("{EXPENSE_SELECT_SQL} WHERE id = ?1"),
            params![id],
            row_to_expense,
        ),
        || anyhow::anyhow!("Expense not found: {id}"),
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_expense(
    conn: &Connection,
    id: i64,
    title: Option<&str>,
    amount_input: Option<&str>,
    currency_input: Option<&str>,
    paid_by_name: Option<&str>,
    expense_date: Option<&str>,
    note: Option<&str>,
) -> Result<()> {
    if title.is_none()
        && amount_input.is_none()
        && currency_input.is_none()
        && paid_by_name.is_none()
        && expense_date.is_none()
        && note.is_none()
    {
        anyhow::bail!(
            "更新する項目を1つ以上指定してください (--title, --amount, --currency, --paid-by-name, --expense-date, --note)"
        );
    }

    let mut expense = get_expense(conn, id)?;
    if let Some(value) = title {
        expense.title = Some(value.to_string());
    }
    if let Some(value) = note {
        expense.note = Some(value.to_string());
    }
    if let Some(value) = paid_by_name {
        expense.paid_by_name = Some(value.to_string());
    }
    if let Some(value) = expense_date {
        validate_expense_date(value)?;
        expense.expense_date = Some(value.to_string());
    }

    let currency = match currency_input {
        Some(code) => validate_currency_code(code)?,
        None => expense.currency.clone(),
    };
    if let Some(input) = amount_input {
        expense.amount = parse_amount_for_currency(input, &currency)?;
    }
    if currency_input.is_some() {
        expense.currency = currency;
    }

    let now = crate::db::now_string();
    conn.execute(
        "UPDATE expenses
         SET title = ?1, amount = ?2, currency = ?3, paid_by_name = ?4, expense_date = ?5,
             note = ?6, updated_at = ?7
         WHERE id = ?8",
        params![
            expense.title,
            expense.amount,
            expense.currency,
            expense.paid_by_name,
            expense.expense_date,
            expense.note,
            &now,
            id,
        ],
    )
    .context("Expense の更新に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_expense(conn: &Connection, id: i64) -> Result<()> {
    get_expense(conn, id)?;
    conn.execute("DELETE FROM expenses WHERE id = ?1", params![id])
        .context("Expense の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_expenses_for_itinerary(conn: &Connection, itinerary_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM expenses WHERE itinerary_id = ?1",
        params![itinerary_id],
    )
    .context("Itinerary 配下 Expense の削除に失敗しました")?;
    Ok(())
}

pub(crate) fn delete_expenses_for_trip(conn: &Connection, trip_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM expenses
         WHERE itinerary_id IN (SELECT id FROM itinerary_items WHERE trip_id = ?1)",
        params![trip_id],
    )
    .context("Trip 配下 Expense の削除に失敗しました")?;
    Ok(())
}

fn row_to_expense(row: &rusqlite::Row) -> rusqlite::Result<Expense> {
    Ok(Expense {
        id: row.get(0)?,
        itinerary_id: row.get(1)?,
        title: row.get(2)?,
        amount: row.get(3)?,
        currency: row.get(4)?,
        paid_by_name: row.get(5)?,
        expense_date: row.get(6)?,
        note: row.get(7)?,
        sort_order: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

pub(crate) fn print_expense_list(target: ExpenseListTarget, expenses: &[Expense]) {
    let label = match target {
        ExpenseListTarget::Trip(id) => format!("Trip {id}"),
        ExpenseListTarget::Itinerary(id) => format!("Itinerary {id}"),
    };
    println!("{label} の Expense ({} 件):", expenses.len());
    if expenses.is_empty() {
        println!("  （なし）");
        return;
    }
    println!(
        "{:<4} {:<6} {:<16} {:<12} {:<10}",
        "ID", "Itin.", "Amount", "Title", "Paid By"
    );
    for expense in expenses {
        println!(
            "{:<4} {:<6} {:<16} {:<12} {:<10}",
            expense.id,
            expense.itinerary_id,
            format_amount_display(expense.amount, &expense.currency),
            fmt_optional_text(&expense.title),
            fmt_optional_text(&expense.paid_by_name),
        );
    }
}

pub(crate) fn print_expense_detail(expense: &Expense) {
    println!("Expense ID  : {}", expense.id);
    println!("Itinerary ID: {}", expense.itinerary_id);
    println!("Title       : {}", fmt_optional_text(&expense.title));
    println!(
        "Amount      : {}",
        format_amount_display(expense.amount, &expense.currency)
    );
    println!("Paid By     : {}", fmt_optional_text(&expense.paid_by_name));
    println!("Date        : {}", fmt_optional_text(&expense.expense_date));
    println!("Note        : {}", fmt_optional_text(&expense.note));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::reset_db;
    use crate::itinerary::add_itinerary_item;
    use crate::trip::add_test_trip;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        crate::db::open_db_at(":memory:").expect("インメモリ DB")
    }

    fn setup_itinerary(conn: &Connection) -> i64 {
        let trip_id = add_test_trip(conn, "Expense Trip").unwrap();
        add_itinerary_item(
            conn, trip_id, 1, "Lunch", None, None, None, None, None, None, None,
        )
        .unwrap()
    }

    #[test]
    fn test_validate_currency_code_normalizes_lowercase() {
        assert_eq!(validate_currency_code("jpy").unwrap(), "JPY");
        assert_eq!(validate_currency_code("Usd").unwrap(), "USD");
    }

    #[test]
    fn test_validate_currency_code_allows_unknown_codes() {
        assert_eq!(validate_currency_code("XXX").unwrap(), "XXX");
    }

    #[test]
    fn test_validate_currency_code_rejects_invalid_format() {
        assert!(validate_currency_code("JP").is_err());
        assert!(validate_currency_code("JPYY").is_err());
        assert!(validate_currency_code("JP1").is_err());
    }

    #[test]
    fn test_parse_amount_jpy_integer() {
        assert_eq!(parse_amount_for_currency("1500", "JPY").unwrap(), 1500);
    }

    #[test]
    fn test_parse_amount_usd_decimal() {
        assert_eq!(parse_amount_for_currency("12.50", "USD").unwrap(), 1250);
        assert_eq!(parse_amount_for_currency("12.5", "USD").unwrap(), 1250);
    }

    #[test]
    fn test_parse_amount_rejects_too_many_decimals_for_jpy() {
        assert!(parse_amount_for_currency("10.5", "JPY").is_err());
    }

    #[test]
    fn test_add_list_show_update_delete_expense() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);

        let id = add_expense(
            &conn,
            itinerary_id,
            "2200",
            "JPY",
            Some("Lunch"),
            None,
            Some("Tomo"),
            Some("2026-04-27"),
        )
        .unwrap();

        let listed = list_expenses_for_itinerary(&conn, itinerary_id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].title.as_deref(), Some("Lunch"));

        let expense = get_expense(&conn, id).unwrap();
        assert_eq!(expense.amount, 2200);
        assert_eq!(expense.currency, "JPY");

        update_expense(
            &conn,
            id,
            None,
            Some("2500"),
            None,
            None,
            None,
            Some("Updated"),
        )
        .unwrap();
        let updated = get_expense(&conn, id).unwrap();
        assert_eq!(updated.amount, 2500);
        assert_eq!(updated.note.as_deref(), Some("Updated"));

        delete_expense(&conn, id).unwrap();
        assert!(get_expense(&conn, id).is_err());
    }

    #[test]
    fn test_list_expenses_for_trip() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(&conn, itinerary_id, "100", "JPY", None, None, None, None).unwrap();

        let trip_expenses = list_expenses_for_trip(&conn, 1).unwrap();
        assert_eq!(trip_expenses.len(), 1);
    }

    #[test]
    fn test_delete_expenses_for_itinerary_cascade() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(&conn, itinerary_id, "500", "JPY", None, None, None, None).unwrap();

        delete_expenses_for_itinerary(&conn, itinerary_id).unwrap();
        assert!(list_expenses_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_delete_expenses_for_trip_cascade() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(&conn, itinerary_id, "500", "JPY", None, None, None, None).unwrap();

        delete_expenses_for_trip(&conn, 1).unwrap();
        assert!(list_expenses_for_itinerary(&conn, itinerary_id)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn test_itinerary_delete_cascades_expenses() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(&conn, itinerary_id, "500", "JPY", None, None, None, None).unwrap();

        crate::itinerary::delete_itinerary_item(&conn, itinerary_id).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM expenses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_trip_delete_cascades_expenses() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(&conn, itinerary_id, "500", "JPY", None, None, None, None).unwrap();

        crate::trip::delete_trip(&conn, 1).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM expenses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_expense_list_json_roundtrip() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(
            &conn,
            itinerary_id,
            "12.50",
            "USD",
            Some("Coffee"),
            None,
            None,
            None,
        )
        .unwrap();

        let expenses = list_expenses_for_itinerary(&conn, itinerary_id).unwrap();
        let json = serde_json::to_string_pretty(&ExpenseListJson {
            trip_id: None,
            itinerary_id: Some(itinerary_id),
            expenses,
        })
        .unwrap();
        let parsed: ExpenseListJson = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.expenses[0].amount, 1250);
        assert_eq!(parsed.expenses[0].currency, "USD");
    }

    #[test]
    fn test_reset_db_clears_expenses() {
        let conn = test_db();
        let itinerary_id = setup_itinerary(&conn);
        add_expense(&conn, itinerary_id, "100", "JPY", None, None, None, None).unwrap();

        reset_db(&conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM expenses", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
