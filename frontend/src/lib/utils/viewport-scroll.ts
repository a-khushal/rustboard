import { viewportOffset } from '$lib/stores/editor';

export function handleViewportScroll(event: WheelEvent): void {
	event.preventDefault();

	if (event.shiftKey) {
		const horizontalDelta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
		viewportOffset.update(offset => ({
			x: offset.x - horizontalDelta,
			y: offset.y,
		}));
	} else {
		viewportOffset.update(offset => ({
			x: offset.x,
			y: offset.y - event.deltaY,
		}));
	}
}

