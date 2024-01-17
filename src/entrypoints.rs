use std::{collections::HashSet, path::Path};

use fehler::throws;

pub struct Entrypoints(String);

impl Entrypoints {
    #[throws(anyhow::Error)]
    pub fn parse_file(entrypoints: impl AsRef<Path>) -> Self {
        Entrypoints(std::fs::read_to_string(entrypoints)?)
    }

    pub fn get(&self) -> HashSet<String> {
        self.0
            .lines()
            .map(|line| line.trim())
            .filter(|line| line.len() > 0 && !line.starts_with('#'))
            .map(ToString::to_string)
            .collect()
    }
}
