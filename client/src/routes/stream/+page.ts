import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
  const res = await fetch('/api/config');
  const config = await res.json();
  return {
    resolution: config.resolution,
  };
};
