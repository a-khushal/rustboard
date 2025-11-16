<script lang="ts">
	import { activeTool, type Tool } from '$lib/stores/tools';

	function setTool(tool: Tool) {
		activeTool.set(tool);
	}

	const tools: Array<{ id: Tool; label: string; icon: string }> = [
		{ id: 'select' as Tool, label: 'Select', icon: 'cursor' },
		{ id: 'rectangle' as Tool, label: 'Rectangle', icon: 'rect' },
		{ id: 'elipse' as Tool, label: 'Ellipse', icon: 'circle' }
	];
</script>

<div class="absolute top-2 left-2 z-50 flex gap-1 bg-white border border-stone-200 shadow-sm rounded-sm p-1">
	{#each tools as tool}
		<button
			on:click={() => setTool(tool.id)}
			class="flex items-center gap-1.5 px-2 py-1.5 text-xs font-sans text-stone-700
				transition-colors duration-150
				{ $activeTool === tool.id
					? 'bg-stone-100 border border-stone-400' 
					: 'bg-white hover:bg-stone-50 border border-stone-200'}"
			title={tool.label}
		>
			{#if tool.icon === 'cursor'}
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
					<path fill="none" d="M3.67 2.14V13.87c0 .3.36.45.57.23l3.24-3.24a.33.33 0 0 1 .23-.1h4.58a.33.33 0 0 0 .23-.57L4.23 1.9a.33.33 0 0 0-.57.23Z"/>
				</svg>
			{:else if tool.icon === 'rect'}
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
					<rect x="3" y="3" width="10" height="10" rx="1"/>
				</svg>
			{:else if tool.icon === 'circle'}
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
					<circle cx="8" cy="8" r="5"/>
				</svg>
			{/if}
		</button>
	{/each}
</div>
