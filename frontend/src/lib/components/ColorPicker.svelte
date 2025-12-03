<script lang="ts">
	import { theme } from '$lib/stores/theme';

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

	function isLightColor(hexColor: string): boolean {
		const hex = hexColor.replace('#', '');
		const r = parseInt(hex.slice(0, 2), 16);
		const g = parseInt(hex.slice(2, 4), 16);
		const b = parseInt(hex.slice(4, 6), 16);
		const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
		return luminance > 0.5;
	}
</script>

<div class="flex flex-col gap-2 w-full min-w-0">
	<div class="flex items-center gap-2 min-w-0">
		<label for="color-picker-{label}" class="sr-only">{label}</label>
		<div class="relative shrink-0">
			<input
				type="color"
				bind:value
				on:input={handleInput}
				class={`w-7 h-7 rounded-full border-2 cursor-pointer shrink-0 opacity-0 absolute inset-0 ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-200'}`}
				title={value}
				id="color-picker-{label}"
			/>
			<div
				class={`w-7 h-7 rounded-full border-2 pointer-events-none ${$theme === 'dark' ? 'border-stone-600' : 'border-stone-200'}`}
				style="background-color: {value};"
			></div>
		</div>
		<input
			type="text"
			bind:value
			on:input={handleInput}
			class={`flex-1 min-w-0 px-2 py-1 text-xs font-mono border rounded focus:outline-none focus:ring-1 h-8 ${$theme === 'dark' ? 'border-stone-600 bg-stone-700 text-stone-200 focus:ring-stone-500' : 'border-stone-200 bg-stone-50 focus:ring-stone-400'}`}
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
				class={`w-6 h-6 rounded-full border-2 transition-all hover:scale-110 hover:border-stone-400 ${value === color ? ($theme === 'dark' ? 'border-stone-400 ring-2 ring-stone-500' : 'border-stone-600 ring-2 ring-stone-300') : ($theme === 'dark' ? 'border-stone-600' : 'border-stone-200')}`}
				style="background-color: {color};"
				title={color}
			>
				{#if value === color}
					<svg class="w-3 h-3 m-auto drop-shadow-lg {isLightColor(color) ? 'text-stone-900' : 'text-white'}" viewBox="0 0 16 16" fill="currentColor">
						<path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.75.75 0 0 1 1.06-1.06L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"/>
					</svg>
				{/if}
			</button>
		{/each}
	</div>
</div>
