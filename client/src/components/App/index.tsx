import React, { useState } from "react";
import './style.css';
import { LoginMenu, LiveCam } from 'components';

const App = () => {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  //const [activityOpen, setActivityOpen] = useState(false);

  if(!isLoggedIn) {
    return <LoginMenu onLogin={() => setIsLoggedIn(true)}/>;
  }

  return <LiveCam />;
};

export default App;
