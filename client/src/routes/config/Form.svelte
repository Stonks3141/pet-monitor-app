<script lang="ts">
	import type { Config, Options, Option } from '$lib';
	import { clearToken } from '$lib';
	import { goto } from '$app/navigation';

	export let options: Options;
	export let config: Config;
	export let onSubmit: (_: Config) => void;

	let selected: Option;
	let newOptionName = '';
	let newOptionValue = '';
</script>

<div class="card h-fit w-full md:w-3/4 lg:w-1/2 bg-base-100">
<div class="flex flex-row justify-between gap-2 mx-8 mt-8">
  <h2 class="card-title">Settings</h2>
  <button class="btn btn-ghost w-fit" on:click={() => {
		clearToken();
		goto('/login');
	}}>Log out</button>
</div>
<form class="form-control card-body" on:submit={e => {
		e.preventDefault();
		config.resolution = selected.resolution;
		config.framerate = selected.framerate;
		onSubmit(config);
	}}>
	<label for="device" class="label">Device</label>
	<select
		id="device"
    class="select bg-base-200 select-bordered"
		bind:value={config.device}
		on:change={() => selected = options[config.device][0]}
	>
		{#each Object.keys(options) as deviceName}
			<option value={deviceName}>{deviceName}</option>
		{/each}
	</select>
	<label for="format" class="label">Format</label>
	<select id="format" class="select bg-base-200 select-bordered" bind:value={selected}>
		{#each options[config.device] as option}
			<option value={option}>
				{`${option.resolution[0]}x${option.resolution[1]}@${option.framerate}fps`}
			</option>
		{/each}
	</select>
	<label for="rotation" class="label">Rotation</label>
	<select id="rotation" class="select bg-base-200 select-bordered" bind:value={config.rotation}>
		<option value={0}>0°</option>
		<option value={90}>90°</option>
		<option value={180}>180°</option>
		<option value={270}>270°</option>
	</select>
	<label for="v4l2Options" class="label">Additional V4L2 options</label>
  <div id="v4l2Options" class="flex">
	{#each Object.keys(config.v4l2Options) as optionName}
    <div class="flex flex-row gap-2">
		<input type="text" class="input input-bordered bg-base-200 w-full" value={optionName} on:change={e => {
			config.v4l2Options[e.target.value] = config.v4l2Options[optionName];
			delete config.v4l2Options[optionName];
		}}>
		<input type="text" class="input input-bordered bg-base-200 w-full" bind:value={config.v4l2Options[optionName]}>
    <button class="btn btn-ghost w-fit mr-2" on:click={e => {
			e.preventDefault();
			delete config.v4l2Options[optionName];
			config.v4l2Options = config.v4l2Options;
		}}>
			<span class="material-icons">remove</span>
		</button>
		</div>
	{/each}
  </div>
  <div class="flex flex-row gap-2">
	<input type="text" class="input input-bordered bg-base-200 w-full" bind:value={newOptionName}>
	<input type="text" class="input input-bordered bg-base-200 w-full" bind:value={newOptionValue}>
	<button class="btn btn-ghost" on:click={e => {
		e.preventDefault();
		if (newOptionName != '' && newOptionValue != '') {
			config.v4l2Options[newOptionName] = newOptionValue;
			newOptionName = '';
			newOptionValue = '';
		}
	}}>Add</button>
  </div>
  <div class="flex flex-row justify-between mt-4">
  <button type="button" class="btn btn-ghost w-fit"><a href="/stream">Cancel</a></button>
	<button type="submit" class="btn btn-primary w-fit">Save</button>
  </div>
</form>
</div>
