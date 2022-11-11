<script lang="ts">
  import type { Config, Capabilities } from '$lib';
  import { clearToken } from '$lib';
  import { goto } from '$app/navigation';

  export let capabilities: Capabilities;
  export let config: Config;
  export let onSubmit: (_: Config) => void;

  console.log(config);

  const updateFormat = () => (config.format = Object.keys(capabilities[config.device])[0]);
  const updateResolution = () =>
    (config.resolution = capabilities[config.device][config.format][0].resolution);
  const updateInterval = () =>
    (config.interval = capabilities[config.device][config.format].find(
      (res) =>
        res.resolution[0] === config.resolution[0] && res.resolution[1] === config.resolution[1]
    )!.intervals[0]);

  const getCurrentIntervals = () =>
    capabilities[config.device][config.format].find(
      (res) =>
        res.resolution[0] === config.resolution[0] && res.resolution[1] === config.resolution[1]
    )!.intervals;
    
  const handleControlEdit = (e: Event & { currentTarget: HTMLInputElement }, optionName: string) => {
    config.v4l2Controls[(<HTMLInputElement>e.target).value] = config.v4l2Controls[optionName];
    delete config.v4l2Controls[optionName];
  };

  let newOptionName = '';
  let newOptionValue = '';
</script>

<div class="card h-fit w-full md:w-3/4 lg:w-1/2 bg-base-100">
  <div class="flex flex-row justify-between gap-2 mx-8 mt-8">
    <h2 class="card-title">Settings</h2>
    <button
      class="btn btn-ghost w-fit"
      on:click={() => {
        clearToken();
        goto('/login');
      }}>Log out</button
    >
  </div>
  <form
    class="form-control card-body"
    on:submit={(e) => {
      e.preventDefault();
      onSubmit(config);
    }}
  >
    <label for="device" class="label">Device</label>
    <select
      id="device"
      class="select bg-base-200 select-bordered"
      bind:value={config.device}
      on:change={() => {
        updateFormat();
        updateResolution();
        updateInterval();
      }}
    >
      {#each Object.keys(capabilities) as deviceName}
        <option value={deviceName}>{deviceName}</option>
      {/each}
    </select>
    <label for="format" class="label">Format</label>
    <select
      id="format"
      class="select bg-base-200 select-bordered"
      bind:value={config.format}
      on:change={() => {
        updateResolution();
        updateInterval();
      }}
    >
      {#each Object.keys(capabilities[config.device]) as format}
        <option value={format}>{format}</option>
      {/each}
    </select>
    <label for="resolution" class="label">Resolution</label>
    <select
      id="resolution"
      class="select bg-base-200 select-bordered"
      bind:value={config.resolution}
      on:change={() => {
        updateInterval();
      }}
    >
      {#each capabilities[config.device][config.format] as { resolution }}
        <option value={resolution}>{`${resolution[0]}x${resolution[1]}`}</option>
      {/each}
    </select>
    <label for="interval" class="label">Framerate</label>
    <select id="interval" class="select bg-base-200 select-bordered" bind:value={config.interval}>
      {#each getCurrentIntervals() as interval}
        <option value={interval}>{interval[1] / interval[0]}</option>
      {/each}
    </select>
    <label for="rotation" class="label">Rotation</label>
    <select id="rotation" class="select bg-base-200 select-bordered" bind:value={config.rotation}>
      <option value={0}>0°</option>
      <option value={90}>90°</option>
      <option value={180}>180°</option>
      <option value={270}>270°</option>
    </select>
    <label for="v4l2Controls" class="label">Additional V4L2 controls</label>
    <div id="v4l2Controls" class="flex flex-col gap-2">
      {#each Object.keys(config.v4l2Controls) as optionName}
        <div class="flex flex-row gap-2">
          <input
            type="text"
            class="input input-bordered bg-base-200 w-full"
            value={optionName}
            on:change={e => handleControlEdit(e, optionName)}
          />
          <input
            type="text"
            class="input input-bordered bg-base-200 w-full"
            bind:value={config.v4l2Controls[optionName]}
          />
          <button
            class="btn btn-ghost w-fit mr-2"
            on:click={(e) => {
              e.preventDefault();
              delete config.v4l2Controls[optionName];
              config.v4l2Controls = config.v4l2Controls;
            }}
          >
            <span class="material-icons">remove</span>
          </button>
        </div>
      {/each}
    </div>
    <div class="flex flex-row gap-2">
      <input
        type="text"
        class="input input-bordered bg-base-200 w-full"
        bind:value={newOptionName}
      />
      <input
        type="text"
        class="input input-bordered bg-base-200 w-full"
        bind:value={newOptionValue}
      />
      <button
        class="btn btn-ghost"
        on:click={(e) => {
          e.preventDefault();
          if (newOptionName != '' && newOptionValue != '') {
            config.v4l2Controls[newOptionName] = newOptionValue;
            newOptionName = '';
            newOptionValue = '';
          }
        }}>Add</button
      >
    </div>
    <div class="flex flex-row justify-between mt-4">
      <button type="button" class="btn btn-ghost w-fit"><a href="/stream">Cancel</a></button>
      <button type="submit" class="btn btn-primary w-fit">Save</button>
    </div>
  </form>
</div>
