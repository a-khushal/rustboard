import type { Rectangle, Ellipse, Line, Arrow, Diamond, Text, Image, Path } from '$lib/stores/editor';

export interface ClipboardData {
	rectangles: Rectangle[];
	ellipses: Ellipse[];
	lines: Line[];
	arrows: Arrow[];
	diamonds: Diamond[];
	texts: Text[];
	images: Image[];
	paths: Path[];
}

let clipboard: ClipboardData = {
	rectangles: [],
	ellipses: [],
	lines: [],
	arrows: [],
	diamonds: [],
	texts: [],
	images: [],
	paths: []
};

export function copyToClipboard(rectangles: Rectangle[], ellipses: Ellipse[], lines: Line[], arrows: Arrow[], diamonds: Diamond[], texts: Text[], images: Image[], paths: Path[] = []): void {
	clipboard = {
		rectangles: rectangles.map(r => ({ ...r })),
		ellipses: ellipses.map(e => ({ ...e })),
		lines: lines.map(l => ({ ...l })),
		arrows: arrows.map(a => ({ ...a })),
		diamonds: diamonds.map(d => ({ ...d })),
		texts: texts.map(t => ({ ...t })),
		images: images.map(i => ({ ...i })),
		paths: paths.map(p => ({ ...p, points: p.points.map(pt => ({ ...pt })) }))
	};
}

export function getClipboard(): ClipboardData {
	return clipboard;
}

export function hasClipboardData(): boolean {
	return clipboard.rectangles.length > 0 || clipboard.ellipses.length > 0 || clipboard.lines.length > 0 || clipboard.arrows.length > 0 || clipboard.diamonds.length > 0 || clipboard.texts.length > 0 || clipboard.images.length > 0 || clipboard.paths.length > 0;
}

