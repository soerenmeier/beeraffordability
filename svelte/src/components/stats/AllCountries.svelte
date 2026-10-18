<script lang="ts">
	import type { Datapoint } from '@/lib/datapoints';
	import Button from '../form/Button.svelte';

	const PAGE_SIZE = 20;

	type Sort = 'affordable' | 'expensive' | 'name';

	// datapoints are expected to be sorted by minutes (most affordable first)
	let { datapoints }: { datapoints: Datapoint[] } = $props();

	let search = $state('');
	let sort: Sort = $state('affordable');
	let limit = $state(PAGE_SIZE);

	let maxMinutes = $derived(Math.max(...datapoints.map(d => d.minutes)));

	let ranked = $derived(datapoints.map((d, i) => ({ ...d, rank: i + 1 })));

	let filtered = $derived.by(() => {
		const query = normalize(search.trim());
		const list = query
			? ranked.filter(d => normalize(d.country ?? '').includes(query))
			: ranked.slice();

		if (sort === 'expensive') list.reverse();
		else if (sort === 'name')
			list.sort((a, b) =>
				(a.country ?? '').localeCompare(b.country ?? ''),
			);

		return list;
	});

	let visible = $derived(filtered.slice(0, limit));

	function normalize(s: string): string {
		return s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
	}

	function resetLimit() {
		limit = PAGE_SIZE;
	}
</script>

<section class="all-countries wrap">
	<header>
		<h2 class="h3">All countries</h2>
		<p class="desc">
			Minutes of labor required for 0.5L supermarket beer in
			{datapoints.length} countries
		</p>
	</header>

	<div class="filters">
		<label class="search">
			<span class="label">Search</span>
			<input
				type="search"
				placeholder="Country"
				bind:value={search}
				oninput={resetLimit}
			/>
		</label>

		<label class="sort">
			<span class="label">Sort by</span>
			<select bind:value={sort} onchange={resetLimit}>
				<option value="affordable">Most affordable</option>
				<option value="expensive">Most expensive</option>
				<option value="name">Name</option>
			</select>
		</label>
	</div>

	{#if visible.length}
		<ol>
			{#each visible as { rank, country, price, wage, minutes, time } (rank)}
				<li>
					<span class="rank">#{rank}</span>

					<span class="ctn">
						<span class="country">{country}</span>
						<span class="stats">0.5L: {price} • Wage: {wage}</span>
					</span>

					<span class="minutes">
						<span class="time">{time}</span>
						<span
							class="progress"
							style:--progress={minutes / maxMinutes}
						></span>
					</span>
				</li>
			{/each}
		</ol>
	{:else}
		<p class="empty">No country matches "{search}".</p>
	{/if}

	<footer>
		{#if filtered.length}
			<p class="count">
				Showing {visible.length} of {filtered.length} countries
			</p>
		{/if}

		{#if visible.length < filtered.length}
			<Button onclick={() => (limit += PAGE_SIZE)}>Show more</Button>
		{/if}
	</footer>
</section>

<style lang="scss">
	.all-countries {
		padding-block: 1.25rem 2.5rem;
	}

	header {
		padding-bottom: 0.5rem;
		margin-bottom: 0.75rem;
		border-bottom: 2px solid var(--black);
	}

	.filters {
		display: grid;
		gap: 0.75rem;
		margin-bottom: 1rem;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.label {
		@include body-sm;
		font-weight: 700;
	}

	input,
	select {
		width: 100%;
		padding: 0.5rem 0.75rem;
		font: inherit;
		color: var(--black);
		background-color: var(--white);
		border: 2px solid var(--black);
		border-radius: 0;
		box-shadow: 3px 3px 0 0 var(--black);

		&:focus {
			outline: 2px solid var(--yellow);
			outline-offset: 2px;
		}
	}

	ol {
		display: flex;
		list-style: none;
		flex-direction: column;
		gap: 0.375rem;
	}

	li {
		display: grid;
		padding: 0.25rem 0.5rem;
		grid-template-columns: 2.75rem 1fr 4.5rem;
		align-items: center;
		background-color: var(--white);
		border: 1px solid var(--border-beige);
	}

	.rank {
		font-weight: 900;
		color: var(--gray);
	}

	.country {
		font-weight: 700;
	}

	.stats {
		display: block;
		@include body-sm;
		opacity: 0.75;
	}

	.time {
		display: block;
		text-align: right;
		font-weight: 900;
	}

	.progress {
		display: block;
		width: 100%;
		height: 0.375rem;
		border: 1px solid var(--gray-2);

		&::after {
			content: '';
			display: block;
			width: 100%;
			height: 100%;
			background-color: var(--black);
			transform: scaleX(var(--progress));
			transform-origin: left;
		}
	}

	.empty {
		padding: 1rem 0;
		color: var(--gray);
	}

	footer {
		display: flex;
		margin-top: 1rem;
		flex-direction: column;
		align-items: center;
		gap: 0.75rem;
	}

	.count {
		@include body-sm;
		color: var(--gray);
	}

	footer :global(.button) {
		padding: 0.5rem 1.5rem;
		font-weight: 700;
		cursor: pointer;
	}

	@include tablet {
		.filters {
			grid-template-columns: 1fr 14rem;
		}

		li {
			padding: 0.5rem 1rem;
			grid-template-columns: 3.5rem 1fr 6rem;
		}

		// price and wage get their own columns on wide screens
		.ctn {
			display: flex;
			align-items: baseline;
			justify-content: space-between;
			gap: 1rem;
			padding-right: 1.5rem;
		}
	}
</style>
