import type { Rectangle } from '$lib/stores/editor';

export function renderRectangles(
	ctx: CanvasRenderingContext2D,
	canvas: HTMLCanvasElement,
	rectangles: Rectangle[],
	selectedRectangles: Rectangle[],
	viewportOffset: { x: number, y: number }
): void {
	ctx.clearRect(0, 0, canvas.width, canvas.height);
	ctx.fillStyle = '#fafaf9';
	ctx.fillRect(0, 0, canvas.width, canvas.height);

	ctx.save();
	ctx.translate(viewportOffset.x, viewportOffset.y);

	rectangles.forEach((rect) => {
		const isSelected = selectedRectangles.some(selected => selected.id === rect.id);
		
		const x = rect.position.x;
		const y = rect.position.y;
		const w = rect.width;
		const h = rect.height;
		
		if (isSelected) {
			ctx.fillStyle = '#ef4444';
			ctx.strokeStyle = '#dc2626';
			ctx.lineWidth = 2.5;
		} else {
			ctx.fillStyle = '#3b82f6';
			ctx.strokeStyle = '#2563eb';
			ctx.lineWidth = 2;
		}
		
		ctx.fillRect(x, y, w, h);
		ctx.strokeRect(x, y, w, h);
		
		if (!isSelected) {
			ctx.strokeStyle = '#60a5fa';
			ctx.lineWidth = 1;
			ctx.strokeRect(x + 1, y + 1, w - 2, h - 2);
		}
	});

	ctx.restore();
}

