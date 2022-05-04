import { configureStore } from '@reduxjs/toolkit';

const store = configureStore({
  reducer: {
    password: passwordReducer,
  },
});

type RootState = ReturnType<typeof store.getState>;
type AppDispatch = typeof store.dispatch;

export default store;
export type { RootState, AppDispatch };
