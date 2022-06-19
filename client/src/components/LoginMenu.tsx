import React, { useReducer, useState, useContext } from 'react';
import Icon from '@mdi/react';
import { mdiEyeOutline, mdiEyeOffOutline } from '@mdi/js';
import { validate } from 'auth';
import { AuthContext } from 'context';

const eyeIcon = <Icon path={mdiEyeOutline} title="Show Password" size={1} />;

const slashedEyeIcon = (
  <Icon path={mdiEyeOffOutline} title="Hide Password" size={1} />
);

interface LoginMenuProps {
  onSuccess?: () => void;
}

/**
 * Login menu component, displays a form that posts password to api on submission to obtain a JSON web token.
 * It also handles using a stored password to get a new session.
 *
 * @param onSuccess Optional callback, called when validation is successful
 */
const LoginMenu = (props: LoginMenuProps) => {
  const [password, setPassword] = useState('');
  const [incorrect, setIncorrect] = useState(false);
  const [showPassword, toggleShowPassword] = useReducer(state => !state, false);
  const { auth: _, setAuth } = useContext(AuthContext);

  const onSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();

    if (await validate(password)) {
      setIncorrect(false);
      setAuth(true);
      props.onSuccess();
    } else {
      setIncorrect(true);
      setAuth(false);
    }
  };

  return (
    <div className="flex flex-col content-center place-content-center place-items-center grow">
      <form
        onSubmit={onSubmit}
        className="flex flex-col gap-3 place-items-center p-4 shadow-md rounded-lg
          bg-white dark:bg-slate-800"
      >
        <label className="flex flex-col gap-2">
          <span className="dark:text-white">Master password</span>
          <div className="flex flex-row gap-2">
            <input
              type={showPassword ? 'text' : 'password'}
              onChange={event => setPassword(event.target.value)}
              className="border w-64 p-1 outline-none rounded focus:border-2
                bg-slate-200 dark:bg-slate-700 border-indigo-400"
            />
            <button
              title={showPassword ? 'Hide password' : 'Show password'}
              onClick={e => {
                e.preventDefault();
                toggleShowPassword();
              }}
              className="text-indigo-500 hover:text-indigo-700 dark:hover:text-indigo-300
                outline-none rounded border-indigo-400 focus:border-2"
            >
              {showPassword ? slashedEyeIcon : eyeIcon}
            </button>
          </div>
        </label>
        {incorrect && <span className="text-red-500">Invalid password</span>}
        <input
          type="submit"
          value="Unlock"
          className="max-w-fit shadow px-4 py-1 rounded-full cursor-pointer outline-none active:ring-2
            active:ring-indigo-300 bg-indigo-500 text-white hover:bg-indigo-600 focus:border-2
            focus:border-indigo-400 dark:bg-indigo-600"
        />
      </form>
      <div className="basis-1/3" />
    </div>
  );
};

export default LoginMenu;
