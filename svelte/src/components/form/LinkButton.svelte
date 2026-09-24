<script lang="ts">
	import Button from './Button.svelte';

	let {
		url,
		target,
		label,
		defaultLabel,
		download,
	}: {
		url: string;
		target?: string | null;
		label?: string | null;
		defaultLabel: string;
		download?: boolean;
	} = $props();
</script>

<Button
	href={url}
	target={target ?? undefined}
	download={download || undefined}
>
	{label ?? defaultLabel}
</Button>
