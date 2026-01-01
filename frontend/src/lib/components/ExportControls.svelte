<script lang="ts">
	import { theme } from '$lib/stores/theme';
	import { 
		rectangles, ellipses, diamonds, lines, arrows, paths, images
	} from '$lib/stores/editor';
	import { exportToPNG, exportToSVG, exportToPDF } from '$lib/utils/export';

	export let canvas: HTMLCanvasElement | undefined = undefined;
	export let ctx: CanvasRenderingContext2D | null = null;

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
			$paths,
			$images,
			$texts,
			ctx,
			'rustboard.svg'
		);
	}

	async function handleExportPDF() {
		if (!canvas) return;
		await exportToPDF(canvas, 'rustboard.pdf');
	}
</script>

<div class={`absolute top-2 right-2 z-50 flex gap-1 shadow-sm rounded-sm p-1 ${$theme === 'dark' ? 'bg-stone-800 border border-stone-700' : 'bg-white border border-stone-200'}`}>
	<button
		on:click={handleExportPNG}
		class={`flex items-center gap-1.5 px-2 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm
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
		class={`flex items-center gap-1.5 px-2 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm
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
		class={`flex items-center gap-1.5 px-2 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm
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

