from deep_trinity import Game, Cell


def test_game():
    game = Game()
    game.fast_mode()
    assert game.should_supply_next_pieces()
    pieces = [getattr(Cell, s).id for s in "SZLJITO"]
    game.supply_next_pieces(pieces)
    game.setup_falling_piece()
    game.firm_drop()
    assert game.lock()
    assert len(str(game)) > 0
    material = game.get_move_decision_material()
    assert len(material.get_dst_candidates()) > 0
