import React, { useEffect, useState } from 'react';

const LiveCam = () => {
  const [stream, setStream] = useState('loading...');

  useEffect(() => {
    fetch('/api/stream')
      .then(res => res.text())
      .then(data => {
        setStream(data);
      });
  }, []);

  return (
    <div className='flex grow content-center place-content-center place-items-center'>
      {stream}
    </div>
  );
};

export default LiveCam;
