enum Msg {
  OpenConfig,
  CloseConfig,
  LogIn,
  LogOut,
}

interface Model {
  loggedIn: boolean;
  page: 'config' | 'camera';
}

const init = (): Model => ({
  loggedIn: false,
  page: 'camera',
});

const update = (model: Model, msg: Msg): Model => {
  let newModel = model;
  switch (msg) {
    case Msg.LogIn:
      newModel.loggedIn = true;
      break;
    case Msg.LogOut:
      newModel.loggedIn = false;
      break;
    case Msg.OpenConfig:
      newModel.page = 'config';
      break;
    case Msg.CloseConfig:
      newModel.page = 'camera';
      break;
  }
  return newModel;
};

export { init, update, Msg };
export type { Model };