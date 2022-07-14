import React, { useEffect, useState } from 'react';
import { Spinner } from 'components';

const LiveCam = () => {
  const [stream, setStream] = useState(null);

  useEffect(() => {
    fetch('/stream.mp4')
      .then((res) => {
        if (res.ok) {
          setStream(res.text());
        }
      });
  }, []);

  return (
    <div className="flex grow content-center place-content-center place-items-center">
      {stream === null ? <Spinner /> : stream}
    </div>
  );
};

export default LiveCam;
