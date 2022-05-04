import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Provider } from 'react-redux';
import { App } from 'components';
import { Login, Camera } from 'routes';
import store from './store';
import 'spectre.css/dist/spectre.min.css';
import './style.css';

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <Provider store={store}>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<App />}>
          <Route path="login" element={<Login />} />
          <Route path="camera" element={<Camera />} />
          <Route path="*" element={
            <main>
              <p>Not found</p>
            </main>
          } />
        </Route>
      </Routes>
    </BrowserRouter>
  </Provider>
);
