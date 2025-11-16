import type { Ellipse, Rectangle } from '$lib/stores/editor';

export function renderRectangles(
	ctx: CanvasRenderingContext2D,
	canvas: HTMLCanvasElement,
	rectangles: Rectangle[],
	selectedRectangles: Rectangle[],
	viewportOffset: { x: number, y: number },
	zoomLevel: number = 1,
	previewRect: { x: number; y: number; width: number; height: number } | null = null
): void {
	ctx.clearRect(0, 0, canvas.width, canvas.height);
	ctx.fillStyle = '#fafaf9';
	ctx.fillRect(0, 0, canvas.width, canvas.height);

	ctx.save();
	ctx.translate(viewportOffset.x, viewportOffset.y);
	ctx.scale(zoomLevel, zoomLevel);

	rectangles.forEach((rect) => {
		const isSelected = selectedRectangles.some(selected => selected.id === rect.id);
		const x = rect.position.x;
		const y = rect.position.y;
		const w = rect.width;
		const h = rect.height;
		
		ctx.strokeStyle = isSelected ? '#ef4444' : '#000000';
		ctx.lineWidth = 2 / zoomLevel;
		ctx.strokeRect(x, y, w, h);
	});

	if (previewRect && previewRect.width > 0 && previewRect.height > 0) {
		ctx.strokeStyle = '#000000';
		ctx.lineWidth = 2 / zoomLevel;
		ctx.globalAlpha = 0.5;
		ctx.strokeRect(previewRect.x, previewRect.y, previewRect.width, previewRect.height);
		ctx.globalAlpha = 1.0;
	}

	ctx.restore();
}

export function renderEllipses(
	ctx: CanvasRenderingContext2D,
	canvas: HTMLCanvasElement,
	ellipses: Ellipse[],
	selectedEllipses: Ellipse[],
	viewportOffset: { x: number, y: number },
	zoomLevel: number = 1,
	previewRect: { x: number; y: number; width: number; height: number } | null = null
): void {
	ctx.clearRect(0, 0, canvas.width, canvas.height);
	ctx.fillStyle = '#fafaf9';
	ctx.fillRect(0, 0, canvas.width, canvas.height);

	ctx.save();
	ctx.translate(viewportOffset.x, viewportOffset.y);
	ctx.scale(zoomLevel, zoomLevel);

	ellipses.forEach((ellipse) => {
		const isSelected = selectedEllipses.some(selected => selected.id === ellipse.id);
		const x = ellipse.position.x;
		const y = ellipse.position.y;
		const rx = ellipse.radius_x;
		const ry = ellipse.radius_y;
		
		ctx.strokeStyle = isSelected ? '#ef4444' : '#000000';
		ctx.lineWidth = 2 / zoomLevel;
		ctx.beginPath();
		ctx.ellipse(x, y, rx, ry, 0, 0, 2 * Math.PI);
		ctx.stroke();
	});

	if (previewRect && previewRect.width > 0 && previewRect.height > 0) {
		ctx.strokeStyle = '#000000';
		ctx.lineWidth = 2 / zoomLevel;
		ctx.globalAlpha = 0.5;
		ctx.beginPath();
		ctx.ellipse(previewRect.x, previewRect.y, previewRect.width / 2, previewRect.height / 2, 0, 0, 2 * Math.PI);
		ctx.stroke();
		ctx.globalAlpha = 1.0;
	}

	ctx.restore();
}