import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Login, Camera, Main } from 'routes';
import './style.css';

const App = () => (
  <>
    {/* TODO: navbar */}
    <div>
      <BrowserRouter>
        <Routes>
          <Route path='/' element={<Main />} /> // redirects to Login
          <Route path='lock' element={<Login />} /> // redirects to Camera if signed in
          <Route path='camera' element={<Camera />} /> // redirects to Login if not signed in
        </Routes>
      </BrowserRouter>
    </div>
  </>
);

export default App;
