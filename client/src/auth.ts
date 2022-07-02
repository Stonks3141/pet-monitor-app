const validate = async (password: string): Promise<boolean> => {
  const res = await fetch('/api/auth', {
    method: 'POST',
    mode: 'cors',
    cache: 'no-cache',
    credentials: 'same-origin',
    body: password,
  });

  if (res.status == 500) {
    alert('Server error when attempting to authenticate. Please try again.');
  }

  return res.ok;
};

export { validate };
