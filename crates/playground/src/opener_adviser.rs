use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use once_cell::sync::Lazy;
use regex::Regex;
use core::prelude::*;
use grid::{X, Y};

struct Node {
    piece: Piece,
    placement: Placement,
    dependencies: Vec<usize>,
}

impl Node {
    fn new(piece: Piece, placement: Placement, dependencies: Vec<usize>) -> Self {
        Self { piece, placement, dependencies }
    }
}

#[derive(Default)]
struct Graph {
    nodes: Vec<Node>,
}

//---

struct NodeNotation {
    label: Option<String>,
    piece: Piece,
    placement: Placement,
    dependencies: Vec<String>,
}

impl FromStr for NodeNotation {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?x)
            (?P<piece>[SZLJITO]),
            (?P<orientation>[0-4]),
            (?P<x>-?\d+),
            (?P<y>-?\d+)
            (?:\s+\[(?P<dependencies>[a-zA-Z0-9,]+)\])?
            (?:\s+\.\.\.\s*(?P<label>[a-zA-Z0-9]+))?
        ").unwrap());

        if let Some(captures) = RE.captures(s) {
            let piece = Piece::from_str(captures.name("piece").unwrap().as_str())?;
            let orientation = Orientation::from_str(captures.name("orientation").unwrap().as_str())?;
            let x = X::from_str(captures.name("x").unwrap().as_str())?;
            let y = Y::from_str(captures.name("y").unwrap().as_str())?;
            let dependencies = if let Some(m) = captures.name("dependencies") {
                m.as_str().split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            } else {
                Vec::new()
            };
            let label = if let Some(m) = captures.name("label") {
                Some(m.as_str().to_string())
            } else {
                None
            };
            Ok(Self {
                label,
                piece,
                placement: Placement::new(orientation, (x, y).into()),
                dependencies,
            })
        } else {
            Err("invalid node str".into())
        }
    }
}

fn build_graph(nodes: &[NodeNotation]) -> Graph {
    let mut label_indices = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if let Some(label) = &node.label {
            label_indices.insert(label.clone(), i);
        }
    }
    let mut graph = Graph::default();
    for node in nodes.iter() {
        let deps = node.dependencies.iter()
            .map(|dep| label_indices.get(dep).copied().unwrap())
            .collect::<Vec<_>>();
        let n = Node::new(node.piece, node.placement, deps);
        graph.nodes.push(n);
    }
    graph
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        // I,0,2,-1 ... I
        // O,0,7,-1
        // L,1,-1,0
        // S,1,5,0 [I]
        // Z,0,3,0 [I] ... Z
        // J,2,3,2 [Z]
        // T,2,1,0 [Z]

        // I,0,2,-1 ... I
        // O,0,7,-1
        // L,1,-1,0
        // J,2,5,0 [I] ... J
        // S,3,3,1 [I] ... S
        // Z,0,4,1 [J,S]
        // T,2,1,0 [J,S]
    }
}
