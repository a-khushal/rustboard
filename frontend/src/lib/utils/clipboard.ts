import type { Rectangle, Ellipse, Line, Arrow } from '$lib/stores/editor';

export interface ClipboardData {
	rectangles: Rectangle[];
	ellipses: Ellipse[];
	lines: Line[];
	arrows: Arrow[];
}

let clipboard: ClipboardData = {
	rectangles: [],
	ellipses: [],
	lines: [],
	arrows: []
};

export function copyToClipboard(rectangles: Rectangle[], ellipses: Ellipse[], lines: Line[], arrows: Arrow[]): void {
	clipboard = {
		rectangles: rectangles.map(r => ({ ...r })),
		ellipses: ellipses.map(e => ({ ...e })),
		lines: lines.map(l => ({ ...l })),
		arrows: arrows.map(a => ({ ...a }))
	};
}

export function getClipboard(): ClipboardData {
	return clipboard;
}

export function hasClipboardData(): boolean {
	return clipboard.rectangles.length > 0 || clipboard.ellipses.length > 0 || clipboard.lines.length > 0 || clipboard.arrows.length > 0;
}

