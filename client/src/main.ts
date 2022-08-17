import './style.css';
import { setupForm } from './login';

const video = `
  <video controls autoplay muted playsinline>
    <source src="/stream.mp4" type='video/mp4; codecs="avc1.64002a"' />
  </video>
`;

const app = document.querySelector<HTMLDivElement>('#app')!;

const res = await fetch('/api/config', {
  method: 'HEAD',
});

let loggedIn = res.ok;

if (!loggedIn) {
  app.innerHTML = `
    <div class="card w-96 h-fit shadow-xl bg-base-100">
      <form class="form-control card-body" id="login">
        <label for="password" class="sr-only">Password</label>
        <input id="password" name="password" type="password" autocomplete="current-password" placeholder="Password" required class="input bg-base-200" />
        <button type="submit" class="btn btn-primary">
          Sign in
        </button>
      </form>
    </div>
    <div class="basis-1/4"></div>
  `;
  const passwordInput = document.querySelector<HTMLInputElement>('#password')!;

  setupForm(
    document.querySelector<HTMLFormElement>('#login')!,
    passwordInput,
    () => {
      passwordInput.classList.replace('input-error', 'input-success');
      app.innerHTML = video;
    },
    () => {
      passwordInput.classList.add('input-error');
    },
  );
} else {
  app.innerHTML = video;
}