import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCookies } from 'react-cookie';

const Main = () => {
  const navigate = useNavigate();
  const [cookies] = useCookies();

  useEffect(() => {
    if ('connect.sid' in cookies) {
      //navigate('/camera');
    }
    else {
      //navigate('/lock');
    }
  }, [cookies]);

  return <p>Loading...</p>;
};

export default Main;
