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
		const isSelected = selectedRectangles.some(r => r.id === rect.id);
		
		ctx.fillStyle = isSelected ? '#ef4444' : '#3b82f6';
		ctx.strokeStyle = isSelected ? '#dc2626' : '#1e40af';
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

