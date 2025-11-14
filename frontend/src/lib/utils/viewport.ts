export function screenToWorld(
    screenX: number,
    screenY: number,
    offset: { x: number; y: number }
): { x: number; y: number } {
    return {
        x: screenX - offset.x,
        y: screenY - offset.y,
    };
}
