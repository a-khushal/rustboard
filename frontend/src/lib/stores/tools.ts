import { writable } from "svelte/store";

export type Tool = 'select' | 'rectangle' | 'ellipse';
export const activeTool = writable<Tool>('select' as Tool);