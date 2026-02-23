import type { Node, Edge } from '@xyflow/svelte';
import { getLayoutedElements } from '$lib/layout/dagre';

interface ApiNode {
	id: string;
	title: string;
	version: number;
	tags: string[];
}

interface ApiEdge {
	source: string;
	target: string;
	edge_type: string;
	trust: number;
	origin: string;
}

interface GraphResponse {
	nodes: ApiNode[];
	edges: ApiEdge[];
}

const EDGE_COLORS: Record<string, string> = {
	DependsOn: 'var(--color-edge-depends)',
	Constrains: 'var(--color-edge-constrains)',
	Implements: 'var(--color-edge-implements)',
	Validates: 'var(--color-edge-validates)',
	Extends: 'var(--color-edge-extends)',
	ConflictsWith: 'var(--color-edge-conflicts)',
	Informs: 'var(--color-edge-informs)'
};

function edgeColor(edgeType: string): string {
	return EDGE_COLORS[edgeType] ?? 'var(--color-text-muted)';
}

export async function fetchGraph(): Promise<{ nodes: Node[]; edges: Edge[] }> {
	const res = await fetch('/api/graph');
	if (!res.ok) throw new Error(`API error: ${res.status}`);

	const data: GraphResponse = await res.json();

	const connectedIds = new Set<string>();
	data.edges.forEach((e) => {
		connectedIds.add(e.source);
		connectedIds.add(e.target);
	});

	const rawNodes: Node[] = data.nodes.map((n) => ({
		id: n.id,
		type: 'spec',
		data: {
			title: n.title,
			specId: n.id,
			tags: n.tags,
			version: n.version,
			isDisconnected: !connectedIds.has(n.id)
		},
		position: { x: 0, y: 0 }
	}));

	const rawEdges: Edge[] = data.edges.map((e) => ({
		id: `${e.source}-${e.target}`,
		source: e.source,
		target: e.target,
		type: 'causal',
		data: {
			edgeType: e.edge_type,
			trust: e.trust,
			origin: e.origin,
			sourceId: e.source,
			targetId: e.target
		},
		style: `stroke: ${edgeColor(e.edge_type)};`,
		markerEnd: { type: 'arrowclosed' as const, color: edgeColor(e.edge_type) }
	}));

	return getLayoutedElements(rawNodes, rawEdges);
}
