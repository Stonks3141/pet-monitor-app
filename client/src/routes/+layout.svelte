<script>
  import './app.css';
</script>

<svelte:head>
  <link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet" />
</svelte:head>

<main>
  <div
    class="flex flex-col min-h-screen w-screen p-4 place-content-center items-center bg-base-300"
  >
    <slot />
  </div>
</main>
