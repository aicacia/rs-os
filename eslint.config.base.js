import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import ts from 'typescript-eslint';

/**
 * Base ESLint configuration for all TypeScript projects in the monorepo.
 * Import and spread this in your project's eslint.config.js:
 * 
 * import baseConfig from '../../eslint.config.base.js';
 * 
 * export default defineConfig(
 *   ...baseConfig,
 *   {
 *     // Your project-specific overrides here
 *   }
 * );
 */
export default defineConfig(
  js.configs.recommended,
  ...ts.configs.recommended,
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
      parserOptions: {
        projectService: true
      }
    },
    rules: {
      'no-undef': 'off',
      '@typescript-eslint/no-explicit-any': 'warn'
    }
  }
);
