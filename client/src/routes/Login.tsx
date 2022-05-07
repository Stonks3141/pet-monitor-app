import React, { useEffect } from "react";
import { useNavigate } from 'react-router-dom';
import { useCookies } from "react-cookie";
import { LoginMenu } from "components";

const Login = () => {
  const [cookies] = useCookies();
  const navigate = useNavigate();

  useEffect(() => {
    if ('token' in cookies) {
      navigate('/camera');
    }
  });

  return (
    <main>
      <LoginMenu />
    </main>
  );
};

export default Login;
