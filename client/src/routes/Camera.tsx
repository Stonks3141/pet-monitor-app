import React from "react";
import { useNavigate } from "react-router-dom";
import { useCookies } from 'react-cookie';
import { LiveCam } from "components";

const Camera = () => {
  const [cookies] = useCookies();
  const navigate = useNavigate();

  if (!('token' in cookies)) {
    navigate('/lock');
  }

  return (
    <main>
      <LiveCam />
    </main>
  );
};

export default Camera;
