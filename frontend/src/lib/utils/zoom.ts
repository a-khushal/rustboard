import { zoom, viewportOffset } from '$lib/stores/editor';
import { get } from 'svelte/store';

const ZOOM_FACTOR = 1.1;
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 32;

function zoomOffset(currentZoom: number, newZoom: number, currentOffset: { x: number; y: number }): { x: number; y: number } {
    const centerX = window.innerWidth / 2;
    const centerY = window.innerHeight / 2;
    const worldX = (centerX - currentOffset.x) / currentZoom;
    const worldY = (centerY - currentOffset.y) / currentZoom;
    return {
        x: centerX - worldX * newZoom,
        y: centerY - worldY * newZoom
    };
}

export function zoomIn() {
    const currentZoom = get(zoom);
    const currentOffset = get(viewportOffset);
    const newZoom = Math.min(MAX_ZOOM, currentZoom * ZOOM_FACTOR);
    const newOffset = zoomOffset(currentZoom, newZoom, currentOffset);
    zoom.set(newZoom);
    viewportOffset.set(newOffset);
}

export function zoomOut() {
    const currentZoom = get(zoom);
    const currentOffset = get(viewportOffset);
    const newZoom = Math.max(MIN_ZOOM, currentZoom / ZOOM_FACTOR);
    const newOffset = zoomOffset(currentZoom, newZoom, currentOffset);
    zoom.set(newZoom);
    viewportOffset.set(newOffset);
}

