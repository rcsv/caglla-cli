use std::process::Command;

#[test]
fn cli_about_prints_english_overview() {
    let output = Command::new(env!("CARGO_BIN_EXE_caglla-cli"))
        .arg("--about")
        .output()
        .expect("failed to run CLI");

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Caglla.Travel CLI"));
    assert!(stdout.contains("local-first travel planning CLI"));
    assert!(stdout.contains("Itinerary is not a venue"));
    assert!(stdout.contains("caglla.db"));
    assert!(stdout.contains("License: MIT"));
}

#[test]
fn cli_about_does_not_require_subcommand() {
    let output = Command::new(env!("CARGO_BIN_EXE_caglla-cli"))
        .args(["--about", "trip", "list"])
        .output()
        .expect("failed to run CLI");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Caglla.Travel CLI"));
}
