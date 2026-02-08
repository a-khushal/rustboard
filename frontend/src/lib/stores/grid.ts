import { writable } from 'svelte/store';

export const gridEnabled = writable<boolean>(false);
export const gridSize = writable<number>(16);
