<script lang="ts">
	import { zoomIn, zoomOut } from '$lib/utils/zoom';
</script>

<div class="absolute bottom-4 right-4 z-50 flex flex-row gap-1 bg-white dark:bg-stone-800 border border-stone-200 dark:border-stone-700 shadow-sm rounded-sm p-1">
	<button
		on:click={zoomIn}
		class="flex items-center justify-center w-8 h-8 text-sm font-sans text-stone-700 dark:text-stone-200
			transition-colors duration-150
			bg-white dark:bg-stone-800 hover:bg-stone-50 dark:hover:bg-stone-700 border border-stone-200 dark:border-stone-700
			active:bg-stone-100 dark:active:bg-stone-600"
		title="Zoom In (Ctrl + Scroll Up)"
	>
		<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<line x1="8" y1="4" x2="8" y2="12"/>
			<line x1="4" y1="8" x2="12" y2="8"/>
		</svg>
	</button>
	<button
		on:click={zoomOut}
		class="flex items-center justify-center w-8 h-8 text-sm font-sans text-stone-700
			transition-colors duration-150
			bg-white hover:bg-stone-50 border border-stone-200
			active:bg-stone-100"
		title="Zoom Out (Ctrl + Scroll Down)"
	>
		<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
			<line x1="4" y1="8" x2="12" y2="8"/>
		</svg>
	</button>
</div>

