<script lang="ts">
	import { get } from 'svelte/store';
	import {
		selectedRectangles,
		selectedEllipses,
		selectedLines,
		selectedArrows,
		selectedDiamonds,
		selectedImages,
		selectedTexts,
		selectedPaths,
		selectedGroups,
		editorApi,
		rectangles,
		ellipses,
		lines,
		arrows,
		diamonds,
		images,
		paths,
		texts,
		type Rectangle,
		type Ellipse,
		type Line,
		type Arrow,
		type Diamond,
		type Image,
		type Text,
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
	import { dashPattern, type DashPattern } from '$lib/stores/dash-pattern';
	import { defaultStrokeWidth } from '$lib/stores/stroke-width';
	import { defaultStrokeColor } from '$lib/stores/stroke-color';
	import { activeTool } from '$lib/stores/tools';

	function getDefaultStrokeColor(): string {
		return $theme === 'dark' ? '#ffffff' : '#000000';
	}

	let strokeColor = '#000000';
	let fillColor: string | null = null;
	let lineWidth = 2;
	let unifiedColor = '#000000';
	let textOpacity = 1.0;

	$: displayStrokeColor = strokeColor;


	$: strokeColors = $theme === 'dark' 
		? ['#ffffff', '#60a5fa', '#34d399', '#fb7185', '#d97706']
		: ['#000000', '#60a5fa', '#34d399', '#fb7185', '#d97706'];

	const fillColors = [
		'#374151',
		'#1e3a8a',
		'#166534',
		'#991b1b',
		'#78350f'
	];

	let strokeColorPickerButton: HTMLButtonElement;
	let fillColorPickerButton: HTMLButtonElement;
	let stylePanelRef: HTMLDivElement;

	function openStrokeColorPicker(event: MouseEvent) {
		const button = event.currentTarget as HTMLButtonElement;
		const buttonRect = button.getBoundingClientRect();
		const panelRect = stylePanelRef.getBoundingClientRect();
		
		const input = document.createElement('input');
		input.type = 'color';
		input.value = displayStrokeColor;
		input.style.position = 'fixed';
		input.style.left = `${panelRect.left - 250}px`;
		input.style.top = `${buttonRect.top}px`;
		input.style.width = '1px';
		input.style.height = '1px';
		input.style.opacity = '0.01';
		input.style.pointerEvents = 'auto';
		input.style.zIndex = '9999';
		document.body.appendChild(input);
		
		setTimeout(() => {
			input.focus();
			input.click();
		}, 10);
		
		input.onchange = (e) => {
			const selectedColor = (e.target as HTMLInputElement).value;
			const denormalizedColor = $theme === 'dark' && selectedColor === '#000000' ? '#ffffff' : 
			                          $theme !== 'dark' && selectedColor === '#ffffff' ? '#000000' : selectedColor;
			updateStrokeColor(denormalizedColor);
			if (document.body.contains(input)) {
				document.body.removeChild(input);
			}
		};
		input.onblur = () => {
			setTimeout(() => {
				if (document.body.contains(input)) {
					document.body.removeChild(input);
				}
			}, 100);
		};
	}

	function openFillColorPicker(event: MouseEvent) {
		const button = event.currentTarget as HTMLButtonElement;
		const buttonRect = button.getBoundingClientRect();
		const panelRect = stylePanelRef.getBoundingClientRect();
		
		const input = document.createElement('input');
		input.type = 'color';
		input.value = fillColor || '#000000';
		input.style.position = 'fixed';
		input.style.left = `${panelRect.left - 250}px`;
		input.style.top = `${buttonRect.top}px`;
		input.style.width = '1px';
		input.style.height = '1px';
		input.style.opacity = '0.01';
		input.style.pointerEvents = 'auto';
		input.style.zIndex = '9999';
		document.body.appendChild(input);
		
		setTimeout(() => {
			input.focus();
			input.click();
		}, 10);
		
		input.onchange = (e) => {
			updateFillColor((e.target as HTMLInputElement).value);
			if (document.body.contains(input)) {
				document.body.removeChild(input);
			}
		};
		input.onblur = () => {
			setTimeout(() => {
				if (document.body.contains(input)) {
					document.body.removeChild(input);
				}
			}, 100);
		};
	}

	function isLightColor(hexColor: string): boolean {
		const hex = hexColor.replace('#', '');
		const r = parseInt(hex.slice(0, 2), 16);
		const g = parseInt(hex.slice(2, 4), 16);
		const b = parseInt(hex.slice(4, 6), 16);
		const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
		return luminance > 0.5;
	}

	let isUpdatingColor = false;

	$: {
		if (!isUpdatingColor && $theme) {
			const totalSelected =
				$selectedRectangles.length +
				$selectedEllipses.length +
				$selectedDiamonds.length +
				$selectedLines.length +
				$selectedArrows.length +
				$selectedImages.length +
				$selectedTexts.length +
				$selectedPaths.length;

			if (totalSelected === 0 && $selectedGroups.length === 0) {
				strokeColor = getDefaultStrokeColor();
				fillColor = null;
				lineWidth = 2;
				textOpacity = 1.0;
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
					.map((s) => {
						if ((s as any).color !== undefined) {
							return (s as any).color || getDefaultStrokeColor();
						}
						return (s as any).stroke_color || getDefaultStrokeColor();
					})
					.filter((c, i, arr) => arr.indexOf(c) === i);
				if (strokeColors.length === 1) {
					let newStrokeColor = strokeColors[0];

					const oppositeThemeDefault = $theme === 'dark' ? '#000000' : '#ffffff';
					const currentThemeDefault = getDefaultStrokeColor();

					if (newStrokeColor === oppositeThemeDefault) {
						newStrokeColor = currentThemeDefault;
					}

					if (newStrokeColor !== strokeColor) {
						strokeColor = newStrokeColor;
					}
				}

				const fillColors = shapes
					.map((s) => (s as any).fill_color ?? null)
					.filter((c, i, arr) => arr.indexOf(c) === i);
				fillColor = fillColors.length === 1 ? fillColors[0] : null;

				const nonTextShapes = shapes.filter((s) => (s as any).color === undefined);
				const lineWidths = nonTextShapes
					.map((s) => (s as any).line_width ?? 2)
					.filter((w, i, arr) => arr.indexOf(w) === i);
				lineWidth = lineWidths.length === 1 ? lineWidths[0] : 2;
				if (lineWidths.length === 1 && (lineWidths[0] === 1 || lineWidths[0] === 2 || lineWidths[0] === 4)) {
					defaultStrokeWidth.set(lineWidths[0]);
				}

				if ($selectedTexts.length > 0) {
					const opacities = $selectedTexts
						.map((t) => (t as any).opacity ?? 1.0)
						.filter((o, i, arr) => arr.indexOf(o) === i);
					textOpacity = opacities.length === 1 ? opacities[0] : 1.0;
				} else {
					textOpacity = 1.0;
				}

			}
		}
	}


	async function updateStrokeColor(color: string) {
		isUpdatingColor = true;
		strokeColor = color;
		defaultStrokeColor.set(color);
		if (!$editorApi) {
			isUpdatingColor = false;
			return;
		}

		const selectedRects = get(selectedRectangles);
		const selectedElls = get(selectedEllipses);
		const selectedDias = get(selectedDiamonds);
		const selectedLns = get(selectedLines);
		const selectedArrs = get(selectedArrows);
		const selectedPths = get(selectedPaths);
		const selectedTxts = get(selectedTexts);

		selectedRects.forEach((rect: Rectangle) => {
			$editorApi.set_rectangle_stroke_color(BigInt(rect.id), color, false);
		});
		selectedElls.forEach((ellipse: Ellipse) => {
			$editorApi.set_ellipse_stroke_color(BigInt(ellipse.id), color, false);
		});
		selectedDias.forEach((diamond: Diamond) => {
			$editorApi.set_diamond_stroke_color(BigInt(diamond.id), color, false);
		});
		selectedLns.forEach((line: Line) => {
			$editorApi.set_line_stroke_color(BigInt(line.id), color, false);
		});
		selectedArrs.forEach((arrow: Arrow) => {
			$editorApi.set_arrow_stroke_color(BigInt(arrow.id), color, false);
		});
		selectedPths.forEach((path: Path) => {
			$editorApi.set_path_stroke_color(BigInt(path.id), color, false);
		});
		selectedTxts.forEach((text: Text) => {
			$editorApi.set_text_color(BigInt(text.id), color, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		await tick();
		isUpdatingColor = false;
		saveStateToLocalStorage();
	}

	async function updateFillColor(color: string | null) {
		isUpdatingColor = true;
		fillColor = color;
		if (!$editorApi) {
			isUpdatingColor = false;
			return;
		}

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
		await tick();
		isUpdatingColor = false;
		saveStateToLocalStorage();
	}

	function updateLineWidth(width: number) {
		lineWidth = Math.max(0.1, width);
		defaultStrokeWidth.set(lineWidth);
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


	function updateFontSize(fontSize: number) {
		if (!$editorApi) return;

		$selectedTexts.forEach((text) => {
			$editorApi.set_text_font_size(BigInt(text.id), fontSize, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateFontWeight(fontWeight: string) {
		if (!$editorApi) return;

		$selectedTexts.forEach((text) => {
			$editorApi.set_text_font_weight(BigInt(text.id), fontWeight, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateTextAlign(textAlign: string) {
		if (!$editorApi) return;

		$selectedTexts.forEach((text) => {
			$editorApi.set_text_text_align(BigInt(text.id), textAlign, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function updateTextOpacity(opacity: number) {
		textOpacity = opacity;
		if (!$editorApi) return;

		const updatedTexts = $texts.map((text) => {
			if ($selectedTexts.some(st => st.id === text.id)) {
				return { ...text, opacity };
			}
			return text;
		});
		texts.set(updatedTexts);

		const updatedSelectedTexts = $selectedTexts.map((text) => ({
			...text,
			opacity
		}));
		selectedTexts.set(updatedSelectedTexts);

		$editorApi.save_snapshot();
		saveStateToLocalStorage();
	}

	function updateStrokeWidthType(width: number) {
		lineWidth = width;
		defaultStrokeWidth.set(width);
		if (!$editorApi) return;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_line_width(BigInt(rect.id), width, false);
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_ellipse_line_width(BigInt(ellipse.id), width, false);
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_line_width(BigInt(diamond.id), width, false);
		});
		$selectedLines.forEach((line) => {
			$editorApi.set_line_line_width(BigInt(line.id), width, false);
		});
		$selectedArrows.forEach((arrow) => {
			$editorApi.set_arrow_line_width(BigInt(arrow.id), width, false);
		});
		$selectedPaths.forEach((path) => {
			$editorApi.set_path_line_width(BigInt(path.id), width, false);
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
		$selectedPaths.forEach(p => $editorApi!.bring_shape_to_front(BigInt(p.id)));
		$selectedImages.forEach(i => $editorApi!.bring_shape_to_front(BigInt(i.id)));
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
		$selectedPaths.forEach(p => $editorApi!.bring_shape_forward(BigInt(p.id)));
		$selectedImages.forEach(i => $editorApi!.bring_shape_forward(BigInt(i.id)));
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
		$selectedPaths.forEach(p => $editorApi!.send_shape_backward(BigInt(p.id)));
		$selectedImages.forEach(i => $editorApi!.send_shape_backward(BigInt(i.id)));
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
		$selectedPaths.forEach(p => $editorApi!.send_shape_to_back(BigInt(p.id)));
		$selectedImages.forEach(i => $editorApi!.send_shape_to_back(BigInt(i.id)));
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
		const selectedImageIds = new Set($selectedImages.map(i => i.id));
		const selectedPathIds = new Set($selectedPaths.map(p => p.id));
		
		const allRectangles = api.get_rectangles() as Rectangle[];
		const allEllipses = api.get_ellipses() as Ellipse[];
		const allLines = api.get_lines() as Line[];
		const allArrows = api.get_arrows() as Arrow[];
		const allDiamonds = api.get_diamonds() as Diamond[];
		const allImages = api.get_images() as Image[];
		const allPaths = api.get_paths() as Path[];
		const allTexts = api.get_texts() as Text[];
		
		const existingTexts = $texts;
		const opacityMap = new Map<number, number>();
		existingTexts.forEach((text) => {
			if ((text as any).opacity !== undefined) {
				opacityMap.set(text.id, (text as any).opacity);
			}
		});

		const textsWithOpacity = allTexts.map((text) => {
			const opacity = opacityMap.get(text.id);
			if (opacity !== undefined) {
				return { ...text, opacity };
			}
			return text;
		});
		
		rectangles.set(allRectangles);
		ellipses.set(allEllipses);
		lines.set(allLines);
		arrows.set(allArrows);
		diamonds.set(allDiamonds);
		images.set(allImages);
		paths.set(allPaths);
		texts.set(textsWithOpacity);
		
		selectedRectangles.set(allRectangles.filter(r => selectedRectIds.has(r.id)));
		selectedEllipses.set(allEllipses.filter(e => selectedEllipseIds.has(e.id)));
		selectedLines.set(allLines.filter(l => selectedLineIds.has(l.id)));
		selectedArrows.set(allArrows.filter(a => selectedArrowIds.has(a.id)));
		selectedDiamonds.set(allDiamonds.filter(d => selectedDiamondIds.has(d.id)));
		selectedImages.set(allImages.filter(i => selectedImageIds.has(i.id)));
		selectedPaths.set(allPaths.filter(p => selectedPathIds.has(p.id)));
		selectedTexts.set(textsWithOpacity.filter(t => {
			const selectedIds = new Set($selectedTexts.map(st => st.id));
			return selectedIds.has(t.id);
		}));
	}

	$: hasSelection =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedImages.length > 0 ||
		$selectedTexts.length > 0 ||
		$selectedPaths.length > 0;

	$: hasFillableShapes =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0;

	$: hasEditableEdges =
		$selectedRectangles.length > 0 ||
		$selectedDiamonds.length > 0;

	$: hasDashPatternShapes =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedImages.length > 0;

	$: showDashPatternControls =
		hasDashPatternShapes ||
		$activeTool === 'rectangle' ||
		$activeTool === 'ellipse' ||
		$activeTool === 'diamond' ||
		$activeTool === 'line' ||
		$activeTool === 'arrow';

	$: hasImagesOnly = $selectedImages.length > 0 &&
		$selectedRectangles.length === 0 &&
		$selectedEllipses.length === 0 &&
		$selectedDiamonds.length === 0 &&
		$selectedLines.length === 0 &&
		$selectedArrows.length === 0 &&
		$selectedTexts.length === 0 &&
		$selectedPaths.length === 0;

	$: hasTexts = $selectedTexts.length > 0;

	$: hasShapes =
		$selectedRectangles.length > 0 ||
		$selectedEllipses.length > 0 ||
		$selectedDiamonds.length > 0 ||
		$selectedLines.length > 0 ||
		$selectedArrows.length > 0 ||
		$selectedTexts.length > 0 ||
		$selectedPaths.length > 0;

	$: isSingleSelection =
		($selectedRectangles.length + $selectedEllipses.length + $selectedDiamonds.length +
		 $selectedLines.length + $selectedArrows.length + $selectedTexts.length + $selectedPaths.length) === 1;

	async function updateUnifiedColor(color: string) {
		unifiedColor = color;
		if (!$editorApi) return;

		const selectedRects = get(selectedRectangles);
		const selectedElls = get(selectedEllipses);
		const selectedDias = get(selectedDiamonds);
		const selectedLns = get(selectedLines);
		const selectedArrs = get(selectedArrows);
		const selectedPths = get(selectedPaths);
		selectedRects.forEach((rect: Rectangle) => {
			$editorApi.set_rectangle_stroke_color(BigInt(rect.id), color, false);
		});
		selectedElls.forEach((ellipse: Ellipse) => {
			$editorApi.set_ellipse_stroke_color(BigInt(ellipse.id), color, false);
		});
		selectedDias.forEach((diamond: Diamond) => {
			$editorApi.set_diamond_stroke_color(BigInt(diamond.id), color, false);
		});
		selectedLns.forEach((line: Line) => {
			$editorApi.set_line_stroke_color(BigInt(line.id), color, false);
		});
		selectedArrs.forEach((arrow: Arrow) => {
			$editorApi.set_arrow_stroke_color(BigInt(arrow.id), color, false);
		});
		selectedPths.forEach((path: Path) => {
			$editorApi.set_path_stroke_color(BigInt(path.id), color, false);
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

		const selectedIds = new Set([
			...$selectedRectangles.map(r => r.id),
			...$selectedEllipses.map(e => e.id),
			...$selectedLines.map(l => l.id),
			...$selectedArrows.map(a => a.id),
			...$selectedDiamonds.map(d => d.id),
			...$selectedTexts.map(t => t.id),
			...$selectedPaths.map(p => p.id),
			...$selectedImages.map(i => i.id)
		]);

		const allRectangles = Array.from($editorApi.get_rectangles() as Rectangle[]);
		const allEllipses = Array.from($editorApi.get_ellipses() as Ellipse[]);
		const allLines = Array.from($editorApi.get_lines() as Line[]);
		const allArrows = Array.from($editorApi.get_arrows() as Arrow[]);
		const allDiamonds = Array.from($editorApi.get_diamonds() as Diamond[]);
		const allTexts = Array.from($editorApi.get_texts() as Text[]);
		const allPaths = Array.from($editorApi.get_paths() as Path[]);
		const allImages = Array.from($editorApi.get_images() as Image[]);

		const currentRectangles = allRectangles.filter(r => selectedIds.has(r.id));
		const currentEllipses = allEllipses.filter(e => selectedIds.has(e.id));
		const currentLines = allLines.filter(l => selectedIds.has(l.id));
		const currentArrows = allArrows.filter(a => selectedIds.has(a.id));
		const currentDiamonds = allDiamonds.filter(d => selectedIds.has(d.id));
		const currentTexts = allTexts.filter((t: Text) => selectedIds.has(t.id));
		const currentPaths = allPaths.filter(p => selectedIds.has(p.id));
		const currentImages = allImages.filter(i => selectedIds.has(i.id));

		copyToClipboard(currentRectangles, currentEllipses, currentLines, currentArrows, currentDiamonds, currentImages, currentTexts, currentPaths);
		const clipboard = getClipboard();

		const bounds: Array<{ minX: number; minY: number }> = [];
		clipboard.rectangles.forEach(r => bounds.push({ minX: r.position.x, minY: r.position.y }));
		clipboard.ellipses.forEach(e => bounds.push({ minX: e.position.x - e.radius_x, minY: e.position.y - e.radius_y }));
		clipboard.diamonds.forEach(d => bounds.push({ minX: d.position.x, minY: d.position.y }));
		clipboard.lines.forEach(l => bounds.push({ minX: Math.min(l.start.x, l.end.x), minY: Math.min(l.start.y, l.end.y) }));
		clipboard.arrows.forEach(a => bounds.push({ minX: Math.min(a.start.x, a.end.x), minY: Math.min(a.start.y, a.end.y) }));
		clipboard.images.forEach(i => bounds.push({ minX: i.position.x, minY: i.position.y }));
		clipboard.texts.forEach(t => bounds.push({ minX: t.position.x, minY: t.position.y }));
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
		const pathIds = $selectedPaths.map(path => path.id);
		const imageIds = $selectedImages.map(image => image.id);
		const textIds = $selectedTexts.map(text => text.id);

		deleteShapes(rectangleIds, ellipseIds, lineIds, arrowIds, diamondIds, textIds, pathIds, imageIds);
	}

	function handleLock() {
		if (!$editorApi) return;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_element_locked(BigInt(rect.id), true, false);
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_element_locked(BigInt(ellipse.id), true, false);
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_element_locked(BigInt(diamond.id), true, false);
		});
		$selectedLines.forEach((line) => {
			$editorApi.set_element_locked(BigInt(line.id), true, false);
		});
		$selectedArrows.forEach((arrow) => {
			$editorApi.set_element_locked(BigInt(arrow.id), true, false);
		});
		$selectedPaths.forEach((path) => {
			$editorApi.set_element_locked(BigInt(path.id), true, false);
		});
		$selectedImages.forEach((image) => {
			$editorApi.set_element_locked(BigInt(image.id), true, false);
		});
		$selectedTexts.forEach((text) => {
			$editorApi.set_element_locked(BigInt(text.id), true, false);
		});
		$selectedGroups.forEach((group) => {
			$editorApi.set_element_locked(BigInt(group.id), true, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	function handleUnlock() {
		if (!$editorApi) return;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_element_locked(BigInt(rect.id), false, false);
		});
		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_element_locked(BigInt(ellipse.id), false, false);
		});
		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_element_locked(BigInt(diamond.id), false, false);
		});
		$selectedLines.forEach((line) => {
			$editorApi.set_element_locked(BigInt(line.id), false, false);
		});
		$selectedArrows.forEach((arrow) => {
			$editorApi.set_element_locked(BigInt(arrow.id), false, false);
		});
		$selectedPaths.forEach((path) => {
			$editorApi.set_element_locked(BigInt(path.id), false, false);
		});
		$selectedImages.forEach((image) => {
			$editorApi.set_element_locked(BigInt(image.id), false, false);
		});
		$selectedTexts.forEach((text) => {
			$editorApi.set_element_locked(BigInt(text.id), false, false);
		});
		$selectedGroups.forEach((group) => {
			$editorApi.set_element_locked(BigInt(group.id), false, false);
		});

		$editorApi.save_snapshot();
		updateStores();
		saveStateToLocalStorage();
	}

	$: allLocked = $editorApi ? (() => {
		const allSelected = [
			...$selectedRectangles.map(r => ({ id: r.id })),
			...$selectedEllipses.map(e => ({ id: e.id })),
			...$selectedDiamonds.map(d => ({ id: d.id })),
			...$selectedLines.map(l => ({ id: l.id })),
			...$selectedArrows.map(a => ({ id: a.id })),
			...$selectedPaths.map(p => ({ id: p.id })),
			...$selectedImages.map(i => ({ id: i.id })),
			...$selectedTexts.map(t => ({ id: t.id })),
			...$selectedGroups.map(g => ({ id: g.id }))
		];
		if (allSelected.length === 0) return false;
		return allSelected.every(el => $editorApi.is_element_locked(BigInt(el.id)));
	})() : false;

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

	function handleDashPatternChange(pattern: DashPattern) {
		if (!$editorApi) return;

		dashPattern.set(pattern);
		const patternStr = pattern;

		$selectedRectangles.forEach((rect) => {
			$editorApi.set_rectangle_dash_pattern(BigInt(rect.id), patternStr, false);
		});

		$selectedEllipses.forEach((ellipse) => {
			$editorApi.set_ellipse_dash_pattern(BigInt(ellipse.id), patternStr, false);
		});

		$selectedDiamonds.forEach((diamond) => {
			$editorApi.set_diamond_dash_pattern(BigInt(diamond.id), patternStr, false);
		});

		$selectedLines.forEach((line) => {
			$editorApi.set_line_dash_pattern(BigInt(line.id), patternStr, false);
		});

		$selectedArrows.forEach((arrow) => {
			$editorApi.set_arrow_dash_pattern(BigInt(arrow.id), patternStr, false);
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

	$: {
		if (hasDashPatternShapes) {
			const allDashPatterns: string[] = [];
			$selectedRectangles.forEach(r => {
				const dp = r.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedEllipses.forEach(e => {
				const dp = e.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedDiamonds.forEach(d => {
				const dp = d.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedLines.forEach(l => {
				const dp = l.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});
			$selectedArrows.forEach(a => {
				const dp = a.dash_pattern || 'solid';
				allDashPatterns.push(dp);
			});

			const allSolid = allDashPatterns.length > 0 && allDashPatterns.every(p => p === 'solid');
			const allDashed = allDashPatterns.length > 0 && allDashPatterns.every(p => p === 'dashed');
			const allDotted = allDashPatterns.length > 0 && allDashPatterns.every(p => p === 'dotted');

			if (allSolid) {
				dashPattern.set('solid');
			} else if (allDashed) {
				dashPattern.set('dashed');
			} else if (allDotted) {
				dashPattern.set('dotted');
			}
		}
	}</script>

{#if hasSelection}
	<div bind:this={stylePanelRef} class={`absolute top-2 right-2 z-50 backdrop-blur-sm border rounded-lg p-3 w-[240px] min-w-[240px] min-h-[100px] overflow-hidden ${$theme === 'dark' ? 'bg-stone-800/95 border-stone-700/50' : 'bg-white/95 border-stone-200/50'} shadow-lg`}>
		<div class="space-y-2.5 min-w-0">
			{#if !hasImagesOnly}
				{#if hasShapes}
					<div class="space-y-1.5">
						<fieldset class="flex flex-col gap-2 w-full min-w-0">
							<legend class={`text-xs font-medium mb-1 ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Stroke</legend>
							<div class="flex items-center gap-1.5 w-full">
								<button
									type="button"
									bind:this={strokeColorPickerButton}
									on:click={openStrokeColorPicker}
									class={`w-8 h-8 rounded-full border-2 transition-all hover:scale-105 ${$theme === 'dark' ? 'border-stone-500' : 'border-stone-400'}`}
									style="background-color: {displayStrokeColor};"
									title="Current color - click to change"
								>
								</button>
								{#each strokeColors as color}
									<button
										type="button"
										on:click={() => updateStrokeColor(color)}
										class={`rounded-full border transition-all hover:scale-105 ${displayStrokeColor === color ? 'w-8 h-8' : 'w-7 h-7'} ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-300'}`}
										style="background-color: {color};"
										title={color}
									>
									</button>
								{/each}
							</div>
						</fieldset>
					</div>
				{/if}
			{/if}

			{#if false}
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
						<legend class={`text-xs font-medium mb-1 ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Background</legend>
						<div class="flex items-center gap-1.5 w-full">
							{#if fillColor === null}
								<button
									type="button"
									bind:this={fillColorPickerButton}
									on:click={openFillColorPicker}
									class={`w-8 h-8 rounded-full border-2 transition-all hover:scale-105 relative ${$theme === 'dark' ? 'border-stone-500' : 'border-stone-400'}`}
									title="No fill - click to change"
								>
									<div class={`w-full h-full rounded-full absolute inset-0 ${$theme === 'dark' ? 'bg-stone-700' : 'bg-stone-200'}`} style="background-image: repeating-conic-gradient(${$theme === 'dark' ? '#374151' : '#e5e7eb'} 0% 25%, transparent 0% 50%); background-size: 50% 50%;"></div>
								</button>
							{:else}
								<button
									type="button"
									bind:this={fillColorPickerButton}
									on:click={openFillColorPicker}
									class={`w-8 h-8 rounded-full border-2 transition-all hover:scale-105 ${$theme === 'dark' ? 'border-stone-500' : 'border-stone-400'}`}
									style="background-color: {fillColor};"
									title="Current color - click to change"
								>
								</button>
							{/if}
							{#each fillColors as color}
								<button
									type="button"
									on:click={() => updateFillColor(color)}
									class={`rounded-full border transition-all hover:scale-105 ${fillColor === color ? 'w-8 h-8' : 'w-7 h-7'} ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-300'}`}
									style="background-color: {color};"
									title={color}
								>
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

			{#if (hasShapes || $activeTool === 'rectangle' || $activeTool === 'ellipse' || $activeTool === 'diamond' || $activeTool === 'line' || $activeTool === 'arrow' || $activeTool === 'freehand') && $selectedTexts.length === 0}
				<div class="space-y-1.5">
					<fieldset class="space-y-1.5">
						<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Stroke width</legend>
						<div class="flex items-center gap-1">
							<button
								on:click={() => updateStrokeWidthType(1)}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$defaultStrokeWidth === 1 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Thin"
								aria-label="Thin stroke"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">
									<line x1="3" y1="12" x2="21" y2="12"></line>
								</svg>
							</button>
							<button
								on:click={() => updateStrokeWidthType(2)}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$defaultStrokeWidth === 2 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Medium"
								aria-label="Medium stroke"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="3" y1="12" x2="21" y2="12"></line>
								</svg>
							</button>
							<button
								on:click={() => updateStrokeWidthType(4)}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$defaultStrokeWidth === 4 ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Thick"
								aria-label="Thick stroke"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
									<line x1="3" y1="12" x2="21" y2="12"></line>
								</svg>
							</button>
						</div>
					</fieldset>
				</div>
			{/if}

			{#if hasTexts}
				<div class="space-y-1.5">
					<fieldset class="flex flex-col gap-2 w-full min-w-0">
						<legend class={`text-xs font-medium mb-1 ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Text Align</legend>
						<div class="flex items-center gap-1">
							<button
								on:click={() => updateTextAlign('left')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Left align"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="17" y1="10" x2="3" y2="10"></line>
									<line x1="21" y1="6" x2="3" y2="6"></line>
									<line x1="21" y1="14" x2="3" y2="14"></line>
									<line x1="17" y1="18" x2="3" y2="18"></line>
								</svg>
							</button>
							<button
								on:click={() => updateTextAlign('center')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Center align"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="18" y1="10" x2="6" y2="10"></line>
									<line x1="21" y1="6" x2="3" y2="6"></line>
									<line x1="21" y1="14" x2="3" y2="14"></line>
									<line x1="18" y1="18" x2="6" y2="18"></line>
								</svg>
							</button>
							<button
								on:click={() => updateTextAlign('right')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700'}`}
								title="Right align"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="21" y1="10" x2="7" y2="10"></line>
									<line x1="21" y1="6" x2="3" y2="6"></line>
									<line x1="21" y1="14" x2="3" y2="14"></line>
									<line x1="21" y1="18" x2="7" y2="18"></line>
								</svg>
							</button>
						</div>
					</fieldset>
				</div>

				<div class="space-y-1.5">
					<fieldset class="flex flex-col gap-2 w-full min-w-0">
						<legend class={`text-xs font-medium mb-1 ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Opacity</legend>
						<div class="flex items-center gap-2 min-w-0">
							<input
								type="range"
								min="0"
								max="1"
								step="0.01"
								bind:value={textOpacity}
								on:input={(e) => updateTextOpacity(parseFloat((e.target as HTMLInputElement).value))}
								class={`flex-1 min-w-0 h-1 rounded-lg appearance-none cursor-pointer ${$theme === 'dark' ? 'bg-stone-600 accent-stone-400' : 'bg-stone-200 accent-stone-600'}`}
								aria-label="Text opacity"
							/>
							<input
								type="number"
								bind:value={textOpacity}
								on:input={(e) => updateTextOpacity(parseFloat((e.target as HTMLInputElement).value))}
								min="0"
								max="1"
								step="0.01"
								class={`w-16 px-1.5 py-1 text-xs border rounded focus:outline-none focus:ring-1 shrink-0 ${$theme === 'dark' ? 'border-stone-600 bg-stone-700 text-stone-200 focus:ring-stone-500' : 'border-stone-200 bg-stone-50 focus:ring-stone-400'}`}
								aria-label="Opacity value"
							/>
						</div>
					</fieldset>
				</div>
			{/if}

			{#if showDashPatternControls}
				<div class="space-y-1.5">
					<fieldset class="space-y-1.5">
						<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Stroke Style</legend>
						<div class="flex items-center gap-1">
							<button
								on:click={() => handleDashPatternChange('solid')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$dashPattern === 'solid' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Solid"
								aria-label="Solid line"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
									<line x1="3" y1="12" x2="21" y2="12"></line>
								</svg>
							</button>
							<button
								on:click={() => handleDashPatternChange('dashed')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$dashPattern === 'dashed' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Dashed"
								aria-label="Dashed line"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="4 4">
									<line x1="3" y1="12" x2="21" y2="12"></line>
								</svg>
							</button>
							<button
								on:click={() => handleDashPatternChange('dotted')}
								class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${$dashPattern === 'dotted' ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
								title="Dotted"
								aria-label="Dotted line"
							>
								<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" stroke-dasharray="2 2">
									<line x1="3" y1="12" x2="21" y2="12"></line>
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
					<legend class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Lock</legend>
					<div class="flex items-center gap-1">
						<button
							on:click={handleLock}
							class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${allLocked ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
							title="Lock"
							aria-label="Lock"
						>
							<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
								<path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
							</svg>
						</button>
						<button
							on:click={handleUnlock}
							class={`flex flex-1 items-center justify-center p-1.5 rounded transition-colors ${!allLocked ? ($theme === 'dark' ? 'bg-stone-600 hover:bg-stone-500 text-stone-200 ring-2 ring-stone-500' : 'bg-stone-300 hover:bg-stone-400 text-stone-800 ring-2 ring-stone-400') : ($theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600 text-stone-200' : 'bg-stone-100 hover:bg-stone-200 text-stone-700')}`}
							title="Unlock"
							aria-label="Unlock"
						>
							<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
								<path d="M7 11V7a5 5 0 0 1 9.9-1"></path>
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
