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

<div class="card w-96 max-w-full h-fit bg-base-100">
  <form class="form-control card-body" on:submit={e => {
    e.preventDefault();
    fetch('/api/login', {
      method: 'POST',
      body: password,
    }).then(res => {
      if (res.status === 200) {
        goto('/stream');
      }
    });
  }}>
    <label for="password" class="sr-only">Password</label>
    <div class="flex flex-row gap-2">
      <input
        name="password"
        id="password"
        autocomplete="current-password"
        placeholder="Password"
        required
        type={showPassword ? 'text' : 'password'}
        on:input={passwordUpdate}
        class="input bg-base-200 w-full"
      />
      <button type="button" style="margin-left:-4.125rem" class="btn btn-ghost" on:click={e => {
        e.preventDefault();
        showPassword = !showPassword;
      }}>
        <span class="material-icons">{showPassword ? 'visibility_off' : 'visibility'}</span>
      </button>
    </div>
    <button type="submit" class="btn btn-primary mt-2">Sign In</button>
  </form>
</div>
