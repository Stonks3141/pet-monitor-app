import React from 'react';
import { useLocation } from 'react-router-dom';
import { NotFound } from 'components';

const NotFoundRoute = () => {
  const loc = useLocation();
  return <NotFound path={loc.pathname} />;
};

export default NotFoundRoute;
