<script lang="ts">
	import type { Config, Options } from '$lib';

	export let options: Options;
	export let config: Config;
	export let onSubmit: (_: Config) => void;

	let selected;
	let newOptionName = '';
	let newOptionValue = '';
</script>

<form on:submit={e => {
		e.preventDefault();
		config.resolution = selected.resolution;
		config.framerate = selected.framerate;
		onSubmit(config);
	}}>
	<label for="device">Device</label>
  <br />
	<select
		id="device"
		bind:value={config.device}
		on:change={() => selected = options[config.device][0]}
	>
		{#each Object.keys(options) as deviceName}
			<option value={deviceName}>{deviceName}</option>
		{/each}
	</select>
  <br />
	<label for="format">Format</label>
  <br />
	<select id="format" bind:value={selected}>
		{#each options[config.device] as option}
			<option value={option}>
				{`${option.resolution[0]}x${option.resolution[1]}@${option.framerate}fps`}
			</option>
		{/each}
	</select>
  <br />
	<label for="rotation">Rotation</label>
  <br />
	<select id="rotation" bind:value={config.rotation}>
		<option value={0}>0°</option>
		<option value={90}>90°</option>
		<option value={180}>180°</option>
		<option value={270}>270°</option>
	</select>
  <br />
	<label for="v4l2Options">Additional V4L2 options</label>
  <br />
	{#each Object.entries(config.v4l2Options) as [optionName, optionValue]}
		<button on:click={e => {
			e.preventDefault();
			delete config.v4l2Options[optionName];
			config.v4l2Options = config.v4l2Options;
			}}>- </button>
		<input type="text" value={optionName} on:change={e => {
			config.v4l2Options[e.target.value] = config.v4l2Options[optionName];
			delete config.v4l2Options[optionName];
		}}>
		<input type="text" bind:value={config.v4l2Options[optionName]}>
		<br />
	{/each}
	<input type="text" bind:value={newOptionName}>
	<input type="text" bind:value={newOptionValue}>
	<button on:click={e => {
		e.preventDefault();
		if (newOptionName != '' && newOptionValue != '') {
			config.v4l2Options[newOptionName] = newOptionValue;
			newOptionName = '';
			newOptionValue = '';
		}
	}}>Add</button>
	<br />
	<input type="submit" value="Save">
</form>
