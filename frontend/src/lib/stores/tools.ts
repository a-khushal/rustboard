import { writable } from "svelte/store";

export type Tool = 'select' | 'rectangle' | 'ellipse' | 'line' | 'arrow' | 'diamond' | 'text' | 'freehand' | 'image' | 'eraser';
export const activeTool = writable<Tool>('select' as Tool);