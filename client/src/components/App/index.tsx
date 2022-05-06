import React from 'react';
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom';
import { Login, Camera } from 'routes';
import './style.css';

const App = () => {
  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path='/' element={<Login />} />
          <Route path='/lock' element={<Login />} />
          <Route path='/camera' element={<Camera />} />
          <Route path='/*' element={
            <main>
              <p>Not found</p>
            </main>
          } />
        </Routes>
      </BrowserRouter>
    </>
  );
};

export default App;
