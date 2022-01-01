"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.SimpleFullScreenSinglePlay = void 0;
const react_1 = __importDefault(require("react"));
const styled_1 = __importDefault(require("@emotion/styled"));
const model = __importStar(require("@deep-trinity/model"));
const Game_1 = __importDefault(require("./Game"));
const Statistics_1 = __importDefault(require("./Statistics"));
const Root = styled_1.default.div `
  display: grid;
  place-items: center;
  width: 100%;
  height: 100%;
`;
const RootInner = styled_1.default.div `
  display: grid;
  grid-template-columns: max-content max-content max-content;
  grid-gap: 2vmin;
`;
const SimpleFullScreenSinglePlay = props => {
    return (react_1.default.createElement("div", null,
        react_1.default.createElement(Root, null,
            react_1.default.createElement(RootInner, null,
                react_1.default.createElement(Game_1.default, { game: props.game }),
                react_1.default.createElement(Statistics_1.default, { game: props.game, types: model.STATISTICS_ENTRY_TYPES }),
                react_1.default.createElement("div", null, props.children)))));
};
exports.SimpleFullScreenSinglePlay = SimpleFullScreenSinglePlay;
exports.default = exports.SimpleFullScreenSinglePlay;
//# sourceMappingURL=SimpleFullScreenSinglePlay.js.map