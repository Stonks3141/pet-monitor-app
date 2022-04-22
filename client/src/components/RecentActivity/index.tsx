import React from 'react';
import './style.css';
import { get } from 'hooks';

const RecentActivity = (): JSX.Element => {
  return <p>{get('recent')}</p>;
};

export default RecentActivity;