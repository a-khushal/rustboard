import { zoom, viewportOffset } from '$lib/stores/editor';
import { get } from 'svelte/store';

const ZOOM_FACTOR = 1.1;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 32;

export function zoomIn() {
    const currentZoom = get(zoom);
    const currentOffset = get(viewportOffset);
    const newZoom = Math.min(MAX_ZOOM, currentZoom * ZOOM_FACTOR);

    const centerX = window.innerWidth / 2;
    const centerY = window.innerHeight / 2;

    const worldX = (centerX - currentOffset.x) / currentZoom;
    const worldY = (centerY - currentOffset.y) / currentZoom;

    const newOffsetX = centerX - worldX * newZoom;
    const newOffsetY = centerY - worldY * newZoom;

    zoom.set(newZoom);
    viewportOffset.set({ x: newOffsetX, y: newOffsetY });
}

export function zoomOut() {
    const currentZoom = get(zoom);
    const currentOffset = get(viewportOffset);
    const newZoom = Math.max(MIN_ZOOM, currentZoom / ZOOM_FACTOR);

    const centerX = window.innerWidth / 2;
    const centerY = window.innerHeight / 2;

    const worldX = (centerX - currentOffset.x) / currentZoom;
    const worldY = (centerY - currentOffset.y) / currentZoom;

    const newOffsetX = centerX - worldX * newZoom;
    const newOffsetY = centerY - worldY * newZoom;

    zoom.set(newZoom);
    viewportOffset.set({ x: newOffsetX, y: newOffsetY });
}

