import React from "react";
import { Outlet } from "react-router-dom";
import './style.css';

const App = () => {
  return (
    <>
      <Outlet />
    </>
  );
};

export default App;
