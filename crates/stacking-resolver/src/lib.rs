use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::str::FromStr;
use core::prelude::*;
use tree::arena::{NodeArena, NodeHandle};

# TODO: Remove move-finder.

pub struct PiecePlacement {
    pub piece: Piece,
    pub placement: Placement,
}

impl PiecePlacement {
    fn new(piece: Piece, placement: Placement) -> Self {
        Self { piece, placement }
    }
}

impl FromStr for PiecePlacement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(",");
        let err_msg = "Invalid format.";
        let mut part0 = parts.next().ok_or::<Self::Err>(err_msg.into())?.chars();
        let part1 = parts.next().ok_or::<Self::Err>(err_msg.into())?;
        let part2 = parts.next().ok_or::<Self::Err>(err_msg.into())?;

        let piece = if let Some(c) = part0.next() {
            if let Ok(p) = Piece::try_from_char(c) {
                p
            } else {
                return Err(format!("'{}' is not piece character.", c).into());
            }
        } else {
            return Err("A piece character is required..".into());
        };
        let orientation = if let Some(c) = part0.next() {
            Orientation::from_str(c.to_string().as_str()).map_err(|e| e.to_string())?
        } else {
            return Err("An orientation value is required..".into());
        };

        let x = i8::from_str(part1).map_err(|_| Self::Err::from("Invalid x value."))?;
        let y = i8::from_str(part2).map_err(|_| Self::Err::from("invalid y value."))?;

        Ok(Self::new(piece, Placement::new(orientation, (x, y).into())))
    }
}

impl Display for PiecePlacement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{},{},{}",
               self.piece.to_char(),
               self.placement.orientation.to_u8(),
               self.placement.pos.0,
               self.placement.pos.1)
    }
}

//---

pub struct NodeData<'a> {
    by_action: Option<Action>,
    game: Game<'a>,
    remains_pps: Vec<Rc<PiecePlacement>>,
    mdr: MoveDecisionResource,
}

impl<'a> NodeData<'a> {
    pub fn new(by_action: Option<Action>, game: Game<'a>, pps: Vec<Rc<PiecePlacement>>) -> Result<Self, &'static str> {
        let mdr = MoveDecisionResource::with_game(&game)?;
        Ok(Self { by_action, game, remains_pps: pps, mdr })
    }
}

pub type VecNodeArena<'a> = tree::arena::VecNodeArena<NodeData<'a>>;

pub fn expand_node(arena: &mut VecNodeArena, node: NodeHandle) {
    if let Some(fp) = arena[node].data.game.state.falling_piece.clone() {
        let pps_len = arena[node].data.remains_pps.len();
        for i in 0..pps_len {
            let pp = arena[node].data.remains_pps.get(i).cloned().unwrap();
            let (game, pps) = {
                let data = &arena[node].data;
                if pp.piece != fp.piece() || !data.mdr.dst_candidates.contains(&pp.placement) {
                    continue;
                }
                let mut game = data.game.clone();
                game.state.falling_piece = Some(FallingPiece::new(fp.piece().default_spec(), pp.placement));
                game.lock().unwrap();
                let mut pps = data.remains_pps.clone();
                pps.remove(i);
                (game, pps)
            };
            if game.state.falling_piece.is_some() {
                arena.append_child(node, NodeData::new(
                    Some(Action::Move(MoveTransition::new(pp.placement.clone(), None))),
                    game,
                    pps,
                ).unwrap());
            }
        }
    }
    if arena[node].data.game.state.can_hold {
        let mut game = arena[node].data.game.clone();
        game.hold().unwrap();
        if game.state.falling_piece.is_some() {
            let pps = arena[node].data.remains_pps.clone();
            let child_data = NodeData::new(Some(Action::Hold), game, pps).unwrap();
            arena.append_child(node, child_data);
        }
    }
}

pub fn expand_all(arena: &mut VecNodeArena, node: NodeHandle) {
    let mut open = vec![node];
    while !open.is_empty() {
        let target = open.pop().unwrap();
        expand_node(arena, target);
        open.extend(arena[target].children());
    }
}

pub struct ResolveStackingResult<'a> {
    pub arena: VecNodeArena<'a>,
    pub root: NodeHandle,
}

impl<'a> ResolveStackingResult<'a> {
    pub fn write_tree(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        fn write(w: &mut impl std::io::Write, indent: &str, n: &tree::arena::Node<NodeData>) -> std::io::Result<()> {
            writeln!(w, "{}- by_action: {:?}", indent, n.data.by_action)?;
            writeln!(w, "{}  game: |-\n{}", indent, n.data.game.to_string().split("\n")
                .map(|line| format!("{}    {}", indent, line)).collect::<Vec<_>>().join("\n"))?;
            writeln!(w, "{}  children: {}", indent, if n.is_leaf() { "[]" } else { "" })
        }
        let mut r = Ok(());
        self.arena.visit_depth_first(self.root, |arena, node, ctx| {
            let indent = "  ".repeat(ctx.depth());
            let n = &arena[node];
            r = write(w, &indent, n);
            if r.is_err() {
                ctx.finish();
            }
        });
        r
    }
    pub fn collect_nodes_by_lock_count(&self, n: u32) -> Vec<NodeHandle> {
        let mut found = Vec::new();
        self.arena.visit_depth_first(self.root, |arena, node, ctx| {
            if arena[node].data.game.stats.lock == n {
                found.push(node);
                ctx.skip();
            }
        });
        found
    }
}

pub fn resolve_stacking(game: Game, pps: Vec<Rc<PiecePlacement>>) -> Result<ResolveStackingResult, &'static str> {
    let mut arena = VecNodeArena::default();
    let root = arena.create(NodeData::new(None, game, pps).unwrap());
    expand_all(&mut arena, root);
    Ok(ResolveStackingResult { arena, root })
}

//---

/*
I0,2,-2:C2
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stacking_resolver() {
        const NEXT_PIECES: &'static str = "ISZTOJLISZTOJL";
        // const PPS: &'static str = "I0,2,-2 O0,7,-1 L1,-1,0 S1,5,0 Z0,3,0 J2,3,2 T2,1,0";
        const PPS: &'static str = "I0,2,-2 O0,7,-1 L1,-1,0 S1,5,0 Z0,3,0 J2,3,2 T2,1,0";

        let next_pieces = NEXT_PIECES.chars().map(|c| Piece::try_from_char(c).unwrap()).collect::<Vec<_>>();
        // TODO: let mirror = false;
        let pps = PPS.split(" ")
            .map(|s| Rc::new(PiecePlacement::from_str(s).unwrap()))
            .collect::<Vec<_>>();
        let pps_len = pps.len();

        let mut initial_game = StdGame::default();
        initial_game.performance_mode();
        initial_game.supply_next_pieces(&next_pieces);
        initial_game.setup_falling_piece(None).unwrap();

        let r = resolve_stacking(initial_game.clone(), pps).unwrap();

        let debug_trace = false;
        if debug_trace {
            r.write_tree(&mut std::io::stdout()).unwrap();
        }

        println!("\n### Result");
        let found = r.collect_nodes_by_lock_count(pps_len as u32);
        for (i, &node) in found.iter().enumerate() {
            println!("--- {} ---", i);
            let route = r.arena.route(node);
            for n in route.iter() {
                let prev_game = r.arena[*n].parent().map_or(&initial_game, |pn| &r.arena[pn].data.game);
                let data = &r.arena[*n].data;
                if let Some(action) = data.by_action {
                    println!(
                        "[{}] {} => {:?}",
                        prev_game.state.hold_piece.map_or(' ', |p| p.to_char()),
                        prev_game.state.falling_piece.as_ref().map_or('?', |fp| fp.piece().to_char()),
                        action,
                    );
                }
            }
        }
    }
}
