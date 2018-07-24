use std;

use super::bot::*;
use super::prelude::*;

use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Trace {
    pub cmds: Vec<Cmd>,
}

impl std::fmt::Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "--start--").unwrap();
        for cmd in &self.cmds {
            writeln!(f, "{:?}", cmd).unwrap();
        }
        writeln!(f, "--end--")
    }
}

impl Trace {
    pub fn write_to(&self, path: impl AsRef<Path>) -> Result<()> {
        std::fs::write(path, self.encode())?;
        Ok(())
    }

    pub fn write_to_trace_dir(&self, filename: &str) -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(&format!("contest/trace/{}", filename));
        self.write_to(path)
    }

    pub fn write_to_submit_dir(&self, name: &str) -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(&format!("contest/submit/{}.nbt", name));
        info!("Writing best trace to: {}", path.display());
        self.write_to(path)
    }

    fn encode(&self) -> Vec<u8> {
        self.cmds.iter().fold(vec![], |mut acc, cmd| {
            acc.extend(Vec::<u8>::from(*cmd));
            acc
        })
    }
}
