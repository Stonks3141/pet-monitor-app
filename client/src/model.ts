enum Msg {
  OpenConfig,
  CloseConfig,
  LogIn,
  LogOut,
  Incorrect,
}

interface Model {
  loggedIn: boolean;
  incorrect: boolean;
  page: 'config' | 'camera';
}

const init = (): Model => ({
  loggedIn: false,
  incorrect: false,
  page: 'camera',
});

const update = (model: Model, msg: Msg): Model => {
  let newModel = model;
  switch (msg) {
    case Msg.LogIn:
      newModel.loggedIn = true;
      newModel.incorrect = false;
      break;
    case Msg.LogOut:
      newModel.loggedIn = false;
      break;
    case Msg.Incorrect:
      newModel.incorrect = true;
      break;
    case Msg.OpenConfig:
      if (model.loggedIn) {
        newModel.page = 'config';
      }
      break;
    case Msg.CloseConfig:
      if (model.loggedIn) {
        newModel.page = 'camera';
      }
      break;
  }
  return newModel;
};

export { init, update, Msg };
export type { Model };