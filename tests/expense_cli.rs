use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run_cli(cwd: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_caglla-cli"))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run CLI")
}

fn temp_workdir() -> std::path::PathBuf {
    let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("caglla-cli-expense-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn setup_trip_with_itinerary(dir: &std::path::Path) {
    assert!(run_cli(dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        dir,
        &[
            "trip",
            "add",
            "Expense Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ],
    )
    .status
    .success());
    assert!(
        run_cli(dir, &["itinerary", "add", "1", "--day", "1", "Lunch"],)
            .status
            .success()
    );
}

#[test]
fn cli_expense_add_and_show() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

    let output = run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "2200",
            "--currency",
            "JPY",
            "--title",
            "Lunch",
            "--paid-by-name",
            "Tomo",
            "--expense-date",
            "2026-04-27",
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Expense を追加しました"));
    assert!(stdout.contains("2,200 JPY"));
    assert!(stdout.contains("Tomo"));
    assert!(stdout.contains("2026-04-27"));

    let show = run_cli(&dir, &["expense", "show", "1"]);
    assert!(show.status.success());
    let show_stdout = String::from_utf8_lossy(&show.stdout);
    assert!(show_stdout.contains("Lunch"));
}

#[test]
fn cli_expense_add_usd_decimal() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

    let output = run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "12.50",
            "--currency",
            "usd",
            "--title",
            "Coffee",
        ],
    );
    assert!(output.status.success());
    let show: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["expense", "show", "1", "--json"]).stdout).unwrap();
    assert_eq!(show["amount"], 1250);
    assert_eq!(show["currency"], "USD");
}

#[test]
fn cli_expense_list_by_itinerary_and_trip() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "100",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let by_itinerary = run_cli(&dir, &["expense", "list", "--itinerary", "1"]);
    assert!(by_itinerary.status.success());
    assert!(String::from_utf8_lossy(&by_itinerary.stdout).contains("100 JPY"));

    let by_trip = run_cli(&dir, &["expense", "list", "--trip", "1"]);
    assert!(by_trip.status.success());
    assert!(String::from_utf8_lossy(&by_trip.stdout).contains("100 JPY"));
}

#[test]
fn cli_expense_update_and_delete() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "500",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let update = run_cli(
        &dir,
        &[
            "expense", "update", "1", "--amount", "600", "--note", "updated",
        ],
    );
    assert!(update.status.success());
    let show: serde_json::Value =
        serde_json::from_slice(&run_cli(&dir, &["expense", "show", "1", "--json"]).stdout).unwrap();
    assert_eq!(show["amount"], 600);
    assert_eq!(show["note"], "updated");

    let delete = run_cli(&dir, &["expense", "delete", "1"]);
    assert!(delete.status.success());
    assert!(!run_cli(&dir, &["expense", "show", "1"]).status.success());
}

#[test]
fn cli_expense_list_json() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "100",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["expense", "list", "--trip", "1", "--json"]);
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(parsed["trip_id"], 1);
    assert_eq!(parsed["expenses"].as_array().unwrap().len(), 1);
}

#[test]
fn cli_expense_rejects_invalid_currency() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);

    let output = run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "100",
            "--currency",
            "JP",
        ],
    );
    assert!(!output.status.success());
}

#[test]
fn cli_expense_cascade_on_itinerary_delete() {
    let dir = temp_workdir();
    setup_trip_with_itinerary(&dir);
    assert!(run_cli(
        &dir,
        &[
            "expense",
            "add",
            "--itinerary",
            "1",
            "--amount",
            "100",
            "--currency",
            "JPY",
        ],
    )
    .status
    .success());

    assert!(run_cli(&dir, &["itinerary", "delete", "1"])
        .status
        .success());
    let list = run_cli(&dir, &["expense", "list", "--trip", "1"]);
    assert!(list.status.success());
    assert!(String::from_utf8_lossy(&list.stdout).contains("（なし）"));
}
