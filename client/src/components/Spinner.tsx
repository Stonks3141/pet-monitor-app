import React from 'react';

const Spinner = () => (
  <svg width='48' height='48' viewBox='0 0 48 48'>
    <clipPath id='spinnerClip' clipPathUnits='objectBoundingBox'>
      <rect width='.6' height='.6' x='-.1' y='-.1' />
    </clipPath>
    <circle cx='24' cy='24' r='20' fill='transparent' className='stroke-indigo-500' stroke-width='5' clip-path='url(#spinnerClip)'>
      <animateTransform attributeName='transform' type='rotate' from='0 24 24' to='360 24 24' dur='1s' repeatCount='indefinite' />
    </circle>
  </svg>
);

export default Spinner;
