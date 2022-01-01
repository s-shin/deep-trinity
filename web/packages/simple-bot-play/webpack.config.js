const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require("path");

module.exports = {
  experiments: {
    // https://github.com/rustwasm/wasm-pack/issues/835
    syncWebAssembly: true,
  },
  mode: "development",
  devtool: "inline-source-map",
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  resolve: {
    extensions: [".ts", ".tsx", ".js"],
  },
  module: {
    rules: [
      { test: /\.tsx?$/, loader: "ts-loader" },
      { test: /\.wasm$/, type: "webassembly/sync" },
    ],
  },
  plugins: [
    new CopyWebpackPlugin({
      patterns: ["index.html"],
    }),
  ],
};
