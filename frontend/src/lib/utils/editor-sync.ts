import { get } from 'svelte/store';
import {
	editorApi,
	rectangles,
	ellipses,
	lines,
	arrows,
	diamonds,
	paths,
	images,
	texts,
	selectedRectangles,
	selectedEllipses,
	selectedLines,
	selectedArrows,
	selectedDiamonds,
	selectedPaths,
	selectedImages,
	selectedTexts,
	type Rectangle,
	type Ellipse,
	type Line,
	type Arrow,
	type Diamond,
	type Path,
	type Image,
	type Text as EditorText,
} from '$lib/stores/editor';

export function updateStores() {
	const api = get(editorApi);
	if (!api) return;

	const $selectedRectangles = get(selectedRectangles);
	const $selectedEllipses = get(selectedEllipses);
	const $selectedLines = get(selectedLines);
	const $selectedArrows = get(selectedArrows);
	const $selectedDiamonds = get(selectedDiamonds);
	const $selectedImages = get(selectedImages);
	const $selectedPaths = get(selectedPaths);
	const $selectedTexts = get(selectedTexts);

	const selectedRectIds = new Set($selectedRectangles.map(r => r.id));
	const selectedEllipseIds = new Set($selectedEllipses.map(e => e.id));
	const selectedLineIds = new Set($selectedLines.map(l => l.id));
	const selectedArrowIds = new Set($selectedArrows.map(a => a.id));
	const selectedDiamondIds = new Set($selectedDiamonds.map(d => d.id));
	const selectedImageIds = new Set($selectedImages.map(i => i.id));
	const selectedPathIds = new Set($selectedPaths.map(p => p.id));
	const selectedTextIds = new Set($selectedTexts.map(t => t.id));

	const allRectangles = api.get_rectangles() as Rectangle[];
	const allEllipses = api.get_ellipses() as Ellipse[];
	const allLines = api.get_lines() as Line[];
	const allArrows = api.get_arrows() as Arrow[];
	const allDiamonds = api.get_diamonds() as Diamond[];
	const allImages = api.get_images() as Image[];
	const allPaths = api.get_paths() as Path[];
	const allTexts = api.get_texts() as EditorText[];

	const $texts = get(texts);
	const opacityMap = new Map<number, number>();
	$texts.forEach((text) => {
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
	selectedTexts.set(allTexts.filter(t => selectedTextIds.has(t.id)));
}

