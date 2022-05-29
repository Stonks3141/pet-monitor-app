import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCookies } from 'react-cookie';
import { Spinner } from 'components';

const Main = () => {
  const navigate = useNavigate();
  const [cookies] = useCookies();

  useEffect(() => {
    if ('token' in cookies) {
      navigate('/camera');
    } else {
      navigate('/lock');
    }
  }, []);

  return (
    <div className="flex grow place-content-center place-items-center content-center">
      <Spinner />
    </div>
  );
};

export default Main;
