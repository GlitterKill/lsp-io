use std::process::Command;

#[test]
fn detect_reports_a_synthetic_polyglot_project() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname='demo'\n").unwrap();
    std::fs::write(dir.path().join("tailwind.config.ts"), "export default {}\n").unwrap();
    std::fs::write(dir.path().join("main.tf"), "terraform {}\n").unwrap();
    std::fs::write(dir.path().join("shader.wgsl"), "").unwrap();
    std::fs::write(dir.path().join("top.vhdl"), "").unwrap();
    std::fs::write(dir.path().join("theorem.lean"), "").unwrap();

    let workflows = dir.path().join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).unwrap();
    std::fs::write(workflows.join("ci.yml"), "name: ci\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_lsp-io"))
        .arg("detect")
        .arg(dir.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    for expected in [
        "Rust",
        "Tailwind CSS",
        "Terraform",
        "WGSL",
        "VHDL",
        "Lean 4",
        "GitHub Actions",
    ] {
        assert!(stdout.contains(expected), "missing {expected} in {stdout}");
    }
}
