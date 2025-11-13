import type { Rectangle } from '$lib/stores/editor';

export function renderRectangles(
	ctx: CanvasRenderingContext2D,
	canvas: HTMLCanvasElement,
	rectangles: Rectangle[],
	selectedRectangles: Rectangle[]
): void {
	ctx.clearRect(0, 0, canvas.width, canvas.height);
	ctx.fillStyle = '#ffffff';
	ctx.fillRect(0, 0, canvas.width, canvas.height);

	rectangles.forEach((rect) => {
		const isSelected = selectedRectangles.some(selected => selected.id === rect.id);
		
		if (isSelected) {
			ctx.fillStyle = '#ef4444';
			ctx.strokeStyle = '#dc2626';
		} else {
			ctx.fillStyle = '#3b82f6';
			ctx.strokeStyle = '#1e40af';
		}
		ctx.lineWidth = 2;
		
		ctx.fillRect(
			rect.position.x,
			rect.position.y,
			rect.width,
			rect.height
		);
		
		ctx.strokeRect(
			rect.position.x,
			rect.position.y,
			rect.width,
			rect.height
		);
	});
}

