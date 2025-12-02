<script lang="ts">
	import { get } from 'svelte/store';
	import {
		selectedRectangles,
		selectedEllipses,
		selectedLines,
		selectedArrows,
		selectedDiamonds,
		selectedTexts,
		editorApi,
		rectangles,
		ellipses,
		lines,
		arrows,
		diamonds,
		texts,
		type Rectangle,
		type Ellipse,
		type Line,
		type Arrow,
		type Diamond,
		type Text
	} from '$lib/stores/editor';
	import { theme } from '$lib/stores/theme';
	import ColorPicker from './ColorPicker.svelte';
	import { saveStateToLocalStorage } from '$lib/utils/storage';
	import { tick } from 'svelte';

	let strokeColor = '#000000';
	let fillColor: string | null = null;
	let lineWidth = 2;
	let textColor = '#000000';
	let unifiedColor = '#000000';

	$: {
		const totalSelected =
			$selectedRectangles.length +
			$selectedEllipses.length +
			$selectedDiamonds.length +
			$selectedLines.length +
			$selectedArrows.length +
			$selectedTexts.length;

		if (totalSelected === 0) {
			strokeColor = '#000000';
			fillColor = null;
			lineWidth = 2;
			textColor = '#000000';
		} else {
		const shapes: Array<Rectangle | Ellipse | Line | Arrow | Diamond | Text> = [
			...$selectedRectangles,
			...$selectedEllipses,
			...$selectedDiamonds,
			...$selectedLines,
			...$selectedArrows,
			...$selectedTexts
		];

		const strokeColors = shapes
			.map((s) => (s as any).stroke_color || '#000000')
			.filter((c, i, arr) => arr.indexOf(c) === i);
		strokeColor = strokeColors.length === 1 ? strokeColors[0] : '#000000';

		const fillColors = shapes
			.map((s) => (s as any).fill_color ?? null)
			.filter((c, i, arr) => arr.indexOf(c) === i);
		fillColor = fillColors.length === 1 ? fillColors[0] : null;

		const lineWidths = shapes
			.map((s) => (s as any).line_width ?? 2)
			.filter((w, i, arr) => arr.indexOf(w) === i);
		lineWidth = lineWidths.length === 1 ? lineWidths[0] : 2;

		const textColors = $selectedTexts
			.map((t) => t.text_color || '#000000')
			.filter((c, i, arr) => arr.indexOf(c) === i);
		textColor = textColors.length === 1 ? textColors[0] : '#000000';
		}
	}

	async function updateStrokeColor(color: string) {
		strokeColor = color;
		if (!$editorApi) return;

		const selectedRects = get(selectedRectangles);
		const selectedElls = get(selectedEllipses);
		const selectedDias = get(selectedDiamonds);
		const selectedLns = get(selectedLines);
		const selectedArrs = get(selectedArrows);

		selectedRects.forEach((rect) => {
			$editorApi.set_rectangle_stroke_color(BigInt(rect.id), color, false);
		});
		selectedElls.forEach((ellipse) => {
			$editorApi.set_ellipse_stroke_color(BigInt(ellipse.id), color, false);
		});
		selectedDias.forEach((diamond) => {
			$editorApi.set_diamond_stroke_color(BigInt(diamond.id), color, false);
		});
		selectedLns.forEach((line) => {
			$editorApi.set_line_stroke_color(BigInt(line.id), color, false);
		});
		selectedArrs.forEach((arrow) => {
			$editorApi.set_arrow_stroke_color(BigInt(arrow.id), color, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateFillColor(color: string | null) {
		fillColor = color;
		if (!$editorApi) return;

		const colorValue = color || null;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_fill_color(BigInt(rect.id), colorValue, false);
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_ellipse_fill_color(BigInt(ellipse.id), colorValue, false);
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_fill_color(BigInt(diamond.id), colorValue, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateLineWidth(width: number) {
		lineWidth = Math.max(0.1, width);
		if (!$editorApi) return;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_line_width(BigInt(rect.id), lineWidth, false);
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_ellipse_line_width(BigInt(ellipse.id), lineWidth, false);
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_line_width(BigInt(diamond.id), lineWidth, false);
		});
		$selectedLines.forEach((line) => {
			$editorApi.set_line_line_width(BigInt(line.id), lineWidth, false);
		});
		$selectedArrows.forEach((arrow) => {
			$editorApi.set_arrow_line_width(BigInt(arrow.id), lineWidth, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateTextColor(color: string) {
		textColor = color;
		if (!$editorApi) return;

		$selectedTexts.forEach((text) => {
			$editorApi.set_text_color(BigInt(text.id), color, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function bringToFront() {
		if (!$editorApi) return;
		$selectedRectangles.forEach(r => $editorApi!.bring_shape_to_front(BigInt(r.id)));
		$selectedEllipses.forEach(e => $editorApi!.bring_shape_to_front(BigInt(e.id)));
		$selectedDiamonds.forEach(d => $editorApi!.bring_shape_to_front(BigInt(d.id)));
		$selectedLines.forEach(l => $editorApi!.bring_shape_to_front(BigInt(l.id)));
		$selectedArrows.forEach(a => $editorApi!.bring_shape_to_front(BigInt(a.id)));
		$selectedTexts.forEach(t => $editorApi!.bring_shape_to_front(BigInt(t.id)));
		updateStores();
		saveStateToLocalStorage();
	}

	function bringForward() {
		if (!$editorApi) return;
		$selectedRectangles.forEach(r => $editorApi!.bring_shape_forward(BigInt(r.id)));
		$selectedEllipses.forEach(e => $editorApi!.bring_shape_forward(BigInt(e.id)));
		$selectedDiamonds.forEach(d => $editorApi!.bring_shape_forward(BigInt(d.id)));
		$selectedLines.forEach(l => $editorApi!.bring_shape_forward(BigInt(l.id)));
		$selectedArrows.forEach(a => $editorApi!.bring_shape_forward(BigInt(a.id)));
		$selectedTexts.forEach(t => $editorApi!.bring_shape_forward(BigInt(t.id)));
		updateStores();
		saveStateToLocalStorage();
	}

	function sendBackward() {
		if (!$editorApi) return;
		$selectedRectangles.forEach(r => $editorApi!.send_shape_backward(BigInt(r.id)));
		$selectedEllipses.forEach(e => $editorApi!.send_shape_backward(BigInt(e.id)));
		$selectedDiamonds.forEach(d => $editorApi!.send_shape_backward(BigInt(d.id)));
		$selectedLines.forEach(l => $editorApi!.send_shape_backward(BigInt(l.id)));
		$selectedArrows.forEach(a => $editorApi!.send_shape_backward(BigInt(a.id)));
		$selectedTexts.forEach(t => $editorApi!.send_shape_backward(BigInt(t.id)));
		updateStores();
		saveStateToLocalStorage();
	}

	function sendToBack() {
		if (!$editorApi) return;
		$selectedRectangles.forEach(r => $editorApi!.send_shape_to_back(BigInt(r.id)));
		$selectedEllipses.forEach(e => $editorApi!.send_shape_to_back(BigInt(e.id)));
		$selectedDiamonds.forEach(d => $editorApi!.send_shape_to_back(BigInt(d.id)));
		$selectedLines.forEach(l => $editorApi!.send_shape_to_back(BigInt(l.id)));
		$selectedArrows.forEach(a => $editorApi!.send_shape_to_back(BigInt(a.id)));
		$selectedTexts.forEach(t => $editorApi!.send_shape_to_back(BigInt(t.id)));
		updateStores();
		saveStateToLocalStorage();
	}

	function updateStores() {
		if (!$editorApi) return;
		const api = get(editorApi);
		
		const selectedRectIds = new Set($selectedRectangles.map(r => r.id));
		const selectedEllipseIds = new Set($selectedEllipses.map(e => e.id));
		const selectedLineIds = new Set($selectedLines.map(l => l.id));
		const selectedArrowIds = new Set($selectedArrows.map(a => a.id));
		const selectedDiamondIds = new Set($selectedDiamonds.map(d => d.id));
		const selectedTextIds = new Set($selectedTexts.map(t => t.id));
		
		const allRectangles = api.get_rectangles() as Rectangle[];
		const allEllipses = api.get_ellipses() as Ellipse[];
		const allLines = api.get_lines() as Line[];
		const allArrows = api.get_arrows() as Arrow[];
		const allDiamonds = api.get_diamonds() as Diamond[];
		const allTexts = api.get_texts() as Text[];
		
		rectangles.set(allRectangles);
		ellipses.set(allEllipses);
		lines.set(allLines);
		arrows.set(allArrows);
		diamonds.set(allDiamonds);
		texts.set(allTexts);
		
		selectedRectangles.set(allRectangles.filter(r => selectedRectIds.has(r.id)));
		selectedEllipses.set(allEllipses.filter(e => selectedEllipseIds.has(e.id)));
		selectedLines.set(allLines.filter(l => selectedLineIds.has(l.id)));
		selectedArrows.set(allArrows.filter(a => selectedArrowIds.has(a.id)));
		selectedDiamonds.set(allDiamonds.filter(d => selectedDiamondIds.has(d.id)));
		selectedTexts.set(allTexts.filter(t => selectedTextIds.has(t.id)));
	}

	$: hasSelection =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedTexts.length > 0;

	$: hasFillableShapes =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0;

	$: hasText = $selectedTexts.length > 0;

	$: hasShapes = 
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0;

	$: isSingleSelection = 
		($selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length + 
		 $selectedLines.length + $selectedArrows.length + $selectedTexts.length) === 1;

	$: {
		if (hasShapes && hasText) {
			const allColors = [
				...$selectedRectangles.map(r => (r as any).stroke_color || '#000000'),
				...$selectedEllipses.map(e => (e as any).stroke_color || '#000000'),
				...$selectedDiamonds.map(d => (d as any).stroke_color || '#000000'),
				...$selectedLines.map(l => (l as any).stroke_color || '#000000'),
				...$selectedArrows.map(a => (a as any).stroke_color || '#000000'),
				...$selectedTexts.map(t => (t as any).text_color || '#000000')
			].filter((c, i, arr) => arr.indexOf(c) === i);
			unifiedColor = allColors.length === 1 ? allColors[0] : '#000000';
		}
	}

	async function updateUnifiedColor(color: string) {
		unifiedColor = color;
		if (!$editorApi) return;

		const selectedRects = get(selectedRectangles);
		const selectedElls = get(selectedEllipses);
		const selectedDias = get(selectedDiamonds);
		const selectedLns = get(selectedLines);
		const selectedArrs = get(selectedArrows);
		const selectedTxts = get(selectedTexts);

		selectedRects.forEach((rect) => {
			$editorApi.set_rectangle_stroke_color(BigInt(rect.id), color, false);
		});
		selectedElls.forEach((ellipse) => {
			$editorApi.set_ellipse_stroke_color(BigInt(ellipse.id), color, false);
		});
		selectedDias.forEach((diamond) => {
			$editorApi.set_diamond_stroke_color(BigInt(diamond.id), color, false);
		});
		selectedLns.forEach((line) => {
			$editorApi.set_line_stroke_color(BigInt(line.id), color, false);
		});
		selectedArrs.forEach((arrow) => {
			$editorApi.set_arrow_stroke_color(BigInt(arrow.id), color, false);
		});
		selectedTxts.forEach((text) => {
			$editorApi.set_text_color(BigInt(text.id), color, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		await tick();
		saveStateToLocalStorage();
	}
</script>

{#if hasSelection}
	<div class={`absolute top-2 right-2 z-50 backdrop-blur-sm border rounded-lg p-3 w-[200px] overflow-hidden ${$theme === 'dark' ? 'bg-stone-800/95 border-stone-700/50' : 'bg-white/95 border-stone-200/50'} shadow-lg`}>
		<div class="space-y-2.5 min-w-0">
			{#if hasShapes && hasText}
				<ColorPicker label="Color" bind:value={unifiedColor} onInput={(val: string) => updateUnifiedColor(val)} />
			{:else if hasShapes}
				<ColorPicker label="Stroke" bind:value={strokeColor} onInput={(val: string) => updateStrokeColor(val)} />
			{:else if hasText}
				<ColorPicker label="Text" bind:value={textColor} onInput={(val: string) => updateTextColor(val)} />
			{/if}

			{#if isSingleSelection && hasShapes}
				<div class="flex items-center gap-2 min-w-0">
					<label for="line-width" class="sr-only">Line Width</label>
					<input
						type="range"
						min="0.5"
						max="20"
						step="0.5"
						bind:value={lineWidth}
						on:input={(e) => updateLineWidth(parseFloat((e.target as HTMLInputElement).value))}
						class={`flex-1 min-w-0 h-1 rounded-lg appearance-none cursor-pointer ${$theme === 'dark' ? 'bg-stone-600 accent-stone-400' : 'bg-stone-200 accent-stone-600'}`}
						id="line-width"
					/>
					<input
						type="number"
						bind:value={lineWidth}
						on:input={(e) => updateLineWidth(parseFloat((e.target as HTMLInputElement).value))}
						min="0.5"
						max="20"
						step="0.5"
						class={`w-12 px-1.5 py-1 text-xs border rounded focus:outline-none focus:ring-1 shrink-0 ${$theme === 'dark' ? 'border-stone-600 bg-stone-700 text-stone-200 focus:ring-stone-500' : 'border-stone-200 bg-stone-50 focus:ring-stone-400'}`}
						aria-label="Line width value"
					/>
				</div>
			{/if}

			<div class="space-y-1.5">
				<fieldset class="space-y-1.5">
					<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Layers</legend>
					<div class="flex items-center gap-1">
						<button
							on:click={sendToBack}
							class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
							title="Send to Back (Ctrl+Shift+[)"
							aria-label="Send to Back"
						>
							<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<line x1="12" y1="5" x2="12" y2="19"></line>
								<polyline points="19 12 12 19 5 12"></polyline>
							</svg>
						</button>
						<button
							on:click={sendBackward}
							class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
							title="Send Backward (Ctrl+[)"
							aria-label="Send Backward"
						>
							<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="19 12 12 19 5 12"></polyline>
							</svg>
						</button>
						<button
							on:click={bringForward}
							class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
							title="Bring Forward (Ctrl+])"
							aria-label="Bring Forward"
						>
							<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="5 12 12 5 19 12"></polyline>
							</svg>
						</button>
						<button
							on:click={bringToFront}
							class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
							title="Bring to Front (Ctrl+Shift+])"
							aria-label="Bring to Front"
						>
							<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<line x1="12" y1="19" x2="12" y2="5"></line>
								<polyline points="5 12 12 5 19 12"></polyline>
							</svg>
						</button>
					</div>
				</fieldset>
			</div>
		</div>
	</div>
{/if}
