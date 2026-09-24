<script lang="ts">
	import type { Snippet } from 'svelte';
	import type {
		HTMLAnchorAttributes,
		HTMLButtonAttributes,
	} from 'svelte/elements';

	let {
		children,
		href = undefined,
		...rest
	}: {
		children: Snippet;
		href?: string;
	} & (HTMLAnchorAttributes | HTMLButtonAttributes) = $props();
</script>

<svelte:element this={href ? 'a' : 'button'} class="button" {href} {...rest}>
	{@render children()}
</svelte:element>

<style lang="scss">
	.button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 0.25rem 0.625rem;
		border: 1px solid var(--black);
		background-color: var(--black);
		color: var(--white);
	}
</style>
