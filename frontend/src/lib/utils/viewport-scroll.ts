import { viewportOffset, zoom } from '$lib/stores/editor';
import { get } from 'svelte/store';

const ZOOM_FACTOR = 1.1;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 32;

export function handleViewportScroll(event: WheelEvent, canvas: HTMLCanvasElement): void {
	event.preventDefault();

	if (event.ctrlKey || event.metaKey) {
		const currentZoom = get(zoom);
		const currentOffset = get(viewportOffset);
		
		const rect = canvas.getBoundingClientRect();
		const mouseX = event.clientX - rect.left;
		const mouseY = event.clientY - rect.top;
		
		const delta = event.deltaY > 0 ? 1 / ZOOM_FACTOR : ZOOM_FACTOR;
		const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, currentZoom * delta));
		
		const zoomRatio = newZoom / currentZoom;
		const newOffsetX = currentOffset.x + (mouseX - currentOffset.x) * (1 - zoomRatio);
		const newOffsetY = currentOffset.y + (mouseY - currentOffset.y) * (1 - zoomRatio);
		
		viewportOffset.set({ x: newOffsetX, y: newOffsetY });
		zoom.set(newZoom);
	} else if (event.shiftKey) {
		const horizontalDelta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
		viewportOffset.update(offset => ({
			x: offset.x - horizontalDelta,
			y: offset.y,
		}));
	} else {
		viewportOffset.update(offset => ({
			x: offset.x - event.deltaX,
			y: offset.y - event.deltaY,
		}));
	}
}
