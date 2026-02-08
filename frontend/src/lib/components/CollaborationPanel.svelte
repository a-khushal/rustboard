<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { collaborationState, isCollaborating, collaboratorCount } from '$lib/stores/collaboration';
	import {
		createSession,
		connectToSession,
		disconnect,
		generateClientId,
		generateClientColor,
		generateClientName,
		checkSessionExists,
	} from '$lib/utils/collaboration';
	import { editorApi } from '$lib/stores/editor';
	import { get } from 'svelte/store';

	let isOpen = false;
	let shareUrl = '';
	let isCreatingSession = false;
	let errorMessage = '';

	onMount(() => {
		const urlParams = new URLSearchParams(window.location.search);
		const sessionId = urlParams.get('session');
		const token = urlParams.get('token');
		const roleParam = urlParams.get('role');
		const role: 'editor' | 'viewer' = roleParam === 'viewer' ? 'viewer' : 'editor';
		if (sessionId && token) {
			joinSession(sessionId, token, role);
		}
	});

	onDestroy(() => {
		disconnect();
	});

	async function startCollaboration() {
		isCreatingSession = true;
		errorMessage = '';

		try {
			const sessionInfo = await createSession();
			const sessionId = sessionInfo.session_id;
			const clientId = generateClientId();
			const name = generateClientName();
			const color = generateClientColor();

			shareUrl = `${window.location.origin}${window.location.pathname}?session=${sessionId}&role=editor&token=${sessionInfo.editor_token}`;

			const api = get(editorApi);
			if (!api) {
				throw new Error('Editor API not available');
			}

			await connectToSession(sessionId, clientId, name, color, api, sessionInfo.editor_token, 'editor');

			collaborationState.update(state => ({
				...state,
				sessionId,
				clientId,
				role: 'editor',
				isHost: true,
			}));
		} catch (error) {
			console.error('Error starting collaboration:', error);
			errorMessage = 'Failed to create session. Please try again.';
		} finally {
			isCreatingSession = false;
		}
	}

	async function joinSession(sessionId: string, token: string, role: 'editor' | 'viewer' = 'editor') {
		errorMessage = '';
		const exists = await checkSessionExists(sessionId, token);
		if (!exists) {
			errorMessage = 'Session not found or token is invalid.';
			return;
		}

		try {
			const clientId = generateClientId();
			const name = generateClientName();
			const color = generateClientColor();

			const api = get(editorApi);
			if (!api) {
				throw new Error('Editor API not available');
			}

			await connectToSession(sessionId, clientId, name, color, api, token, role);

			collaborationState.update(state => ({
				...state,
				sessionId,
				clientId,
				role,
				isHost: false,
			}));

			shareUrl = `${window.location.origin}${window.location.pathname}?session=${sessionId}&role=editor&token=${token}`;
		} catch (error) {
			console.error('Error joining session:', error);
			errorMessage = 'Failed to join session. Please try again.';
		}
	}

	function copyShareLink() {
		navigator.clipboard.writeText(shareUrl).then(() => {
			const button = document.getElementById('copy-button');
			if (button) {
				const originalText = button.textContent;
				button.textContent = 'Copied!';
				setTimeout(() => {
					if (button) button.textContent = originalText;
				}, 2000);
			}
		}).catch(err => {
			console.error('Failed to copy:', err);
		});
	}

	function stopCollaboration() {
		disconnect();
		collaborationState.update(state => ({
			...state,
			isConnected: false,
			connectionStatus: 'disconnected',
			isResyncing: false,
			lastError: null,
			sessionId: null,
			clientId: null,
			isHost: false,
			collaborators: [],
			presenceByClient: {},
			role: 'editor',
		}));
		shareUrl = '';
		window.history.replaceState({}, '', window.location.pathname);
	}
</script>

<div class="fixed top-4 right-4 z-50">
	<button
		on:click={() => isOpen = !isOpen}
		class="bg-white dark:bg-stone-800 border border-stone-300 dark:border-stone-700 rounded-lg px-4 py-2 shadow-lg hover:bg-stone-50 dark:hover:bg-stone-700 transition-colors flex items-center gap-2"
	>
		{#if $isCollaborating}
			<div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
			<span class="text-sm font-medium">Collaborating ({$collaboratorCount})</span>
		{:else}
			<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
			</svg>
			<span class="text-sm font-medium">Share</span>
		{/if}
	</button>

	{#if isOpen}
		<div class="absolute top-full right-0 mt-2 w-80 bg-white dark:bg-stone-800 border border-stone-300 dark:border-stone-700 rounded-lg shadow-xl p-4">
			{#if $isCollaborating}
				<div class="space-y-4">
					<div>
						<h3 class="text-sm font-semibold mb-2">Collaboration Active</h3>
						<div class="flex items-center gap-2 mb-3">
							<div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
							<span class="text-xs text-stone-600 dark:text-stone-400">
								{$collaboratorCount} {$collaboratorCount === 1 ? 'person' : 'people'} connected
							</span>
						</div>
					</div>

					<div>
						<label for="share-link-input" class="text-xs font-medium text-stone-700 dark:text-stone-300 mb-1 block">Share Link</label>
						<div class="flex gap-2">
							<input
								id="share-link-input"
								type="text"
								readonly
								value={shareUrl}
								class="flex-1 px-3 py-2 text-xs bg-stone-50 dark:bg-stone-900 border border-stone-300 dark:border-stone-700 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
							/>
							<button
								id="copy-button"
								on:click={copyShareLink}
								class="px-3 py-2 bg-blue-500 text-white text-xs rounded hover:bg-blue-600 transition-colors"
							>
								Copy
							</button>
						</div>
					</div>

					<div>
						<div class="text-xs font-medium text-stone-700 dark:text-stone-300 mb-2 block">Collaborators</div>
						<div class="space-y-2">
							{#each $collaborationState.collaborators as collaborator}
								<div class="flex items-center gap-2 text-xs">
									<div
										class="w-3 h-3 rounded-full"
										style="background-color: {collaborator.color};"
									></div>
									<span class="text-stone-700 dark:text-stone-300">{collaborator.name}</span>
									{#if collaborator.id === $collaborationState.clientId}
										<span class="text-xs text-stone-500">(You)</span>
									{/if}
								</div>
							{/each}
						</div>
					</div>

					<button
						on:click={stopCollaboration}
						class="w-full px-4 py-2 bg-red-500 text-white text-sm rounded hover:bg-red-600 transition-colors"
					>
						Stop Collaboration
					</button>
				</div>
			{:else}
				<div class="space-y-4">
					<h3 class="text-sm font-semibold">Start Collaboration</h3>
					<p class="text-xs text-stone-600 dark:text-stone-400">
						Create a session to collaborate with others in real-time.
					</p>

					{#if errorMessage}
						<div class="p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-xs text-red-700 dark:text-red-400">
							{errorMessage}
						</div>
					{/if}

					<button
						on:click={startCollaboration}
						disabled={isCreatingSession}
						class="w-full px-4 py-2 bg-blue-500 text-white text-sm rounded hover:bg-blue-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
					>
						{isCreatingSession ? 'Creating...' : 'Create Session'}
					</button>
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	:global(body) {
		position: relative;
	}
</style>
