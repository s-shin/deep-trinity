use deep_trinity_core::{Game, RandomPieceGenerator, MovePlayer};
use rand::prelude::StdRng;
use rand::SeedableRng;
use anyhow::Result;
use std::io::{stdout, Write};
use deep_trinity_bot::{Bot, Action};
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();

    let mut game: Game = Default::default();
    let mut pg = RandomPieceGenerator::new(StdRng::seed_from_u64(0));
    game.supply_next_pieces(&pg.generate());
    game.setup_falling_piece(None)?;

    let mut bot: deep_trinity_bot::tree::TreeBot = Default::default();
    // let mut bot: deep_trinity_bot::simple_tree::SimpleTreeBot = Default::default();
    // let mut bot: deep_trinity_bot::simple::SimpleBot = Default::default();
    // let mut bot: deep_trinity_bot::mcts_puct::MctsPuctBot = Default::default();
    let mut action = None;
    let mut move_player = None;
    loop {
        if game.should_supply_next_pieces() {
            game.supply_next_pieces(&pg.generate());
        }
        if action.is_none() {
            action = Some(bot.think(&game)?);
        }
        match action.unwrap() {
            Action::Move(mt) => {
                if move_player.is_none() {
                    let path = game.get_almost_good_move_path(&mt)?;
                    move_player = Some(MovePlayer::new(path));
                }
                if !move_player.as_mut().unwrap().step(&mut game)? {
                    game.lock()?;
                    if game.state.is_game_over() {
                        break;
                    }
                    action = None;
                    move_player = None;
                }
            }
            Action::Hold => {
                game.hold()?;
                action = None;
                move_player = None;
            }
        }
        write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1))?;
        writeln!(stdout, "{} [ms] / {} [nodes] = {} [us/node] ", bot.expansion_duration.as_millis(), bot.num_expanded, bot.expansion_duration.as_micros() as usize / bot.num_expanded)?;
        write!(stdout, "{}", game)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}