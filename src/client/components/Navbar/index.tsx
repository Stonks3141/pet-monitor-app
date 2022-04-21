import React from 'react';
import './style.css';

interface NavbarProps {
  isOpen?: boolean;
  children: JSX.Element[] | JSX.Element;
};

/**
 * Reusable navbar component, mostly for styling purposes
 * @param children array of NavButton elements
 * @returns navbar component
 */
const Navbar = ({isOpen=true, children}: NavbarProps) => {
  if(isOpen)
    return <div className="navbar">{children}</div>;
};

export default Navbar;