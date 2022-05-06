import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCookies } from 'react-cookie';
import axios from 'axios';
import './style.css';

const LoginMenu = () => {
  const [password, setPassword] = useState("");
  let navigate = useNavigate();

  const [cookies, setCookie] = useCookies();

  useEffect(() => {
    if (!('password' in cookies)) {
      setCookie('password', '');
    }

    axios.post('/auth', { password: password })
      .then(res => {
        if (res.data == 'ok') {
          navigate('/camera');
        }
      })
      .catch(err => console.log('Auth error: ' + err));
  });

  return (
    <form className="LoginMenu" onSubmit={() => setCookie('password', password)}>
      <label>Password<br/></label>
      <input type="password" name="password" required onChange={(event) => setPassword(event.target.value)} />
      <br/>
      <input type="submit" className="submit" value="Unlock" />
    </form>
  );
};

export default LoginMenu;
