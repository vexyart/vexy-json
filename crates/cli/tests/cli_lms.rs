// this_file: crates/cli/tests/cli_lms.rs

#![cfg(unix)]

use serde_json::Value;
use std::env;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::{tempdir, TempDir};

const PIPE_BYTES: usize = 65_536;
const FULL_BYTES: usize = 153_247;
const SENTINEL: &str = "sentinel:end";

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vexy-json"))
}

fn run_with_stdin(arguments: &[&str], input: &[u8]) -> Output {
    let mut child = cli()
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the CLI binary should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input)
        .expect("the test input should be written");
    child.wait_with_output().expect("the CLI should finish")
}

fn full_fixture() -> Vec<u8> {
    let mut json = Vec::with_capacity(FULL_BYTES);
    json.push(b'[');
    for _ in 0..32_767 {
        json.extend_from_slice(b"0,");
    }
    json.push(b'0');
    assert_eq!(
        json.len(),
        PIPE_BYTES,
        "the pipe prefix must end after a complete array item"
    );

    let sentinel = format!(",\"{SENTINEL}\"");
    while json.len() + 2 + sentinel.len() < FULL_BYTES {
        json.extend_from_slice(b",0");
    }
    json.extend_from_slice(sentinel.as_bytes());
    json.resize(FULL_BYTES - 1, b' ');
    json.push(b']');
    assert_eq!(
        json.len(),
        FULL_BYTES,
        "the regular-file fixture size drifted"
    );
    serde_json::from_slice::<Value>(&json).expect("the complete fixture must be valid JSON");
    json
}

struct FakeLms {
    _directory: TempDir,
    bin_directory: PathBuf,
    fixture_path: PathBuf,
}

impl FakeLms {
    fn regular_or_pipe() -> Self {
        Self::with_script(
            r#"#!/bin/sh
if [ "$#" -ne 3 ] || [ "$1" != "ls" ] || [ "$2" != "--json" ] || [ "$3" != "--llm" ]; then
  echo "unexpected argv" >&2
  exit 64
fi
if [ -f /dev/stdout ]; then
  cp "$FAKE_LMS_FIXTURE" /dev/stdout
else
  head -c 65536 "$FAKE_LMS_FIXTURE"
fi
"#,
        )
    }

    fn with_script(script: &str) -> Self {
        let directory = tempdir().expect("a fake-lms directory should be created");
        let bin_directory = directory.path().join("bin");
        fs::create_dir(&bin_directory).expect("the fake bin directory should be created");
        let executable = bin_directory.join("lms");
        fs::write(&executable, script).expect("the fake lms should be written");
        let mut permissions = fs::metadata(&executable)
            .expect("the fake lms metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).expect("the fake lms should be executable");

        let fixture_path = directory.path().join("fixture.json");
        fs::write(&fixture_path, full_fixture()).expect("the full fixture should be written");
        Self {
            _directory: directory,
            bin_directory,
            fixture_path,
        }
    }

    fn command(&self) -> Command {
        let mut command = cli();
        let inherited_path = env::var_os("PATH").unwrap_or_default();
        let mut paths = vec![self.bin_directory.clone()];
        paths.extend(env::split_paths(&inherited_path));
        command.env(
            "PATH",
            env::join_paths(paths).expect("the fake PATH should be valid"),
        );
        command.env("FAKE_LMS_FIXTURE", &self.fixture_path);
        command
    }

    fn piped_output(&self) -> Output {
        let mut command = Command::new(self.bin_directory.join("lms"));
        command
            .args(["ls", "--json", "--llm"])
            .env("FAKE_LMS_FIXTURE", &self.fixture_path);
        command.output().expect("the fake piped lms should run")
    }
}

fn assert_directory_empty(path: &Path) {
    let entries: Vec<_> = fs::read_dir(path)
        .expect("the temporary directory should be readable")
        .collect();
    assert!(
        entries.is_empty(),
        "the CLI must clean only and all of its lms temporary captures: {entries:?}"
    );
}

#[test]
fn repair_is_explicit_truthful_and_json_only_at_65536_bytes() {
    let fake = FakeLms::regular_or_pipe();
    let piped = fake.piped_output();
    assert!(
        piped.status.success(),
        "the fake pipe producer should succeed"
    );
    assert_eq!(
        piped.stdout.len(),
        PIPE_BYTES,
        "the fake pipe must reproduce the 65,536-byte truncation"
    );

    let without_repair = run_with_stdin(&["--compact"], &piped.stdout);
    assert!(
        !without_repair.status.success(),
        "unexpected EOF must fail without explicit --repair"
    );
    assert!(
        without_repair.stdout.is_empty(),
        "a failed parse must emit no stdout"
    );

    let repaired = run_with_stdin(&["--repair", "--compact"], &piped.stdout);
    assert!(repaired.status.success(), "safe EOF repair should succeed");
    let repaired_value: Value =
        serde_json::from_slice(&repaired.stdout).expect("repair stdout must contain only JSON");
    assert!(
        repaired_value.is_array(),
        "the repaired prefix should remain an array"
    );

    let stderr = String::from_utf8(repaired.stderr).expect("stderr should be UTF-8");
    for expected in [
        "unexpected EOF at byte 65536",
        "Repair actions:",
        "inserted closing ']'",
        "partial/lossy",
        "Missing upstream bytes were not recovered",
    ] {
        assert!(
            stderr.contains(expected),
            "repair stderr should contain {expected:?}; stderr was {stderr:?}"
        );
    }
}

#[test]
fn repair_refuses_incomplete_strings_and_invalid_utf8() {
    let incomplete_string = run_with_stdin(&["--repair", "--compact"], br#"["unfinished"#);
    assert!(!incomplete_string.status.success());
    assert!(incomplete_string.stdout.is_empty());
    let stderr = String::from_utf8(incomplete_string.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("inside a quoted string"),
        "stderr was {stderr:?}"
    );
    assert!(
        stderr.contains("fabricate string contents"),
        "stderr was {stderr:?}"
    );

    let invalid_utf8 = run_with_stdin(&["--repair", "--compact"], b"[0,\xff");
    assert!(!invalid_utf8.status.success());
    assert!(invalid_utf8.stdout.is_empty());
    let stderr = String::from_utf8(invalid_utf8.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("not valid UTF-8 at byte 3"),
        "stderr was {stderr:?}"
    );
}

#[test]
fn lms_capture_is_lossless_153247_bytes_semantic_and_clean() {
    let fake = FakeLms::regular_or_pipe();
    let capture_directory = tempdir().expect("a capture directory should be created");
    let output = fake
        .command()
        .args(["--lms", "--compact"])
        .env("TMPDIR", capture_directory.path())
        .output()
        .expect("the CLI lms capture should run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value =
        serde_json::from_slice(&output.stdout).expect("lms stdout must contain only valid JSON");
    let expected: Value = serde_json::from_slice(&full_fixture()).expect("fixture should parse");
    assert_eq!(
        actual, expected,
        "regular-file capture must be semantically lossless"
    );
    assert!(
        actual
            .as_array()
            .and_then(|values| values.last())
            .and_then(Value::as_str)
            == Some(SENTINEL),
        "the end-of-dataset sentinel must survive capture"
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("Captured 153247 bytes"),
        "stderr was {stderr:?}"
    );
    assert!(
        stderr.contains("regular temporary file"),
        "stderr was {stderr:?}"
    );
    assert_directory_empty(capture_directory.path());
}

#[test]
fn lms_capture_reports_missing_nonzero_and_invalid_children() {
    let missing_capture_directory = tempdir().expect("a capture directory should be created");
    let missing = cli()
        .arg("--lms")
        .env("PATH", "")
        .env("TMPDIR", missing_capture_directory.path())
        .output()
        .expect("the CLI should start even when lms is absent");
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());
    let stderr = String::from_utf8(missing.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("Failed to execute `lms ls --json --llm`"),
        "stderr was {stderr:?}"
    );
    assert_directory_empty(missing_capture_directory.path());

    let nonzero = FakeLms::with_script("#!/bin/sh\necho 'fake child diagnostic' >&2\nexit 23\n");
    let nonzero_capture_directory = tempdir().expect("a capture directory should be created");
    let output = nonzero
        .command()
        .arg("--lms")
        .env("TMPDIR", nonzero_capture_directory.path())
        .output()
        .expect("the nonzero child scenario should run");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("fake child diagnostic"),
        "stderr was {stderr:?}"
    );
    assert!(
        stderr.contains("exited unsuccessfully (23)"),
        "stderr was {stderr:?}"
    );
    assert_directory_empty(nonzero_capture_directory.path());

    let invalid = FakeLms::with_script("#!/bin/sh\nprintf '%s' '{\"key\":'\n");
    let invalid_capture_directory = tempdir().expect("a capture directory should be created");
    let output = invalid
        .command()
        .arg("--lms")
        .env("TMPDIR", invalid_capture_directory.path())
        .output()
        .expect("the invalid child scenario should run");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("<lms regular-file capture>"),
        "stderr was {stderr:?}"
    );
    assert_directory_empty(invalid_capture_directory.path());
}

#[test]
fn repair_rejects_partial_tokens_and_lms_rejects_other_input_modes() {
    let partial_number = run_with_stdin(&["--repair", "--compact"], b"[1e");
    assert!(!partial_number.status.success());
    assert!(partial_number.stdout.is_empty());
    let stderr = String::from_utf8(partial_number.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("final token is incomplete"),
        "stderr was {stderr:?}"
    );

    let conflict = cli()
        .args(["--lms", "--ndjson"])
        .output()
        .expect("clap should reject conflicting input modes");
    assert!(!conflict.status.success());
    assert!(conflict.stdout.is_empty());
    let stderr = String::from_utf8(conflict.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("cannot be used with"),
        "stderr was {stderr:?}"
    );
}

#[test]
fn lms_capture_detects_downstream_early_close_without_panicking() {
    let fake = FakeLms::regular_or_pipe();
    let mut child = fake
        .command()
        .args(["--lms", "--compact"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the CLI should start");
    drop(child.stdout.take());
    let output = child.wait_with_output().expect("the CLI should finish");

    assert!(!output.status.success(), "a closed downstream must fail");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(stderr.contains("broken pipe"), "stderr was {stderr:?}");
    assert!(!stderr.contains("panicked"), "stderr was {stderr:?}");
    assert!(
        !stderr.contains("Captured 153247 bytes"),
        "must not report completed output"
    );
}
