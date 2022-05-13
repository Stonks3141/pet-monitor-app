import React from "react";
import { useNavigate } from 'react-router-dom';
import { LoginMenu } from "components";

const Login = () => {
  const navigate = useNavigate();
  return <LoginMenu onSuccess={() => navigate('/camera')} />;
};

export default Login;
