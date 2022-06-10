import React from 'react';
import { useCookies } from 'react-cookie';
import { useNavigate } from 'react-router-dom';

const Logout = () => {
  const [_cookies, _setCookie, deleteCookie] = useCookies();
  const navigate = useNavigate();

  const handleClick = () => {
    deleteCookie('token');
    navigate('/lock');
  };

  return (
    <a href='' onClick={handleClick}>
      Log out
    </a>
  );
};

export default Logout;
