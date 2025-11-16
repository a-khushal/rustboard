import type { Rectangle, Ellipse } from '$lib/stores/editor';

export interface ClipboardData {
	rectangles: Rectangle[];
	ellipses: Ellipse[];
}

let clipboard: ClipboardData = {
	rectangles: [],
	ellipses: []
};

export function copyToClipboard(rectangles: Rectangle[], ellipses: Ellipse[]): void {
	clipboard = {
		rectangles: rectangles.map(r => ({ ...r })),
		ellipses: ellipses.map(e => ({ ...e }))
	};
}

export function getClipboard(): ClipboardData {
	return clipboard;
}

export function hasClipboardData(): boolean {
	return clipboard.rectangles.length > 0 || clipboard.ellipses.length > 0;
}

