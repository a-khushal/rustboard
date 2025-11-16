import type { Ellipse, Rectangle, Line } from '$lib/stores/editor';

export function isPointInRectangle(x: number, y: number, rect: Rectangle): boolean {
	return (
		x >= rect.position.x &&
		x <= rect.position.x + rect.width &&
		y >= rect.position.y &&
		y <= rect.position.y + rect.height
	);
}

export function isPointInEllipse(x: number, y: number, ellipse: Ellipse): boolean {
	return (
		(x - ellipse.position.x) ** 2 / ellipse.radius_x ** 2 +
		(y - ellipse.position.y) ** 2 / ellipse.radius_y ** 2 <= 1
	);
}

export function isPointOnLine(x: number, y: number, line: Line, threshold: number = 5): boolean {
	const { start, end } = line;
	const dx = end.x - start.x;
	const dy = end.y - start.y;
	const length = Math.sqrt(dx * dx + dy * dy);
	
	if (length === 0) return false;
	
	const t = Math.max(0, Math.min(1, ((x - start.x) * dx + (y - start.y) * dy) / (length * length)));
	const projX = start.x + t * dx;
	const projY = start.y + t * dy;
	const dist = Math.sqrt((x - projX) ** 2 + (y - projY) ** 2);
	
	return dist <= threshold;
}