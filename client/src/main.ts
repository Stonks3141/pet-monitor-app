import './style.css';
import { init, update, Msg } from './model';

interface Config {
  resolution: [number, number];
  rotation: number;
  framerate: number;
  device: string;
}

const app = document.querySelector<HTMLDivElement>('#app')!;

let model = init();

const getToken = (): string => {
  const match = document.cookie.match(new RegExp('(^| )token=([^;]+)'));
  if (match) {
    return match[2];
  } else {
    return '';
  }
};

const setupLogin = () => {
  let show = false;
  app.innerHTML = `
    <div class="card w-96 max-w-full h-fit bg-base-100">
      <form class="form-control card-body" id="login">
        <label for="password" class="sr-only">Password</label>
        <div class="flex flex-row gap-2">
          <input id="password" name="password" type="password" autocomplete="current-password" placeholder="Password" required class="input bg-base-200 w-full" />
          <button type="button" style="margin-left:-4.125rem" id="toggleShow" class="btn btn-ghost">
            <span id="showIcon" class="material-icons">visibility</span>
          </button>
        </div>
        <button type="submit" class="btn btn-primary mt-2">Sign in</button>
      </form>
    </div>
  `;

  const form = document.querySelector<HTMLFormElement>('#login')!;
  const toggleShow = document.querySelector<HTMLButtonElement>('#toggleShow')!;
  const showIcon = document.querySelector<HTMLSpanElement>('#showIcon')!;
  const passwordInput = document.querySelector<HTMLInputElement>('#password')!;

  toggleShow.onclick = () => {
    show = !show;
    passwordInput.type = show ? 'text' : 'password';
    showIcon.innerHTML = show ? 'visibility_off' : 'visibility';
  }

  form.onsubmit = async (event: SubmitEvent) => {
    event.preventDefault();

    const res = await fetch('/api/login', {
      method: 'POST',
      mode: 'cors',
      cache: 'no-cache',
      credentials: 'same-origin',
      body: passwordInput.value,
    });

    if (res.ok) {
      passwordInput.classList.replace('input-error', 'input-success');
      model = update(model, Msg.LogIn);
    } else if (res.status == 500) {
      alert('Server error. Please try again.');
    } else {
      passwordInput.classList.add('input-error');
    }

    view();
  }
};

const setupCamera = async () => {
  const res = await fetch('/api/config');
  if (res.status === 401) {
    model = update(model, Msg.LogOut);
    view();
  } else if (res.status === 500) {
    alert('Server error. Please reload.');
  }
  let config: Config = await res.json()!;

  app.innerHTML = `
    <button id="settings" class="btn m-4 absolute top-0 right-0">Settings</button>
    <video controls autoplay muted playsinline width="${config.resolution[0]}" height="${config.resolution[1]}">
      <source src="/stream.mp4" type='video/mp4; codecs="avc1.64002a"' />
    </video>
  `;

  document.querySelector<HTMLButtonElement>('#settings')!.onclick = () => {
    model = update(model, Msg.OpenConfig);
    view();
  }
}

const setupConfig = async () => {
  const res = await fetch('/api/config');
  if (res.status === 401) {
    model = update(model, Msg.LogOut);
    view();
  } else if (res.status === 500) {
    alert('Server error. Please reload.');
  }
  let config: Config = await res.json()!;

  app.innerHTML = `
    <div class="card h-fit w-full md:w-1/2 bg-base-100">
      <div class="flex flex-row justify-between gap-2 mx-8 mt-8">
        <h2 class="card-title">Settings</h2>
        <button id="logout" class="btn btn-ghost w-fit">Log out</button>
      </div>
      <form id="config" class="form-control card-body">
        <label for="width" class="label">Width</label>
        <input id="width" type="text" value="${config.resolution[0]}" class="input input-bordered bg-base-200" required />
        <label for="height" class="label">Height</label>
        <input id="height" type="text" value="${config.resolution[1]}" class="input input-bordered bg-base-200" required />
        <label for="rotation" class="label">Rotation</label>
        <select id="rotation" class="select bg-base-200 select-bordered" required>
          <option ${config.rotation === 0   && 'selected'} value="0"  >0°</option>
          <option ${config.rotation === 90  && 'selected'} value="90" >90°</option>
          <option ${config.rotation === 180 && 'selected'} value="180">180°</option>
          <option ${config.rotation === 270 && 'selected'} value="270">270°</option>
        </select>
        <label for="framerate" class="label">Framerate</label>
        <input id="framerate" type="text" value="${config.framerate}" class="input input-bordered bg-base-200" required />
        <label for="device" class="label">Camera Device</label>
        <input id="device" type="text" value="${config.device}" class="input input-bordered bg-base-200" required />
        <div class="flex flex-row justify-between mt-4">
          <button id="close" type="button" class="btn btn-ghost w-fit">Cancel</button>
          <button class="btn btn-primary w-fit">Save</button>
        </div>
      </form>
    </div>
  `;

  document.querySelector<HTMLButtonElement>('#close')!.onclick = () => {
    model = update(model, Msg.CloseConfig);
    view();
  }

  document.querySelector<HTMLButtonElement>('#logout')!.onclick = async () => {
    document.cookie = 'token=; Max-Age=0';
    model = update(model, Msg.LogOut);
    model = update(model, Msg.CloseConfig);
    view();
  }

  const form = document.querySelector<HTMLFormElement>('#config')!;
  const width = document.querySelector<HTMLInputElement>('#width');
  const height = document.querySelector<HTMLInputElement>('#height');
  const rotation = document.querySelector<HTMLInputElement>('#rotation');
  const framerate = document.querySelector<HTMLInputElement>('#framerate');
  const device = document.querySelector<HTMLInputElement>('#device');

  form.onsubmit = async (event: SubmitEvent) => {
    event.preventDefault();

    const res = await fetch('/api/config', {
      method: 'PUT',
      headers: new Headers({
        'Content-Type': 'application/json',
        'x-csrf-token': getToken(),
      }),
      body: JSON.stringify({
        resolution: [
          parseInt(width!.value),
          parseInt(height!.value),
        ],
        rotation: parseInt(rotation!.value),
        framerate: parseInt(framerate!.value),
        device: device!.value,
      }),
    });

    if (res.status === 401) {
      model = update(model, Msg.LogOut);
      view();
    } else if (res.status === 500) {
      alert('Server error. Please try again.');
    } else if (res.ok) {
      model = update(model, Msg.CloseConfig);
      view();
    }
  };
}

const view = () => {
  if (!model.loggedIn) {
    setupLogin();
  } else {
    if (model.page === 'camera') {
      setupCamera();
    } else if (model.page === 'config') {
      setupConfig();
    }
  }
}

view();

fetch('/api/config', {
  method: 'HEAD',
}).then(res => {
  let old = model.loggedIn;
  if (res.ok) {
    model = update(model, Msg.LogIn);
  } else {
    model = update(model, Msg.LogOut);
  }
  if (old !== model.loggedIn) {
    view();
  }
});