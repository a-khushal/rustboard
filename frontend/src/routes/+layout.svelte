<script>
	import '../app.css';
	import { theme } from '$lib/stores/theme';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	if (browser) {
		onMount(() => {
			if (document.documentElement) {
				const currentTheme = $theme;
				if (currentTheme === 'dark') {
					document.documentElement.classList.add('dark');
				} else {
					document.documentElement.classList.remove('dark');
				}
			}
		});
	}
</script>

<slot />
