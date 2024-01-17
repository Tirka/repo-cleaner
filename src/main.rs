mod compilation_units;
mod entrypoints;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::Context as _;
use fehler::throws;

use compilation_units::collect_compilation_units;
use itertools::Itertools;
use petgraph::{graph::DiGraph, visit::Dfs};

use crate::entrypoints::Entrypoints;

#[throws(anyhow::Error)]
fn main() {
    let velas_chain_root = std::env::args()
        .skip(1)
        .next()
        .context("provide path to `velas-chain` repo")?;

    let repo_root = PathBuf::from(&velas_chain_root);

    let compilation_units = collect_compilation_units(repo_root)?;

    let entrypoints = Entrypoints::parse_file("./entrypoints.txt")?.get();

    let mut dependency_graph: DiGraph<DependencyNode, u32> = Default::default();
    let mut node_indices = HashMap::new();
    let mut entrypoint_indices = HashSet::new();

    // 1) fill graph with nodes
    for cu in compilation_units.clone() {
        let is_entrypoint = entrypoints.contains(&cu.name);
        let node = DependencyNode::new(&cu.name, is_entrypoint);
        let node_idx = dependency_graph.add_node(node);
        node_indices.insert(cu.name, node_idx);

        if is_entrypoint {
            entrypoint_indices.insert(node_idx);
        }
    }

    // 2) connect nodes with edges
    for cu in compilation_units {
        let unit = node_indices.get(&cu.name).unwrap();
        for dep in cu
            .dependencies
            .into_iter()
            .chain(cu.dev_dependencies.into_iter())
        {
            if let Some(dep) = node_indices.get(&dep) {
                dependency_graph.add_edge(*unit, *dep, 1);
            }
        }
    }

    // 3) find dependency subgraphs for each entrypoint and mark its nodes as required
    for ep in entrypoint_indices {
        let mut discovered = Dfs::new(&dependency_graph, ep);
        while let Some(idx) = discovered.next(&dependency_graph) {
            dependency_graph[idx].required = true;
        }
    }

    // 4) find nodes which are not required
    let orphans = dependency_graph
        .node_weights()
        .filter(|node| !node.required)
        .sorted_by_key(|o| &o.name)
        .collect_vec();

    // 5) print crates to remove
    for o in orphans.into_iter() {
        println!("{}", o.name);
    }

    // serialize graph into graphviz format for later visualization
    // let graph_display = petgraph::dot::Dot::new(&dependency_graph);
    // println!("{}", graph_display)
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct DependencyNode {
    name: String,
    is_entry_point: bool,
    required: bool,
}

impl DependencyNode {
    pub fn new(name: &str, is_entry_point: bool) -> Self {
        Self {
            name: name.to_string(),
            is_entry_point,
            required: false,
        }
    }
}

// impl std::fmt::Display for DependencyNode {
//     #[throws(std::fmt::Error)]
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) {
//         write!(f, "{}", self.name)?;

//         if self.is_entry_point {
//             write!(f, " [entrypoint]")?;
//         }
//     }
// }
