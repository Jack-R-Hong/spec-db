<script lang="ts">
	import { BaseEdge, EdgeLabel, getBezierPath } from '@xyflow/svelte';

	let {
		sourceX,
		sourceY,
		targetX,
		targetY,
		sourcePosition,
		targetPosition,
		data,
		markerEnd,
		style
	} = $props<{
		id: string;
		sourceX: number;
		sourceY: number;
		targetX: number;
		targetY: number;
		sourcePosition: any;
		targetPosition: any;
		data: { edgeType: string; trust: number; origin: string };
		markerEnd: string;
		style: string;
	}>();

	const result = $derived(
		getBezierPath({
			sourceX,
			sourceY,
			sourcePosition,
			targetX,
			targetY,
			targetPosition
		})
	);

	const label = $derived(data?.edgeType?.replace(/([A-Z])/g, ' $1').trim() ?? '');
</script>

<BaseEdge path={result[0]} {markerEnd} {style} />

<EdgeLabel x={result[1]} y={result[2]}>
	<span class="edge-label">{label}</span>
</EdgeLabel>

<style>
	.edge-label {
		font-size: 9px;
		font-family: var(--font-sans);
		color: var(--color-text-muted);
		background: var(--color-base);
		padding: 2px 6px;
		border-radius: 3px;
		border: 1px solid var(--color-border);
		white-space: nowrap;
	}
</style>
