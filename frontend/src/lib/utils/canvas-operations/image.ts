import { get } from 'svelte/store';
import { editorApi, images, selectedImages, type Image } from '$lib/stores/editor';
import { sendOperation } from '$lib/utils/collaboration';

export function addImage(x: number, y: number, width: number, height: number, imageData: string): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const newId = api.add_image(x, y, width, height, imageData);
    const updatedImages = Array.from(api.get_images() as Image[]);
    images.set(updatedImages);

    const newIdNum = Number(newId);
    const newImage = updatedImages.find((i: Image) => i.id === newIdNum);
    if (newImage) {
        selectedImages.set([newImage]);
    }
    
    sendOperation({
        op: 'AddImage',
        id: newIdNum,
        position: { x, y },
        width,
        height,
        image_data: imageData
    });
    
    return newIdNum;
}

export function moveImage(id: number, x: number, y: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_image(BigInt(id), x, y, saveHistory);
    updateImages();
    
    sendOperation({
        op: 'MoveImage',
        id,
        position: { x, y }
    });
}

export function resizeImage(id: number, width: number, height: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.resize_image(BigInt(id), width, height, saveHistory);
    updateImages();
    
    sendOperation({
        op: 'ResizeImage',
        id,
        width,
        height
    });
}

export function setImageRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_image_rotation(BigInt(id), angle, saveHistory);
    updateImages();
}

export function updateImages(): void {
    const api = get(editorApi);
    if (!api) return;
    const updatedImages = Array.from(api.get_images() as Image[]);
    images.set(updatedImages);

    const currentSelection = get(selectedImages);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((i: Image) => i.id));
        const updatedSelection = updatedImages.filter((i: Image) => selectedIds.has(i.id));
        selectedImages.set(updatedSelection.length > 0 ? updatedSelection : []);
    }
}

