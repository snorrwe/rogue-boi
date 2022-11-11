import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { ViteRsw } from 'vite-plugin-rsw';
import path from 'path';
import { child_process } from 'vite-plugin-child-process';

const production = process.env.BUILD === 'production';

export default defineConfig({
	appType: 'spa',
	resolve: {
		alias: {
			'@rogueBoi': path.resolve(__dirname, './ui/lib')
		}
	},
	build: {
		target: 'es2021'
	},
	plugins: [
		ViteRsw(),
		svelte({
			include: ['ui/**/*svelte'],
			compilerOptions: {
				dev: !production
			},
			emitCss: false
		}),
		child_process({
			name: 'icons',
			command: ['cargo', 'xtask', 'copy-icons'],
			watch: []
		}),
		production &&
			child_process({
				name: 'bundle',
				command: ['cargo', 'xtask', 'bundle'],
				watch: []
			})
	]
});
