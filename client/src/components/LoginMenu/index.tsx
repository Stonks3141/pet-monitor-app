import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCookies } from 'react-cookie';
import axios from 'axios';
import './style.css';

const LoginMenu = () => {
  const [password, setPassword] = useState('');
  const [incorrect, setIncorrect] = useState(false);
  const [cookies, setCookie] = useCookies();
  const navigate = useNavigate();

  const validate = (pwd: string) => {
    axios.post('/api/auth', null, {headers: {password: pwd}})
      .then(res => {
        if (res.status == 401) {
          setIncorrect(true);
        }
        else if (res.status == 200) {
          setCookie('password', password);
          setIncorrect(false);
          navigate('/camera');
        }
        else {
          alert('An error has occurred: ' + res.statusText + res.data);
          throw new Error(res.statusText + res.data);
        }
      })
      .catch(err => console.error(err));
  }

  useEffect(() => {
    if ('password' in cookies) {
      setPassword(cookies.password);
      validate(password);
    }
  }, []);

  return (
    <form className='LoginMenu' onSubmit={(e) => {
      e.preventDefault();
      validate(password);
    }}>
      <label>Password</label>
      <input type='password' onChange={event => setPassword(event.target.value)} />
      {incorrect && <label className='incorrect'>Incorrect password</label>}
      <input type='submit' className='submit' value='Unlock' />
    </form>
  );
};

export default LoginMenu;
