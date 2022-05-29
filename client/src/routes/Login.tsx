import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCookies } from 'react-cookie';
import { LoginMenu } from 'components';

const Login = () => {
  const navigate = useNavigate();
  const [cookies] = useCookies();

  useEffect(() => {
    if ('token' in cookies) {
      navigate('/camera');
    }
  }, []);

  return <LoginMenu onSuccess={() => navigate('/camera')} />;
};

export default Login;
