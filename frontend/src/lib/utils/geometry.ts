import type { Ellipse, Rectangle, Line, Arrow } from '$lib/stores/editor';

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

function rectsIntersect(rect1: { x: number; y: number; width: number; height: number }, rect2: { x: number; y: number; width: number; height: number }): boolean {
	return !(
		rect1.x + rect1.width < rect2.x ||
		rect2.x + rect2.width < rect1.x ||
		rect1.y + rect1.height < rect2.y ||
		rect2.y + rect2.height < rect1.y
	);
}

export function rectangleIntersectsBox(rect: Rectangle, box: { x: number; y: number; width: number; height: number }): boolean {
	const rectBounds = { x: rect.position.x, y: rect.position.y, width: rect.width, height: rect.height };
	return (
		rectBounds.x >= box.x &&
		rectBounds.y >= box.y &&
		rectBounds.x + rectBounds.width <= box.x + box.width &&
		rectBounds.y + rectBounds.height <= box.y + box.height
	);
}

export function ellipseIntersectsBox(ellipse: Ellipse, box: { x: number; y: number; width: number; height: number }): boolean {
	const ellipseBounds = {
		x: ellipse.position.x - ellipse.radius_x,
		y: ellipse.position.y - ellipse.radius_y,
		width: ellipse.radius_x * 2,
		height: ellipse.radius_y * 2
	};
	
	if (
		ellipseBounds.x < box.x ||
		ellipseBounds.y < box.y ||
		ellipseBounds.x + ellipseBounds.width > box.x + box.width ||
		ellipseBounds.y + ellipseBounds.height > box.y + box.height
	) {
		return false;
	}
	
	for (let angle = 0; angle < Math.PI * 2; angle += Math.PI / 16) {
		const testX = ellipse.position.x + Math.cos(angle) * ellipse.radius_x;
		const testY = ellipse.position.y + Math.sin(angle) * ellipse.radius_y;
		
		if (
			testX < box.x || testX > box.x + box.width ||
			testY < box.y || testY > box.y + box.height
		) {
			return false;
		}
	}
	
	return true;
}

export function lineIntersectsBox(line: Line, box: { x: number; y: number; width: number; height: number }): boolean {
	const lineStartInBox = (
		line.start.x >= box.x && line.start.x <= box.x + box.width &&
		line.start.y >= box.y && line.start.y <= box.y + box.height
	);
	const lineEndInBox = (
		line.end.x >= box.x && line.end.x <= box.x + box.width &&
		line.end.y >= box.y && line.end.y <= box.y + box.height
	);
	
	return lineStartInBox && lineEndInBox;
}

export function arrowIntersectsBox(arrow: Arrow, box: { x: number; y: number; width: number; height: number }): boolean {
	return lineIntersectsBox(arrow as any, box);
}
