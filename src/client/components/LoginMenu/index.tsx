import React from 'react';
import './style.css';

interface LoginMenuProps {
  onLogin: () => void;
};

const LoginMenu = ({onLogin}: LoginMenuProps) => {
  return <p onClick={onLogin}>login here</p>;
};

export default LoginMenu;