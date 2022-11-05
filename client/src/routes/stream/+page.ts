import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
  const res = await fetch('/api/config');
  const config = await res.json();
  console.log(config);
  return {
    resolution: config.resolution,
  };
};
