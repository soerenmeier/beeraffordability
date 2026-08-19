<script>
	let { entry } = $props();
</script>

<h1>Welcome</h1>

<p>Welcome to your new Crelte project. This is the starter template.</p>
<p>
	If you're new to crelte we strongly recommend to read the documentation
	first.
</p>

<h2>Configure Craft</h2>
<ol>
	<li>
		First setup more <a
			href={entry.cpUrl + '/settings/sites'}
			target="_blank"
		>
			sites
		</a>
		if you want to support more languages
		<br />
		Remember to add the env variables in all
		<code>.env*</code>
		files
	</li>

	<li>
		Then setup the first <a
			href={entry.cpUrl + '/settings/sections/new'}
			target="_blank"
		>
			section
		</a>
	</li>

	<li>
		Then setup your first <a
			href={entry.cpUrl + '/settings/globals/new'}
			target="_blank"
		>
			global
		</a>
	</li>

	<li>
		At last allow the sites, sections and globals to be queried via <a
			href={entry.cpUrl + '/graphql/schemas/1'}
			target="_blank"
		>
			GraphQl
		</a>
	</li>
</ol>

<h2>Configure Svelte</h2>
<ol>
	<li>
		Modify the <code>global.graphql</code>
		file removing crelte starter and replacing it with your own global query
	</li>
	<li>
		Modify the <code>entry.graphql</code>
		file removing crelte starter and replacing it with the default entry query
		<strong>Note:</strong>
		You will not be able to access this page after you remove it.
		<br />
		And you can now delete
		<code>templates/crelte-starter.svelte</code>
	</li>
</ol>

<style lang="scss">
	h1 {
		margin-top: 2rem;
		font-size: 2rem;
		font-weight: bold;
	}

	h2 {
		margin-top: 1.5rem;
		font-size: 1.5rem;
		font-weight: bold;
	}

	ol {
		margin-top: 1rem;
		list-style: decimal;
		padding-left: 2rem;

		li {
			margin-top: 0.25rem;
		}
	}

	strong {
		font-weight: bold;
	}

	code {
		background-color: #eaeaea;
		padding: 0.1rem 0.2rem;
	}
</style>
