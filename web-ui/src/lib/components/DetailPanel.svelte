<script lang="ts">
	import snarkdown from 'snarkdown';

	interface SpecDetail {
		id: string;
		title: string;
		version: number;
		tags: string[];
		owner: string | null;
		created: string;
		depends_on: string[];
		body: string;
		inbound_edges: { source: string; edge_type: string; trust: number; origin: string }[];
		outbound_edges: { target: string; edge_type: string; trust: number; origin: string }[];
	}

	let {
		selectedNodeId,
		downstreamIds = [],
		onselectnode,
		onsave
	} = $props<{
		selectedNodeId: string | null;
		downstreamIds: string[];
		onselectnode: (id: string) => void;
		onsave?: (specId: string, changes: Record<string, unknown>) => void;
	}>();

	let spec = $state<SpecDetail | null>(null);
	let loading = $state(false);
	let isEditing = $state(false);

	let editTitle = $state('');
	let editOwner = $state('');
	let editTagsStr = $state('');
	let editDepsStr = $state('');

	$effect(() => {
		if (!selectedNodeId) {
			spec = null;
			isEditing = false;
			return;
		}
		isEditing = false;
		loading = true;
		fetch(`/api/spec/${encodeURIComponent(selectedNodeId)}`)
			.then((r) => (r.ok ? r.json() : null))
			.then((data) => {
				spec = data;
				loading = false;
			})
			.catch(() => {
				spec = null;
				loading = false;
			});
	});

	function enterEditMode() {
		if (!spec) return;
		editTitle = spec.title;
		editOwner = spec.owner ?? '';
		editTagsStr = spec.tags.join(', ');
		editDepsStr = spec.depends_on.join(', ');
		isEditing = true;
	}

	function cancelEdit() {
		isEditing = false;
	}

	function saveEdit() {
		if (!spec || !onsave) return;
		const changes: Record<string, unknown> = {};
		if (editTitle !== spec.title) changes.title = editTitle;
		if (editOwner !== (spec.owner ?? '')) changes.owner = editOwner;
		const newTags = editTagsStr
			.split(',')
			.map((t) => t.trim())
			.filter(Boolean);
		if (JSON.stringify(newTags) !== JSON.stringify(spec.tags)) changes.tags = newTags;
		const newDeps = editDepsStr
			.split(',')
			.map((d) => d.trim())
			.filter(Boolean);
		if (JSON.stringify(newDeps) !== JSON.stringify(spec.depends_on)) changes.depends_on = newDeps;

		if (Object.keys(changes).length === 0) {
			isEditing = false;
			return;
		}
		onsave(spec.id, changes);
		isEditing = false;
	}

	function handlePanelKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && isEditing) {
			e.stopPropagation();
			cancelEdit();
		}
	}

	function trustColor(trust: number): string {
		if (trust > 0.8) return 'var(--color-success)';
		if (trust >= 0.5) return 'var(--color-warning)';
		return 'var(--color-impact)';
	}

	const renderedBody = $derived(spec?.body ? snarkdown(spec.body) : '');
</script>

{#if selectedNodeId}
	<aside class="detail-panel" class:open={!!selectedNodeId} onkeydown={handlePanelKeydown}>
		{#if loading}
			<div class="panel-loading">Loading…</div>
		{:else if spec}
			<header class="panel-header">
				<h2>{spec.title}</h2>
				<div class="panel-header-right">
					{#if !isEditing}
						<button class="edit-btn" onclick={enterEditMode}>Edit</button>
					{/if}
					<span class="panel-version">v{spec.version}</span>
				</div>
			</header>

			{#if isEditing}
				<section class="section">
					<h3>Edit Frontmatter</h3>
					<div class="edit-form">
						<label>
							<span class="edit-label">Title</span>
							<input type="text" bind:value={editTitle} class="edit-input" />
						</label>
						<label>
							<span class="edit-label">Owner</span>
							<input type="text" bind:value={editOwner} class="edit-input" />
						</label>
						<label>
							<span class="edit-label">Tags</span>
							<input
								type="text"
								bind:value={editTagsStr}
								class="edit-input"
								placeholder="comma-separated"
							/>
						</label>
						<label>
							<span class="edit-label">Depends On</span>
							<input
								type="text"
								bind:value={editDepsStr}
								class="edit-input"
								placeholder="comma-separated spec IDs"
							/>
						</label>
						<div class="edit-actions">
							<button class="save-btn" onclick={saveEdit}>Save</button>
							<button class="cancel-btn" onclick={cancelEdit}>Cancel</button>
						</div>
					</div>
				</section>
			{:else}
				<section class="section">
					<h3>Frontmatter</h3>
					<dl class="fields">
						<dt>ID</dt>
						<dd class="mono">{spec.id}</dd>
						<dt>Owner</dt>
						<dd>{spec.owner ?? '—'}</dd>
						<dt>Created</dt>
						<dd>{spec.created}</dd>
						{#if spec.tags.length > 0}
							<dt>Tags</dt>
							<dd>
								<div class="tags">
									{#each spec.tags as tag}
										<span class="tag">{tag}</span>
									{/each}
								</div>
							</dd>
						{/if}
						{#if spec.depends_on.length > 0}
							<dt>Depends On</dt>
							<dd>
								{#each spec.depends_on as dep}
									<button class="link" onclick={() => onselectnode(dep)}>{dep}</button>
								{/each}
							</dd>
						{/if}
					</dl>
				</section>
			{/if}

			{#if renderedBody}
				<section class="section">
					<h3>Content</h3>
					<div class="body-preview">{@html renderedBody}</div>
				</section>
			{/if}

			{#if spec.inbound_edges.length > 0}
				<section class="section">
					<h3>Inbound Edges ({spec.inbound_edges.length})</h3>
					<ul class="edge-list">
						{#each spec.inbound_edges as edge}
							<li>
								<button class="link" onclick={() => onselectnode(edge.source)}>{edge.source}</button>
								<span class="edge-type">{edge.edge_type}</span>
								<span class="trust" style="color: {trustColor(edge.trust)}">
									{edge.trust.toFixed(1)}
								</span>
							</li>
						{/each}
					</ul>
				</section>
			{/if}

			{#if spec.outbound_edges.length > 0}
				<section class="section">
					<h3>Outbound Edges ({spec.outbound_edges.length})</h3>
					<ul class="edge-list">
						{#each spec.outbound_edges as edge}
							<li>
								<button class="link" onclick={() => onselectnode(edge.target)}>{edge.target}</button>
								<span class="edge-type">{edge.edge_type}</span>
								<span class="trust" style="color: {trustColor(edge.trust)}">
									{edge.trust.toFixed(1)}
								</span>
							</li>
						{/each}
					</ul>
				</section>
			{/if}

			{#if downstreamIds.length > 0}
				<section class="section">
					<h3>Downstream Impact ({downstreamIds.length})</h3>
					<ul class="impact-list">
						{#each downstreamIds as id}
							<li>
								<button class="link" onclick={() => onselectnode(id)}>{id}</button>
							</li>
						{/each}
					</ul>
				</section>
			{/if}
		{/if}
	</aside>
{/if}

<style>
	.detail-panel {
		position: fixed;
		top: 40px;
		right: 0;
		width: 360px;
		height: calc(100vh - 40px);
		background: var(--color-surface);
		border-left: 1px solid var(--color-border);
		overflow-y: auto;
		z-index: 50;
		padding: 16px;
		transform: translateX(100%);
		transition: transform 0.2s ease;
		font-family: var(--font-sans);
	}

	.detail-panel.open {
		transform: translateX(0);
	}

	.panel-loading {
		color: var(--color-text-muted);
		padding: 32px;
		text-align: center;
	}

	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 16px;
		padding-bottom: 12px;
		border-bottom: 1px solid var(--color-border);
	}

	.panel-header h2 {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
		margin: 0;
	}

	.panel-version {
		font-size: 11px;
		color: var(--color-accent);
		background: rgba(0, 210, 255, 0.1);
		padding: 2px 8px;
		border-radius: 4px;
	}

	.section {
		margin-bottom: 16px;
	}

	.section h3 {
		font-size: 11px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 8px;
	}

	.fields {
		display: grid;
		grid-template-columns: 80px 1fr;
		gap: 4px 8px;
		font-size: 12px;
	}

	.fields dt {
		color: var(--color-text-muted);
	}

	.fields dd {
		color: var(--color-text);
		margin: 0;
	}

	.mono {
		font-family: var(--font-mono);
		font-size: 11px;
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.tag {
		font-size: 10px;
		padding: 1px 6px;
		border-radius: 3px;
		background: var(--color-surface-alt);
		color: var(--color-upstream);
	}

	.body-preview {
		font-size: 12px;
		line-height: 1.5;
		color: var(--color-text);
		max-height: 200px;
		overflow-y: auto;
		padding: 8px;
		background: var(--color-base);
		border-radius: 4px;
	}

	.body-preview :global(code) {
		font-family: var(--font-mono);
		font-size: 11px;
		background: var(--color-surface-alt);
		padding: 1px 4px;
		border-radius: 2px;
	}

	.edge-list,
	.impact-list {
		list-style: none;
		padding: 0;
		font-size: 12px;
	}

	.edge-list li {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 0;
		border-bottom: 1px solid rgba(42, 42, 74, 0.5);
	}

	.impact-list li {
		padding: 2px 0;
	}

	.edge-type {
		font-size: 10px;
		color: var(--color-text-muted);
		background: var(--color-base);
		padding: 1px 6px;
		border-radius: 3px;
	}

	.trust {
		font-size: 10px;
		font-weight: 600;
		margin-left: auto;
	}

	.link {
		background: none;
		border: none;
		color: var(--color-accent);
		font-family: var(--font-mono);
		font-size: 11px;
		cursor: pointer;
		padding: 0;
		text-decoration: none;
	}

	.link:hover {
		text-decoration: underline;
	}

	.panel-header-right {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.edit-btn {
		font-size: 11px;
		padding: 2px 10px;
		border-radius: 4px;
		border: 1px solid var(--color-border);
		background: var(--color-surface-alt);
		color: var(--color-text-muted);
		cursor: pointer;
		font-family: var(--font-sans);
	}

	.edit-btn:hover {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	.edit-form {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.edit-form label {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.edit-label {
		font-size: 11px;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.3px;
	}

	.edit-input {
		font-size: 12px;
		padding: 6px 8px;
		border-radius: 4px;
		border: 1px solid var(--color-border);
		background: var(--color-base);
		color: var(--color-text);
		font-family: var(--font-sans);
		outline: none;
	}

	.edit-input:focus {
		border-color: var(--color-accent);
	}

	.edit-actions {
		display: flex;
		gap: 8px;
		margin-top: 4px;
	}

	.save-btn,
	.cancel-btn {
		font-size: 12px;
		padding: 5px 14px;
		border-radius: 4px;
		border: 1px solid var(--color-border);
		cursor: pointer;
		font-family: var(--font-sans);
	}

	.save-btn {
		background: var(--color-success);
		color: var(--color-base);
		border-color: var(--color-success);
	}

	.save-btn:hover {
		opacity: 0.9;
	}

	.cancel-btn {
		background: var(--color-surface-alt);
		color: var(--color-text-muted);
	}

	.cancel-btn:hover {
		color: var(--color-text);
	}
</style>
