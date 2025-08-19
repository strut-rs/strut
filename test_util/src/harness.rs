use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub struct Harness;

#[allow(dead_code)]
impl Harness {
    /// Runs (`cargo run`) a binary crate at the given path, expecting success.
    pub fn pass(path: &str) -> Output {
        Self::run(path, true, &[])
    }

    /// Runs (`cargo run`) a binary crate at the given path, expecting failure.
    pub fn fail(path: &str) -> Output {
        Self::run(path, false, &[])
    }

    /// Runs (`cargo run`) a binary crate at the given path, with environment
    /// variables, expecting success.
    pub fn pass_with_env(path: &str, envs: &[(&str, &str)]) -> Output {
        Self::run(path, true, envs)
    }

    /// Runs (`cargo run`) a binary crate at the given path, with environment
    /// variables, expecting failure.
    pub fn fail_with_env(path: &str, envs: &[(&str, &str)]) -> Output {
        Self::run(path, false, envs)
    }

    /// Captures the `stdout` and `stderr` of running (`cargo run`) a binary
    /// crate at the given path, and dumps the captured text into the
    /// correspondingly named files next to the crate’s `Cargo.toml`.
    pub fn dump_output(path: &str) {
        let (crate_dir, output) = Self::run_cargo(path, &[]);

        Self::write_stream(&output.stdout, &crate_dir.join("stdout"), "stdout");
        Self::write_stream(&output.stderr, &crate_dir.join("stderr"), "stderr");

        println!("✅ Output for '{}' dumped successfully.", path);
    }

    /// Runs the test crate, checks its exit status, and compares its output if
    /// expectation files exist.
    fn run(path: &str, expect_success: bool, envs: &[(&str, &str)]) -> Output {
        let (crate_dir, output) = Self::run_cargo(path, envs);

        // Assert the exit status.
        if expect_success {
            assert!(
                output.status.success(),
                "Expected '{}' to succeed, but it failed.\nStderr: {}",
                path,
                String::from_utf8_lossy(&output.stderr),
            );
        } else {
            assert!(
                !output.status.success(),
                "Expected '{}' to fail, but it succeeded.",
                path,
            );
        }

        // Check output streams against expectation files, if they exist.
        Self::check_stream(&output.stdout, &crate_dir.join("stdout"), "stdout");
        Self::check_stream(&output.stderr, &crate_dir.join("stderr"), "stderr");

        output
    }

    /// A centralized helper to execute `cargo run` for a specific test crate.
    fn run_cargo(path: &str, envs: &[(&str, &str)]) -> (PathBuf, Output) {
        let crate_dir = PathBuf::from("tests").join(path);
        let manifest_path = crate_dir.join("Cargo.toml");

        assert!(
            manifest_path.exists(),
            "Manifest path does not exist: {}",
            manifest_path.display(),
        );

        let env_map = envs.iter().cloned().collect::<HashMap<_, _>>();

        let output = Command::new("cargo")
            .arg("run")
            .arg("--quiet")
            .arg("--manifest-path")
            .arg(&manifest_path)
            .env("NO_COLOR", "1")
            .envs(&env_map)
            .output()
            .expect("Failed to execute `cargo run` command");

        (crate_dir, output)
    }

    /// Compares actual output with the content of an expectation file, if it
    /// exists.
    fn check_stream(actual_output: &[u8], file_path: &Path, stream_name: &str) {
        if file_path.exists() {
            let expected_output = fs::read_to_string(file_path).unwrap_or_else(|e| {
                panic!(
                    "Failed to read expected {} file at {}: {}",
                    stream_name,
                    file_path.display(),
                    e,
                )
            });
            let actual_output_str = String::from_utf8_lossy(actual_output);

            assert_eq!(
                expected_output,
                actual_output_str,
                "{} mismatch for {}",
                stream_name.to_uppercase(),
                file_path.display(),
            );
        }
    }

    /// Writes the given output to a specified file.
    fn write_stream(output: &[u8], file_path: &Path, stream_name: &str) {
        fs::write(file_path, output).unwrap_or_else(|e| {
            panic!(
                "Failed to write {} to {}: {}",
                stream_name,
                file_path.display(),
                e,
            )
        });
    }
}
