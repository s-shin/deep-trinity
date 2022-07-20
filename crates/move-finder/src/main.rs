use std::ops::Deref;
use std::process::exit;
use std::rc::Rc;
use std::str::FromStr;
use clap::Parser;
use rand::prelude::*;
use core::prelude::*;
use bot::Action;
use grid::Grid;
use tree::arena::{NodeArena, NodeHandle};

struct NodeData {
    by_action: Option<Action>,
    game: Game,
    pps: Vec<Rc<PiecePlacement>>,
    resource: MoveDecisionResource,
}

impl NodeData {
    pub fn new(by_action: Option<Action>, game: Game, pps: Vec<Rc<PiecePlacement>>) -> Result<Self, &'static str> {
        let resource = MoveDecisionResource::with_game(&game)?;
        Ok(Self { by_action, game, pps, resource })
    }
}

type VecNodeArena = tree::arena::VecNodeArena<NodeData>;

fn expand_node(arena: &mut VecNodeArena, node: NodeHandle) {
    if let Some(fp) = arena[node].data.game.state.falling_piece.clone() {
        let pps_len = arena[node].data.pps.len();
        for i in 0..pps_len {
            let pp = arena[node].data.pps.get(i).cloned().unwrap();
            let (game, pps) = {
                let data = &arena[node].data;
                if pp.piece != fp.piece() || !data.resource.dst_candidates.contains(&pp.placement) {
                    continue;
                }
                let mut game = data.game.clone();
                game.state.falling_piece = Some(FallingPiece::new(fp.piece().default_spec(), pp.placement));
                game.lock().unwrap();
                let mut pps = data.pps.clone();
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
            let pps = arena[node].data.pps.clone();
            let child_data = NodeData::new(Some(Action::Hold), game, pps).unwrap();
            arena.append_child(node, child_data);
        }
    }
}

// macro_rules! pp {
//     ($piece_name:ident, $orientation:literal, $x:literal, $y:literal) => {
//         PiecePlacement::new(
//             core::Piece::$piece_name,
//             Placement::new(
//                 core::ORIENTATIONS[$orientation],
//                 grid::Vec2($x, $y),
//             ),
//         )
//     }
// }

#[derive(Copy, Clone, Debug)]
struct PiecePlacement {
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
            if let Ok(p) = Piece::from_char(c) {
                p
            } else {
                return Err(format!("'{}' is not piece character.", c).into());
            }
        } else {
            return Err("A piece character is required..".into());
        };
        let orientation = if let Some(c) = part0.next() {
            if let Ok(n) = u8::from_str(c.to_string().as_str()) {
                Orientation::new(n)
            } else {
                return Err(format!("'{}' is invalid orientation value.", c).into());
            }
        } else {
            return Err("An orientation value is required..".into());
        };

        let x = i8::from_str(part1).map_err(|_| Self::Err::from("Invalid x value."))?;
        let y = i8::from_str(part2).map_err(|_| Self::Err::from("invalid y value."))?;

        Ok(Self::new(piece, Placement::new(orientation, (x, y).into())))
    }
}

#[derive(Debug)]
struct PieceList(Vec<Piece>);

impl FromStr for PieceList {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut r = Vec::with_capacity(s.len());
        for c in s.chars() {
            r.push(Piece::from_char(c)?);
        }
        Ok(PieceList(r))
    }
}

impl Deref for PieceList {
    type Target = Vec<Piece>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 's', long, default_value = "0")]
    random_seed: u64,
    #[clap(short, long, default_value = "")]
    pieces: PieceList,
    #[clap(long)]
    debug: bool,
    positions: Vec<PiecePlacement>,
}

fn main() {
    let args: Args = Args::parse();
    if args.positions.is_empty() {
        println!("ERROR: At least one position is required.");
        exit(1);
    }

    let debug_trace = args.debug;

    let pps = args.positions.iter().map(|&pp| Rc::new(pp)).collect::<Vec<_>>();
    let pps_len = pps.len();
    println!("### Positions");
    {
        let mut game: Game = Default::default();
        for ps in pps.iter() {
            println!("{} {} {}", ps.piece.char(), ps.placement.orientation.id(), ps.placement.pos);
            game.state.playfield.grid.put_fast(ps.placement.pos, game.piece_specs.get(ps.piece).grid(ps.placement.orientation));
        }
        println!("Try to find:\n{}", game);
    }

    let mut initial_game: Game = Default::default();
    initial_game.state.playfield.grid.disable_basic_grid();
    initial_game.supply_next_pieces(args.pieces.as_slice());
    {
        let mut n = args.pieces.len();
        if n < pps_len {
            let mut rpg = RandomPieceGenerator::new(StdRng::seed_from_u64(args.random_seed));
            while n < pps_len {
                let pieces = rpg.generate();
                initial_game.supply_next_pieces(&pieces);
                n += pieces.len();
            }
        }
    }
    initial_game.setup_falling_piece(None).unwrap();
    println!("\n### Initial Game\n{}", initial_game);

    let mut arena = VecNodeArena::default();
    let root = arena.create(NodeData::new(None, initial_game.clone(), pps).unwrap());

    let mut open = vec![root];
    while !open.is_empty() {
        let target = open.pop().unwrap();
        expand_node(&mut arena, target);
        open.extend(arena[target].children());
    }
    if debug_trace {
        arena.visit_depth_first(root, |arena, node, ctx| {
            let indent = "  ".repeat(ctx.depth);
            let n = &arena[node];
            println!("{}- by_action: {:?}", indent, n.data.by_action);
            println!("{}  game: |-\n{}", indent, n.data.game.to_string().split("\n")
                .map(|line| format!("{}    {}", indent, line)).collect::<Vec<_>>().join("\n"));
            println!("{}  children: {}", indent, if n.is_leaf() { "[]" } else { "" });
        });
    }

    println!("\n### Result");

    let mut found = Vec::new();
    arena.visit_depth_first(root, |arena, node, ctx| {
        if arena[node].data.game.stats.lock == pps_len as u32 {
            found.push(arena.route(node));
            ctx.skip();
        }
    });

    for (i, route) in found.iter().enumerate() {
        println!("--- {} ---", i);
        for n in route.iter() {
            let prev_game = arena[*n].parent().map_or(&initial_game, |pn| &arena[pn].data.game);
            let data = &arena[*n].data;
            if let Some(action) = data.by_action {
                println!(
                    "[{}] {} => {:?}",
                    prev_game.state.hold_piece.map_or(' ', |p| p.char()),
                    prev_game.state.falling_piece.as_ref().map_or('?', |fp| fp.piece().char()),
                    action,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::io::Write;
    use assert_cmd::Command;

    #[test]
    fn basic() {
        let r = Command::cargo_bin("move-finder")
            .unwrap()
            .args("-p ISZTOJLISZTOJL I0,2,-2 O0,7,-1 L1,-1,0 S1,5,0 Z0,3,0 J2,3,2 T2,1,0".split(" ").collect::<Vec<_>>())
            .assert()
            .success();
        io::stdout().write_all(&r.get_output().stdout).unwrap()
    }
}
