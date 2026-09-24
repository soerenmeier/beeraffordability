<script lang="ts">
	import LinkButton from '@/components/form/LinkButton.svelte';

	let { btns }: { btns: any[] } = $props();
</script>

<div class="buttons">
	{#each btns as btn}
		<LinkButton {...btn} />
	{/each}
</div>

<style lang="scss">
	.buttons {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}
</style>
