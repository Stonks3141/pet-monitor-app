import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Login, Camera, Main } from 'routes';

const App = () => (
  <>
    {/* TODO: navbar/logout */}
    <div className='flex bg-gray-50 dark:bg-gray-900 h-screen dark:text-white'>
      <BrowserRouter>
        <Routes>
          <Route path='/' element={<Main />} />
          <Route path='lock' element={<Login />} />
          <Route path='camera' element={<Camera />} />
        </Routes>
      </BrowserRouter>
    </div>
  </>
);

export default App;
