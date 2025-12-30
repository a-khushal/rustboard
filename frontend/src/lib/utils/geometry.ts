import type { Ellipse, Rectangle, Line, Arrow, Diamond, Image } from '$lib/stores/editor';

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

export function isPointOnLine(x: number, y: number, line: Line | Arrow, threshold: number = 5): boolean {
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

export function isPointOnPath(x: number, y: number, path: { points: Array<{ x: number; y: number }>; line_width?: number }, threshold?: number): boolean {
	if (path.points.length < 2) return false;
	
	const lineWidth = path.line_width || 2;
	const effectiveThreshold = threshold !== undefined ? threshold : Math.max(lineWidth / 2 + 1, 2);
	
	for (let i = 0; i < path.points.length - 1; i++) {
		const start = path.points[i];
		const end = path.points[i + 1];
		const dx = end.x - start.x;
		const dy = end.y - start.y;
		const length = Math.sqrt(dx * dx + dy * dy);
		
		if (length === 0) continue;
		
		const t = Math.max(0, Math.min(1, ((x - start.x) * dx + (y - start.y) * dy) / (length * length)));
		const projX = start.x + t * dx;
		const projY = start.y + t * dy;
		const dist = Math.sqrt((x - projX) ** 2 + (y - projY) ** 2);
		
		if (dist <= effectiveThreshold) return true;
	}
	
	return false;
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

export function isPointInDiamond(x: number, y: number, diamond: Diamond): boolean {
	const centerX = diamond.position.x + diamond.width / 2;
	const centerY = diamond.position.y + diamond.height / 2;
	const halfWidth = diamond.width / 2;
	const halfHeight = diamond.height / 2;
	const dx = Math.abs(x - centerX);
	const dy = Math.abs(y - centerY);
	return (dx / halfWidth) + (dy / halfHeight) <= 1;
}

export function diamondIntersectsBox(diamond: Diamond, box: { x: number; y: number; width: number; height: number }): boolean {
	const centerX = diamond.position.x + diamond.width / 2;
	const centerY = diamond.position.y + diamond.height / 2;
	const halfWidth = diamond.width / 2;
	const halfHeight = diamond.height / 2;

	const corners = [
		{ x: centerX, y: centerY - halfHeight },
		{ x: centerX + halfWidth, y: centerY },
		{ x: centerX, y: centerY + halfHeight },
		{ x: centerX - halfWidth, y: centerY }
	];

	return corners.every(corner =>
		corner.x >= box.x && corner.x <= box.x + box.width &&
		corner.y >= box.y && corner.y <= box.y + box.height
	);
}

export function isPointInImage(x: number, y: number, image: Image): boolean {
	return (
		x >= image.position.x &&
		x <= image.position.x + image.width &&
		y >= image.position.y &&
		y <= image.position.y + image.height
	);
}

export function imageIntersectsBox(image: Image, box: { x: number; y: number; width: number; height: number }): boolean {
	const imageBounds = {
		x: image.position.x,
		y: image.position.y,
		width: image.width,
		height: image.height
	};
	return (
		imageBounds.x >= box.x &&
		imageBounds.y >= box.y &&
		imageBounds.x + imageBounds.width <= box.x + box.width &&
		imageBounds.y + imageBounds.height <= box.y + box.height
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

export function lineIntersectsBox(line: Line | Arrow, box: { x: number; y: number; width: number; height: number }): boolean {
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
	return lineIntersectsBox(arrow, box);
}

export function getPathBoundingBox(path: { points: Array<{ x: number; y: number }> }): { x: number; y: number; width: number; height: number } | null {
	if (path.points.length === 0) return null;
	
	let minX = path.points[0].x;
	let minY = path.points[0].y;
	let maxX = path.points[0].x;
	let maxY = path.points[0].y;
	
	for (const point of path.points) {
		minX = Math.min(minX, point.x);
		minY = Math.min(minY, point.y);
		maxX = Math.max(maxX, point.x);
		maxY = Math.max(maxY, point.y);
	}
	
	return {
		x: minX,
		y: minY,
		width: maxX - minX,
		height: maxY - minY
	};
}

export function pathIntersectsBox(path: { points: Array<{ x: number; y: number }> }, box: { x: number; y: number; width: number; height: number }): boolean {
	if (path.points.length === 0) return false;
	
	const boxRight = box.x + box.width;
	const boxBottom = box.y + box.height;
	
	for (const point of path.points) {
		if (!(point.x >= box.x && point.x <= boxRight && point.y >= box.y && point.y <= boxBottom)) {
			return false;
		}
	}
	
	return true;
}

