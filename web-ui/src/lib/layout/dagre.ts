import dagre from '@dagrejs/dagre';
import { Position, type Node, type Edge } from '@xyflow/svelte';

const NODE_WIDTH = 240;
const NODE_HEIGHT = 100;

export function getLayoutedElements(
	nodes: Node[],
	edges: Edge[],
	direction = 'TB'
): { nodes: Node[]; edges: Edge[] } {
	const g = new dagre.graphlib.Graph();
	g.setDefaultEdgeLabel(() => ({}));

	const isHorizontal = direction === 'LR';
	g.setGraph({ rankdir: direction, nodesep: 60, ranksep: 80 });

	const connectedIds = new Set<string>();
	edges.forEach((edge) => {
		connectedIds.add(edge.source);
		connectedIds.add(edge.target);
	});

	const connected = nodes.filter((n) => connectedIds.has(n.id));
	const disconnected = nodes.filter((n) => !connectedIds.has(n.id));

	connected.forEach((node) => {
		g.setNode(node.id, { width: NODE_WIDTH, height: NODE_HEIGHT });
	});

	edges.forEach((edge) => {
		g.setEdge(edge.source, edge.target);
	});

	dagre.layout(g);

	const layoutedConnected: Node[] = connected.map((node) => {
		const pos = g.node(node.id);
		return {
			...node,
			targetPosition: isHorizontal ? Position.Left : Position.Top,
			sourcePosition: isHorizontal ? Position.Right : Position.Bottom,
			position: {
				x: pos.x - NODE_WIDTH / 2,
				y: pos.y - NODE_HEIGHT / 2
			}
		};
	});

	const graphBounds = layoutedConnected.reduce(
		(acc, n) => ({
			maxX: Math.max(acc.maxX, n.position.x + NODE_WIDTH),
			maxY: Math.max(acc.maxY, n.position.y + NODE_HEIGHT)
		}),
		{ maxX: 0, maxY: 0 }
	);

	const layoutedDisconnected: Node[] = disconnected.map((node, i) => {
		const col = i % 4;
		const row = Math.floor(i / 4);
		return {
			...node,
			targetPosition: isHorizontal ? Position.Left : Position.Top,
			sourcePosition: isHorizontal ? Position.Right : Position.Bottom,
			position: {
				x: graphBounds.maxX + 100 + col * (NODE_WIDTH + 40),
				y: graphBounds.maxY + 100 + row * (NODE_HEIGHT + 40)
			}
		};
	});

	return {
		nodes: [...layoutedConnected, ...layoutedDisconnected],
		edges
	};
}
