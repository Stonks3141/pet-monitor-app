import type { PageLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const prerender = true;

export const load: PageLoad = async ({ fetch }) => {
  const res = await fetch('/api/config', {
    method: 'HEAD',
  });
  if (res.status === 200) {
    throw redirect(307, '/stream');
  } else {
    throw redirect(307, '/login');
  }
};
