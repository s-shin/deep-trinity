use core::{Game, FallingPiece, MoveTransition, Piece};
use std::error::Error;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Action {
    Move(MoveTransition),
    Hold,
}

type NodeId = u64;

#[derive(Debug, Clone)]
struct NodeData {
    game: Game,
    current_piece: Piece,
    move_candidates: HashSet<MoveTransition>,
}

impl NodeData {
    fn new(game: Game) -> Self {
        let current_piece = game.state.falling_piece.as_ref().unwrap().piece;
        let move_candidates = game.get_move_candidates().unwrap();
        Self { game, current_piece, move_candidates }
    }
}

#[derive(Debug, Default)]
struct NodeDataRegistry {
    last_node_id: NodeId,
    nodes: HashMap<NodeId, NodeData>,
}

impl NodeDataRegistry {
    fn get(&mut self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(&id)
    }
    fn register(&mut self, data: NodeData) -> NodeId {
        self.last_node_id += 1;
        let id = self.last_node_id;
        self.nodes.insert(id, data);
        id
    }
}

#[derive(Debug, Clone)]
struct Node {
    id: NodeId,
    parent: Option<NodeId>,
    children: HashMap<Action, NodeId>,
}

impl Node {
    fn new(id: NodeId, parent: Option<NodeId>) -> Self {
        Self {
            id,
            parent,
            children: HashMap::new(),
        }
    }
    fn is_root(&self) -> bool { self.parent.is_none() }
    fn expand(&mut self, reg: &mut NodeDataRegistry) {
        let data = reg.get(self.id).unwrap().clone();
        for mt in data.move_candidates.iter() {
            let mut game = data.game.clone();
            game.state.falling_piece = Some(FallingPiece::new_with_last_move_transition(data.current_piece, mt));
            game.lock().unwrap();
            let child = Node::new(reg.register(NodeData::new(game)), Some(self.id));
            self.children.insert(Action::Move(*mt), child.id);
        }
        if data.game.state.can_hold {
            let mut game = data.game.clone();
            game.hold().unwrap();
            let child = Node::new(reg.register(NodeData::new(game)), Some(self.id));
            self.children.insert(Action::Hold, child.id);
        }
    }
}

fn find_tspin(game: &Game) -> Result<(), Box<dyn Error>> {
    let mut reg: NodeDataRegistry = Default::default();
    let mut root = Node::new(reg.register(NodeData::new(game.clone())), None);
    root.expand(&mut reg);
    println!("{:?}", root);

    // let candidates = game.get_move_candidates()?;
    // let piece = game.state.falling_piece.as_ref().unwrap().piece;
    //
    // for mt in candidates.iter() {
    //     let mut next_game = game.clone();
    //     next_game.state.falling_piece = Some(FallingPiece::new_with_last_move_transition(piece, *mt));
    //     next_game.lock()?;
    //     find_tspin(&next_game);
    // }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use core::{RandomPieceGenerator};

    #[test]
    fn test() {
        let mut game: Game = Default::default();
        let mut pg = RandomPieceGenerator::new(StdRng::seed_from_u64(0));
        game.supply_next_pieces(&pg.generate());
        game.setup_falling_piece(None).unwrap();
        find_tspin(&game).unwrap();
    }
}
