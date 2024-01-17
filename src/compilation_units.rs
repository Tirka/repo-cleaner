use std::{collections::HashSet, path::PathBuf, str::FromStr};

use fehler::throws;

#[throws(anyhow::Error)]
pub fn collect_compilation_units(repo_root: PathBuf) -> HashSet<CompilationUnit> {
    let max_depth = 5;
    let mut compilation_units = HashSet::new();
    parse_units(&mut compilation_units, repo_root, max_depth, true)?;
    return compilation_units;
}

#[throws(anyhow::Error)]
fn parse_units(
    compilation_units: &mut HashSet<CompilationUnit>,
    path: PathBuf,
    max_depth: usize,
    is_root_call: bool,
) {
    let dir = std::fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;

    for entry in dir {
        let entry_meta = entry.metadata()?;

        if entry_meta.is_file() && entry.file_name() == "Cargo.toml" && !is_root_call {
            let unit = CompilationUnit::parse_manifest(entry.path()).unwrap();
            compilation_units.insert(unit);
        }

        if entry_meta.is_dir() && max_depth > 0 {
            parse_units(compilation_units, entry.path(), max_depth - 1, false)?;
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CompilationUnit {
    pub name: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub manifest: PathBuf,
}

impl CompilationUnit {
    #[throws(anyhow::Error)]
    fn parse_manifest(manifest_path: PathBuf) -> Self {
        let manifest = toml::Value::from_str(&std::fs::read_to_string(&manifest_path)?)?;
        let manifest = manifest.as_table().unwrap();

        let package = manifest.get("package").unwrap();

        let name = package.get("name").unwrap().as_str().unwrap();

        let description = package
            .get("description")
            .and_then(|description| description.as_str())
            .unwrap_or("<no description>");

        let dependencies = manifest
            .get("dependencies")
            .and_then(|d| d.as_table())
            .map(|deps| deps.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        Self {
            name: name.to_string(),
            description: description.to_string(),
            dependencies: dependencies,
            manifest: manifest_path,
        }
    }
}
