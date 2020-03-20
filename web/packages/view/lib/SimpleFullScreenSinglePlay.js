"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (Object.hasOwnProperty.call(mod, k)) result[k] = mod[k];
    result["default"] = mod;
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
const react_1 = __importDefault(require("react"));
const core_1 = require("@emotion/core");
const styled_1 = __importDefault(require("@emotion/styled"));
const model = __importStar(require("@deep-trinity/model"));
const Game_1 = __importDefault(require("./Game"));
const Statistics_1 = __importDefault(require("./Statistics"));
const Root = styled_1.default.div `
  display: grid;
  align-items: center;
  place-items: center;
  width: 100%;
  height: 100%;
`;
const RootInner = styled_1.default.div `
  display: grid;
  grid-template-columns: max-content max-content;
  grid-gap: 2vmin;
`;
exports.SimpleFullScreenSinglePlay = props => {
    return (react_1.default.createElement("div", null,
        react_1.default.createElement(core_1.Global, { styles: core_1.css `
        body { margin: 0; padding: 0; }
      ` }),
        react_1.default.createElement(Root, null,
            react_1.default.createElement(RootInner, null,
                react_1.default.createElement(Game_1.default, { game: props.game }),
                react_1.default.createElement(Statistics_1.default, { game: props.game, types: model.STATISTICS_ENTRY_TYPES })))));
};
exports.default = exports.SimpleFullScreenSinglePlay;
//# sourceMappingURL=SimpleFullScreenSinglePlay.js.map