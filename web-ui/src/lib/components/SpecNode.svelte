<script lang="ts">
	import { Handle, Position } from '@xyflow/svelte';

	let { data } = $props<{
		data: {
			title: string;
			specId: string;
			tags: string[];
			version: number;
			isDisconnected: boolean;
			highlight?: 'selected' | 'downstream' | 'upstream' | 'dimmed' | null;
		};
	}>();
</script>

<div
	class="spec-node"
	class:disconnected={data.isDisconnected}
	class:selected={data.highlight === 'selected'}
	class:downstream={data.highlight === 'downstream'}
	class:upstream={data.highlight === 'upstream'}
	class:dimmed={data.highlight === 'dimmed'}
>
	<Handle type="target" position={Position.Top} />

	<div class="header">
		<span class="title">{data.title}</span>
		<span class="version">v{data.version}</span>
	</div>

	<span class="id">{data.specId}</span>

	{#if data.tags.length > 0}
		<div class="tags">
			{#each data.tags as tag}
				<span class="tag">{tag}</span>
			{/each}
		</div>
	{/if}

	<Handle type="source" position={Position.Bottom} />
</div>

<style>
	.spec-node {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 12px;
		min-width: 200px;
		max-width: 260px;
		font-family: var(--font-sans);
		transition: opacity 0.2s;
	}

	.spec-node.disconnected {
		opacity: 0.4;
	}

	.spec-node.selected {
		border-color: var(--color-accent);
		box-shadow: 0 0 12px rgba(0, 210, 255, 0.3);
	}

	.spec-node.downstream {
		border-color: var(--color-impact);
		box-shadow: 0 0 8px rgba(233, 69, 96, 0.2);
	}

	.spec-node.upstream {
		border-color: var(--color-upstream);
		box-shadow: 0 0 8px rgba(92, 124, 250, 0.2);
	}

	.spec-node.dimmed {
		opacity: 0.15;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 4px;
	}

	.title {
		font-weight: 600;
		font-size: 13px;
		color: var(--color-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
	}

	.version {
		font-size: 10px;
		color: var(--color-accent);
		background: rgba(0, 210, 255, 0.1);
		padding: 2px 6px;
		border-radius: 4px;
		margin-left: 8px;
		flex-shrink: 0;
	}

	.id {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--color-text-muted);
		display: block;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		margin-bottom: 6px;
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.tag {
		font-size: 10px;
		padding: 2px 6px;
		border-radius: 4px;
		background: var(--color-surface-alt);
		color: var(--color-upstream);
		border: 1px solid rgba(92, 124, 250, 0.2);
	}

	.spec-node :global(.svelte-flow__handle) {
		width: 8px;
		height: 8px;
		background: var(--color-border);
		border: 1px solid var(--color-surface);
		opacity: 0;
		transition:
			opacity 0.15s,
			transform 0.15s,
			background 0.15s;
	}

	.spec-node:hover :global(.svelte-flow__handle) {
		opacity: 1;
	}

	.spec-node :global(.svelte-flow__handle:hover) {
		opacity: 1;
		transform: scale(1.5);
		background: var(--color-accent);
	}
</style>
