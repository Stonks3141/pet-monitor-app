import React, { useEffect } from "react";
import { useNavigate } from 'react-router-dom';
import { useCookies } from "react-cookie";
import { LoginMenu } from "components";

const Login = () => {
  const navigate = useNavigate();
  const [cookies] = useCookies();

  useEffect(() => {
    if ('connect.sid' in cookies) {
      navigate('/camera');
    }
  }, [cookies]);

  return <LoginMenu onSuccess={() => navigate('/camera')} />;
};

export default Login;
