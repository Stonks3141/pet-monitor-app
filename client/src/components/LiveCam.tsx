import React from 'react';

const LiveCam = () => {
  const time = Math.floor(Date.now() / 1000);
  return (
    <div className="flex content-center place-content-center place-items-center grow">
      <video controls autoPlay muted playsInline className="flex max-w-full max-h-full">
        <source src={'stream.mp4?' + time.toString()} type="video/mp4; codecs=&quot;avc1.64002a&quot;" />
        <source src="stream.m3u8" type="applicaton/x-mpegURL" />
      </video>
    </div>
  );
};

export default LiveCam;
