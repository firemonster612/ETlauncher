import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import svelte from 'eslint-plugin-svelte';
import globals from 'globals';

/** @type {import('eslint').Linter.Config[]} */
export default [
  {
    ignores: [
      'build/',
      '.svelte-kit/',
      'dist/',
      'src-tauri/target/',
      '*.config.js',
      '*.config.ts',
      '**/*.svelte.ts',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
      ecmaVersion: 2022,
      sourceType: 'module',
    },
  },
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
      },
    },
    rules: {
      // Disable the rule for links - external URLs don't need SvelteKit path resolution
      // The rule only makes sense for goto/pushState/replaceState calls
      'svelte/no-navigation-without-resolve': ['error', {
        ignoreLinks: true,
      }],
      // Disable unused svelte-ignore check since we removed the comments
      'svelte/no-unused-svelte-ignore': 'off',
    },
  },
];
