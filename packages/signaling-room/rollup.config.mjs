import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import typescript from '@rollup/plugin-typescript';
import terser from '@rollup/plugin-terser';
// @ts-expect-error :no-types
import esmImportToUrl from 'rollup-plugin-esm-import-to-url';

export default [
	{
		input: 'src/index.ts',
		output: [
			{
				file: 'browser/index.js',
				format: 'es',
				sourcemap: true,
				// @ts-expect-error :bad-type
				plugins: [terser()]
			}
		],
		plugins: [
			esmImportToUrl({
				imports: {
					tslib: 'https://unpkg.com/tslib@2/tslib.es6.js'
				}
			}),
			// @ts-expect-error :bad-type
			resolve({ browser: true }),
			// @ts-expect-error :bad-type
			commonjs({
				transformMixedEsModules: true
			}),
			// @ts-expect-error :bad-type
			typescript({
				tsconfig: './tsconfig.json',
				compilerOptions: {
					outDir: 'browser',
					declaration: false,
					declarationMap: false,
					declarationDir: undefined
				}
			})
		]
	}
];
