<script module>
	import datapointsQuery from '@/queries/datapoints.graphql';

	/** @type {import('crelte').LoadData} */
	export const loadData = cr =>
		cr.query(datapointsQuery, {
			id: cr.getGlobal('currentQuarter').id,
		});
</script>

<script>
	import PageIntro from '@/components/intro/PageIntro.svelte';
	import TopStats from '@/components/stats/TopStats.svelte';

	let { entry, datapoints } = $props();
</script>

<PageIntro
	title={entry.pageTitle ?? entry.title}
	pageIntroCke={entry.pageIntroCke}
/>

<TopStats {datapoints} />
