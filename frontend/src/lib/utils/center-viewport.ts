import { get } from 'svelte/store';
import { rectangles, ellipses, lines, arrows, viewportOffset, zoom } from '$lib/stores/editor';
import type { Rectangle, Ellipse, Line, Arrow } from '$lib/stores/editor';

function getRectangleBounds(rect: Rectangle): { minX: number; minY: number; maxX: number; maxY: number } {
	return {
		minX: rect.position.x,
		minY: rect.position.y,
		maxX: rect.position.x + rect.width,
		maxY: rect.position.y + rect.height
	};
}

function getEllipseBounds(ellipse: Ellipse): { minX: number; minY: number; maxX: number; maxY: number } {
	return {
		minX: ellipse.position.x - ellipse.radius_x,
		minY: ellipse.position.y - ellipse.radius_y,
		maxX: ellipse.position.x + ellipse.radius_x,
		maxY: ellipse.position.y + ellipse.radius_y
	};
}

function getLineBounds(line: Line): { minX: number; minY: number; maxX: number; maxY: number } {
	return {
		minX: Math.min(line.start.x, line.end.x),
		minY: Math.min(line.start.y, line.end.y),
		maxX: Math.max(line.start.x, line.end.x),
		maxY: Math.max(line.start.y, line.end.y)
	};
}

function getArrowBounds(arrow: Arrow): { minX: number; minY: number; maxX: number; maxY: number } {
	return {
		minX: Math.min(arrow.start.x, arrow.end.x),
		minY: Math.min(arrow.start.y, arrow.end.y),
		maxX: Math.max(arrow.start.x, arrow.end.x),
		maxY: Math.max(arrow.start.y, arrow.end.y)
	};
}

export function centerViewportOnShapes(): void {
	const $rectangles = get(rectangles);
	const $ellipses = get(ellipses);
	const $lines = get(lines);
	const $arrows = get(arrows);
	const $zoom = get(zoom);

	if ($rectangles.length === 0 && $ellipses.length === 0 && $lines.length === 0 && $arrows.length === 0) {
		return;
	}

	let minX = Infinity;
	let minY = Infinity;
	let maxX = -Infinity;
	let maxY = -Infinity;

	$rectangles.forEach(rect => {
		const bounds = getRectangleBounds(rect);
		minX = Math.min(minX, bounds.minX);
		minY = Math.min(minY, bounds.minY);
		maxX = Math.max(maxX, bounds.maxX);
		maxY = Math.max(maxY, bounds.maxY);
	});

	$ellipses.forEach(ellipse => {
		const bounds = getEllipseBounds(ellipse);
		minX = Math.min(minX, bounds.minX);
		minY = Math.min(minY, bounds.minY);
		maxX = Math.max(maxX, bounds.maxX);
		maxY = Math.max(maxY, bounds.maxY);
	});

	$lines.forEach(line => {
		const bounds = getLineBounds(line);
		minX = Math.min(minX, bounds.minX);
		minY = Math.min(minY, bounds.minY);
		maxX = Math.max(maxX, bounds.maxX);
		maxY = Math.max(maxY, bounds.maxY);
	});

	$arrows.forEach(arrow => {
		const bounds = getArrowBounds(arrow);
		minX = Math.min(minX, bounds.minX);
		minY = Math.min(minY, bounds.minY);
		maxX = Math.max(maxX, bounds.maxX);
		maxY = Math.max(maxY, bounds.maxY);
	});

	const centerX = (minX + maxX) / 2;
	const centerY = (minY + maxY) / 2;

	const screenCenterX = window.innerWidth / 2;
	const screenCenterY = window.innerHeight / 2;

	const newOffsetX = screenCenterX - centerX * $zoom;
	const newOffsetY = screenCenterY - centerY * $zoom;

	viewportOffset.set({ x: newOffsetX, y: newOffsetY });
}

