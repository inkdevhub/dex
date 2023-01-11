'use strict';

module.exports = {
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:prettier/recommended',
    'plugin:jest/recommended',
  ],
  root: true,
  parserOptions: {
    project: './tsconfig.json',
  },
  plugins: ['@typescript-eslint/eslint-plugin', 'jest'],
  env: {
    node: true,
    jest: true,
  },
  ignorePatterns: ['.eslintrc.js'],
  rules: {
    'dot-notation': [
      2,
      {
        allowKeywords: true,
        allowPattern: '^[a-z]+(_[a-z]+)+$',
      },
    ],
    '@typescript-eslint/no-namespace': 'off',
    '@typescript-eslint/interface-name-prefix': 'off',
    '@typescript-eslint/explicit-function-return-type': 'error',
    '@typescript-eslint/explicit-module-boundary-types': 'error',
    '@typescript-eslint/no-explicit-any': 'error',
    '@typescript-eslint/require-await': 'error',
    '@typescript-eslint/naming-convention': [
      'error',
      {
        selector: 'variableLike',
        format: ['strictCamelCase'],
      },
      {
        selector: 'variable',
        format: ['strictCamelCase', 'UPPER_CASE'],
      },
      {
        selector: 'memberLike',
        format: ['strictCamelCase'],
      },
      {
        selector: 'enumMember',
        format: ['StrictPascalCase'],
      },
      {
        selector: 'typeLike',
        format: ['StrictPascalCase'],
      },
      {
        selector: ['parameter'],
        modifiers: ['unused'],
        format: ['strictCamelCase'],
        leadingUnderscore: 'require',
      },
      {
        selector: 'typeParameter',
        format: ['StrictPascalCase'],
      },
      {
        selector: 'property',
        format: ['strictCamelCase'],
      },
      {
        selector: 'method',
        format: ['strictCamelCase'],
      },
    ],
    '@typescript-eslint/ban-ts-comment': 'off',
  },
};
