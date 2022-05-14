import React, { useState, useEffect } from 'react';
import { useCookies } from 'react-cookie';
import axios from 'axios';

const eyeIcon = (
  <svg width='24px' height='24px' viewBox='0 0 24 24'>
    <title>Eye icon</title>
    <path fill='currentColor' d='M12,9A3,3 0 0,1 15,12A3,3 0 0,1 12,15A3,3 0 0,1 9,12A3,3 0 0,1 12,9M12,4.5C17,4.5 21.27,7.61 23,12C21.27,16.39 17,19.5 12,19.5C7,19.5 2.73,16.39 1,12C2.73,7.61 7,4.5 12,4.5M3.18,12C4.83,15.36 8.24,17.5 12,17.5C15.76,17.5 19.17,15.36 20.82,12C19.17,8.64 15.76,6.5 12,6.5C8.24,6.5 4.83,8.64 3.18,12Z' />
  </svg>
);

const slashedEyeIcon = (
  <svg width='24px' height='24px' viewBox='0 0 24 24'>
    <title>Slashed eye icon</title>
    <path fill='currentColor' d='M2,5.27L3.28,4L20,20.72L18.73,22L15.65,18.92C14.5,19.3 13.28,19.5 12,19.5C7,19.5 2.73,16.39 1,12C1.69,10.24 2.79,8.69 4.19,7.46L2,5.27M12,9A3,3 0 0,1 15,12C15,12.35 14.94,12.69 14.83,13L11,9.17C11.31,9.06 11.65,9 12,9M12,4.5C17,4.5 21.27,7.61 23,12C22.18,14.08 20.79,15.88 19,17.19L17.58,15.76C18.94,14.82 20.06,13.54 20.82,12C19.17,8.64 15.76,6.5 12,6.5C10.91,6.5 9.84,6.68 8.84,7L7.3,5.47C8.74,4.85 10.33,4.5 12,4.5M3.18,12C4.83,15.36 8.24,17.5 12,17.5C12.69,17.5 13.37,17.43 14,17.29L11.72,15C10.29,14.85 9.15,13.71 9,12.28L5.6,8.87C4.61,9.72 3.78,10.78 3.18,12Z' />
  </svg>
);

interface LoginMenuProps {
  onSuccess?: () => void;
}

/**
 * Login menu component, displays a form that posts password to api on submission to establish an Express session.
 * It also handles using a stored password to get a new session.
 * 
 * @param onSuccess Optional callback, called when validation is successful
*/
const LoginMenu = (props: LoginMenuProps) => {
  const [password, setPassword] = useState('');
  const [incorrect, setIncorrect] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [cookies, setCookie] = useCookies();

  const validate = (pwd: string) => (
    axios.post('/api/auth', {hash: pwd})
      .then(res => {
        if (res.status == 401) {
          setIncorrect(true);
        }
        else if (res.status == 200) {
          setCookie('password', password);
          setIncorrect(false);
          props.onSuccess();
        }
        else {
          throw new Error(res.statusText + res.data);
        }
      })
      .catch(err => console.error(err))
  );

  useEffect(() => {
    if ('password' in cookies && !('connect.sid' in cookies)) {
      setPassword(cookies.password);
      validate(cookies.password);
    }
  }, []);

  return (
    <div className='flex  flex-col content-center place-content-center place-items-center grow'>
      <form className='flex flex-col gap-3 place-items-center h-fit w-fit p-4 shadow-md rounded-lg bg-white dark:bg-slate-800' onSubmit={(event) => {
        event.preventDefault();
        validate(password);
      }}>
        <label className='flex flex-col gap-2'>
          <span className='dark:text-white'>Master password</span>
          <div className='flex flex-row gap-2'>
            <input type={showPassword ? 'text' : 'password'} className='border w-64 p-1 outline-none rounded focus:border-2 border-indigo-400 bg-slate-200 dark:bg-slate-700' onChange={event => setPassword(event.target.value)} />
            <button title={showPassword ? 'Hide password' : 'Show password'} className='text-indigo-500 hover:text-indigo-700 dark:hover:text-indigo-300 outline-none rounded border-indigo-400 focus:border-2' onClick={(e) => {
              e.preventDefault();
              setShowPassword(!showPassword);
            }}>
              {showPassword ? slashedEyeIcon : eyeIcon}
            </button>
          </div>
        </label>
        {incorrect && <span className='text-red'>Invalid password</span>}
        <input type='submit' className='max-w-fit shadow-md px-4 py-2 rounded cursor-pointer outline-none active:ring-2 active:ring-indigo-300 bg-indigo-500 text-white hover:bg-indigo-600 focus:border-2 focus:border-indigo-400 dark:bg-indigo-600' value='Unlock' />
      </form>
      <div className='basis-1/3'></div>
    </div>
  );
};

export default LoginMenu;
