//! Output locations and observed environment for the standalone benchmarks.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Run {
    pub id: String,
    pub output_dir: PathBuf,
    pub timestamp_unix_seconds: u64,
    pub rustc: String,
    pub cargo: String,
}

impl Run {
    pub fn new(game: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch");
        let id = format!("run-{}-{}", timestamp.as_nanos(), std::process::id());
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("game crate must belong to the workspace");
        let output_dir = workspace.join("target/benchmarks").join(game).join(&id);

        Self {
            id,
            output_dir,
            timestamp_unix_seconds: timestamp.as_secs(),
            rustc: tool_version("rustc"),
            cargo: tool_version("cargo"),
        }
    }
}

fn tool_version(tool: &str) -> String {
    let output = Command::new(tool)
        .arg("--version")
        .output()
        .expect("Rust toolchain version command must be available");
    assert!(output.status.success(), "{tool} --version failed");
    String::from_utf8(output.stdout)
        .expect("toolchain version must be UTF-8")
        .trim()
        .to_owned()
}
