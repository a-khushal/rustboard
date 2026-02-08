import { writable, derived, get } from 'svelte/store';

export interface Collaborator {
	id: string;
	name: string;
	color: string;
}

export interface CollaboratorPresence {
	cursor: { x: number; y: number } | null;
	selectedIds: number[];
	updatedAt: number;
}

export interface CollaborationState {
	isConnected: boolean;
	connectionStatus: 'disconnected' | 'connecting' | 'connected' | 'reconnecting' | 'error';
	isResyncing: boolean;
	lastError: string | null;
	sessionId: string | null;
	clientId: string | null;
	collaborators: Collaborator[];
	presenceByClient: Record<string, CollaboratorPresence>;
	role: 'editor' | 'viewer';
	isHost: boolean;
}

const initialState: CollaborationState = {
	isConnected: false,
	connectionStatus: 'disconnected',
	isResyncing: false,
	lastError: null,
	sessionId: null,
	clientId: null,
	collaborators: [],
	presenceByClient: {},
	role: 'editor',
	isHost: false,
};

export const collaborationState = writable<CollaborationState>(initialState);

export const isCollaborating = derived(collaborationState, ($state) => $state.isConnected && $state.sessionId !== null);

export const collaboratorCount = derived(collaborationState, ($state) => $state.collaborators.length);
