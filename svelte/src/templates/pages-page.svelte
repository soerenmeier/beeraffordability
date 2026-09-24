<script>
	import Richtext, { hasChunks } from '@/components/cke/Richtext.svelte';
	import PageIntro from '@/components/intro/PageIntro.svelte';

	let { entry } = $props();
</script>

<PageIntro
	title={entry.pageTitle ?? entry.title}
	pageIntroCke={entry.pageIntroCke}
/>

{#if hasChunks(entry.ctnCke)}
	<div class="wrap ctn">
		<Richtext {...entry.ctnCke} />
	</div>
{/if}

<style lang="scss">
	.ctn {
		padding-block: 1rem;
	}

	@include tablet {
		.ctn {
			padding-block: 1.25rem;
		}
	}

	@include desktop {
		.ctn {
			padding-block: 2rem;
		}
	}
</style>
