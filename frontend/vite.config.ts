import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), wasm(), topLevelAwait()],
	resolve: {
		alias: {
			'rustboard-wasm': path.resolve(__dirname, '../pkg'),
			'rustboard_wasm': path.resolve(__dirname, '../pkg')
		}
	},
	optimizeDeps: {
		exclude: ['rustboard-wasm', 'rustboard_wasm']
	},
	server: {
		fs: {
			allow: [
				path.resolve(__dirname, '../pkg'),
				path.resolve(__dirname, '..')
			]
		}
	}
});
