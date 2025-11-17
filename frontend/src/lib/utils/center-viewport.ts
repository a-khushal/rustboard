import { get } from 'svelte/store';
import { rectangles, ellipses, lines, arrows, diamonds, viewportOffset, zoom } from '$lib/stores/editor';
import type { Rectangle, Ellipse, Line, Arrow, Diamond } from '$lib/stores/editor';

export function centerViewportOnShapes(): void {
	const $rectangles = get(rectangles);
	const $ellipses = get(ellipses);
	const $lines = get(lines);
	const $arrows = get(arrows);
	const $diamonds = get(diamonds);
	const $zoom = get(zoom);

	if ($rectangles.length === 0 && $ellipses.length === 0 && $lines.length === 0 && $arrows.length === 0 && $diamonds.length === 0) {
		return;
	}

	let minX = Infinity;
	let minY = Infinity;
	let maxX = -Infinity;
	let maxY = -Infinity;

	$rectangles.forEach(rect => {
		minX = Math.min(minX, rect.position.x);
		minY = Math.min(minY, rect.position.y);
		maxX = Math.max(maxX, rect.position.x + rect.width);
		maxY = Math.max(maxY, rect.position.y + rect.height);
	});

	$ellipses.forEach(ellipse => {
		minX = Math.min(minX, ellipse.position.x - ellipse.radius_x);
		minY = Math.min(minY, ellipse.position.y - ellipse.radius_y);
		maxX = Math.max(maxX, ellipse.position.x + ellipse.radius_x);
		maxY = Math.max(maxY, ellipse.position.y + ellipse.radius_y);
	});

	$lines.forEach(line => {
		minX = Math.min(minX, Math.min(line.start.x, line.end.x));
		minY = Math.min(minY, Math.min(line.start.y, line.end.y));
		maxX = Math.max(maxX, Math.max(line.start.x, line.end.x));
		maxY = Math.max(maxY, Math.max(line.start.y, line.end.y));
	});

	$arrows.forEach(arrow => {
		minX = Math.min(minX, Math.min(arrow.start.x, arrow.end.x));
		minY = Math.min(minY, Math.min(arrow.start.y, arrow.end.y));
		maxX = Math.max(maxX, Math.max(arrow.start.x, arrow.end.x));
		maxY = Math.max(maxY, Math.max(arrow.start.y, arrow.end.y));
	});

	$diamonds.forEach(diamond => {
		minX = Math.min(minX, diamond.position.x);
		minY = Math.min(minY, diamond.position.y);
		maxX = Math.max(maxX, diamond.position.x + diamond.width);
		maxY = Math.max(maxY, diamond.position.y + diamond.height);
	});

	const centerX = (minX + maxX) / 2;
	const centerY = (minY + maxY) / 2;

	const screenCenterX = window.innerWidth / 2;
	const screenCenterY = window.innerHeight / 2;

	const newOffsetX = screenCenterX - centerX * $zoom;
	const newOffsetY = screenCenterY - centerY * $zoom;

	viewportOffset.set({ x: newOffsetX, y: newOffsetY });
}

