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
    let dir = std::env::temp_dir().join(format!("caglla-cli-day-{n}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn setup_trip(dir: &std::path::Path) {
    assert!(run_cli(dir, &["db", "reset"]).status.success());
    assert!(run_cli(
        dir,
        &[
            "trip",
            "add",
            "Okinawa Family Trip",
            "--start",
            "2026-04-26",
            "--end",
            "2026-04-29",
        ],
    )
    .status
    .success());
}

#[test]
fn cli_day_list_shows_days_with_dates() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(&dir, &["day", "list", "1"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Trip: Okinawa Family Trip"));
    assert!(stdout.contains("Day 1  2026-04-26"));
    assert!(stdout.contains("Day 4  2026-04-29"));
}

#[test]
fn cli_day_show_lists_itineraries_for_day() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "09:00",
            "美ら海水族館",
        ],
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "13:00",
            "海邦丸"
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["day", "show", "1", "2"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Day 2"));
    assert!(stdout.contains("Date: 2026-04-27"));
    assert!(stdout.contains("Itineraries:"));
    assert!(stdout.contains("- 09:00 美ら海水族館"));
    assert!(stdout.contains("- 13:00 海邦丸"));
}

#[test]
fn cli_day_show_empty_day() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(&dir, &["day", "show", "1", "3"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Day 3"));
    assert!(stdout.contains("Date: 2026-04-28"));
    assert!(stdout.contains("Itineraries:"));
    assert!(!stdout.contains("- "));
}

#[test]
fn cli_day_show_rejects_invalid_day_number() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(&dir, &["day", "show", "1", "99"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Day not found: trip 1 day 99"));
}

#[test]
fn cli_day_swap_exchanges_itineraries() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "09:00",
            "Day2 Plan"
        ],
    )
    .status
    .success());
    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "3",
            "--time",
            "10:00",
            "Day3 Plan"
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["day", "swap", "1", "2", "3"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Day 2 と Day 3 の日程を入れ替えました"));

    let day2 = run_cli(&dir, &["day", "show", "1", "2"]);
    let day3 = run_cli(&dir, &["day", "show", "1", "3"]);
    assert!(String::from_utf8_lossy(&day2.stdout).contains("Day3 Plan"));
    assert!(String::from_utf8_lossy(&day3.stdout).contains("Day2 Plan"));
}

#[test]
fn cli_day_swap_rejects_same_day() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(
        run_cli(&dir, &["itinerary", "add", "1", "--day", "2", "Plan"],)
            .status
            .success()
    );

    let output = run_cli(&dir, &["day", "swap", "1", "2", "2"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("同じ Day"));
}

#[test]
fn cli_day_list_json() {
    let dir = temp_workdir();
    setup_trip(&dir);

    let output = run_cli(&dir, &["day", "list", "1", "--json"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["trip_name"], "Okinawa Family Trip");
    assert_eq!(parsed["days"].as_array().unwrap().len(), 4);
    assert_eq!(parsed["days"][1]["date"], "2026-04-27");
}

#[test]
fn cli_day_show_json() {
    let dir = temp_workdir();
    setup_trip(&dir);
    assert!(run_cli(
        &dir,
        &[
            "itinerary",
            "add",
            "1",
            "--day",
            "2",
            "--time",
            "09:00",
            "Museum"
        ],
    )
    .status
    .success());

    let output = run_cli(&dir, &["day", "show", "1", "2", "--json"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(parsed["day_number"], 2);
    assert_eq!(parsed["date"], "2026-04-27");
    assert_eq!(parsed["itineraries"].as_array().unwrap().len(), 1);
}
