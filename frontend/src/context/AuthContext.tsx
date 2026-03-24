import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { auth, ApiError } from '../api/client';
import type { User } from '../types/api';

interface AuthState {
  user: User | null;
  loading: boolean;
  login: () => void;
  logout: () => void;
}

const AuthContext = createContext<AuthState>({
  user: null,
  loading: true,
  login: () => {},
  logout: () => {},
});

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Check for token in URL (OIDC callback redirect)
    const params = new URLSearchParams(window.location.search);
    const token = params.get('token');
    if (token) {
      localStorage.setItem('bcf_token', token);
      window.history.replaceState({}, '', window.location.pathname);
    }

    // Try to load user from stored token
    const stored = localStorage.getItem('bcf_token');
    if (stored) {
      auth.me()
        .then(setUser)
        .catch((err) => {
          if (err instanceof ApiError && err.status === 401) {
            localStorage.removeItem('bcf_token');
          }
        })
        .finally(() => setLoading(false));
    } else {
      setLoading(false);
    }
  }, []);

  const login = () => {
    window.location.href = auth.loginUrl();
  };

  const logout = () => {
    localStorage.removeItem('bcf_token');
    setUser(null);
  };

  return (
    <AuthContext.Provider value={{ user, loading, login, logout }}>
      {children}
    </AuthContext.Provider>
  );
}

export const useAuth = () => useContext(AuthContext);
