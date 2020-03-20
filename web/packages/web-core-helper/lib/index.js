"use strict";
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const model = __importStar(require("@deep-trinity/model"));
exports.getGameModel = (game, onlyVisibleNext) => {
    const cells = new Uint8Array(game.width() * game.height()).fill(model.Cell.Empty);
    for (let y = 0; y < game.height(); y++) {
        for (let x = 0; x < game.width(); x++) {
            cells[model.getIndex(game.width(), x, y)] = game.getCell(x, y);
        }
    }
    let g = {
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