import React, { useContext, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { LoginMenu } from 'components';
import { AuthContext } from 'context';

const Login = () => {
  const navigate = useNavigate();
  const { auth } = useContext(AuthContext);

  useEffect(() => {
    if (auth) {
      navigate('/camera');
    }
  }, []);

  return <LoginMenu onSuccess={() => navigate('/camera')} />;
};

export default Login;
