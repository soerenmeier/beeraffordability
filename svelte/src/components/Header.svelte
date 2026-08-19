<script>
	import { getSite, getGlobal } from 'crelte';

	const header = getGlobal('header');
	const site = getSite();
</script>

<div class="header">
	<nav>
		<ul>
			<li>
				<a href={$site.url.href}>Home / {$header.title}</a>
			</li>
			<li>
				<a href="https://docs.crelte.com" target="_blank">Docs</a>
			</li>
		</ul>
	</nav>
</div>

<style lang="scss">
	.header {
		margin-bottom: 2rem;
	}

	ul {
		display: flex;
		gap: 1rem;
	}
</style>
