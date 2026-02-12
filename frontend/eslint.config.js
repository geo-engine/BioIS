// @ts-check
const eslint = require('@eslint/js');
const { defineConfig } = require('eslint/config');
const tseslint = require('typescript-eslint');
const angular = require('angular-eslint');

const appPrefix = 'app';

module.exports = defineConfig([
  {
    files: ['**/*.ts'],
    extends: [
      eslint.configs.recommended,
      tseslint.configs.recommendedTypeChecked,
      tseslint.configs.stylisticTypeChecked,
      angular.configs.tsRecommended,
    ],
    processor: angular.processInlineTemplates,
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        // project: true,
        project: ['tsconfig.app.json', 'tsconfig.spec.json'], // Explicitly list Angular's configs
        tsconfigRootDir: __dirname, // Recommended to ensure it resolves correctly from the project root
      },
    },
    rules: {
      'arrow-parens': ['off', 'always'],
      'brace-style': ['off', '1tbs'], // prettier is currently inconsistent, re-enable if possible
      'guard-for-in': 'error',
      'id-blacklist': 'off',
      'id-match': 'off',
      'import/order': 'off',
      'no-redeclare': ['error', { builtinGlobals: false }],
      'no-underscore-dangle': 'off',
      'valid-typeof': 'error',
      'no-bitwise': 'error',
      'no-empty-function': 'error',
      'no-unused-vars': 'off',
      'no-shadow': 'off',
      'no-console': [
        'error',
        {
          allow: ['warn', 'error'],
        },
      ],
      camelcase: 'off',

      '@angular-eslint/component-selector': [
        'error',
        { type: 'element', prefix: appPrefix, style: 'kebab-case' },
      ],
      '@angular-eslint/directive-selector': [
        'error',
        {
          type: 'attribute',
          prefix: appPrefix,
          style: 'camelCase',
        },
      ],
      '@angular-eslint/prefer-signals': 'error',

      '@typescript-eslint/array-type': ['off', { default: 'generic' }],
      '@typescript-eslint/consistent-type-definitions': 'error',
      '@typescript-eslint/dot-notation': 'off',
      '@typescript-eslint/explicit-member-accessibility': [
        'off',
        {
          accessibility: 'explicit',
        },
      ],
      '@typescript-eslint/member-ordering': [
        'error',
        {
          default: {
            memberTypes: [
              'signature',
              'field',
              'static-initialization',
              'constructor',
              'accessor',
              ['get', 'set', 'method'],
            ],
          },
        },
      ],
      '@typescript-eslint/naming-convention': [
        'error',
        {
          selector: 'default',
          format: ['camelCase'],
        },
        {
          selector: 'import',
          format: ['camelCase', 'PascalCase'],
        },
        {
          selector: 'variable',
          format: ['camelCase', 'UPPER_CASE'],
          leadingUnderscore: 'allow',
        },
        {
          selector: 'parameter',
          format: ['camelCase'],
          leadingUnderscore: 'allow',
        },
        {
          selector: 'parameterProperty',
          format: ['camelCase'],
          leadingUnderscore: 'allow',
        },
        {
          selector: 'memberLike',
          modifiers: ['private'],
          format: ['camelCase'],
          leadingUnderscore: 'allow',
        },
        {
          selector: 'property',
          format: ['camelCase', 'UPPER_CASE', 'PascalCase'],
          leadingUnderscore: 'allow',
        },
        {
          selector: 'enumMember',
          format: ['PascalCase', 'UPPER_CASE'],
          leadingUnderscore: 'allow',
        },
        {
          selector: 'accessor',
          format: ['camelCase', 'UPPER_CASE'],
        },
        {
          selector: 'typeLike',
          format: ['PascalCase'],
        },
      ],
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
        },
      ],
      '@typescript-eslint/no-shadow': 'error',
      '@typescript-eslint/explicit-function-return-type': 'error',
      '@typescript-eslint/no-unsafe-argument': 'error',
      '@typescript-eslint/no-unsafe-assignment': 'error',
      '@typescript-eslint/no-unsafe-call': 'error',
      '@typescript-eslint/no-unsafe-member-access': 'error',
      '@typescript-eslint/no-unsafe-return': 'error',
      '@typescript-eslint/unbound-method': ['error', { ignoreStatic: true }],
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/no-inferrable-types': [
        'error',
        { ignoreParameters: true, ignoreProperties: true },
      ],
    },
  },
  {
    files: ['**/*.html'],
    extends: [angular.configs.templateRecommended, angular.configs.templateAccessibility],
    rules: {},
  },
]);
