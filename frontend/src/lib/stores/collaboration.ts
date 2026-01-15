import { writable, derived, get } from 'svelte/store';

export interface Collaborator {
	id: string;
	name: string;
	color: string;
}

export interface CollaborationState {
	isConnected: boolean;
	sessionId: string | null;
	clientId: string | null;
	collaborators: Collaborator[];
	isHost: boolean;
}

const initialState: CollaborationState = {
	isConnected: false,
	sessionId: null,
	clientId: null,
	collaborators: [],
	isHost: false,
};

export const collaborationState = writable<CollaborationState>(initialState);

export const isCollaborating = derived(collaborationState, ($state) => $state.isConnected && $state.sessionId !== null);

export const collaboratorCount = derived(collaborationState, ($state) => $state.collaborators.length);

