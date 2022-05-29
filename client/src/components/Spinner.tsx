import React from 'react';
import Icon from '@mdi/react';
import { mdiLoading } from '@mdi/js';

const Spinner = () => (
  <Icon path={mdiLoading} size={1} className="stroke-indigo-500" />
);

export default Spinner;
