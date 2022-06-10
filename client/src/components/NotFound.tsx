import React from 'react';

interface NotFoundProps {
  path: string;
}

const NotFound = (props: NotFoundProps) => (
  <p>{"couldn't find " + props.path}</p>
);

export default NotFound;
