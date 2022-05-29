import React, { useState } from 'react';
import Icon from '@mdi/react';
import { mdiEyeOutline, mdiEyeOffOutline } from '@mdi/js';
import { validate } from 'auth';

const eyeIcon = <Icon path={mdiEyeOutline} title='Show Password' size={1} />;

const slashedEyeIcon = (
  <Icon path={mdiEyeOffOutline} title='Hide Password' size={1} />
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

  const onSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (validate(password)) {
      setIncorrect(false);
      props.onSuccess();
    }
  };

  const onShowButtonClicked = () => {
    console.log('hi');
    setShowPassword(!showPassword);
  };

  const onTextChanged = (event: React.ChangeEvent<HTMLInputElement>) => {
    setPassword(event.target.value);
  };

  return (
    <div className='flex  flex-col content-center place-content-center place-items-center grow'>
      <form
        onSubmit={onSubmit}
        className='flex flex-col gap-3 place-items-center h-fit w-fit p-4 shadow-md rounded-lg
          bg-white dark:bg-slate-800'
      >
        <label className='flex flex-col gap-2'>
          <span className='dark:text-white'>Master password</span>
          {/* Reverse flex direction so the `<button>` is higher in the DOM
          and pressing enter in the text box submits the form instead of toggling the button*/}
          <div className='flex flex-row-reverse gap-2'>
            <button
              title={showPassword ? 'Hide password' : 'Show password'}
              onClick={onShowButtonClicked}
              className='text-indigo-500 hover:text-indigo-700 dark:hover:text-indigo-300
                outline-none rounded border-indigo-400 focus:border-2'
            >
              {showPassword ? slashedEyeIcon : eyeIcon}
            </button>
            <input
              type={showPassword ? 'text' : 'password'}
              onChange={onTextChanged}
              className='border w-64 p-1 outline-none rounded focus:border-2 border-indigo-400
                bg-slate-200 dark:bg-slate-700'
            />
          </div>
        </label>
        {incorrect && <span className='text-red'>Invalid password</span>}
        <input
          type='submit'
          value='Unlock'
          className='max-w-fit shadow px-4 py-2 rounded cursor-pointer outline-none active:ring-2
            active:ring-indigo-300 bg-indigo-500 text-white hover:bg-indigo-600 focus:border-2
            focus:border-indigo-400 dark:bg-indigo-600'
        />
      </form>
      <div className='basis-1/3' />
    </div>
  );
};

export default LoginMenu;
