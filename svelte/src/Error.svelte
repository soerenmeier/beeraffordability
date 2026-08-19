<script>
	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	let { status, message } = $props();
</script>

<svelte:head>
	<title>Error</title>
</svelte:head>

<div id="error">
	<div class="center">An error occurred</div>
</div>

<style lang="scss">
	#error {
		position: absolute;
		display: flex;
		width: 100%;
		height: 100%;
		align-items: center;
		justify-content: center;
	}
</style>
