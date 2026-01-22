// Alert Dialog Store - provides app-wide alert/confirm functionality
// Use this instead of native browser alert() and confirm()

type AlertType = 'info' | 'success' | 'warning' | 'error';

interface AlertOptions {
	title?: string;
	message: string;
	type?: AlertType;
	confirmText?: string;
	cancelText?: string;
}

interface AlertState {
	isOpen: boolean;
	title: string;
	message: string;
	type: AlertType;
	confirmText: string;
	cancelText: string | null;
	resolve: ((value: boolean) => void) | null;
}

function createAlertDialogStore() {
	const state = $state<AlertState>({
		isOpen: false,
		title: '',
		message: '',
		type: 'info',
		confirmText: 'OK',
		cancelText: null,
		resolve: null,
	});

	function alert(options: AlertOptions | string): Promise<void> {
		const opts = typeof options === 'string' ? { message: options } : options;

		return new Promise((resolve) => {
			state.isOpen = true;
			state.title = opts.title ?? '';
			state.message = opts.message;
			state.type = opts.type ?? 'info';
			state.confirmText = opts.confirmText ?? 'OK';
			state.cancelText = null;
			state.resolve = () => resolve();
		});
	}

	function confirm(options: AlertOptions | string): Promise<boolean> {
		const opts = typeof options === 'string' ? { message: options } : options;

		return new Promise((resolve) => {
			state.isOpen = true;
			state.title = opts.title ?? '';
			state.message = opts.message;
			state.type = opts.type ?? 'warning';
			state.confirmText = opts.confirmText ?? 'Confirm';
			state.cancelText = opts.cancelText ?? 'Cancel';
			state.resolve = resolve;
		});
	}

	function handleConfirm() {
		const resolve = state.resolve;
		state.isOpen = false;
		state.resolve = null;
		resolve?.(true);
	}

	function handleCancel() {
		const resolve = state.resolve;
		state.isOpen = false;
		state.resolve = null;
		resolve?.(false);
	}

	return {
		get isOpen() {
			return state.isOpen;
		},
		get title() {
			return state.title;
		},
		get message() {
			return state.message;
		},
		get type() {
			return state.type;
		},
		get confirmText() {
			return state.confirmText;
		},
		get cancelText() {
			return state.cancelText;
		},
		alert,
		confirm,
		handleConfirm,
		handleCancel,
	};
}

export const alertDialogStore = createAlertDialogStore();
