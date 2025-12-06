<script lang="ts">
	import { get } from 'svelte/store';
	import {
		selectedRectangles,
		selectedEllipses,
		selectedLines,
		selectedArrows,
		selectedDiamonds,
		selectedTexts,
		selectedImages,
		selectedPaths,
		editorApi,
		rectangles,
		ellipses,
		lines,
		arrows,
		diamonds,
		texts,
		images,
		paths,
		type Rectangle,
		type Ellipse,
		type Line,
		type Arrow,
		type Diamond,
		type Text,
		type Image,
		type Path
	} from '$lib/stores/editor';
	import { theme } from '$lib/stores/theme';
	import ColorPicker from './ColorPicker.svelte';
	import { saveStateToLocalStorage } from '$lib/utils/storage';
	import { tick } from 'svelte';
	import { copyToClipboard, getClipboard } from '$lib/utils/clipboard';
	import { pasteShapes } from '$lib/utils/paste-shapes';
	import { deleteShapes } from '$lib/utils/delete-shapes';
	import { edgeStyle, type EdgeStyle } from '$lib/stores/edge-style';

	let strokeColor = '#000000';
	let fillColor: string | null = null;
	let lineWidth = 2;
	let textColor = '#000000';
	let unifiedColor = '#000000';

	const fillColors = ['#ef4444', '#3b82f6', '#10b981', '#f97316'];

	function isLightColor(hexColor: string): boolean {
		const hex = hexColor.replace('#', '');
		const r = parseInt(hex.slice(0, 2), 16);
		const g = parseInt(hex.slice(2, 4), 16);
		const b = parseInt(hex.slice(4, 6), 16);
		const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
		return luminance > 0.5;
	}

	$: {
		const totalSelected =
			$selectedRectangles.length +
			$selectedEllipses.length +
			$selectedDiamonds.length +
			$selectedLines.length +
			$selectedArrows.length +
			$selectedTexts.length +
			$selectedImages.length +
			$selectedPaths.length;

		if (totalSelected === 0) {
			strokeColor = '#000000';
			fillColor = null;
			lineWidth = 2;
			textColor = '#000000';
		} else {
		const shapes: Array<Rectangle | Ellipse | Line | Arrow | Diamond | Text | Path> = [
			...$selectedRectangles,
			...$selectedEllipses,
			...$selectedDiamonds,
			...$selectedLines,
			...$selectedArrows,
			...$selectedTexts,
			...$selectedPaths
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
		const selectedPths = get(selectedPaths);

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
		selectedPths.forEach((path) => {
			$editorApi.set_path_stroke_color(BigInt(path.id), color, false);
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
		$selectedPaths.forEach((path) => {
			$editorApi.set_path_line_width(BigInt(path.id), lineWidth, false);
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
		$selectedPaths.forEach(p => $editorApi!.bring_shape_to_front(BigInt(p.id)));
		$selectedImages.forEach(i => $editorApi!.bring_shape_to_front(BigInt(i.id)));
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
		$selectedPaths.forEach(p => $editorApi!.bring_shape_forward(BigInt(p.id)));
		$selectedImages.forEach(i => $editorApi!.bring_shape_forward(BigInt(i.id)));
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
		$selectedPaths.forEach(p => $editorApi!.send_shape_backward(BigInt(p.id)));
		$selectedImages.forEach(i => $editorApi!.send_shape_backward(BigInt(i.id)));
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
		$selectedPaths.forEach(p => $editorApi!.send_shape_to_back(BigInt(p.id)));
		$selectedImages.forEach(i => $editorApi!.send_shape_to_back(BigInt(i.id)));
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
		const selectedImageIds = new Set($selectedImages.map(i => i.id));
		const selectedPathIds = new Set($selectedPaths.map(p => p.id));
		
		const allRectangles = api.get_rectangles() as Rectangle[];
		const allEllipses = api.get_ellipses() as Ellipse[];
		const allLines = api.get_lines() as Line[];
		const allArrows = api.get_arrows() as Arrow[];
		const allDiamonds = api.get_diamonds() as Diamond[];
		const allTexts = api.get_texts() as Text[];
		const allImages = api.get_images() as Image[];
		const allPaths = api.get_paths() as Path[];
		
		rectangles.set(allRectangles);
		ellipses.set(allEllipses);
		lines.set(allLines);
		arrows.set(allArrows);
		diamonds.set(allDiamonds);
		texts.set(allTexts);
		images.set(allImages);
		paths.set(allPaths);
		
		selectedRectangles.set(allRectangles.filter(r => selectedRectIds.has(r.id)));
		selectedEllipses.set(allEllipses.filter(e => selectedEllipseIds.has(e.id)));
		selectedLines.set(allLines.filter(l => selectedLineIds.has(l.id)));
		selectedArrows.set(allArrows.filter(a => selectedArrowIds.has(a.id)));
		selectedDiamonds.set(allDiamonds.filter(d => selectedDiamondIds.has(d.id)));
		selectedTexts.set(allTexts.filter(t => selectedTextIds.has(t.id)));
		selectedImages.set(allImages.filter(i => selectedImageIds.has(i.id)));
		selectedPaths.set(allPaths.filter(p => selectedPathIds.has(p.id)));
	}

	$: hasSelection =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedTexts.length > 0 ||
		$selectedImages.length > 0 ||
		$selectedPaths.length > 0;

	$: hasFillableShapes =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0;

	$: hasEditableEdges =
		$selectedRectangles.length > 0 ||
		$selectedDiamonds.length > 0;

	$: hasImagesOnly = $selectedImages.length > 0 && 
		$selectedRectangles.length === 0 &&
		$selectedEllipses.length === 0 &&
		$selectedDiamonds.length === 0 &&
		$selectedLines.length === 0 &&
		$selectedArrows.length === 0 &&
		$selectedTexts.length === 0 &&
		$selectedPaths.length === 0;

	$: hasText = $selectedTexts.length > 0;

	$: hasShapes = 
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedPaths.length > 0;

	$: isSingleSelection = 
		($selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length + 
		 $selectedLines.length + $selectedArrows.length + $selectedTexts.length + $selectedPaths.length) === 1;

	$: {
		if (hasShapes && hasText) {
			const allColors = [
				...$selectedRectangles.map(r => (r as any).stroke_color || '#000000'),
				...$selectedEllipses.map(e => (e as any).stroke_color || '#000000'),
				...$selectedDiamonds.map(d => (d as any).stroke_color || '#000000'),
				...$selectedLines.map(l => (l as any).stroke_color || '#000000'),
				...$selectedArrows.map(a => (a as any).stroke_color || '#000000'),
				...$selectedPaths.map(p => (p as any).stroke_color || '#000000'),
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
		const selectedPths = get(selectedPaths);
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
		selectedPths.forEach((path) => {
			$editorApi.set_path_stroke_color(BigInt(path.id), color, false);
		});
		selectedTxts.forEach((text) => {
			$editorApi.set_text_color(BigInt(text.id), color, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		await tick();
		saveStateToLocalStorage();
	}

	function handleDuplicate() {
		if (!$editorApi) return;

		const hasSelection = $selectedRectangles.length > 0 || $selectedEllipses.length > 0 || $selectedLines.length > 0 || $selectedArrows.length > 0 || $selectedDiamonds.length > 0 || $selectedTexts.length > 0 || $selectedPaths.length > 0 || $selectedImages.length > 0;
		if (!hasSelection) return;

		copyToClipboard($selectedRectangles, $selectedEllipses, $selectedLines, $selectedArrows, $selectedDiamonds, $selectedTexts, $selectedImages, $selectedPaths);
		const clipboard = getClipboard();

		const bounds: Array<{ minX: number; minY: number }> = [];
		clipboard.rectangles.forEach(r => bounds.push({ minX: r.position.x, minY: r.position.y }));
		clipboard.ellipses.forEach(e => bounds.push({ minX: e.position.x - e.radius_x, minY: e.position.y - e.radius_y }));
		clipboard.diamonds.forEach(d => bounds.push({ minX: d.position.x, minY: d.position.y }));
		clipboard.lines.forEach(l => bounds.push({ minX: Math.min(l.start.x, l.end.x), minY: Math.min(l.start.y, l.end.y) }));
		clipboard.arrows.forEach(a => bounds.push({ minX: Math.min(a.start.x, a.end.x), minY: Math.min(a.start.y, a.end.y) }));
		clipboard.texts.forEach(t => bounds.push({ minX: t.position.x, minY: t.position.y }));
		clipboard.images.forEach(i => bounds.push({ minX: i.position.x, minY: i.position.y }));
		clipboard.paths.forEach(p => {
			if (p.points.length > 0) {
				const minX = Math.min(...p.points.map(pt => pt.x));
				const minY = Math.min(...p.points.map(pt => pt.y));
				bounds.push({ minX, minY });
			}
		});

		if (bounds.length === 0) return;

		const minX = Math.min(...bounds.map(b => b.minX));
		const minY = Math.min(...bounds.map(b => b.minY));
		const duplicateX = minX + 20;
		const duplicateY = minY + 20;

		pasteShapes(clipboard, duplicateX, duplicateY);
	}

	function handleDelete() {
		if (!$editorApi) return;

		const rectangleIds = $selectedRectangles.map(rect => rect.id);
		const ellipseIds = $selectedEllipses.map(ellipse => ellipse.id);
		const diamondIds = $selectedDiamonds.map(diamond => diamond.id);
		const lineIds = $selectedLines.map(line => line.id);
		const arrowIds = $selectedArrows.map(arrow => arrow.id);
		const textIds = $selectedTexts.map(text => text.id);
		const pathIds = $selectedPaths.map(path => path.id);
		const imageIds = $selectedImages.map(image => image.id);

		deleteShapes(rectangleIds, ellipseIds, lineIds, arrowIds, diamondIds, textIds, pathIds, imageIds);
	}

	function handleEdgeStyleChange(style: EdgeStyle) {
		if (!$editorApi) return;

		edgeStyle.set(style);
		const radius = style === 'rounded' ? 4.0 : 0.0;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_border_radius(BigInt(rect.id), radius, false);
		});

		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_border_radius(BigInt(diamond.id), radius, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	$: {
		if (hasEditableEdges) {
			const allBorderRadii: number[] = [];
			$selectedRectangles.forEach(r => {
				const br = r.border_radius ?? 0;
				allBorderRadii.push(br);
			});
			$selectedDiamonds.forEach(d => {
				const br = d.border_radius ?? 0;
				allBorderRadii.push(br);
			});
			
			const allSharp = allBorderRadii.length > 0 && allBorderRadii.every(r => r === 0);
			const allRounded = allBorderRadii.length > 0 && allBorderRadii.every(r => r > 0);
			
			if (allSharp) {
				edgeStyle.set('sharp');
			} else if (allRounded) {
				edgeStyle.set('rounded');
			}
		}
	}
</script>

{#if hasSelection}
	<div class={`absolute top-2 right-2 z-50 backdrop-blur-sm border rounded-lg p-3 w-[200px] min-w-[200px] min-h-[100px] overflow-hidden ${$theme === 'dark' ? 'bg-stone-800/95 border-stone-700/50' : 'bg-white/95 border-stone-200/50'} shadow-lg`}>
		<div class="space-y-2.5 min-w-0">
			{#if !hasImagesOnly}
				{#if hasShapes && hasText}
					<ColorPicker label="Color" bind:value={unifiedColor} onInput={(val: string) => updateUnifiedColor(val)} />
				{:else if hasShapes}
					<ColorPicker label="Stroke" bind:value={strokeColor} onInput={(val: string) => updateStrokeColor(val)} />
				{:else if hasText}
					<ColorPicker label="Text" bind:value={textColor} onInput={(val: string) => updateTextColor(val)} />
				{/if}
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

			{#if hasFillableShapes}
				<div class="space-y-1.5">
					<fieldset class="flex flex-col gap-2 w-full min-w-0">
						<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Fill</legend>
						<div class="flex items-center gap-2 min-w-0">
							<div class="relative shrink-0">
								<input
									type="color"
									value={fillColor || '#000000'}
									on:input={(e) => updateFillColor((e.target as HTMLInputElement).value)}
									class={`w-7 h-7 rounded-full border-2 cursor-pointer shrink-0 opacity-0 absolute inset-0 ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-200'}`}
									title={fillColor || 'No fill'}
								/>
								<div
									class={`w-7 h-7 rounded-full border-2 pointer-events-none ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-200'}`}
									style="background-color: {fillColor || 'transparent'};"
								></div>
							</div>
							<input
								type="text"
								value={fillColor || ''}
								on:input={(e) => {
									const val = (e.target as HTMLInputElement).value;
									updateFillColor(val || null);
								}}
								class={`flex-1 min-w-0 px-2 py-1 text-xs font-mono border rounded focus:outline-none focus:ring-1 h-8 ${$theme === 'dark' ? 'border-stone-600 bg-stone-700 text-stone-200 focus:ring-stone-500' : 'border-stone-200 bg-stone-50 focus:ring-stone-400'}`}
								placeholder="No fill"
								maxlength="7"
								aria-label="Fill color hex value"
							/>
						</div>
						<div class="grid grid-cols-4 gap-1.5 w-full mt-1">
							{#each fillColors as color}
								<button
									type="button"
									on:click={() => updateFillColor(color)}
									class={`w-6 h-6 rounded-full border-2 transition-all hover:scale-110 hover:border-stone-400 ${fillColor === color ? ($theme === 'dark' ? 'border-stone-400 ring-2 ring-stone-500' : 'border-stone-600 ring-2 ring-stone-300') : ($theme === 'dark' ? 'border-stone-600' : 'border-stone-200')}`}
									style="background-color: {color};"
									title={color}
								>
									{#if fillColor === color}
										<svg class="w-3 h-3 m-auto drop-shadow-lg {isLightColor(color) ? 'text-stone-900' : 'text-white'}" viewBox="0 0 16 16" fill="currentColor">
											<path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
										</svg>
									{/if}
								</button>
							{/each}
						</div>
					</fieldset>
				</div>
			{/if}

			{#if hasEditableEdges}
				<div class="space-y-1.5">
					<fieldset class="space-y-1.5">
						<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Edges</legend>
						<div class="flex items-center gap-1">
							<button
								on:click={() => handleEdgeStyleChange('sharp')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$edgeStyle === 'sharp' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Sharp"
								aria-label="Sharp edges"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
									<rect x="3" y="3" width="18" height="18" rx="0" ry="0" stroke-dasharray="2 2"></rect>
								</svg>
							</button>
							<button
								on:click={() => handleEdgeStyleChange('rounded')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$edgeStyle === 'rounded' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Rounded"
								aria-label="Rounded edges"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
									<rect x="3" y="3" width="18" height="18" rx="2" ry="2" stroke-dasharray="2 2"></rect>
									<path d="M15 3h6v6" stroke-width="2.5" fill="none"></path>
								</svg>
							</button>
						</div>
					</fieldset>
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

			<div class="space-y-1.5">
				<fieldset class="space-y-1.5">
					<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Actions</legend>
					<div class="flex items-center gap-1">
						<button
							on:click={handleDuplicate}
							class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
							title="Duplicate (Ctrl+D)"
							aria-label="Duplicate"
						>
							<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
								<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
							</svg>
						</button>
						<button
							on:click={handleDelete}
							class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
							title="Delete"
							aria-label="Delete"
						>
							<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<polyline points="3 6 5 6 21 6"></polyline>
								<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
								<line x1="10" y1="11" x2="10" y2="17"></line>
								<line x1="14" y1="11" x2="14" y2="17"></line>
							</svg>
						</button>
					</div>
				</fieldset>
			</div>
		</div>
	</div>
{/if}
