export type UiStatus = {
	ready: boolean;
	busy: boolean;
};

export const status = $state<UiStatus>({
	ready: false,
	busy: false
});
