<script lang="ts">
	import {
		SvelteFlow,
		Controls,
		MiniMap,
		Background,
		BackgroundVariant,
		useSvelteFlow,
		type Node,
		type Edge,
		type NodeTypes,
		type EdgeTypes,
		type Connection
	} from '@xyflow/svelte';
	import '@xyflow/svelte/dist/style.css';

	import SpecNode from '$lib/components/SpecNode.svelte';
	import CausalEdge from '$lib/components/CausalEdge.svelte';
	import SearchFilter from '$lib/components/SearchFilter.svelte';
	import DetailPanel from '$lib/components/DetailPanel.svelte';
	import HeaderBar from '$lib/components/HeaderBar.svelte';
	import ToastNotification from '$lib/components/ToastNotification.svelte';
	import { fetchGraph } from '$lib/stores/graph';
	import { computeImpactChain } from '$lib/utils/traversal';

	const nodeTypes: NodeTypes = { spec: SpecNode } as unknown as NodeTypes;
	const edgeTypes: EdgeTypes = { causal: CausalEdge } as unknown as EdgeTypes;

	let baseNodes = $state<Node[]>([]);
	let baseEdges = $state<Edge[]>([]);
	let nodes = $state.raw<Node[]>([]);
	let edges = $state.raw<Edge[]>([]);
	let error = $state<string | null>(null);
	let loading = $state(true);
	let selectedNodeId = $state<string | null>(null);
	let selectedEdgeId = $state<string | null>(null);
	let searchMatchIds = $state<string[] | null>(null);
	let toast: ToastNotification = undefined!;

	const { fitView } = useSvelteFlow();

	function applyHighlighting() {
		if (selectedNodeId) {
			const chain = computeImpactChain(selectedNodeId, baseEdges);
			nodes = baseNodes.map((n) => ({
				...n,
				data: {
					...n.data,
					highlight:
						n.id === selectedNodeId
							? 'selected'
							: chain.downstream.has(n.id)
								? 'downstream'
								: chain.upstream.has(n.id)
									? 'upstream'
									: 'dimmed'
				}
			}));
			edges = baseEdges.map((e) => ({
				...e,
				style: chain.downstreamEdges.has(e.id)
					? 'stroke: var(--color-impact); stroke-width: 2;'
					: chain.upstreamEdges.has(e.id)
						? 'stroke: var(--color-upstream); stroke-width: 2;'
						: 'stroke: var(--color-border); opacity: 0.15;'
			}));
		} else if (searchMatchIds) {
			const matchSet = new Set(searchMatchIds);
			nodes = baseNodes.map((n) => ({
				...n,
				data: { ...n.data, highlight: matchSet.has(n.id) ? null : 'dimmed' }
			}));
			edges = baseEdges;
		} else {
			nodes = baseNodes.map((n) => ({
				...n,
				data: { ...n.data, highlight: null }
			}));
			edges = baseEdges;
		}
	}

	function handleNodeClick({ node }: { node: Node; event: MouseEvent | TouchEvent }) {
		selectedNodeId = selectedNodeId === node.id ? null : node.id;
		searchMatchIds = null;
		applyHighlighting();
	}

	function handlePaneClick() {
		selectedNodeId = null;
		searchMatchIds = null;
		applyHighlighting();
	}

	function handleSearchFilter(matchingIds: string[]) {
		selectedNodeId = null;
		searchMatchIds = matchingIds;
		applyHighlighting();
		if (matchingIds.length > 0) {
			requestAnimationFrame(() => fitView({ nodes: matchingIds.map((id) => ({ id })) }));
		}
	}

	function handleSearchClear() {
		searchMatchIds = null;
		applyHighlighting();
	}

	const downstreamIds = $derived.by(() => {
		if (!selectedNodeId) return [];
		const chain = computeImpactChain(selectedNodeId, baseEdges);
		return [...chain.downstream];
	});

	function handleSelectFromPanel(id: string) {
		selectedNodeId = id;
		searchMatchIds = null;
		applyHighlighting();
		requestAnimationFrame(() => fitView({ nodes: [{ id }] }));
	}

	function handleConnect(connection: Connection) {
		if (!connection.source || !connection.target || connection.source === connection.target) return;

		const sourceData = baseNodes.find((n) => n.id === connection.source)?.data;
		const sourceId = sourceData?.specId ?? connection.source;
		const targetData = baseNodes.find((n) => n.id === connection.target)?.data;
		const targetId = targetData?.specId ?? connection.target;

		const exists = baseEdges.some(
			(e) =>
				e.data?.sourceId === sourceId && e.data?.targetId === targetId
		);
		if (exists) {
			toast.showError('Edge already exists');
			return;
		}

		toast.showConfirm(
			`Write to ${sourceId}? This will create a git commit.`,
			async () => {
				try {
					const res = await fetch('/api/writeback', {
						method: 'POST',
						headers: { 'Content-Type': 'application/json' },
						body: JSON.stringify({ type: 'edge_add', source: sourceId, target: targetId })
					});
					if (!res.ok) {
						const err = await res.json();
						toast.showError(err.message ?? 'Write-back failed');
						return;
					}
					const undoFn = async () => {
						await fetch('/api/writeback/undo', { method: 'POST' });
						refreshGraph();
					};
					pendingUndo = undoFn;
					setTimeout(() => { pendingUndo = null; }, 5000);
					toast.showUndo(undoFn);
					refreshGraph();
				} catch {
					toast.showError('Network error');
				}
			},
			() => {}
		);
	}

	function handleEdgeClick({ edge }: { edge: Edge; event: MouseEvent | TouchEvent }) {
		selectedEdgeId = selectedEdgeId === edge.id ? null : edge.id;
	}

	function handleDeleteEdge() {
		if (!selectedEdgeId) return;
		const edge = baseEdges.find((e) => e.id === selectedEdgeId);
		if (!edge) return;

		const sourceId = edge.data?.sourceId ?? edge.source;
		const targetId = edge.data?.targetId ?? edge.target;

		toast.showConfirm(
			`Remove edge ${sourceId} → ${targetId}? This will create a git commit.`,
			async () => {
				try {
					const res = await fetch('/api/writeback', {
						method: 'POST',
						headers: { 'Content-Type': 'application/json' },
						body: JSON.stringify({ type: 'edge_remove', source: sourceId, target: targetId })
					});
					if (!res.ok) {
						const err = await res.json();
						toast.showError(err.message ?? 'Write-back failed');
						return;
					}
					selectedEdgeId = null;
					const undoFn = async () => {
						await fetch('/api/writeback/undo', { method: 'POST' });
						refreshGraph();
					};
					pendingUndo = undoFn;
					setTimeout(() => { pendingUndo = null; }, 5000);
					toast.showUndo(undoFn);
					refreshGraph();
				} catch {
					toast.showError('Network error');
				}
			},
			() => {
				selectedEdgeId = null;
			}
		);
	}

	let pendingUndo: (() => void) | null = $state(null);

	async function handleFrontmatterSave(specId: string, changes: Record<string, unknown>) {
		toast.showConfirm(
			`Write to ${specId}? This will create a git commit.`,
			async () => {
				try {
					const res = await fetch('/api/writeback', {
						method: 'POST',
						headers: { 'Content-Type': 'application/json' },
						body: JSON.stringify({ type: 'frontmatter_edit', spec_id: specId, changes })
					});
					if (!res.ok) {
						const err = await res.json();
						toast.showError(err.message ?? 'Write-back failed');
						return;
					}
					const undoFn = async () => {
						await fetch('/api/writeback/undo', { method: 'POST' });
						refreshGraph();
					};
					pendingUndo = undoFn;
					setTimeout(() => {
						pendingUndo = null;
					}, 5000);
					toast.showUndo(undoFn);
					refreshGraph();
				} catch {
					toast.showError('Network error');
				}
			},
			() => {}
		);
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === '0') {
			e.preventDefault();
			fitView();
		}
		if (e.key === 'Escape' && !searchMatchIds && selectedNodeId) {
			selectedNodeId = null;
			selectedEdgeId = null;
			applyHighlighting();
		}
		const target = e.target as HTMLElement;
		const isInputFocused = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
		if ((e.key === 'Delete' || e.key === 'Backspace') && !isInputFocused && selectedEdgeId) {
			e.preventDefault();
			handleDeleteEdge();
		}
		if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !isInputFocused && pendingUndo) {
			e.preventDefault();
			pendingUndo();
			pendingUndo = null;
		}
	}

	function refreshGraph() {
		fetchGraph()
			.then((graph) => {
				baseNodes = graph.nodes;
				baseEdges = graph.edges;
				selectedNodeId = null;
				searchMatchIds = null;
				nodes = graph.nodes;
				edges = graph.edges;
			})
			.catch(() => {});
	}

	$effect(() => {
		fetchGraph()
			.then((graph) => {
				baseNodes = graph.nodes;
				baseEdges = graph.edges;
				nodes = graph.nodes;
				edges = graph.edges;
				loading = false;
			})
			.catch((err: Error) => {
				error = err.message;
				loading = false;
			});
	});
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<main>
	<HeaderBar onrefresh={refreshGraph} />
	{#if loading}
		<div class="status">Loading graph…</div>
	{:else if error}
		<div class="status error">
			<p>Failed to load graph</p>
			<p class="detail">{error}</p>
		</div>
	{:else}
		<div class="graph-area">
			<SearchFilter
				nodes={baseNodes}
				onfilter={handleSearchFilter}
				onclear={handleSearchClear}
			/>
			<SvelteFlow
				bind:nodes
				bind:edges
				{nodeTypes}
				{edgeTypes}
				fitView
				onnodeclick={handleNodeClick}
				onpaneclick={handlePaneClick}
				onconnect={handleConnect}
				onedgeclick={handleEdgeClick}
			>
				<Controls />
				<Background variant={BackgroundVariant.Dots} gap={20} />
				<MiniMap
					style="background: var(--color-surface); border: 1px solid var(--color-border);"
				/>
			</SvelteFlow>
			<DetailPanel
				{selectedNodeId}
				{downstreamIds}
				onselectnode={handleSelectFromPanel}
				onsave={handleFrontmatterSave}
			/>
		</div>
	{/if}
	<ToastNotification bind:this={toast} />
</main>

<style>
	main {
		height: 100vh;
		width: 100vw;
		display: flex;
		flex-direction: column;
	}

	.graph-area {
		flex: 1;
		position: relative;
		min-height: 0;
	}

	.status {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-text-muted);
		font-size: 16px;
		gap: 8px;
	}

	.status.error {
		color: var(--color-impact);
	}

	.detail {
		font-size: 12px;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
	}
</style>
