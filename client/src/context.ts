import React from 'react';

interface AuthContextInterface {
  auth: boolean;
  setAuth: (_: boolean) => void;
}

const AuthContext = React.createContext<AuthContextInterface | null>(null);

export { AuthContext };
