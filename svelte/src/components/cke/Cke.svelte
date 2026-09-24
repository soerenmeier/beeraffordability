<script lang="ts">
	import Buttons from './Buttons.svelte';

	let { chunks }: { chunks: any[] } = $props();
</script>

{#each chunks as chunk}
	{#if chunk.html}
		<div class="html">{@html chunk.html}</div>
	{:else if chunk.btns}
		<Buttons {...chunk} />
	{/if}
{/each}
