// Pet Montitor App
// Copyright (C) 2022  Samuel Nystrom
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

import React, { useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Login, Camera, Main, NotFoundRoute } from 'routes';
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
          <Route path="*" element={<NotFoundRoute />} />
        </Routes>
      </BrowserRouter>
    </AuthContext.Provider>
  );
};

export default App;
