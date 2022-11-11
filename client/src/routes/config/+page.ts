import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
  const res = await fetch('/api/config');
  const config = await res.json();
  const caps_res = await fetch('/api/capabilities');
  const caps = await caps_res.json();
  return {
    config,
    capabilities: caps,
  };
};
