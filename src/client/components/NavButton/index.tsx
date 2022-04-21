import React from 'react';
import './style.css';

interface NavButtonProps {
  onClick: () => void;
  active?: boolean;
  children: string;
};

/**
 * NavButton component, should be inside a NavBar component
 * @param onClick callback handler for onClick event
 * @param active optional, whether the component corresponds to the current page, default false
 * @param children must be a string
 * @returns button with text and onclick handler
 */
const NavButton = ({onClick, active=false, children}: NavButtonProps) => {
  return <button onClick={onClick} className={'navbutton' + (active ? ' active' : '')}>{children}</button>;
};

export default NavButton;
