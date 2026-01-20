<script lang="ts">
	import { theme } from '$lib/stores/theme';
	import { onMount } from 'svelte';

	export let isOpen = false;

	function close() {
		isOpen = false;
	}

	function handleBackdropKeyDown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			close();
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape' && isOpen) {
			event.preventDefault();
			event.stopPropagation();
			close();
			return;
		}
		const isSlashKey = event.code === 'Slash' || event.key === '/' || event.key === '?';
		if ((event.ctrlKey || event.metaKey) && isSlashKey) {
			event.preventDefault();
			event.stopPropagation();
			close();
			return;
		}
		if (event.key === 'F1') {
			event.preventDefault();
			event.stopPropagation();
			close();
			return;
		}
	}

	let cleanup: (() => void) | null = null;

	$: {
		if (cleanup) {
			cleanup();
			cleanup = null;
		}
		if (isOpen) {
			window.addEventListener('keydown', handleKeyDown, true);
			cleanup = () => {
				window.removeEventListener('keydown', handleKeyDown, true);
			};
		}
	}

	onMount(() => {
		return () => {
			if (cleanup) {
				cleanup();
			}
		};
	});

	const shortcuts = [
		{
			category: 'General',
			items: [
				{ keys: ['Ctrl', '?'], description: 'Show keyboard shortcuts' },
				{ keys: ['F1'], description: 'Show keyboard shortcuts' },
			]
		},
		{
			category: 'Edit',
			items: [
				{ keys: ['Ctrl', 'Z'], description: 'Undo' },
				{ keys: ['Ctrl', 'Shift', 'Z'], description: 'Redo' },
				{ keys: ['Ctrl', 'Y'], description: 'Redo' },
				{ keys: ['Ctrl', 'C'], description: 'Copy' },
				{ keys: ['Ctrl', 'V'], description: 'Paste' },
				{ keys: ['Ctrl', 'D'], description: 'Duplicate' },
				{ keys: ['Delete'], description: 'Delete selected' },
			]
		},
		{
			category: 'Selection',
			items: [
				{ keys: ['Escape'], description: 'Clear selection / Cancel tool' },
				{ keys: ['Shift'], description: 'Multi-select (hold while clicking)' },
			]
		},
		{
			category: 'Transform',
			items: [
				{ keys: ['R'], description: 'Rotate selected (clockwise)' },
				{ keys: ['Shift', 'R'], description: 'Rotate selected (counter-clockwise)' },
				{ keys: ['Ctrl', 'G'], description: 'Group selected' },
				{ keys: ['Ctrl', 'Shift', 'G'], description: 'Ungroup selected' },
			]
		},
		{
			category: 'Layering',
			items: [
				{ keys: ['Ctrl', ']'], description: 'Bring forward' },
				{ keys: ['Ctrl', 'Shift', ']'], description: 'Bring to front' },
				{ keys: ['Ctrl', '['], description: 'Send backward' },
				{ keys: ['Ctrl', 'Shift', '['], description: 'Send to back' },
			]
		},
		{
			category: 'Image',
			items: [
				{ keys: ['C'], description: 'Crop image (when image selected)' },
				{ keys: ['F'], description: 'Apply filter to image' },
				{ keys: ['O'], description: 'Adjust image opacity' },
			]
		},
		{
			category: 'View',
			items: [
				{ keys: ['Ctrl', '+'], description: 'Zoom in' },
				{ keys: ['Ctrl', '-'], description: 'Zoom out' },
				{ keys: ['Space'], description: 'Pan mode (hold and drag)' },
				{ keys: ['Ctrl', 'Wheel'], description: 'Zoom in/out' },
				{ keys: ['Wheel'], description: 'Pan (scroll without Ctrl)' },
			]
		},
	];

	function formatKey(key: string): string {
		const keyMap: Record<string, string> = {
			'Ctrl': 'Ctrl',
			'Shift': 'Shift',
			'Alt': 'Alt',
			'Meta': '⌘',
			'Delete': 'Del',
			'Escape': 'Esc',
			'Space': 'Space',
			'Wheel': 'Scroll',
		};
		return keyMap[key] || key.toUpperCase();
	}
</script>

{#if isOpen}
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
		on:click={close}
		on:keydown={handleBackdropKeyDown}
		role="dialog"
		aria-modal="true"
		aria-label="Keyboard shortcuts"
		tabindex="-1"
	>
		<div
			class={`relative w-full max-w-2xl max-h-[90vh] overflow-y-auto rounded-lg shadow-xl ${
				$theme === 'dark' ? 'bg-stone-800 border border-stone-700' : 'bg-white border border-stone-200'
			}`}
			on:click|stopPropagation
			role="presentation"
		>
			<div
				class={`sticky top-0 flex items-center justify-between p-4 border-b ${
					$theme === 'dark' ? 'border-stone-700' : 'border-stone-200'
				}`}
			>
				<h2
					class={`text-xl font-semibold ${
						$theme === 'dark' ? 'text-stone-100' : 'text-stone-900'
					}`}
				>
					Keyboard Shortcuts
				</h2>
				<button
					on:click={close}
					class={`p-1 rounded transition-colors ${
						$theme === 'dark'
							? 'text-stone-400 hover:text-stone-200 hover:bg-stone-700'
							: 'text-stone-500 hover:text-stone-900 hover:bg-stone-100'
					}`}
					aria-label="Close shortcuts panel"
				>
					<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<line x1="18" y1="6" x2="6" y2="18"></line>
						<line x1="6" y1="6" x2="18" y2="18"></line>
					</svg>
				</button>
			</div>

			<div class="p-6">
				{#each shortcuts as category}
					<div class="mb-6 last:mb-0">
						<h3
							class={`text-sm font-semibold uppercase tracking-wide mb-3 ${
								$theme === 'dark' ? 'text-stone-400' : 'text-stone-600'
							}`}
						>
							{category.category}
						</h3>
						<div class="space-y-2">
							{#each category.items as item}
								<div
									class={`flex items-center justify-between py-2 px-3 rounded ${
										$theme === 'dark'
											? 'bg-stone-700/50 hover:bg-stone-700'
											: 'bg-stone-50 hover:bg-stone-100'
									}`}
								>
									<span
										class={`text-sm ${
											$theme === 'dark' ? 'text-stone-200' : 'text-stone-700'
										}`}
									>
										{item.description}
									</span>
									<div class="flex items-center gap-1">
										{#each item.keys as key, i}
											{#if i > 0}
												<span
													class={`text-xs ${
														$theme === 'dark' ? 'text-stone-500' : 'text-stone-400'
													}`}
												>
													+
												</span>
											{/if}
											<kbd
												class={`px-2 py-1 text-xs font-mono rounded ${
													$theme === 'dark'
														? 'bg-stone-900 text-stone-200 border border-stone-600'
														: 'bg-white text-stone-700 border border-stone-300 shadow-sm'
												}`}
											>
												{formatKey(key)}
											</kbd>
										{/each}
									</div>
								</div>
							{/each}
						</div>
					</div>
				{/each}
			</div>

			<div
				class={`sticky bottom-0 p-4 border-t text-center text-xs ${
					$theme === 'dark'
						? 'border-stone-700 text-stone-500'
						: 'border-stone-200 text-stone-500'
				}`}
			>
				Press <kbd
					class={`px-1.5 py-0.5 text-xs font-mono rounded ${
						$theme === 'dark'
							? 'bg-stone-900 text-stone-200 border border-stone-600'
							: 'bg-white text-stone-700 border border-stone-300'
					}`}
				>Esc</kbd> to close
			</div>
		</div>
	</div>
{/if}

