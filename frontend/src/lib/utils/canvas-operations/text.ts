import { get } from 'svelte/store';
import { editorApi, texts, selectedTexts, type Text } from '$lib/stores/editor';

function updateTexts(): void {
    const api = get(editorApi);
    if (!api) return;

    const updatedTexts = Array.from(api.get_texts() as Text[]);
    texts.set(updatedTexts);

    const currentSelection = get(selectedTexts);
    if (currentSelection.length > 0) {
        const selectedIds = new Set(currentSelection.map((t: Text) => t.id));
        const updatedSelection = updatedTexts.filter((t: Text) => selectedIds.has(t.id));
        selectedTexts.set(updatedSelection);
    }
}

export function addText(x: number, y: number, content: string, width: number = 100, height: number = 20): number | null {
    const api = get(editorApi);
    if (!api) return null;

    const newId = api.add_text(x, y, width, height, content);

    const updatedTexts = Array.from(api.get_texts() as Text[]);
    texts.set(updatedTexts);

    const newText = updatedTexts.find((t: Text) => t.id === newId);
    if (newText) {
        selectedTexts.set([newText]);
    }

    return newId;
}

export function moveText(id: number, x: number, y: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.move_text(BigInt(id), x, y, saveHistory);
    updateTexts();
}

export function resizeText(id: number, width: number, height: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.resize_text(BigInt(id), width, height, saveHistory);
    updateTexts();
}

export function setTextContent(id: number, content: string, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_text_content(BigInt(id), content, saveHistory);
    updateTexts();
}

export function setTextFontSize(id: number, fontSize: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_text_font_size(BigInt(id), fontSize, saveHistory);
    updateTexts();
}

export function setTextFontFamily(id: number, fontFamily: string, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_text_font_family(BigInt(id), fontFamily, saveHistory);
    updateTexts();
}

export function setTextTextAlign(id: number, textAlign: string, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_text_text_align(BigInt(id), textAlign, saveHistory);
    updateTexts();
}

export function setTextColor(id: number, color: string, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_text_color(BigInt(id), color, saveHistory);
    updateTexts();
}

export function setTextRotation(id: number, angle: number, saveHistory: boolean = true): void {
    const api = get(editorApi);
    if (!api) return;

    api.set_text_rotation(BigInt(id), angle, saveHistory);
    updateTexts();
}
