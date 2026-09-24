<script>
	import { getGlobal } from 'crelte';

	const footer = getGlobal('footer');
</script>

<div class="footer">
	<div class="inner wrap">
		<p>Copyright Meier Labs © {new Date().getFullYear()}</p>

		<nav>
			<ul>
				{#each $footer.navigation.filter(n => !!n.navLink?.url) as { navLink }}
					<li>
						<a href={navLink.url} target={navLink.target}>
							{navLink.label ?? navLink.defaultLabel}
						</a>
					</li>
				{/each}
			</ul>
		</nav>
	</div>
</div>

<style lang="scss">
	.footer {
		margin-top: 1rem;
		border-top: 2px solid var(--black);
		padding-block: 1rem;
		background-color: var(--beige-dark);
	}

	.inner {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
	}

	nav ul {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}

	nav a {
		opacity: 0.6;
	}

	@include tablet {
		.footer {
			margin-top: 2rem;
		}

		nav ul {
			gap: 1rem;
		}
	}
</style>
