import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import './style.css';

const LoginMenu = () => {
  const [password, setPassword] = useState("");
  let navigate = useNavigate();

  return (
    <form className="LoginMenu" onSubmit={() => {alert(password); navigate("/camera");}}>
      <label>Password<br/></label>
      <input type="password" name="password" required onChange={(event) => setPassword(event.target.value)} />
      <br/>
      <input type="submit" className="submit" value="Unlock" />
    </form>
  );
};

export default LoginMenu;
