import * as model from "@deep-trinity/model";
export const getGameModel = (game, onlyVisibleNext) => {
    const cells = new Uint8Array(game.width() * game.height()).fill(model.Cell.Empty);
    for (let y = 0; y < game.height(); y++) {
        for (let x = 0; x < game.width(); x++) {
            cells[model.getIndex(game.width(), x, y)] = game.getCell(x, y);
        }
    }
    const g = {
        width: game.width(),
        height: game.height(),
        visibleHeight: game.visibleHeight(),
        cells,
        nextPieces: game.getNextPieces(onlyVisibleNext),
        holdPiece: game.getHoldPiece(),
        currentNumBTBs: game.getCurrentNumBTBs(),
        currentNumCombos: game.getCurrentNumCombos(),
        stats: model.STATISTICS_ENTRY_TYPES.reduce((p, c) => {
            p[c] = game.getStatsCount(c);
            return p;
        }, {}),
    };
    return g;
};
//# sourceMappingURL=index.js.map