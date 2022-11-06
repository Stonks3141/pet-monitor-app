<script lang="ts">
  import type { PageData } from './$types';
  import type { Config, Options } from '$lib';
  import { getToken } from '$lib';
  import { goto } from '$app/navigation';
  import Form from './Form.svelte';

  export let data: PageData & { config: Config; options: Options };

  const onSubmit = (newConfig: Config) => {
    fetch('/api/config', {
      method: 'PUT',
      headers: {
        'content-type': 'application/json',
        'x-csrf-token': getToken()!,
      },
      body: JSON.stringify(newConfig),
    });
    goto('/stream');
  };
</script>

<Form options={data.options} config={data.config} {onSubmit} />
