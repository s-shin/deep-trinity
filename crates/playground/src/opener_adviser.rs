use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::str::FromStr;
use std::fmt::Write;
use once_cell::sync::Lazy;
use regex::Regex;
use deep_trinity_core::prelude::*;
use deep_trinity_grid::{X, Y};

#[derive(Clone, Debug)]
pub struct Constraint {
    pub label: Option<String>,
    pub piece: Piece,
    pub placement: Placement,
    pub is_end: bool,
    pub dependencies: Vec<String>,
}

impl FromStr for Constraint {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?x)
            (?P<piece>[SZLJITO]),
            (?P<orientation>[0-4]),
            (?P<x>-?\d+),
            (?P<y>-?\d+)
            (?:\s+\^(?P<dependencies>[a-zA-Z0-9,]+))?
            (?:\s+!(?P<end>END))?
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
            let is_end = captures.name("end").is_some();
            let label = if let Some(m) = captures.name("label") {
                Some(m.as_str().to_string())
            } else {
                None
            };
            Ok(Self {
                label,
                piece,
                placement: Placement::new(orientation, (x, y).into()),
                is_end,
                dependencies,
            })
        } else {
            Err("invalid node str".into())
        }
    }
}

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub constraint: Constraint,
    pub dependencies: Vec<NodeId>,
}

impl Node {
    fn new(id: usize, constraint: Constraint, dependencies: Vec<NodeId>) -> Self {
        Self { id, constraint, dependencies }
    }
    pub fn piece(&self) -> Piece { self.constraint.piece }
    pub fn placement(&self) -> Placement { self.constraint.placement }
    pub fn is_end(&self) -> bool { self.constraint.is_end }
}

#[derive(Default, Clone, Debug)]
pub struct Graph {
    nodes: Vec<Node>,
}

impl Graph {
    pub fn new(constraints: &[Constraint]) -> Self {
        let mut label_indices = HashMap::<String, Vec<usize>>::new();
        for (i, constraint) in constraints.iter().enumerate() {
            if let Some(label) = &constraint.label {
                if let Some(indices) = label_indices.get_mut(label) {
                    indices.push(i);
                } else {
                    label_indices.insert(label.clone(), vec![i]);
                }
            }
        }
        let mut graph = Graph::default();
        for (i, constraint) in constraints.iter().enumerate() {
            let deps = constraint.dependencies.iter()
                .map(|dep| label_indices.get(dep).unwrap())
                .fold(vec![], |mut acc, v| {
                    acc.extend(v);
                    acc
                });
            let n = Node::new(i, constraint.clone(), deps);
            graph.nodes.push(n);
        }
        graph
    }
    pub fn get_node(&self, id: NodeId) -> Option<&Node> { self.nodes.get(id) }
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> { self.nodes.get_mut(id) }
    pub fn to_plant_uml_string(&self) -> String {
        let mut s = String::new();
        writeln!(&mut s, "@startuml").unwrap();
        writeln!(&mut s, "hide empty description").unwrap();
        let mut end_nodes = self.nodes.iter().map(|n| n.id).collect::<HashSet<_>>();
        for node in self.nodes.iter() {
            writeln!(
                &mut s, r#"state "{} ({},{},{})" as S{}"#,
                node.piece(), node.placement().orientation,
                node.placement().pos.0, node.placement().pos.1,
                node.id,
            ).unwrap();
            if node.dependencies.is_empty() {
                writeln!(&mut s, "[*] --> S{}", node.id).unwrap();
            } else {
                for &i in node.dependencies.iter() {
                    end_nodes.remove(&i);
                    let dep = self.nodes.get(i).unwrap();
                    writeln!(&mut s, "S{} --> S{}", dep.id, node.id).unwrap();
                }
            }
            if node.is_end() {
                writeln!(&mut s, "S{} --> [*]", node.id).unwrap();
            }
        }
        writeln!(&mut s, "@enduml").unwrap();
        s
    }
}

pub struct Advisor<'a> {
    graph: &'a Graph,
    consumed: Vec<bool>,
}

impl<'a> Advisor<'a> {
    pub fn new(graph: &'a Graph) -> Self {
        Self { graph, consumed: vec![false; graph.nodes.len()] }
    }
    pub fn candidates(&self) -> Vec<&Node> {
        let mut r = Vec::new();
        for node in self.graph.nodes.iter() {
            if self.consumed[node.id] {
                continue;
            }
            if node.dependencies.iter().find(|&&dep_id| !self.consumed[dep_id]).is_none() {
                r.push(node);
            }
        }
        r
    }
    pub fn consume(&mut self, node_id: usize) {
        self.consumed[node_id] = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    #[ignore]
    fn test() {
        fn build(txt: &str) -> Graph {
            let nns = txt.split('\n')
                .map(str::trim)
                .map(|line| { line.split('#').next().unwrap_or("") })
                .filter(|s| !s.is_empty())
                .map(|s| Constraint::from_str(s).unwrap())
                .collect::<Vec<_>>();
            Graph::new(&nns)
        }

        // let graph = build(r"
        //     # TKI 1 (left-side, only hard drop)
        //     I,0,2,-2 ... I
        //     O,0,7,-1 ... O
        //     L,1,-1,0 ... L
        //     S,1,5,0 ^I ... S
        //     Z,0,3,0 ^I ... Z
        //     J,2,3,2 ^Z
        //     J,0,7,1 ^S,O
        //     T,2,1,0 ^I,O,L,S,Z !END
        // ");
        // println!("{}", graph.to_plant_uml_string());

        let graph = build(r"
            # Mountainous Stacking v2
            # base
            I,1,-2,0 ... I1
            J,0,1,-1 ... J1
            S,3,1,1 ^J1 ... S1
            Z,0,4,-1 ... Z1
            T,3,6,0 ... T1
            O,0,7,-1 ... O1
            S,3,1,3 ^S1 ... S2
            # pattern 1
            J,1,-1,4 ^I1
            O,0,1,4 ^S2 ... O2
            Z,0,4,1 ^Z1 ... Z2
            L,1,5,3 ^T1 ... L1
            L,3,7,2 ^L1,O1 ... L2
            I,1,7,2 ^O1 ... I2
            T,1,2,1 ^I1,J1,O2,Z2,L2,I2 !END
        ");
        println!("{}", graph.to_plant_uml_string());

        let mut rpg = RandomPieceGenerator::new(StdRng::seed_from_u64(0));
        let mut game = StdGame::default();
        game.supply_next_pieces(&rpg.generate());
        game.supply_next_pieces(&rpg.generate());
        game.supply_next_pieces(&rpg.generate());
        game.setup_falling_piece(None).unwrap();

        let mut adviser = Advisor::new(&graph);
        for i in 0..20 {
            let piece = game.state.falling_piece.as_ref().unwrap().piece();
            println!("{}: {}", i, piece);
            let candidates = adviser.candidates();
            println!("candidates: {:?}", candidates);
            if let Some(found) = candidates.iter().find(|n| n.piece() == piece).map(|&n| n.clone()) {
                println!("=> {:?}", &found);
                adviser.consume(found.id);
                if found.is_end() {
                    println!("=> end");
                    break;
                }
                game.state.falling_piece.as_mut().unwrap().placement = found.placement();
                game.lock().unwrap();
            } else if game.state.can_hold {
                println!("=> try hold");
                game.hold().unwrap();
            } else {
                println!("=> not found");
                break;
            }
        }
        println!("{}", &game);

        // println!("---");
        //
        // let graph = build(r"
        //     # TKI 1 (left-side, only hard drop)
        //     I,0,2,-1 ... I
        //     O,0,7,1
        //     L,1,-1,0
        //     S,1,5,0 [I]
        //     Z,0,3,0 [I] ... Z
        //     J,2,3,2 [Z]
        //     T,2,1,0 [Z]
        // ");
        // println!("{}", graph.to_plant_uml_string());
        //
        // println!("---");
        //
        // let graph = build(r"
        //     # TKI 2 (left-side, only hard drop)
        //     I,0,2,-1 ... I
        //     O,0,7,-1
        //     L,1,-1,0
        //     J,2,5,0 [I] ... J
        //     S,3,3,1 [I] ... S
        //     Z,0,4,1 [S,J]
        //     T,2,1,0 [S,J]
        // ");
        // println!("{}", graph.to_plant_uml_string());
    }
}
