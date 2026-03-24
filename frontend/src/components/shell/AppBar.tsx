import type { User } from '../../types/api';
import './AppBar.css';

interface AppBarProps {
  user: User | null;
  onLogin: () => void;
  onLogout: () => void;
}

export default function AppBar({ user, onLogin, onLogout }: AppBarProps) {
  return (
    <div className="app-bar">
      <div className="app-bar__brand">
        <span className="app-bar__brand-open">Open</span>
        <span className="app-bar__brand-aec">AEC</span>
        <span className="app-bar__brand-subtitle">BCF Platform</span>
      </div>
      <div className="app-bar__right">
        {user ? (
          <>
            <span className="app-bar__username">{user.name}</span>
            <button
              className="app-bar__btn app-bar__btn--logout"
              onClick={onLogout}
            >
              Uitloggen
            </button>
          </>
        ) : (
          <button
            className="app-bar__btn app-bar__btn--login"
            onClick={onLogin}
          >
            Inloggen
          </button>
        )}
      </div>
    </div>
  );
}
