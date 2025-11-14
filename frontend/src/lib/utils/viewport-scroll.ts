import { viewportOffset } from '$lib/stores/editor';

export function handleViewportScroll(event: WheelEvent): void {
	event.preventDefault();
	const scrollSpeed = 1.0

	if (event.shiftKey) {
		const horizontalDelta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
		viewportOffset.update(offset => ({
			x: offset.x - horizontalDelta * scrollSpeed,
			y: offset.y,
		}));
	} else {
		viewportOffset.update(offset => ({
			x: offset.x - event.deltaX * scrollSpeed,
			y: offset.y - event.deltaY * scrollSpeed,
		}));
	}
}
