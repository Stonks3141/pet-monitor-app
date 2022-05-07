import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCookies } from 'react-cookie';

const Main = () => {
  const navigate = useNavigate();
  const [cookies] = useCookies();

  useEffect(() => {
    if ('password' in cookies) {
      navigate('/camera');
    }
    else {
      navigate('/lock');
    }
  });
};

export default Main;
