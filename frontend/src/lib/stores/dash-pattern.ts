import { writable } from 'svelte/store';

export type DashPattern = 'solid' | 'dashed' | 'dotted';

export const dashPattern = writable<DashPattern>('solid');

