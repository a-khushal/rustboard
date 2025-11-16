import type { Ellipse, Rectangle } from '$lib/stores/editor';

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