import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
  const res = await fetch('/api/config');
  const config = await res.json();
  return {
    config,
    options: {
      '/dev/video0': [
        { resolution: [640, 480], framerate: 30 },
        { resolution: [640, 360], framerate: 30 },
      ],
    },
  };
};
