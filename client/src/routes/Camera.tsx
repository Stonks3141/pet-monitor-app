import React, { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useCookies } from 'react-cookie';
import { LiveCam } from "components";

const Camera = () => {
  const [cookies] = useCookies();
  const navigate = useNavigate();

  useEffect(() => {
    if (!('connect.sid' in cookies)) {
      navigate('/lock');
    }
  }, [cookies['connect.sid']]);

  return (
    <main>
      <LiveCam />
    </main>
  );
};

export default Camera;
