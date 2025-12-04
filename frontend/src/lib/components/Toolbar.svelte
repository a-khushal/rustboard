<script lang="ts">
	import { activeTool, type Tool } from '$lib/stores/tools';
	import { theme } from '$lib/stores/theme';

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
		{ id: 'text' as Tool, label: 'Text', icon: 'text' },
		{ id: 'freehand' as Tool, label: 'Freehand', icon: 'freehand' }
	];

	function toggleTheme() {
		theme.update(t => t === 'light' ? 'dark' : 'light');
		console.log('theme', $theme);
	}
</script>

<div class={`absolute top-2 left-2 z-50 flex gap-1 shadow-sm rounded-sm p-1 ${$theme === 'dark' ? 'bg-stone-800 border border-stone-700' : 'bg-white border border-stone-200'}`}>
	{#each tools as tool (tool.id)}
		<button
			on:click={() => setTool(tool.id)}
			class={`flex items-center gap-1.5 px-2 py-1.5 text-xs font-sans ${$theme === 'dark' ? 'text-stone-200' : 'text-stone-700'}
				transition-colors duration-150 rounded-sm
				${$activeTool === tool.id
					? $theme === 'dark'
						? 'bg-stone-700 border border-stone-500'
						: 'bg-stone-100 border border-stone-400'
					: $theme === 'dark'
						? 'bg-stone-800 hover:bg-stone-700 border border-stone-500'
						: 'bg-white hover:bg-stone-50 border border-stone-200'}`}
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
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
					<line x1="4" y1="3" x2="12" y2="3" />
					<line x1="8" y1="3" x2="8" y2="13" />
				</svg>
			{:else if tool.icon === 'freehand'}
				<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-pencil" viewBox="0 0 16 16">
					<path d="M12.146.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1 0 .708l-10 10a.5.5 0 0 1-.168.11l-5 2a.5.5 0 0 1-.65-.65l2-5a.5.5 0 0 1 .11-.168zM11.207 2.5 13.5 4.793 14.793 3.5 12.5 1.207zm1.586 3L10.5 3.207 4 9.707V10h.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.5h.293zm-9.761 5.175-.106.106-1.528 3.821 3.821-1.528.106-.106A.5.5 0 0 1 5 12.5V12h-.5a.5.5 0 0 1-.5-.5V11h-.5a.5.5 0 0 1-.468-.325"/>
				</svg>
			{/if}
		</button>
	{/each}
	
	<div class={`w-px ${$theme === 'dark' ? 'bg-stone-700' : 'bg-stone-200'} mx-1`}></div>

	<button
		on:click={toggleTheme}
		class={`flex items-center gap-1.5 px-2 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm
			${$theme === 'dark'
				? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
				: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
		title={$theme === 'light' ? 'Switch to Dark Mode' : 'Switch to Light Mode'}
	>
		{#if $theme === 'light'}
			<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
				<circle cx="12" cy="12" r="4" />
				<line x1="12" y1="2" x2="12" y2="6" />
				<line x1="12" y1="18" x2="12" y2="22" />
				<line x1="2" y1="12" x2="6" y2="12" />
				<line x1="18" y1="12" x2="22" y2="12" />
				<line x1="4.2" y1="4.2" x2="6.8" y2="6.8" />
				<line x1="17.2" y1="17.2" x2="19.8" y2="19.8" />
				<line x1="17.2" y1="6.8" x2="19.8" y2="4.2" />
				<line x1="4.2" y1="19.8" x2="6.8" y2="17.2" />
			</svg>
		{:else}
			<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
				<path d="M13 9.5a5 5 0 1 1-6-6 3.5 3.5 0 0 0 6 6z" />
			</svg>
		{/if}
	</button>
</div>
