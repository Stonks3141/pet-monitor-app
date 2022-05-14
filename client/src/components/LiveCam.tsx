import React, { useState, useEffect } from 'react';
import { useCookies } from 'react-cookie';
import axios from 'axios';

const LiveCam = () => {
  const [stream, setStream] = useState('');
  const [cookies] = useCookies();

  useEffect(() => {
    axios.get('/api/stream')
    .then(res => {
      if (res.status == 200) {
        setStream(res.data);
      }
      else {
        throw new Error(res.statusText + res.data);
      }
    })
    .catch(err => console.error(err));
  }, []);

  return <p>{stream}</p>;
};

export default LiveCam;
