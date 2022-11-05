<script lang="ts">
	import type { PageData } from './$types';
	import type { Config, Options } from '$lib';
	import Form from './Form.svelte';

	export let data: PageData & { config: Config, options: Options };
</script>

<Form options={data.options} config={data.config} onSubmit={(newConfig) => {
	fetch('/api/config', {
		method: 'PUT',
		headers: {
			'content-type': 'application/json',
		},
		body: JSON.stringify(newConfig),
	});
}} />
