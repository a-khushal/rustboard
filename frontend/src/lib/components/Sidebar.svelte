<script lang="ts">
	import { theme } from '$lib/stores/theme';
	import { 
		rectangles, ellipses, diamonds, lines, arrows, texts, paths
	} from '$lib/stores/editor';
	import { exportToPNG, exportToSVG, exportToPDF } from '$lib/utils/export';

	export let canvas: HTMLCanvasElement | undefined = undefined;
	export let ctx: CanvasRenderingContext2D | null = null;

	let isOpen = false;

	function toggleSidebar() {
		isOpen = !isOpen;
	}

	async function handleExportPNG() {
		if (!canvas) return;
		await exportToPNG(canvas, 'rustboard.png');
	}

	function handleExportSVG() {
		if (!ctx) return;
		exportToSVG(
			$rectangles,
			$ellipses,
			$diamonds,
			$lines,
			$arrows,
			$texts,
			$paths,
			ctx,
			'rustboard.svg'
		);
	}

	async function handleExportPDF() {
		if (!canvas) return;
		await exportToPDF(canvas, 'rustboard.pdf');
	}
</script>

<div class="absolute top-2 right-2 z-50 flex items-start gap-2">
	{#if isOpen}
		<div class={`flex flex-col gap-2 shadow-lg rounded-lg p-3 w-[200px] backdrop-blur-sm ${$theme === 'dark' ? 'bg-stone-800/95 border border-stone-700/50' : 'bg-white/95 border border-stone-200/50'}`}>
			<div class="flex items-center justify-between mb-1">
				<h3 class={`text-xs font-semibold ${$theme === 'dark' ? 'text-stone-200' : 'text-stone-700'}`}>Menu</h3>
				<button
					on:click={toggleSidebar}
					class={`flex items-center justify-center w-6 h-6 rounded-sm transition-colors duration-150
						${$theme === 'dark'
							? 'text-stone-200 hover:bg-stone-700'
							: 'text-stone-700 hover:bg-stone-100'}`}
					title="Close Menu"
				>
					<svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
						<line x1="3" y1="3" x2="13" y2="13"/>
						<line x1="13" y1="3" x2="3" y2="13"/>
					</svg>
				</button>
			</div>
			
			<div class="flex flex-col gap-2">
				<div class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-600'}`}>Export</div>
				
				<div class="flex flex-row gap-2">
					<button
						on:click={handleExportPNG}
						class={`flex flex-col items-center gap-1 px-3 py-2 text-xs font-sans transition-colors duration-150 rounded-sm
							${$theme === 'dark'
								? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
								: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
						title="Export as PNG"
					>
						<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M14 10v2a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-2"/>
							<polyline points="11 7 8 10 5 7"/>
							<line x1="8" y1="10" x2="8" y2="2"/>
						</svg>
						<span>PNG</span>
					</button>
					
					<button
						on:click={handleExportSVG}
						class={`flex flex-col items-center gap-1 px-3 py-2 text-xs font-sans transition-colors duration-150 rounded-sm
							${$theme === 'dark'
								? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
								: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
						title="Export as SVG"
					>
						<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M14 10v2a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-2"/>
							<polyline points="11 7 8 10 5 7"/>
							<line x1="8" y1="10" x2="8" y2="2"/>
						</svg>
						<span>SVG</span>
					</button>
					
					<button
						on:click={handleExportPDF}
						class={`flex flex-col items-center gap-1 px-3 py-2 text-xs font-sans transition-colors duration-150 rounded-sm
							${$theme === 'dark'
								? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
								: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
						title="Export as PDF"
					>
						<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M14 10v2a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-2"/>
							<polyline points="11 7 8 10 5 7"/>
							<line x1="8" y1="10" x2="8" y2="2"/>
						</svg>
						<span>PDF</span>
					</button>
				</div>  
			</div>
		</div>
	{:else}
		<button
			on:click={toggleSidebar}
			class={`flex items-center justify-center w-8 h-8 rounded-sm shadow-sm transition-colors duration-150
				${$theme === 'dark'
					? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
					: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
			title="Open Menu"
		>
			<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
				<line x1="2" y1="4" x2="14" y2="4"/>
				<line x1="2" y1="8" x2="14" y2="8"/>
				<line x1="2" y1="12" x2="14" y2="12"/>
			</svg>
		</button>
	{/if}
</div>

