export function screenToWorld(
    screenX: number,
    screenY: number,
    offset: { x: number; y: number },
    zoomLevel: number = 1
): { x: number; y: number } {
    return {
        x: (screenX - offset.x) / zoomLevel,
        y: (screenY - offset.y) / zoomLevel,
    };
}
