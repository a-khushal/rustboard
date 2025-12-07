<script lang="ts">
	import { get } from 'svelte/store';
	import { theme } from '$lib/stores/theme';
	import { 
		rectangles, ellipses, diamonds, lines, arrows, texts, paths, images,
		editorApi,
		type Rectangle, type Ellipse, type Line, type Arrow, type Diamond, type Text, type Path, type Image
	} from '$lib/stores/editor';
	import { exportToPNG, exportToSVG, exportToPDF } from '$lib/utils/export';
	import { deleteShapes } from '$lib/utils/delete-shapes';
	import { clearAllSelections } from '$lib/utils/selection';

	export let canvas: HTMLCanvasElement | undefined = undefined;
	export let ctx: CanvasRenderingContext2D | null = null;

	let isOpen = false;
	let showResetModal = false;
	let fileInputRef: HTMLInputElement;

	export function closeSidebar() {
		isOpen = false;
	}

	function toggleSidebar() {
		isOpen = !isOpen;
	}

	function openResetModal() {
		showResetModal = true;
		isOpen = false;
	}

	function closeResetModal() {
		showResetModal = false;
	}

	function handleModalKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			closeResetModal();
		}
	}

	function handleBackdropKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape' || event.key === 'Enter') {
			closeResetModal();
		}
	}

	function handleStopPropagation(event: MouseEvent | KeyboardEvent) {
		event.stopPropagation();
	}

	function handleResetCanvas() {
		const api = get(editorApi);
		if (!api) return;

		const rectangleIds = $rectangles.map(r => r.id);
		const ellipseIds = $ellipses.map(e => e.id);
		const diamondIds = $diamonds.map(d => d.id);
		const lineIds = $lines.map(l => l.id);
		const arrowIds = $arrows.map(a => a.id);
		const textIds = $texts.map(t => t.id);
		const pathIds = $paths.map(p => p.id);
		const imageIds = $images.map(i => i.id);

		deleteShapes(rectangleIds, ellipseIds, lineIds, arrowIds, diamondIds, textIds, pathIds, imageIds);
		clearAllSelections();
		closeResetModal();
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
			$images,
			ctx,
			'rustboard.svg'
		);
	}

	async function handleExportPDF() {
		if (!canvas) return;
		await exportToPDF(canvas, 'rustboard.pdf');
	}

	function handleSaveAs() {
		const api = get(editorApi);
		if (!api) return;

		const jsonData = api.serialize();
		const blob = new Blob([jsonData], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = 'rustboard.json';
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}

	async function handleLoadFile(event: Event) {
		const target = event.target as HTMLInputElement;
		const file = target.files?.[0];
		if (!file) return;

		const api = get(editorApi);
		if (!api) return;

		try {
			const text = await file.text();
			const success = api.deserialize(text);
			
			if (success) {
				const updatedRectangles = api.get_rectangles() as Rectangle[];
				const updatedEllipses = api.get_ellipses() as Ellipse[];
				const updatedLines = api.get_lines() as Line[];
				const updatedArrows = api.get_arrows() as Arrow[];
				const updatedDiamonds = api.get_diamonds() as Diamond[];
				const updatedTexts = api.get_texts() as Text[];
				const updatedPaths = api.get_paths() as Path[];
				const updatedImages = api.get_images() as Image[];
				
				rectangles.set(updatedRectangles);
				ellipses.set(updatedEllipses);
				lines.set(updatedLines);
				arrows.set(updatedArrows);
				diamonds.set(updatedDiamonds);
				texts.set(updatedTexts);
				paths.set(updatedPaths);
				images.set(updatedImages);
				
				clearAllSelections();
			} else {
				alert('Failed to load file. Please ensure it is a valid Rustboard JSON file.');
			}
		} catch (error) {
			console.error('Error loading file:', error);
			alert('Error loading file. Please try again.');
		}

		target.value = '';
	}
</script>

<div class="absolute top-2 right-2 z-50 flex items-start gap-2" role="complementary" aria-label="Sidebar menu">
	{#if isOpen}
		<div 
			class={`flex flex-col gap-2 shadow-lg rounded-lg p-3 w-[200px] backdrop-blur-sm ${$theme === 'dark' ? 'bg-stone-800/95 border border-stone-700/50' : 'bg-white/95 border border-stone-200/50'}`}
			on:click={handleStopPropagation}
			on:keydown={handleStopPropagation}
			role="menu"
			tabindex="-1"
		>
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
				<div class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-600'}`}>File</div>
				
				<div class="flex flex-col gap-2">
					<button
						on:click={handleSaveAs}
						class={`flex items-center justify-center gap-2 px-3 py-2 text-xs font-sans transition-colors duration-150 rounded-sm
							${$theme === 'dark'
								? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
								: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
						title="Save As JSON"
					>
						<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M14 10v2a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-2"/>
							<polyline points="11 7 8 10 5 7"/>
							<line x1="8" y1="10" x2="8" y2="2"/>
						</svg>
						<span>Save As</span>
					</button>
					
					<input
						bind:this={fileInputRef}
						type="file"
						accept=".json"
						on:change={handleLoadFile}
						class="hidden"
						aria-label="Load JSON file"
					/>
					<button
						type="button"
						on:click={() => fileInputRef?.click()}
						class={`flex items-center justify-center gap-2 px-3 py-2 text-xs font-sans transition-colors duration-150 rounded-sm w-full
							${$theme === 'dark'
								? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
								: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
						title="Load JSON file"
					>
						<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M14 10v2a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-2"/>
							<polyline points="5 7 8 4 11 7"/>
							<line x1="8" y1="4" x2="8" y2="12"/>
						</svg>
						<span>Load</span>
					</button>
				</div>

				<div class="border-t ${$theme === 'dark' ? 'border-stone-700' : 'border-stone-200'} my-1"></div>

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

				<div class="border-t ${$theme === 'dark' ? 'border-stone-700' : 'border-stone-200'} my-1"></div>

				<button
					on:click={openResetModal}
					class={`flex items-center justify-center gap-2 px-3 py-2 text-xs font-sans transition-colors duration-150 rounded-sm
						${$theme === 'dark'
							? 'text-red-400 bg-stone-800 hover:bg-stone-700 border border-red-500/30 hover:border-red-500/50'
							: 'text-red-600 bg-white hover:bg-stone-50 border border-red-300 hover:border-red-400'}`}
					title="Reset Canvas"
				>
					<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
						<polyline points="3 6 5 6 21 6"/>
						<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
						<line x1="10" y1="11" x2="10" y2="17"/>
						<line x1="14" y1="11" x2="14" y2="17"/>
					</svg>
					<span>Reset canvas</span>
				</button>
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

{#if showResetModal}
	<div 
		class="fixed inset-0 z-100 flex items-center justify-center ${$theme === 'dark' ? 'bg-black/50' : 'bg-black/30'}"
		on:click={closeResetModal}
		on:keydown={handleBackdropKeyDown}
		role="dialog"
		aria-modal="true"
		aria-labelledby="modal-title"
		tabindex="-1"
	>
		<div 
			class={`rounded-lg shadow-xl p-6 max-w-md w-full mx-4 ${$theme === 'dark' ? 'bg-stone-800 border border-stone-700' : 'bg-white border border-stone-200'}`}
			on:click={handleStopPropagation}
			on:keydown={handleStopPropagation}
			role="dialog"
			tabindex="-1"
		>
			<div class="flex items-start gap-4 mb-4">
				<div class={`shrink-0 w-10 h-10 rounded-full flex items-center justify-center ${$theme === 'dark' ? 'bg-red-500/20' : 'bg-red-100'}`}>
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" class={$theme === 'dark' ? 'text-red-400' : 'text-red-600'}>
						<polyline points="3 6 5 6 21 6"/>
						<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
						<line x1="10" y1="11" x2="10" y2="17"/>
						<line x1="14" y1="11" x2="14" y2="17"/>
					</svg>
				</div>
				<div class="flex-1">
					<h3 id="modal-title" class={`text-lg font-semibold mb-2 ${$theme === 'dark' ? 'text-stone-200' : 'text-stone-900'}`}>
						Reset Canvas
					</h3>
					<p class={`text-sm ${$theme === 'dark' ? 'text-stone-400' : 'text-stone-600'}`}>
						Are you sure you want to reset the canvas? This will permanently delete all shapes and cannot be undone.
					</p>
				</div>
			</div>
			<div class="flex gap-3 justify-end">
				<button
					on:click={closeResetModal}
					class={`px-4 py-2 text-sm font-medium rounded-sm transition-colors duration-150
						${$theme === 'dark'
							? 'text-stone-300 bg-stone-700 hover:bg-stone-600 border border-stone-600'
							: 'text-stone-700 bg-stone-100 hover:bg-stone-200 border border-stone-300'}`}
				>
					Cancel
				</button>
				<button
					on:click={handleResetCanvas}
					class={`px-4 py-2 text-sm font-medium rounded-sm transition-colors duration-150
						${$theme === 'dark'
							? 'text-white bg-red-600 hover:bg-red-700 border border-red-700'
							: 'text-white bg-red-600 hover:bg-red-700 border border-red-700'}`}
				>
					Reset
				</button>
			</div>
		</div>
	</div>
{/if}

