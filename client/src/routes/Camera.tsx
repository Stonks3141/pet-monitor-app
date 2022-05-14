import React, { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useCookies } from 'react-cookie';
import { LiveCam } from "components";

const Camera = () => {
  const [cookies] = useCookies();
  const navigate = useNavigate();

  useEffect(() => {
    console.log(cookies);
    if (!('connect.sid' in cookies)) {
      navigate('/lock');
    }
  }, []);

  return <LiveCam />;
};

export default Camera;
