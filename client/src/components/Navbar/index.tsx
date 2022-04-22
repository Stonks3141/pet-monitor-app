import React from 'react';
import './style.css';

interface NavbarProps {
  isOpen?: boolean;
  onClose?: () => void;
  children: JSX.Element[] | JSX.Element;
};

/**
 * Reusable navbar component, mostly for styling purposes
 * @param isOpen optional, defaults to true, renders a menu button if closed
 * @param children array of NavButton elements
 * @returns navbar component
 */
const Navbar = ({isOpen=true, onClose=null, children}: NavbarProps) => {
  if(isOpen) {
    return (
    <div className="navbar">
      {onClose != null && <button onClick={onClose}>Close</button>}
      {children}
    </div>
    );
  }
  return;
};

export default Navbar;