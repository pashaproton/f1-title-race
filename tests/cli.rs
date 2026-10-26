use std::process::Command;

#[test]
fn binary_runs_end_to_end() {
    let output = Command::new(env!("CARGO_BIN_EXE_f1-title-race"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("tests/fixtures/2026-golden.toml")
        .output()
        .expect("binary should launch");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("623"));
}
