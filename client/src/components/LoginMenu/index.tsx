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
    axios.post('/api/auth', {password: pwd})
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
    <form className='flex flex-col grid-cols-1 place-content-center place-items-center shadow-lg mx-auto max-w-sm rounded-xl items-center p-5 space-y-3 bg-white dark:bg-gray-800' onSubmit={(e) => {
      e.preventDefault();
      validate(password);
    }}>
      <label>
        <span className='dark:text-white my-1'>Master password</span>
        <input type='password' className='border p-1 text-sm outline-none rounded-md focus:border-2 border-indigo-400 bg-slate-200 dark:bg-slate-700 dark:text-white peer invalid:border-red' onChange={event => setPassword(event.target.value)} />
        <span className='invisible peer-invalid:visible text-red'>Invalid password</span>
      </label>
      <input type='submit' className='shadow-lg px-4 py-1 rounded-full outline-none active:ring-2 active:ring-indigo-300 bg-indigo-500 text-white hover:bg-indigo-600 focus:border-2 focus:border-indigo-400 dark:bg-indigo-600' value='Unlock' />
    </form>
  );
};

export default LoginMenu;
