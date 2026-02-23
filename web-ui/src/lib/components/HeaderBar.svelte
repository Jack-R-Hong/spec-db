<script lang="ts">
	interface StatusData {
		spec_count: number;
		last_sync_sha: string;
		consistency: string;
		drift_detected: boolean;
	}

	let { onrefresh } = $props<{ onrefresh: () => void }>();

	let status = $state<StatusData | null>(null);
	let syncing = $state(false);
	let syncError = $state<string | null>(null);

	function fetchStatus() {
		fetch('/api/status')
			.then((r) => (r.ok ? r.json() : null))
			.then((data) => {
				status = data;
			})
			.catch(() => {});
	}

	async function triggerSync(mode: string) {
		syncing = true;
		syncError = null;
		try {
			const res = await fetch(`/api/sync?mode=${mode}`, { method: 'POST' });
			if (!res.ok) {
				const err = await res.json();
				syncError = err.message ?? 'Sync failed';
			} else {
				fetchStatus();
				onrefresh();
			}
		} catch (e) {
			syncError = 'Network error';
		}
		syncing = false;
	}

	function shortSha(sha: string): string {
		return sha ? sha.substring(0, 7) : '—';
	}

	$effect(() => {
		fetchStatus();
		const interval = setInterval(fetchStatus, 30_000);
		return () => clearInterval(interval);
	});
</script>

<header class="header-bar">
	<div class="left">
		<span class="logo">◇ Lattice</span>
	</div>

	<div class="center">
		{#if status}
			<span class="stat">{status.spec_count} specs</span>
			<span class="separator">·</span>
			<span class="stat mono">{shortSha(status.last_sync_sha)}</span>
			{#if status.drift_detected}
				<span class="drift" title="Index drift detected — consider a full rebuild">⚠</span>
			{/if}
		{/if}
		{#if syncError}
			<span class="sync-error">{syncError}</span>
		{/if}
	</div>

	<div class="right">
		<button class="btn" disabled={syncing} onclick={() => triggerSync('incremental')}>
			{syncing ? '…' : 'Sync'}
		</button>
		<button class="btn btn-secondary" disabled={syncing} onclick={() => triggerSync('full')}>
			Rebuild
		</button>
	</div>
</header>

<style>
	.header-bar {
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 16px;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border);
		z-index: 60;
		position: relative;
		flex-shrink: 0;
	}

	.left,
	.center,
	.right {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.logo {
		font-weight: 700;
		font-size: 14px;
		color: var(--color-accent);
	}

	.stat {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.mono {
		font-family: var(--font-mono);
	}

	.separator {
		color: var(--color-border);
		font-size: 12px;
	}

	.drift {
		color: var(--color-warning);
		font-size: 14px;
		cursor: help;
	}

	.sync-error {
		font-size: 11px;
		color: var(--color-impact);
	}

	.btn {
		font-size: 11px;
		padding: 4px 12px;
		border-radius: 4px;
		border: 1px solid var(--color-border);
		background: var(--color-surface-alt);
		color: var(--color-text);
		cursor: pointer;
		font-family: var(--font-sans);
	}

	.btn:hover:not(:disabled) {
		background: var(--color-upstream);
		color: white;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-secondary {
		border-color: var(--color-warning);
		color: var(--color-warning);
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--color-warning);
		color: var(--color-base);
	}
</style>
