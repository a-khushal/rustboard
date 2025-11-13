import type { Rectangle } from '$lib/stores/editor';

export function isPointInRectangle(x: number, y: number, rect: Rectangle): boolean {
	return (
		x >= rect.position.x &&
		x <= rect.position.x + rect.width &&
		y >= rect.position.y &&
		y <= rect.position.y + rect.height
	);
}

