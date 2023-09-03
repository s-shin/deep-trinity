import { FlatCompat } from "@eslint/eslintrc";
import path from "path";
import { fileURLToPath } from "url";
import js from "@eslint/js";
import tsParser from "@typescript-eslint/parser";
import tsPlugin from "@typescript-eslint/eslint-plugin";
import * as emotionPlugin from "@emotion/eslint-plugin";
import reactPlugin from "eslint-plugin-react";
import reactRecommendedConfig from "eslint-plugin-react/configs/recommended.js";
import reactJsxRecommendedConfig from "eslint-plugin-react/configs/jsx-runtime.js";
import globals from "globals";
import pretttierConfig  from "eslint-config-prettier";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const compat = new FlatCompat({
    baseDirectory: __dirname,
});

export default [
  js.configs.recommended,
  ...compat.extends("plugin:@typescript-eslint/recommended"),
  {
    settings: {
      react: {
        version: "detect",
      },
    },
  },
  reactRecommendedConfig,
  reactJsxRecommendedConfig,
  pretttierConfig,
  {
    files: ["src/**/*.{js,jsx,ts,tsx}"],
    plugins: {
      ts: tsPlugin,
      react: reactPlugin,
      emotion: emotionPlugin,
    },
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaFeatures: {
          jsx: true,
        },
      },
      globals: {
        ...globals.node,
        ...globals.browser,
      },
    },
    rules: {},
  },
  // {
  //   root: true,
  //   env: {
  //     node: true,
  //   },
  //   parser: "@typescript-eslint/parser",
  //   parserOptions: {
  //     tsconfigRootDir: __dirname,
  //     project: ["./tsconfig.json", "./packages/*/tsconfig.json"],
  //   },
  //   plugins: ["@typescript-eslint", "@emotion"],
  //   settings: {
  //     react: {
  //       version: "detect",
  //     },
  //   },
  //   extends: [
  //     "eslint:recommended",
  //     "plugin:react/recommended",
  //     "plugin:@typescript-eslint/recommended",
  //     "prettier",
  //   ],
  //   rules: {
  //     "react/prop-types": "off",
  //     "@typescript-eslint/explicit-function-return-type": ["error", { allowExpressions: true, allowTypedFunctionExpressions: true, allowHigherOrderFunctions: true }],
  //     "@emotion/pkg-renaming": "error",
  //   },
  // },
];
