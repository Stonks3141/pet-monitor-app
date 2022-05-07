import React, { useState } from 'react';
import { useCookies } from 'react-cookie';
import axios from 'axios';
import './style.css';

const LoginMenu = () => {
  const [password, setPassword] = useState('');
  const [incorrect, setIncorrect] = useState(null);
  const [_cookies, setCookie] = useCookies();

  const validate = (event: React.FormEvent) => {
    event.preventDefault();
    axios.post('/api/auth', { password: password })
      .then(res => {
        if (res.status == 401) {
          setIncorrect(
            <label className='incorrect'>Incorrect password</label>
          );
        }
        else {
          setCookie('token', res.data);
          setCookie('password', password);
          setIncorrect(null);
        }
      })
      .catch(err => console.log('Auth error: ' + err));
  }

  return (
    <form className='LoginMenu' onSubmit={validate}>
      <label>Password</label>
      <input type='password' name='password' required onChange={(event) => setPassword(event.target.value)} />
      {incorrect}
      <input type='submit' className='submit' value='Unlock' />
    </form>
  );
};

export default LoginMenu;
