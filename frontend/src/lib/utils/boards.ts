export type BoardMeta = {
	id: string;
	name: string;
	updatedAt: number;
};

const BOARDS_META_KEY = 'rustboard-boards-meta';
const CURRENT_BOARD_ID_KEY = 'rustboard-current-board-id';
const DEFAULT_BOARD_ID = 'default-board';

function boardStateKey(boardId: string): string {
	return `rustboard-state-${boardId}`;
}

function readBoards(): BoardMeta[] {
	if (typeof window === 'undefined') return [];
	try {
		const raw = localStorage.getItem(BOARDS_META_KEY);
		if (!raw) return [];
		const parsed = JSON.parse(raw) as BoardMeta[];
		if (!Array.isArray(parsed)) return [];
		return parsed.filter((board) => board && typeof board.id === 'string' && typeof board.name === 'string');
	} catch {
		return [];
	}
}

function writeBoards(boards: BoardMeta[]): void {
	if (typeof window === 'undefined') return;
	localStorage.setItem(BOARDS_META_KEY, JSON.stringify(boards));
}

function nowTs(): number {
	return Date.now();
}

function makeBoardId(): string {
	return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export function ensureDefaultBoard(): BoardMeta {
	const boards = readBoards();
	const existing = boards.find((board) => board.id === DEFAULT_BOARD_ID);
	if (existing) return existing;
	const created: BoardMeta = { id: DEFAULT_BOARD_ID, name: 'My Board', updatedAt: nowTs() };
	writeBoards([created, ...boards]);
	if (typeof window !== 'undefined' && !localStorage.getItem(CURRENT_BOARD_ID_KEY)) {
		localStorage.setItem(CURRENT_BOARD_ID_KEY, created.id);
	}
	return created;
}

export function listBoards(): BoardMeta[] {
	ensureDefaultBoard();
	return readBoards().sort((a, b) => b.updatedAt - a.updatedAt);
}

export function getCurrentBoardId(): string {
	if (typeof window === 'undefined') return DEFAULT_BOARD_ID;
	ensureDefaultBoard();
	return localStorage.getItem(CURRENT_BOARD_ID_KEY) || DEFAULT_BOARD_ID;
}

export function setCurrentBoardId(boardId: string): void {
	if (typeof window === 'undefined') return;
	localStorage.setItem(CURRENT_BOARD_ID_KEY, boardId);
}

export function saveBoardSnapshot(boardId: string, serialized: string): void {
	if (typeof window === 'undefined') return;
	localStorage.setItem(boardStateKey(boardId), serialized);
	const boards = readBoards();
	const index = boards.findIndex((board) => board.id === boardId);
	if (index >= 0) {
		boards[index] = { ...boards[index], updatedAt: nowTs() };
		writeBoards(boards);
	}
}

export function loadBoardSnapshot(boardId: string): string | null {
	if (typeof window === 'undefined') return null;
	return localStorage.getItem(boardStateKey(boardId));
}

export function createBoard(name: string, sourceSerialized: string = ''): BoardMeta {
	const boards = readBoards();
	const board: BoardMeta = {
		id: makeBoardId(),
		name: name.trim() || 'Untitled Board',
		updatedAt: nowTs(),
	};
	writeBoards([board, ...boards]);
	localStorage.setItem(boardStateKey(board.id), sourceSerialized);
	setCurrentBoardId(board.id);
	return board;
}

export function renameBoard(boardId: string, name: string): void {
	const nextName = name.trim();
	if (!nextName) return;
	const boards = readBoards();
	const index = boards.findIndex((board) => board.id === boardId);
	if (index < 0) return;
	boards[index] = { ...boards[index], name: nextName, updatedAt: nowTs() };
	writeBoards(boards);
}

export function duplicateBoard(boardId: string, newName: string): BoardMeta | null {
	const snapshot = loadBoardSnapshot(boardId);
	if (snapshot === null) return null;
	return createBoard(newName, snapshot);
}

export function deleteBoard(boardId: string): string {
	const boards = readBoards();
	if (boards.length <= 1) {
		return boardId;
	}
	const filtered = boards.filter((board) => board.id !== boardId);
	writeBoards(filtered);
	localStorage.removeItem(boardStateKey(boardId));
	const nextCurrent = filtered[0].id;
	setCurrentBoardId(nextCurrent);
	return nextCurrent;
}

export function serializeCurrentBoard(api: { serialize: () => string }): void {
	const boardId = getCurrentBoardId();
	saveBoardSnapshot(boardId, api.serialize());
}
