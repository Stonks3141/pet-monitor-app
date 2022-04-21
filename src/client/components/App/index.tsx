import React, { useState } from "react";
import './style.css';
import { Navbar, NavButton, LoginMenu, LiveCam, RecentActivity } from '/src/client/components';

const App = () => {
  const [isLoggedIn, setIsLoggedIn] = useState(false);
  const [currentPage, setCurrentPage] = useState(<LiveCam />);
  const [sidebarOpen, setSidebarOpen] = useState(true);

  if(!isLoggedIn) {
    return <LoginMenu onLogin={() => setIsLoggedIn(true)}/>;
  }

  return (
    <>
      <Navbar isOpen={sidebarOpen}>
        <NavButton onClick={() => setCurrentPage(<LiveCam />)}        active={currentPage === <LiveCam />}       >Live Cam</NavButton>
        <NavButton onClick={() => setCurrentPage(<RecentActivity />)} active={currentPage === <RecentActivity />}>Recent Activity</NavButton>
        <NavButton onClick={() => setIsLoggedIn(false)}               active={false}                             >Log Out</NavButton>
      </Navbar>
      {currentPage}
    </>
  );
};

export default App;