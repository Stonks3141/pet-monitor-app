<script lang="ts">
  import type { PageData } from './$types';
  export let data: PageData;
  let [width, height] = data.resolution;
</script>

<video controls autoplay muted playsinline {width} {height}>
  <source src="/stream.mp4" type="video/mp4" />
</video>
