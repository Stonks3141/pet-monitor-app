import React, { useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Login, Camera, Main } from 'routes';
import { AuthContext } from 'context';
import { useCookies } from 'react-cookie';

const App = () => {
  const [auth, setAuth] = useState(false);
  const [cookies] = useCookies();

  useEffect(() => {
    if ('token' in cookies) {
      setAuth(true);
    }
  }, []);

  return (
    <AuthContext.Provider value={{ auth, setAuth }}>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Main />} />
          <Route path="lock" element={<Login />} />
          <Route path="camera" element={<Camera />} />
          <Route path="*" element={<Main />} />
        </Routes>
      </BrowserRouter>
    </AuthContext.Provider>
  );
};

export default App;
