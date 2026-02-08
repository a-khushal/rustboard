<script lang="ts">
	import { activeTool, type Tool } from '$lib/stores/tools';
	import { theme } from '$lib/stores/theme';
	import { viewportOffset, zoom, selectedRectangles, selectedEllipses, selectedLines, selectedArrows, selectedDiamonds, selectedPaths, selectedImages, selectedTexts, selectedGroups, editorApi } from '$lib/stores/editor';
	import { addImage } from '$lib/utils/canvas-operations/index';
	import { screenToWorld } from '$lib/utils/viewport';
	import { onMount, onDestroy } from 'svelte';
	import { collaborationState, isCollaborating, collaboratorCount } from '$lib/stores/collaboration';
	import { gridEnabled, gridSize } from '$lib/stores/grid';
	import { zoomIn, zoomOut } from '$lib/utils/zoom';
	import { updateAllStoresAfterUndoRedo } from '$lib/utils/undo-redo';
	import {
		createSession,
		connectToSession,
		disconnect,
		generateClientId,
		generateClientColor,
		generateClientName,
		checkSessionExists,
		revokeSessionToken,
		rotateSessionToken,
		issueInviteToken,
	} from '$lib/utils/collaboration';
	import { get } from 'svelte/store';
	import ShortcutsPanel from './ShortcutsPanel.svelte';

	let collaborationMenuOpen = false;
	let shareUrl = '';
	let viewerUrl = '';
	let isCreatingSession = false;
	let errorMessage = '';
	let editorToken = '';
	let viewerToken = '';
	let tokenActionInProgress = false;
	let shortcutsPanelOpen = false;
	let currentTimeMs = Date.now();

	function parseTokenExpiryMs(token: string): number | null {
		if (!token) return null;
		const parts = token.split('.');
		if (parts.length !== 2) return null;
		try {
			const base64 = parts[0].replace(/-/g, '+').replace(/_/g, '/');
			const padded = base64 + '='.repeat((4 - (base64.length % 4)) % 4);
			const payload = JSON.parse(atob(padded)) as { exp?: number };
			if (typeof payload.exp !== 'number') return null;
			return payload.exp * 1000;
		} catch {
			return null;
		}
	}

	function formatExpiryCountdown(expiryMs: number): string {
		const remaining = expiryMs - currentTimeMs;
		if (remaining <= 0) return 'Expired';
		const seconds = Math.floor(remaining / 1000);
		const days = Math.floor(seconds / 86400);
		const hours = Math.floor((seconds % 86400) / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (days > 0) return `${days}d ${hours}h`;
		if (hours > 0) return `${hours}h ${minutes}m`;
		return `${minutes}m`;
	}

	$: viewerTokenExpiryMs = parseTokenExpiryMs(viewerToken);
	$: viewerTokenExpiryLabel = viewerToken
		? (viewerTokenExpiryMs ? formatExpiryCountdown(viewerTokenExpiryMs) : 'Unknown')
		: 'No active viewer link';

	onMount(() => {
		function isTypingTarget(target: EventTarget | null): boolean {
			if (!(target instanceof HTMLElement)) return false;
			if (target.isContentEditable) return true;
			if (target.closest('[contenteditable="true"]')) return true;
			if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) {
				return true;
			}
			return false;
		}

		const urlParams = new URLSearchParams(window.location.search);
		const sessionId = urlParams.get('session');
		const token = urlParams.get('token');
		const roleParam = urlParams.get('role');
		const role: 'editor' | 'viewer' = roleParam === 'viewer' ? 'viewer' : 'editor';
		if (sessionId && token) {
			if (role === 'editor') {
				editorToken = token;
			} else {
				viewerToken = token;
			}
			const wasHost = localStorage.getItem(`session_host_${sessionId}`) === 'true';
			joinSession(sessionId, token, wasHost, role);
		}

		function handleClickOutside(event: MouseEvent) {
			const target = event.target as HTMLElement;
			if (!target.closest('.collaboration-menu-container')) {
				collaborationMenuOpen = false;
			}
		}

		function handleKeyDown(event: KeyboardEvent) {
			if (isTypingTarget(event.target)) return;

			if (!event.ctrlKey && !event.metaKey && !event.altKey) {
				const codeToTool: Record<string, Tool> = {
					Digit1: 'select',
					Digit2: 'rectangle',
					Digit3: 'diamond',
					Digit4: 'ellipse',
					Digit5: 'arrow',
					Digit6: 'line',
					Digit7: 'freehand',
					Digit8: 'text',
					Digit9: 'image',
					Digit0: 'eraser',
					Numpad1: 'select',
					Numpad2: 'rectangle',
					Numpad3: 'diamond',
					Numpad4: 'ellipse',
					Numpad5: 'arrow',
					Numpad6: 'line',
					Numpad7: 'freehand',
					Numpad8: 'text',
					Numpad9: 'image',
					Numpad0: 'eraser'
				};

				const keyToTool: Record<string, Tool> = {
					'1': 'select',
					'2': 'rectangle',
					'3': 'diamond',
					'4': 'ellipse',
					'5': 'arrow',
					'6': 'line',
					'7': 'freehand',
					'8': 'text',
					'9': 'image',
					'0': 'eraser'
				};

				const mappedTool = codeToTool[event.code] ?? keyToTool[event.key];
				if (mappedTool) {
					event.preventDefault();
					setTool(mappedTool);
					return;
				}
			}

			const isSlashKey = event.code === 'Slash' || event.key === '/' || event.key === '?';
			if ((event.ctrlKey || event.metaKey) && isSlashKey) {
				event.preventDefault();
				shortcutsPanelOpen = true;
			}
			if (event.key === 'F1') {
				event.preventDefault();
				shortcutsPanelOpen = true;
			}
		}

		document.addEventListener('click', handleClickOutside);
		window.addEventListener('keydown', handleKeyDown);
		const timer = window.setInterval(() => {
			currentTimeMs = Date.now();
		}, 30000);
		return () => {
			document.removeEventListener('click', handleClickOutside);
			window.removeEventListener('keydown', handleKeyDown);
			window.clearInterval(timer);
		};
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

			const api = get(editorApi);
			if (!api) {
				throw new Error('Editor API not available');
			}

			await connectToSession(sessionId, clientId, name, color, api, sessionInfo.editor_token, 'editor');

			shareUrl = `${window.location.origin}${window.location.pathname}?session=${sessionId}&role=editor&token=${sessionInfo.editor_token}`;
			viewerUrl = `${window.location.origin}${window.location.pathname}?session=${sessionId}&role=viewer&token=${sessionInfo.viewer_token}`;
			editorToken = sessionInfo.editor_token;
			viewerToken = sessionInfo.viewer_token;
			window.history.replaceState({}, '', `?session=${sessionId}&role=editor&token=${sessionInfo.editor_token}`);
			localStorage.setItem(`session_host_${sessionId}`, 'true');

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

	async function joinSession(sessionId: string, token: string, isHost: boolean = false, role: 'editor' | 'viewer' = 'editor') {
		errorMessage = '';
		const exists = await checkSessionExists(sessionId, token);
		if (!exists) {
			errorMessage = 'Session not found or token is invalid.';
			localStorage.removeItem(`session_host_${sessionId}`);
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

			shareUrl = `${window.location.origin}${window.location.pathname}?session=${sessionId}&role=${role}&token=${token}`;
			viewerUrl = role === 'viewer' ? shareUrl : '';
			if (role === 'editor') {
				editorToken = token;
			} else {
				viewerToken = token;
			}
			window.history.replaceState({}, '', `?session=${sessionId}&role=${role}&token=${token}`);

			collaborationState.update(state => ({
				...state,
				sessionId,
				clientId,
				role,
				isHost,
			}));
		} catch (error) {
			console.error('Error joining session:', error);
			errorMessage = 'Failed to join session. Please try again.';
		}
	}

	function copyShareLink(url: string = shareUrl, buttonId: string = 'copy-button') {
		navigator.clipboard.writeText(url).then(() => {
			const button = document.getElementById(buttonId);
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
		const state = get(collaborationState);
		if (state.sessionId) {
			localStorage.removeItem(`session_host_${state.sessionId}`);
		}
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
		viewerUrl = '';
		editorToken = '';
		viewerToken = '';
		tokenActionInProgress = false;
		collaborationMenuOpen = false;
		window.history.replaceState({}, '', window.location.pathname);
	}

	async function rotateViewerLink() {
		const state = get(collaborationState);
		if (!state.sessionId || !editorToken) return;
		tokenActionInProgress = true;
		errorMessage = '';
		try {
			const result = await rotateSessionToken(state.sessionId, editorToken, 'viewer');
			if (!result.rotated || !result.token) {
				throw new Error('Viewer token rotation failed');
			}
			viewerToken = result.token;
			viewerUrl = `${window.location.origin}${window.location.pathname}?session=${state.sessionId}&role=viewer&token=${result.token}`;
		} catch (error) {
			console.error('Failed to rotate viewer link:', error);
			errorMessage = 'Failed to rotate viewer link';
		} finally {
			tokenActionInProgress = false;
		}
	}

	async function revokeViewerLink() {
		const state = get(collaborationState);
		if (!state.sessionId || !editorToken || !viewerToken) return;
		tokenActionInProgress = true;
		errorMessage = '';
		try {
			const result = await revokeSessionToken(state.sessionId, editorToken, viewerToken);
			if (!result.revoked) {
				throw new Error('Viewer token revocation failed');
			}
			viewerToken = '';
			viewerUrl = '';
		} catch (error) {
			console.error('Failed to revoke viewer link:', error);
			errorMessage = 'Failed to revoke viewer link';
		} finally {
			tokenActionInProgress = false;
		}
	}

	async function generateShortLivedViewerLink() {
		const state = get(collaborationState);
		if (!state.sessionId || !editorToken) return;
		tokenActionInProgress = true;
		errorMessage = '';
		try {
			const result = await issueInviteToken(state.sessionId, editorToken, 'viewer', 60 * 60);
			if (!result.issued || !result.token) {
				throw new Error('Failed to issue short-lived invite');
			}
			viewerToken = result.token;
			viewerUrl = `${window.location.origin}${window.location.pathname}?session=${state.sessionId}&role=viewer&token=${result.token}`;
		} catch (error) {
			console.error('Failed to issue short-lived viewer link:', error);
			errorMessage = 'Failed to generate short-lived viewer link';
		} finally {
			tokenActionInProgress = false;
		}
	}

	function setTool(tool: Tool) {
		if ($collaborationState.isConnected && $collaborationState.role === 'viewer' && tool !== 'select') {
			activeTool.set('select');
			return;
		}
		if (tool === 'image') {
			const fileInput = document.createElement('input');
			fileInput.type = 'file';
			fileInput.accept = 'image/*';
			fileInput.style.display = 'none';
			fileInput.onchange = (e) => {
				const file = (e.target as HTMLInputElement).files?.[0];
				if (file) {
					const reader = new FileReader();
					reader.onload = (event) => {
						const imageData = event.target?.result as string;
						if (imageData) {
							const img = new Image();
							img.onload = () => {
								const canvas = document.querySelector('canvas');
								if (canvas) {
									const rect = canvas.getBoundingClientRect();
									const centerScreenX = rect.width / 2;
									const centerScreenY = rect.height / 2;
									const worldPos = screenToWorld(centerScreenX, centerScreenY, $viewportOffset, $zoom);
									const x = worldPos.x - img.width / 2;
									const y = worldPos.y - img.height / 2;
									addImage(x, y, img.width, img.height, imageData);
								} else {
									addImage(0, 0, img.width, img.height, imageData);
								}
								activeTool.set('select');
							};
							img.src = imageData;
						}
					};
					reader.readAsDataURL(file);
				}
				document.body.removeChild(fileInput);
			};
			document.body.appendChild(fileInput);
			fileInput.click();
		} else {
			activeTool.set(tool);
			if (tool === 'eraser') {
				selectedRectangles.set([]);
				selectedEllipses.set([]);
				selectedLines.set([]);
				selectedArrows.set([]);
				selectedDiamonds.set([]);
				selectedPaths.set([]);
				selectedImages.set([]);
				selectedTexts.set([]);
				selectedGroups.set([]);
			}
		}
	}

	const tools: Array<{ id: Tool; label: string; icon: string; shortcut: string }> = [
		{ id: 'select' as Tool, label: 'Select', icon: 'cursor', shortcut: '1' },
		{ id: 'rectangle' as Tool, label: 'Rectangle', icon: 'rect', shortcut: '2' },
		{ id: 'diamond' as Tool, label: 'Diamond', icon: 'diamond', shortcut: '3' },
		{ id: 'ellipse' as Tool, label: 'Ellipse', icon: 'circle', shortcut: '4' },
		{ id: 'arrow' as Tool, label: 'Arrow', icon: 'arrow', shortcut: '5' },
		{ id: 'line' as Tool, label: 'Line', icon: 'line', shortcut: '6' },
		{ id: 'freehand' as Tool, label: 'Freehand', icon: 'freehand', shortcut: '7' },
		{ id: 'text' as Tool, label: 'Text', icon: 'text', shortcut: '8' },
		{ id: 'image' as Tool, label: 'Image', icon: 'image', shortcut: '9' },
		{ id: 'eraser' as Tool, label: 'Eraser', icon: 'eraser', shortcut: '0' }
	];

	function toggleTheme() {
		theme.update(t => t === 'light' ? 'dark' : 'light');
		console.log('theme', $theme);
	}

	function toggleGridSnap() {
		gridEnabled.update((enabled) => !enabled);
	}

	function undo() {
		if ($isCollaborating || !$editorApi) return;
		const success = $editorApi.undo();
		if (success) {
			updateAllStoresAfterUndoRedo();
		}
	}

	function redo() {
		if ($isCollaborating || !$editorApi) return;
		const success = $editorApi.redo();
		if (success) {
			updateAllStoresAfterUndoRedo();
		}
	}
</script>

<div class={`fixed right-1.5 bottom-2 left-1.5 z-50 flex flex-wrap items-center justify-center gap-1 overflow-visible rounded-sm p-1 shadow-sm md:absolute md:top-2 md:right-auto md:bottom-auto md:left-2 md:flex-nowrap md:justify-start ${$theme === 'dark' ? 'bg-stone-800 border border-stone-700' : 'bg-white border border-stone-200'}`}>
	{#each tools as tool, index (tool.id)}
		<button
			on:click={() => setTool(tool.id)}
			class={`relative flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans md:h-auto md:w-auto md:px-2 ${$theme === 'dark' ? 'text-stone-200' : 'text-stone-700'}
				transition-colors duration-150 rounded-sm
				${$activeTool === tool.id
					? $theme === 'dark'
						? 'bg-stone-700 border border-stone-500'
						: 'bg-stone-100 border border-stone-400'
					: $theme === 'dark'
						? 'bg-stone-800 hover:bg-stone-700 border border-stone-500'
						: 'bg-white hover:bg-stone-50 border border-stone-200'}`}
			title={`${tool.label} (${tool.shortcut})`}
		>
			<span
				class={`absolute bottom-0.5 right-0.5 hidden text-[9px] font-medium leading-none md:block ${$theme === 'dark' ? 'text-stone-400' : 'text-stone-500'}`}
				aria-hidden="true"
			>
				{tool.shortcut}
			</span>
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
			{:else if tool.icon === 'freehand'}
				<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-pencil" viewBox="0 0 16 16">
					<path d="M12.146.146a.5.5 0 0 1 .708 0l3 3a.5.5 0 0 1 0 .708l-10 10a.5.5 0 0 1-.168.11l-5 2a.5.5 0 0 1-.65-.65l2-5a.5.5 0 0 1 .11-.168zM11.207 2.5 13.5 4.793 14.793 3.5 12.5 1.207zm1.586 3L10.5 3.207 4 9.707V10h.5a.5.5 0 0 1 .5.5v.5h.5a.5.5 0 0 1 .5.5v.5h.293zm-9.761 5.175-.106.106-1.528 3.821 3.821-1.528.106-.106A.5.5 0 0 1 5 12.5V12h-.5a.5.5 0 0 1-.5-.5V11h-.5a.5.5 0 0 1-.468-.325"/>
				</svg>
			{:else if tool.icon === 'image'}
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<rect x="2" y="2" width="12" height="12" rx="1"/>
					<circle cx="5.5" cy="5.5" r="1.5"/>
					<polyline points="2,10 5.5,6.5 8,9 11,6 14,10"/>
					<line x1="14" y1="10" x2="14" y2="14"/>
					<line x1="2" y1="14" x2="14" y2="14"/>
				</svg>
			{:else if tool.icon === 'eraser'}
				<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="currentColor" class="bi bi-eraser" viewBox="0 0 16 16">
					<path d="M8.086 2.207a2 2 0 0 1 2.828 0l3.879 3.879a2 2 0 0 1 0 2.828l-5.5 5.5A2 2 0 0 1 7.879 15H5.12a2 2 0 0 1-1.414-.586l-2.5-2.5a2 2 0 0 1 0-2.828zm2.121.707a1 1 0 0 0-1.414 0L4.16 7.547l5.293 5.293 4.633-4.633a1 1 0 0 0 0-1.414zM8.746 13.547 3.453 8.254 1.914 9.793a1 1 0 0 0 0 1.414l2.5 2.5a1 1 0 0 0 .707.293H7.88a1 1 0 0 0 .707-.293z"/>
				</svg>
			{:else if tool.icon === 'text'}
				<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<line x1="2" y1="2" x2="14" y2="2"/>
					<line x1="8" y1="2" x2="8" y2="15"/>
				</svg>
			{/if}
		</button>
		{#if index === 7}
			<div class="basis-full md:hidden"></div>
		{/if}
	{/each}

	<button
		on:click={toggleGridSnap}
		class={`flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm md:h-auto md:w-auto md:px-2
			${$gridEnabled
				? $theme === 'dark'
					? 'text-stone-100 bg-stone-700 border border-stone-500'
					: 'text-stone-800 bg-stone-100 border border-stone-400'
				: $theme === 'dark'
					? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-500'
					: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
		title={`Snap to grid (${$gridSize}px)`}
	>
		<svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.25">
			<path d="M1 1h14v14H1z" />
			<path d="M1 5.5h14M1 10.5h14M5.5 1v14M10.5 1v14" />
		</svg>
	</button>
	
	<div class="relative collaboration-menu-container">
		<button
			on:click={(e) => {
				e.stopPropagation();
				collaborationMenuOpen = !collaborationMenuOpen;
			}}
			class={`flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm relative md:h-auto md:w-auto md:px-2
				${collaborationMenuOpen
					? $theme === 'dark'
						? 'bg-stone-700 border border-stone-500'
						: 'bg-stone-100 border border-stone-400'
					: $theme === 'dark'
						? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-500'
						: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
			title={$isCollaborating ? `Collaborating (${$collaboratorCount})` : 'Share & Collaborate'}
		>
			{#if $isCollaborating}
				<div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
				<span class="hidden text-xs md:inline">{$collaboratorCount}</span>
			{:else}
				<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<path d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
				</svg>
			{/if}
		</button>

		{#if collaborationMenuOpen}
			<div 
				on:click|stopPropagation
				on:keydown|stopPropagation
				role="dialog"
				aria-label="Collaboration menu"
				tabindex="-1"
				class={`absolute bottom-full left-1/2 mb-2 w-[min(22rem,calc(100vw-0.75rem))] -translate-x-1/2 md:bottom-auto md:left-0 md:top-full md:mt-1 md:mb-0 md:w-80 md:translate-x-0 ${$theme === 'dark' ? 'bg-stone-800 border border-stone-700' : 'bg-white border border-stone-200'} rounded-sm shadow-lg p-3 z-50`}
			>
				{#if $isCollaborating}
					<div class="space-y-3">
						<div>
							<h3 class={`text-xs font-semibold mb-2 ${$theme === 'dark' ? 'text-stone-200' : 'text-stone-700'}`}>Collaboration Active</h3>
							<div class="flex items-center gap-2 mb-2">
								<div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
								<span class={`text-xs ${$theme === 'dark' ? 'text-stone-400' : 'text-stone-600'}`}>
									{$collaboratorCount} {$collaboratorCount === 1 ? 'person' : 'people'} connected
								</span>
							</div>
						</div>

							<div>
								<label for="share-link-input" class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'} mb-1 block`}>Share Link</label>
								<div class="flex gap-2">
								<input
									id="share-link-input"
									type="text"
									readonly
									value={shareUrl}
									class={`flex-1 px-2 py-1.5 text-xs ${$theme === 'dark' ? 'bg-stone-900 border-stone-700 text-stone-200' : 'bg-stone-50 border-stone-300 text-stone-700'} border rounded-sm focus:outline-none focus:ring-2 focus:ring-blue-500`}
								/>
									<button
										id="copy-button"
										on:click={() => copyShareLink(shareUrl, 'copy-button')}
										class={`px-3 py-1.5 text-xs rounded-sm transition-colors ${$theme === 'dark' ? 'bg-blue-600 hover:bg-blue-700' : 'bg-blue-500 hover:bg-blue-600'} text-white`}
									>
										Copy
									</button>
								</div>
							</div>
					<div>
						<label for="viewer-link-input" class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'} mb-1 block`}>Viewer Link</label>
						<div class="flex gap-2">
									<input
										id="viewer-link-input"
										type="text"
										readonly
										value={viewerUrl}
										class={`flex-1 px-2 py-1.5 text-xs ${$theme === 'dark' ? 'bg-stone-900 border-stone-700 text-stone-200' : 'bg-stone-50 border-stone-300 text-stone-700'} border rounded-sm focus:outline-none focus:ring-2 focus:ring-blue-500`}
									/>
							<button
								id="copy-viewer-button"
								on:click={() => copyShareLink(viewerUrl, 'copy-viewer-button')}
								disabled={!viewerUrl}
								class={`px-3 py-1.5 text-xs rounded-sm transition-colors ${$theme === 'dark' ? 'bg-stone-700 hover:bg-stone-600' : 'bg-stone-500 hover:bg-stone-600'} text-white`}
							>
								Copy
							</button>
						</div>
						{#if $collaborationState.role === 'editor'}
							<div class="mt-2 flex gap-2">
								<button
									on:click={generateShortLivedViewerLink}
									disabled={tokenActionInProgress || !editorToken}
									class={`px-2 py-1 text-[11px] rounded-sm transition-colors disabled:opacity-50 ${$theme === 'dark' ? 'bg-blue-700 hover:bg-blue-600 text-white' : 'bg-blue-500 hover:bg-blue-600 text-white'}`}
								>
									1h Viewer Link
								</button>
								<button
									on:click={rotateViewerLink}
									disabled={tokenActionInProgress || !editorToken}
									class={`px-2 py-1 text-[11px] rounded-sm transition-colors disabled:opacity-50 ${$theme === 'dark' ? 'bg-amber-700 hover:bg-amber-600 text-white' : 'bg-amber-500 hover:bg-amber-600 text-white'}`}
								>
									Rotate Viewer Link
								</button>
								<button
									on:click={revokeViewerLink}
									disabled={tokenActionInProgress || !viewerToken || !editorToken}
									class={`px-2 py-1 text-[11px] rounded-sm transition-colors disabled:opacity-50 ${$theme === 'dark' ? 'bg-red-700 hover:bg-red-600 text-white' : 'bg-red-500 hover:bg-red-600 text-white'}`}
								>
									Revoke Viewer Link
								</button>
							</div>
						{/if}
					</div>

					{#if errorMessage}
						<div class={`p-2 rounded-sm text-xs ${$theme === 'dark' ? 'bg-red-900/20 border-red-800 text-red-400' : 'bg-red-50 border-red-200 text-red-700'} border`}>
							{errorMessage}
						</div>
					{/if}

					<div class={`rounded-sm border p-2 ${$theme === 'dark' ? 'border-stone-700 bg-stone-900/60' : 'border-stone-200 bg-stone-50'}`}>
						<div class={`text-[11px] font-semibold ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>Link Security</div>
						<div class={`mt-1 text-[11px] ${$theme === 'dark' ? 'text-stone-400' : 'text-stone-600'}`}>
							Viewer link: {viewerToken ? 'Active' : 'Revoked'}
						</div>
						<div class={`text-[11px] ${$theme === 'dark' ? 'text-stone-400' : 'text-stone-600'}`}>
							Expires in: {viewerTokenExpiryLabel}
						</div>
					</div>

						<div>
							<div class={`text-xs font-medium ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'} mb-2 block`}>Collaborators</div>
							<div class="space-y-1.5" role="list">
								{#each $collaborationState.collaborators as collaborator}
									<div class={`flex items-center gap-2 text-xs ${$theme === 'dark' ? 'text-stone-300' : 'text-stone-700'}`}>
										<div
											class="w-3 h-3 rounded-full"
											style="background-color: {collaborator.color};"
										></div>
										<span>{collaborator.name}</span>
										{#if collaborator.id === $collaborationState.clientId}
											<span class={`text-xs ${$theme === 'dark' ? 'text-stone-500' : 'text-stone-500'}`}>(You)</span>
										{/if}
									</div>
								{/each}
							</div>
						</div>

						<button
							on:click={stopCollaboration}
							class={`w-full px-3 py-2 text-xs rounded-sm transition-colors ${$theme === 'dark' ? 'bg-red-600 hover:bg-red-700' : 'bg-red-500 hover:bg-red-600'} text-white`}
						>
							Stop Collaboration
						</button>
					</div>
				{:else}
					<div class="space-y-3">
						<h3 class={`text-xs font-semibold ${$theme === 'dark' ? 'text-stone-200' : 'text-stone-700'}`}>Start Collaboration</h3>
						<p class={`text-xs ${$theme === 'dark' ? 'text-stone-400' : 'text-stone-600'}`}>
							Create a session to collaborate with others in real-time.
						</p>

						{#if errorMessage}
							<div class={`p-2 rounded-sm text-xs ${$theme === 'dark' ? 'bg-red-900/20 border-red-800 text-red-400' : 'bg-red-50 border-red-200 text-red-700'} border`}>
								{errorMessage}
							</div>
						{/if}

						<button
							on:click={startCollaboration}
							disabled={isCreatingSession}
							class={`w-full px-3 py-2 text-xs rounded-sm transition-colors disabled:opacity-50 disabled:cursor-not-allowed ${$theme === 'dark' ? 'bg-blue-600 hover:bg-blue-700' : 'bg-blue-500 hover:bg-blue-600'} text-white`}
						>
							{isCreatingSession ? 'Creating...' : 'Create Session'}
						</button>
					</div>
				{/if}
			</div>
		{/if}
	</div>

	<div class="flex items-center gap-1 md:hidden">
		<button
			type="button"
			on:click={undo}
			disabled={$isCollaborating}
			class={`flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm
				${$theme === 'dark'
					? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700 disabled:opacity-50 disabled:cursor-not-allowed'
					: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200 disabled:opacity-50 disabled:cursor-not-allowed'}`}
			title={$isCollaborating ? 'Undo disabled in collaboration' : 'Undo'}
		>
			<svg width="16" height="16" viewBox="0 0 8 8" fill="currentColor">
				<path d="M4.5 1C2.57 1 1 2.57 1 4.5V5H0l2 2 2-2H3v-.5a2.5 2.5 0 0 1 5 0C8 2.57 6.43 1 4.5 1z"/>
			</svg>
		</button>
		<button
			type="button"
			on:click={redo}
			disabled={$isCollaborating}
			class={`flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm
				${$theme === 'dark'
					? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700 disabled:opacity-50 disabled:cursor-not-allowed'
					: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200 disabled:opacity-50 disabled:cursor-not-allowed'}`}
			title={$isCollaborating ? 'Redo disabled in collaboration' : 'Redo'}
		>
			<svg width="16" height="16" viewBox="0 0 8 8" fill="currentColor">
				<path d="M3.5 1C1.57 1 0 2.57 0 4.5a2.5 2.5 0 0 1 5 0V5H4l2 2 2-2H7v-.5C7 2.57 5.43 1 3.5 1z"/>
			</svg>
		</button>
		<button
			type="button"
			on:click={zoomIn}
			class={`flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm
				${$theme === 'dark'
					? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
					: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
			title="Zoom in"
		>
			+
		</button>
		<button
			type="button"
			on:click={zoomOut}
			class={`flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm
				${$theme === 'dark'
					? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
					: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
			title="Zoom out"
		>
			-
		</button>
	</div>

	<button
		on:click={toggleTheme}
	class={`flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm md:h-auto md:w-auto md:px-2
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

	<button
		on:click={() => shortcutsPanelOpen = true}
	class={`flex h-8 w-8 shrink-0 items-center justify-center gap-1.5 px-0 py-1.5 text-xs font-sans transition-colors duration-150 rounded-sm md:h-auto md:w-auto md:px-2
			${$theme === 'dark'
				? 'text-stone-200 bg-stone-800 hover:bg-stone-700 border border-stone-700'
				: 'text-stone-700 bg-white hover:bg-stone-50 border border-stone-200'}`}
		title="Keyboard Shortcuts"
	>
		<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
			<circle cx="12" cy="12" r="10" />
			<line x1="12" y1="16" x2="12" y2="12" />
			<line x1="12" y1="8" x2="12.01" y2="8" />
		</svg>
	</button>
</div>

<ShortcutsPanel bind:isOpen={shortcutsPanelOpen} />
