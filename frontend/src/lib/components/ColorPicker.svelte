<script lang="ts">
	export let value: string = '#000000';
	export let label: string = 'Color';
	export let onInput: ((value: string) => void) | undefined = undefined;

	const modernColors = [
		'#ffffff', '#e5e5e5', '#fbbf24', '#f97316',
		'#ef4444', '#ec4899', '#a855f7', '#6366f1',
		'#3b82f6', '#06b6d4', '#10b981', '#000000'
	];

	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		value = target.value;
		onInput?.(value);
	}

	function selectColor(color: string) {
		value = color;
		onInput?.(value);
	}
</script>

<div class="flex flex-col gap-2 w-full min-w-0">
	<div class="flex items-center gap-2 min-w-0">
		<label for="color-picker-{label}" class="sr-only">{label}</label>
		<input
			type="color"
			bind:value
			on:input={handleInput}
			class="w-8 h-8 rounded-full border-2 border-stone-200 cursor-pointer overflow-hidden shrink-0"
			title={value}
			id="color-picker-{label}"
		/>
		<input
			type="text"
			bind:value
			on:input={handleInput}
			class="flex-1 min-w-0 px-2 py-1 text-xs font-mono border border-stone-200 rounded focus:outline-none focus:ring-1 focus:ring-stone-400 bg-stone-50 h-8"
			placeholder="#000000"
			maxlength="7"
			aria-label="{label} hex value"
		/>
	</div>
	<div class="grid grid-cols-4 gap-1.5">
		{#each modernColors as color}
			<button
				type="button"
				on:click={() => selectColor(color)}
				class="w-6 h-6 rounded-full border-2 transition-all hover:scale-110 hover:border-stone-400 {value === color ? 'border-stone-600 ring-2 ring-stone-300' : 'border-stone-200'}"
				style="background-color: {color};"
				title={color}
			>
				{#if value === color}
					<svg class="w-3 h-3 m-auto text-white drop-shadow-lg" viewBox="0 0 16 16" fill="currentColor">
						<path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
					</svg>
				{/if}
			</button>
		{/each}
	</div>
</div>

