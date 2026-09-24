<script>
	import Richtext, { hasChunks } from '../cke/Richtext.svelte';

	let { title, pageIntroCke } = $props();
</script>

<div class="page-intro wrap">
	<h1 class="h1">{title}</h1>

	{#if hasChunks(pageIntroCke)}
		<Richtext {...pageIntroCke} />
	{/if}
</div>

<style lang="scss">
	.page-intro {
		margin-top: 1rem;
		padding-bottom: 1.25rem;
		border-bottom: 2px solid var(--black);
	}

	.page-intro :global .rt {
		margin-top: 0.75rem;
		padding-top: 0.75rem;
		border-top: 1px solid var(--black);
		font-weight: 700;
		color: var(--gray);
	}
</style>
