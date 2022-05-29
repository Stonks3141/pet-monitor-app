import React, { useState } from 'react';

const LiveCam = () => {
  const [stream, setStream] = useState('');

  return (
    <div className="flex grow content-center place-content-center place-items-center">
      <video
        crossOrigin="anonymous"
        controls
        autoPlay
        width={1280}
        height={720}
      >
        <source
          src="https://nzp-ms05.si.edu/live_edge_panda/smil:panda02_all.smil/playlist.m3u8"
          type="application/x-mpegURL"
        />
      </video>
    </div>
  );
};

export default LiveCam;
