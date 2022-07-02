import React, { useContext, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { LiveCam } from 'components';
import { AuthContext } from 'context';

const Camera = () => {
  const navigate = useNavigate();
  const { auth } = useContext(AuthContext);

  useEffect(() => {
    if (!auth) {
      navigate('/lock');
    }
  }, []);

  return <LiveCam />;
};

export default Camera;
