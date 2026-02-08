import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import path from 'path';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit(), wasm(), topLevelAwait()],
	build: {
		chunkSizeWarningLimit: 650,
		rollupOptions: {
			output: {
				manualChunks(id) {
					if (id.includes('/pkg/')) {
						return 'rustboard-wasm-bridge';
					}

					if (!id.includes('node_modules')) return undefined;

					if (id.includes('/svelte/')) {
						return 'vendor-svelte';
					}
					if (id.includes('/@sveltejs/')) {
						return 'vendor-sveltekit';
					}

					const parts = id.split('/node_modules/')[1]?.split('/') ?? [];
					if (parts.length === 0) return 'vendor';

					const packageName = parts[0].startsWith('@') && parts.length > 1
						? `${parts[0]}-${parts[1]}`
						: parts[0];

					return `vendor-${packageName.replace('@', '').replace('/', '-')}`;
				}
			}
		}
	},
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
