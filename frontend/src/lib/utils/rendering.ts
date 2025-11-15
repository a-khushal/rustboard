import type { Rectangle } from '$lib/stores/editor';

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

