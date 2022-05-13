import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Login, Camera, Main } from 'routes';

const App = () => (
  <>
    {/* TODO: navbar */}
    <div className='flex bg-stone-50 dark:bg-gray-900 h-screen'>
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
