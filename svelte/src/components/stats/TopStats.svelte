<script lang="ts">
	import type { Datapoint } from '@/lib/datapoints';
	import TopFive from './TopFive.svelte';

	let { datapoints }: { datapoints: Datapoint[] } = $props();

	let maxMinutes = $derived(Math.max(...datapoints.map(d => d.minutes)));
	let first = $derived(datapoints.slice(0, 5));
	let last = $derived(datapoints.slice(-5).reverse());
</script>

<div class="top-stats wrap">
	<TopFive
		title="Most affordable beers"
		desc="Lowest minutes of labor required for 0.5L supermarket beer"
		datapoints={first}
		{maxMinutes}
		style="green"
	/>
	<TopFive
		title="Most expensive beers"
		desc="Highest minutes of labor required for 0.5L supermarket"
		datapoints={last}
		{maxMinutes}
		style="red"
	/>
</div>

<style lang="scss">
	.top-stats {
		display: grid;
		padding-block: 1.25rem;
		gap: 1rem;
	}

	@include tablet {
		.top-stats {
			grid-template-columns: repeat(2, 1fr);
		}
	}
</style>
