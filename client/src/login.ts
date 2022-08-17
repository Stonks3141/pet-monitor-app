const setupForm = (
  form: HTMLFormElement, 
  pwdInput: HTMLInputElement,
  onSuccess: () => void,
  onFailure: () => void
) => {
  form.onsubmit = async (event: SubmitEvent) => {
    event.preventDefault();
    const res = await fetch('/api/login', {
      method: 'POST',
      mode: 'cors',
      cache: 'no-cache',
      credentials: 'same-origin',
      body: pwdInput.value,
    });

    if (res.ok) {
      onSuccess();
    } else if (res.status == 500) {
      alert('Server error. Please try again.');
    } else {
      onFailure();
    }
  };
};

export { setupForm };