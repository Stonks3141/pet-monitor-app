import React, { useContext, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Spinner } from 'components';
import { AuthContext } from 'context';

const Main = () => {
  const navigate = useNavigate();
  const { auth } = useContext(AuthContext);

  useEffect(() => {
    if (auth) {
      navigate('/camera');
    } else {
      navigate('/lock');
    }
  }, []);

  return (
    <div className='flex grow place-content-center place-items-center content-center'>
      <Spinner />
    </div>
  );
};

export default Main;
