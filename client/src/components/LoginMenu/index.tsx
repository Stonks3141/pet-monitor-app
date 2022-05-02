import React, { useState } from 'react';
import './style.css';

interface LoginMenuProps {
  onLogin: () => void;
};

const LoginMenu = ({onLogin}: LoginMenuProps) => {
  const [password, setPassword] = useState("");

  return (
    <form onSubmit={() => {alert(password); onLogin();}} className="LoginMenu">
      <label>Password</label>
      <input type="password" onChange={(event) => setPassword(event.target.value)} />
      <br />
      <hr />
      <input type="submit" className="submit" value="Unlock" />
    </form>
  );
};

export default LoginMenu;
