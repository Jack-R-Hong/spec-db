<script lang="ts">
	import type { Node } from '@xyflow/svelte';

	let {
		nodes,
		onfilter,
		onclear
	} = $props<{
		nodes: Node[];
		onfilter: (matchingIds: string[]) => void;
		onclear: () => void;
	}>();

	let query = $state('');
	let inputEl = $state<HTMLInputElement | null>(null);
	let visible = $state(false);

	function handleInput() {
		if (!query.trim()) {
			onclear();
			return;
		}
		const q = query.toLowerCase();
		const matching = nodes.filter((n) => {
			const d = n.data as { title: string; specId: string; tags: string[] };
			return (
				d.title.toLowerCase().includes(q) ||
				d.specId.toLowerCase().includes(q) ||
				d.tags.some((t: string) => t.toLowerCase().includes(q))
			);
		});
		onfilter(matching.map((n) => n.id));
	}

	function dismiss() {
		query = '';
		visible = false;
		onclear();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			dismiss();
		}
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
			e.preventDefault();
			visible = true;
			requestAnimationFrame(() => inputEl?.focus());
		}
		if ((e.ctrlKey || e.metaKey) && e.key === '0') {
			e.preventDefault();
		}
	}
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

{#if visible}
	<div class="search-overlay">
		<input
			bind:this={inputEl}
			bind:value={query}
			oninput={handleInput}
			onkeydown={handleKeydown}
			type="text"
			placeholder="Search specs… (Esc to close)"
			class="search-input"
		/>
	</div>
{/if}

<style>
	.search-overlay {
		position: fixed;
		top: 52px;
		left: 50%;
		transform: translateX(-50%);
		z-index: 100;
	}

	.search-input {
		width: 400px;
		padding: 10px 16px;
		font-size: 14px;
		font-family: var(--font-sans);
		background: var(--color-surface);
		color: var(--color-text);
		border: 1px solid var(--color-upstream);
		border-radius: 8px;
		outline: none;
		box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
	}

	.search-input::placeholder {
		color: var(--color-text-muted);
	}
</style>
