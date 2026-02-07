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
	sessionId: string | null;
	clientId: string | null;
	collaborators: Collaborator[];
	presenceByClient: Record<string, CollaboratorPresence>;
	isHost: boolean;
}

const initialState: CollaborationState = {
	isConnected: false,
	sessionId: null,
	clientId: null,
	collaborators: [],
	presenceByClient: {},
	isHost: false,
};

export const collaborationState = writable<CollaborationState>(initialState);

export const isCollaborating = derived(collaborationState, ($state) => $state.isConnected && $state.sessionId !== null);

export const collaboratorCount = derived(collaborationState, ($state) => $state.collaborators.length);
