import deep_trinity as detris


def test_game():
    game = detris.Game()
    game.fast_mode()
    assert game.should_supply_next_pieces()
    pieces = [getattr(detris.Cell, s).id for s in "SZLJITO"]
    game.supply_next_pieces(pieces)
    game.setup_falling_piece()
    game.firm_drop()
    assert game.lock()
    print(game)
