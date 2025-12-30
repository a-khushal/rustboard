import { selectedRectangles, selectedEllipses, selectedLines, selectedArrows, selectedDiamonds, selectedPaths, selectedImages, selectedGroups } from '$lib/stores/editor';

export function clearAllSelections(): void {
    selectedRectangles.set([]);
    selectedEllipses.set([]);
    selectedLines.set([]);
    selectedArrows.set([]);
    selectedDiamonds.set([]);
    selectedPaths.set([]);
    selectedImages.set([]);
    selectedGroups.set([]);
}
