import React, { useEffect } from "react";
import { useNavigate } from 'react-router-dom';
import { useCookies } from "react-cookie";
import { LoginMenu } from "components";

const Login = () => {
  const [cookies] = useCookies();
  const navigate = useNavigate();

  useEffect(() => {
    if ('password' in cookies) {
      navigate('/camera');
    }
  }, [cookies.password]);

  return (
    <main>
      <LoginMenu />
    </main>
  );
};

export default Login;
