<script module lang="ts">
	export function hasChunks(cke: any): boolean {
		return !!cke?.chunks?.length;
	}
</script>

<script lang="ts">
	import Cke from './Cke.svelte';

	let { chunks }: { chunks: any[] } = $props();
</script>

<div class="rt">
	<Cke {chunks} />
</div>
