<script lang="ts">
	import { activeTool, type Tool } from '$lib/stores/tools';

	function setTool(tool: Tool) {
		activeTool.set(tool);
	}

	const tools: Array<{ id: Tool; label: string; icon: string }> = [
		{ id: 'select' as Tool, label: 'Select', icon: 'cursor' },
		{ id: 'rectangle' as Tool, label: 'Rectangle', icon: 'rect' },
		{ id: 'ellipse' as Tool, label: 'Ellipse', icon: 'circle' },
		{ id: 'diamond' as Tool, label: 'Diamond', icon: 'diamond' },
		{ id: 'line' as Tool, label: 'Line', icon: 'line' },
		{ id: 'arrow' as Tool, label: 'Arrow', icon: 'arrow' },
		{ id: 'text' as Tool, label: 'Text', icon: 'text' }
	];
</script>

<div class="absolute top-2 left-2 z-50 flex gap-1 bg-white border border-stone-200 shadow-sm rounded-sm p-1">
	{#each tools as tool (tool.id)}
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
			{:else if tool.icon === 'diamond'}
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
					<polygon points="8,3 13,8 8,13 3,8"/>
				</svg>
			{:else if tool.icon === 'line'}
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
					<line x1="3" y1="3" x2="13" y2="13"/>
				</svg>
			{:else if tool.icon === 'arrow'}
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<line x1="2" y1="8" x2="12" y2="8"/>
					<polyline points="9,5 12,8 9,11"/>
				</svg>
			{:else if tool.icon === 'text'}
				<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 15 15">
					<path fill="currentColor" fill-rule="evenodd" d="M3.95 2.95V4.5a.45.45 0 0 1-.9 0v-2a.45.45 0 0 1 .45-.45h8a.45.45 0 0 1 .45.45v2a.45.45 0 1 1-.9 0V2.95h-3v9.1h1.204a.45.45 0 0 1 0 .9h-3.5a.45.45 0 1 1 0-.9H6.95v-9.1h-3Z" clip-rule="evenodd">
					</path>
				</svg>
			{/if}
		</button>
	{/each}
</div>
