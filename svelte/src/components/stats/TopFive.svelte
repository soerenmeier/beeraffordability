<script lang="ts">
	import type { Datapoint } from '@/lib/datapoints';

	let {
		title,
		desc,
		datapoints,
		maxMinutes,
		style,
	}: {
		title: string;
		desc: string;
		datapoints: Datapoint[];
		maxMinutes: number;
		style: 'green' | 'red';
	} = $props();
</script>

<section class="style-{style}">
	<header>
		<h2 class="h3">{title}</h2>
		<p class="desc">{desc}</p>
	</header>

	<ol>
		{#each datapoints as { country, price, wage, minutes, time }, i}
			<li>
				<span class="rank">#{i + 1}</span>

				<span class="ctn">
					<span class="country h4">{country}</span>
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
</section>

<style lang="scss">
	section {
		--accent: var(--black);

		padding: 1.25rem 1rem;
		background-color: var(--white);
		border: 2px solid var(--black);
		box-shadow: 3px 3px 0 0 var(--black);
	}

	.style-green {
		--accent: var(--green);
	}

	.style-red {
		--accent: var(--red);
	}

	header {
		// display: block;
		padding-bottom: 0.5rem;
		margin-bottom: 0.75rem;
		border-bottom: 2px solid var(--black);
	}

	ol {
		display: flex;
		list-style: none;
		flex-direction: column;
		gap: 0.5rem;
	}

	li {
		display: grid;
		padding: 0.25rem;
		grid-template-columns: 1.75rem 1fr 4.5rem;
		align-items: center;
		background-color: var(--beige);
		border: 1px solid var(--border-beige);
	}

	.rank {
		font-weight: 900;
		@include body-xl;
		color: var(--accent);
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
		color: var(--accent);
		font-weight: 900;
		@include body-xl;
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
			background-color: var(--accent);
			transform: scaleX(var(--progress));
			transform-origin: left;
		}
	}

	@include tablet {
		li {
			padding: 0.5rem;
			grid-template-columns: 2.25rem 1fr 5.5rem;
		}

		.country {
			// @include h4;
		}
	}

	@include desktop {
		li {
			padding: 1rem 1.5rem;
			// grid-template-columns: 2.25rem 1fr 5.5rem;
		}
	}
</style>
