const validate = async (password: string): Promise<boolean> => {
  const res = await fetch('/api/auth', {
    method: 'POST',
    mode: 'cors',
    cache: 'no-cache',
    credentials: 'same-origin',
    body: password,
  });

  return res.ok;
};

export { validate };
