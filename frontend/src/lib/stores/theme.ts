import { writable } from 'svelte/store';
import { browser } from '$app/environment';

type Theme = 'light' | 'dark';

const initialTheme: Theme = browser ? (localStorage.getItem('theme') as Theme) || 'light' : 'light';

export const theme = writable<Theme>(initialTheme);

function applyTheme(value: Theme) {
    if (!browser || !document.documentElement) return;
    if (value === 'dark') {
        document.documentElement.classList.add('dark');
    } else {
        document.documentElement.classList.remove('dark');
    }
}

if (browser) {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            applyTheme(initialTheme);
        });
    } else {
        applyTheme(initialTheme);
    }
    
    theme.subscribe((value) => {
        if (browser) {
            localStorage.setItem('theme', value);
            applyTheme(value);
        }
    });
}
