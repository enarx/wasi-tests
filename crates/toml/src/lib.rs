use std::collections::{BTreeMap, HashSet};

use serde::{de::Error, Deserialize};
use wasmparser::{Chunk, Parser, Payload};

/// A test execution environment description.
#[derive(Deserialize, Default, Debug)]
pub struct Environment {
    /// The exit value that the test should produce.
    #[serde(default)]
    pub exit: i32,

    /// Environment variables to be set.
    #[serde(default)]
    pub vars: BTreeMap<String, String>,

    /// Arguments to be passed to the test.
    #[serde(default)]
    pub args: Vec<String>,

    /// The preopened directories the test expects.
    #[serde(default)]
    pub dirs: HashSet<String>,

    /// The stdin data to be passed to the test.
    #[serde(default)]
    pub stdin: String,

    /// The stdout data that the test should produce.
    #[serde(default)]
    pub stdout: String,

    /// The stderr data that the test should produce.
    #[serde(default)]
    pub stderr: String,
}

impl Environment {
    pub fn load(wasm: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(0);
        let mut data = &b""[..];
        let mut read = 0;

        loop {
            match parser.parse(&wasm[read..], true)? {
                Chunk::NeedMoreData(_) => Err(toml::de::Error::custom("invalid wasm"))?,
                Chunk::Parsed { consumed, payload } => {
                    read += consumed;

                    match payload {
                        Payload::CustomSection(section) if section.name() == ".test.data" => {
                            data = section.data();
                        }

                        Payload::End(..) => break,
                        _ => continue,
                    }
                }
            }
        }

        Ok(toml::from_slice(data)?)
    }
}
