<script>
	import { getSite, getEntry } from 'crelte';

	const site = getSite();
	const entry = getEntry();
</script>

<div class="header">
	{#if $entry.slug !== '__home__'}
		<a href={$site.url.href}>Back Home</a>
	{/if}
</div>

<style lang="scss">
	.header {
		display: flex;
		height: 1.81rem;
		padding-inline: var(--sx-body);
		background-color: var(--black);
		align-items: center;
	}

	a {
		color: var(--yellow);
		font-family: var(--font-mono);
		text-transform: uppercase;
	}
</style>
