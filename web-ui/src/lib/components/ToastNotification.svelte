<script lang="ts">
	type ToastType = 'confirm' | 'error' | 'undo';

	interface Toast {
		type: ToastType;
		message: string;
		onconfirm?: () => void;
		oncancel?: () => void;
		onundo?: () => void;
	}

	let active = $state<Toast | null>(null);
	let undoTimer = $state<ReturnType<typeof setTimeout> | null>(null);
	let undoCountdown = $state(5);
	let undoInterval = $state<ReturnType<typeof setInterval> | null>(null);

	export function showConfirm(message: string, onconfirm: () => void, oncancel: () => void) {
		active = { type: 'confirm', message, onconfirm, oncancel };
	}

	export function showError(message: string) {
		active = { type: 'error', message };
		setTimeout(() => {
			if (active?.type === 'error') active = null;
		}, 5000);
	}

	export function showUndo(onundo: () => void) {
		undoCountdown = 5;
		active = { type: 'undo', message: 'Change committed.', onundo };

		if (undoInterval) clearInterval(undoInterval);
		undoInterval = setInterval(() => {
			undoCountdown -= 1;
			if (undoCountdown <= 0) {
				dismiss();
			}
		}, 1000);

		if (undoTimer) clearTimeout(undoTimer);
		undoTimer = setTimeout(dismiss, 5200);
	}

	function dismiss() {
		if (undoTimer) clearTimeout(undoTimer);
		if (undoInterval) clearInterval(undoInterval);
		undoTimer = null;
		undoInterval = null;
		active = null;
	}

	function handleConfirm() {
		active?.onconfirm?.();
		dismiss();
	}

	function handleCancel() {
		active?.oncancel?.();
		dismiss();
	}

	function handleUndo() {
		active?.onundo?.();
		dismiss();
	}
</script>

{#if active}
	<div class="toast" class:error={active.type === 'error'}>
		<span class="toast-msg">{active.message}</span>
		{#if active.type === 'confirm'}
			<button class="toast-btn confirm" onclick={handleConfirm}>Confirm</button>
			<button class="toast-btn cancel" onclick={handleCancel}>Cancel</button>
		{:else if active.type === 'undo'}
			<button class="toast-btn undo" onclick={handleUndo}>Undo ({undoCountdown}s)</button>
		{/if}
	</div>
{/if}

<style>
	.toast {
		position: fixed;
		bottom: 24px;
		left: 50%;
		transform: translateX(-50%);
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 10px 16px;
		display: flex;
		align-items: center;
		gap: 12px;
		z-index: 100;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
		font-family: var(--font-sans);
		font-size: 13px;
		color: var(--color-text);
	}

	.toast.error {
		border-color: var(--color-impact);
		color: var(--color-impact);
	}

	.toast-msg {
		white-space: nowrap;
	}

	.toast-btn {
		font-size: 12px;
		padding: 4px 12px;
		border-radius: 4px;
		border: 1px solid var(--color-border);
		background: var(--color-surface-alt);
		color: var(--color-text);
		cursor: pointer;
		font-family: var(--font-sans);
		white-space: nowrap;
	}

	.toast-btn:hover {
		background: var(--color-upstream);
		color: white;
	}

	.toast-btn.confirm {
		border-color: var(--color-success);
		color: var(--color-success);
	}

	.toast-btn.confirm:hover {
		background: var(--color-success);
		color: var(--color-base);
	}

	.toast-btn.cancel {
		color: var(--color-text-muted);
	}

	.toast-btn.cancel:hover {
		background: var(--color-impact);
		color: white;
	}

	.toast-btn.undo {
		border-color: var(--color-warning);
		color: var(--color-warning);
	}

	.toast-btn.undo:hover {
		background: var(--color-warning);
		color: var(--color-base);
	}
</style>
