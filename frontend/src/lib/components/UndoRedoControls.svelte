<script lang="ts">
	import { editorApi } from '$lib/stores/editor';
	import { theme } from '$lib/stores/theme';
	import { updateAllStoresAfterUndoRedo } from '$lib/utils/undo-redo';

	function undo() {
		if (!$editorApi) return;
		const success = $editorApi.undo();
		if (success) {
			updateAllStoresAfterUndoRedo();
		}
	}

	function redo() {
		if (!$editorApi) return;
		const success = $editorApi.redo();
		if (success) {
			updateAllStoresAfterUndoRedo();
		}
	}
</script>

<div class={`absolute bottom-4 left-4 z-50 flex flex-row gap-1 shadow-sm rounded-sm p-1 ${$theme === 'dark' ? 'bg-stone-800 border border-stone-700' : 'bg-white border border-stone-200'}`}>
	<button
		type="button"
		on:click={undo}
		class={`flex items-center justify-center w-8 h-8 text-sm font-sans transition-colors duration-150 rounded-sm
			${$theme === 'dark'
				? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700 active:bg-stone-600'
				: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200 active:bg-stone-100'}
			disabled:opacity-50 disabled:cursor-not-allowed`}
		title="Undo (Ctrl + Z)"
	>
		<svg width="16" height="16" viewBox="0 0 8 8" fill="currentColor">
			<path d="M4.5 1C2.57 1 1 2.57 1 4.5V5H0l2 2 2-2H3v-.5a2.5 2.5 0 0 1 5 0C8 2.57 6.43 1 4.5 1z"/>
		</svg>
	</button>
	<button
		type="button"
		on:click={redo}
		class={`flex items-center justify-center w-8 h-8 text-sm font-sans transition-colors duration-150 rounded-sm
			${$theme === 'dark'
				? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700 active:bg-stone-600'
				: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200 active:bg-stone-100'}
			disabled:opacity-50 disabled:cursor-not-allowed`}
		title="Redo (Ctrl + Y)"
	>
		<svg width="16" height="16" viewBox="0 0 8 8" fill="currentColor">
			<path d="M3.5 1C1.57 1 0 2.57 0 4.5a2.5 2.5 0 0 1 5 0V5H4l2 2 2-2H7v-.5C7 2.57 5.43 1 3.5 1z"/>
		</svg>
	</button>
</div>
