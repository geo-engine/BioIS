// Removes unused imports from the generated code (run with `--fix`).
//
// ESLint is run via `npx -p eslint -p typescript-eslint -p eslint-plugin-unused-imports`,
// so the plugins are not resolvable from this file.
// Instead, resolve them next to the running ESLint binary, where `npx` installed them.

import { realpathSync } from "node:fs";
import { createRequire } from "node:module";

const require = createRequire(realpathSync(process.argv[1]));
const tseslint = require("typescript-eslint");
const unusedImports = require("eslint-plugin-unused-imports");

export default [
  {
    files: ["typescript/**/*.ts"],
    linterOptions: { reportUnusedDisableDirectives: "off" },
    languageOptions: { parser: tseslint.parser },
    plugins: { "unused-imports": unusedImports },
    rules: { "unused-imports/no-unused-imports": "error" },
  },
];
