import { writable } from 'svelte/store';

export type UiStatus = {
	ready: boolean;
	busy: boolean;
};

const initialStatus: UiStatus = {
	ready: false,
	busy: false
};

export const status = writable<UiStatus>(initialStatus);
