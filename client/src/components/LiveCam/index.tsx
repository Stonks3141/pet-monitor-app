import React, { useState, useEffect } from 'react';
import { useCookies } from 'react-cookie';
import axios from 'axios';
import './style.css';

const LiveCam = () => {
  const [stream, setStream] = useState('');
  const [cookies] = useCookies();

  useEffect(() => {
    axios.get('/api/stream', {headers: {session: cookies.id}})
    .then(res => {
      if (res.status == 200) {
        setStream(res.data);
      }
      else {
        alert('An error has occurred: ' + res.statusText + res.data);
        throw new Error(res.statusText + res.data);
      }
    })
    .catch(err => console.error(err));
  }, []);

  return <p>{stream}</p>;
};

export default LiveCam;
