module.exports = {
  root: true,
  env: {
    node: true,
  },
  parser: "@typescript-eslint/parser",
  parserOptions: {
    tsconfigRootDir: __dirname,
    project: ["./tsconfig.json", "./packages/*/tsconfig.json"],
  },
  plugins: ["@typescript-eslint"],
  settings: {
    react: {
      version: "detect",
    },
  },
  extends: [
    "eslint:recommended",
    "plugin:react/recommended",
    "plugin:@typescript-eslint/recommended",
    "prettier",
  ],
  rules: {
    "react/prop-types": "off",
    "@typescript-eslint/explicit-function-return-type": ["error", { allowExpressions: true, allowTypedFunctionExpressions: true, allowHigherOrderFunctions: true }],
  },
};
