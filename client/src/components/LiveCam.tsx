import React, { useState, useEffect } from 'react';
import axios from 'axios';

const LiveCam = () => {
  const [stream, setStream] = useState('');

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

  return (
    <div className='flex grow content-center place-content-center place-items-center'>
      <video crossOrigin='anonymous' controls autoPlay width={1280} height={720}>
        <source src='https://nzp-ms05.si.edu/live_edge_panda/smil:panda02_all.smil/playlist.m3u8' type='application/x-mpegURL' />
      </video>
    </div>
  );
};

export default LiveCam;
