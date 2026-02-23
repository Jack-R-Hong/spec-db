import type { Edge } from '@xyflow/svelte';

export interface ImpactChain {
	downstream: Set<string>;
	upstream: Set<string>;
	downstreamEdges: Set<string>;
	upstreamEdges: Set<string>;
}

export function computeImpactChain(nodeId: string, edges: Edge[]): ImpactChain {
	const downstream = new Set<string>();
	const upstream = new Set<string>();
	const downstreamEdges = new Set<string>();
	const upstreamEdges = new Set<string>();

	const forwardAdj = new Map<string, { target: string; edgeId: string }[]>();
	const backwardAdj = new Map<string, { source: string; edgeId: string }[]>();

	for (const e of edges) {
		if (!forwardAdj.has(e.source)) forwardAdj.set(e.source, []);
		forwardAdj.get(e.source)!.push({ target: e.target, edgeId: e.id });

		if (!backwardAdj.has(e.target)) backwardAdj.set(e.target, []);
		backwardAdj.get(e.target)!.push({ source: e.source, edgeId: e.id });
	}

	const queue: string[] = [nodeId];
	while (queue.length > 0) {
		const current = queue.shift()!;
		for (const { target, edgeId } of forwardAdj.get(current) ?? []) {
			if (!downstream.has(target)) {
				downstream.add(target);
				downstreamEdges.add(edgeId);
				queue.push(target);
			}
		}
	}

	const bQueue: string[] = [nodeId];
	while (bQueue.length > 0) {
		const current = bQueue.shift()!;
		for (const { source, edgeId } of backwardAdj.get(current) ?? []) {
			if (!upstream.has(source)) {
				upstream.add(source);
				upstreamEdges.add(edgeId);
				bQueue.push(source);
			}
		}
	}

	return { downstream, upstream, downstreamEdges, upstreamEdges };
}
