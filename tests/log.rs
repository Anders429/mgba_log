//! Tests for logging in mGBA.
//!
//! For now, this needs to be run on nightly-2023-03-24. Not sure why it doesn't work for newer
//! versions.

use cargo_metadata::Message;
use mgba_logs::{Level, Record};
use std::{
    convert::AsRef,
    io::BufReader,
    path::Path,
    process::{Command, Stdio},
};

fn build_rom<P>(path: P) -> String
where
    P: AsRef<Path>,
{
    let mut command = Command::new("cargo")
        .args(["build", "--message-format=json-render-diagnostics"])
        .stdout(Stdio::piped())
        .current_dir(path)
        .spawn()
        .expect("failed to build rom");

    // Find the executable name.
    let reader = BufReader::new(command.stdout.as_mut().expect("failed to read stdout"));
    for message in Message::parse_stream(reader) {
        match message.expect("failed to obtain message from stdout") {
            Message::CompilerArtifact(artifact) => {
                if let Some(executable) = artifact.executable {
                    return executable.into();
                }
            }
            Message::BuildFinished(_) => {
                break;
            }
            _ => (), // Unknown message
        }
    }
    panic!("failed to find executable name")
}

fn execute_rom(rom: &str) -> Vec<Record> {
    let mut command = Command::new("cargo")
        .args(["run", rom])
        .stdout(Stdio::piped())
        .current_dir("tests/mgba_logs")
        .spawn()
        .expect("failed to run rom");

    serde_json::from_reader(command.stdout.as_mut().expect("failed to read stdout"))
        .expect("failed to deserialize output")
}

#[test]
fn debug() {
    // Build ROM.
    let rom = build_rom("tests/debug");

    // Execute ROM.
    let records = execute_rom(&rom);

    // Assert logs.
    assert!(records.contains(&Record {
        level: Level::Debug,
        message: "Hello, world!".to_owned(),
    }));
}
