// @ts-check

import tseslint from "typescript-eslint";
import eslint from "@eslint/js";
import globals from "globals";
import jsdoc from "eslint-plugin-jsdoc";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import eslintPluginPrettierRecommended from "eslint-plugin-prettier/recommended";

// noinspection JSUnusedGlobalSymbols
export default tseslint.config(
    eslint.configs.recommended,
    eslintPluginPrettierRecommended,
    {
        ignores: [
            "**/dist/**",
            "**/docs/**",
            "packages/selfhosted/test_assets/swfobject.js",
        ],
    },
    {
        languageOptions: {
            ecmaVersion: 2021,
        },
        rules: {
            camelcase: [
                "error",
                { properties: "never", allow: ["__webpack_public_path__"] },
            ],
            curly: "error",
            eqeqeq: "error",
            "no-constructor-return": "error",
            "no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
            "prefer-const": "error",
            "spaced-comment": [
                "error",
                "always",
                { block: { balanced: true }, markers: ["/"] },
            ],
        },
    },
    {
        files: ["**/*.ts"],
        extends: tseslint.configs.strict,
        rules: {
            "@typescript-eslint/no-non-null-assertion": "off",
            "@typescript-eslint/consistent-type-assertions": [
                "error",
                { assertionStyle: "as" },
            ],
            "@typescript-eslint/no-unused-vars": [
                "error",
                {
                    args: "all",
                    argsIgnorePattern: "^_",
                    caughtErrors: "all",
                    caughtErrorsIgnorePattern: "^_",
                    destructuredArrayIgnorePattern: "^_",
                    varsIgnorePattern: "^_",
                    ignoreRestSiblings: true,
                },
            ],
            // Disallow const enums, as they can't be used by consumers.
            // See https://www.typescriptlang.org/docs/handbook/enums.html#const-enum-pitfalls
            "no-restricted-syntax": [
                "error",
                {
                    selector: "TSEnumDeclaration[const=true]",
                    message: "Don't declare const enums",
                },
            ],
        },
    },
    {
        files: ["packages/core/src/**", "packages/selfhosted/js/**"],
        languageOptions: {
            globals: globals.browser,
        },
    },
    {
        files: ["packages/extension/src/**"],
        languageOptions: {
            globals: { ...globals.browser, ...globals.webextensions },
        },
    },
    {
        files: ["packages/extension/tools/**"],
        languageOptions: {
            globals: globals.node,
            ecmaVersion: 2022, // Needed for top-level-await.
        },
    },
    {
        files: ["packages/**/webpack.config.js", "packages/**/wdio.config.ts"],
        languageOptions: {
            globals: globals.node,
        },
    },
    {
        files: ["packages/core/tools/**"],
        languageOptions: {
            globals: globals.node,
        },
    },
    {
        files: ["packages/core/src/**"],
        extends: [jsdoc.configs["flat/recommended-typescript-error"]],
        rules: {
            "jsdoc/tag-lines": [
                "error",
                "always",
                {
                    count: 0,
                    startLines: 1,
                },
            ],
        },
        settings: {
            jsdoc: {
                ignorePrivate: true,
                ignoreInternal: true,
            },
        },
    },
    {
        files: ["packages/demo/**"],
        languageOptions: {
            ecmaVersion: 2020,
            globals: globals.browser,
        },
        plugins: { "react-hooks": reactHooks, "react-refresh": reactRefresh },
        rules: {
            ...reactHooks.configs.recommended.rules,
            "react-refresh/only-export-components": [
                "warn",
                { allowConstantExport: true },
            ],
        },
    },
);
