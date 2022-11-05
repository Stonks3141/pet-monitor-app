<script lang="ts">
  import { goto } from '$app/navigation';
  let showPassword = false;
  let password = '';
  const passwordUpdate = (e: Event & { currentTarget: HTMLInputElement }) => {
    password = (<HTMLInputElement>e.target).value;
  };
</script>

<svlete:head>
  <title>Login</title>
</svlete:head>

<form
  on:submit={(e) => {
    e.preventDefault();
    fetch('/api/login', {
      method: 'POST',
      body: password,
    }).then((res) => {
      if (res.status === 200) {
        goto('/stream');
      }
    });
  }}
>
  <label for="password">Password</label>
  <br />
  <input type={showPassword ? 'text' : 'password'} id="password" on:input={passwordUpdate} />
  <button
    on:click={(e) => {
      e.preventDefault();
      showPassword = !showPassword;
    }}>{showPassword ? 'hide' : 'show'}</button
  >
  <br />
  <input type="submit" value="Sign In" />
</form>
