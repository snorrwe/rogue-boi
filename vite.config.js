import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { ViteRsw } from 'vite-plugin-rsw';
import path from 'path';

export default defineConfig({
	appType: 'spa',
	resolve: {
		alias: {
			'@rogueBoi': path.resolve(__dirname, './ui/lib')
		}
	},
	assetsInclude: ['public/**/*.svg'],
	build: {
		target: 'es2021'
	},
	plugins: [
		ViteRsw(),
		svelte({
			include: ['ui/**/*svelte'],
			emitCss: false
		})
	]
});
