import type { User } from '../../types/api';
import './AppBar.css';

const iconSave = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>`;

const iconSettingsSmall = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>`;

interface AppBarProps {
  user: User | null;
  onLogin: () => void;
  onLogout: () => void;
}

export default function AppBar({ user, onLogin, onLogout }: AppBarProps) {
  return (
    <div className="app-bar">
      <div className="app-bar__left">
        {/* Quick-access buttons */}
        <button
          className="app-bar__quick-btn"
          title="Opslaan (Ctrl+S)"
          dangerouslySetInnerHTML={{ __html: iconSave }}
        />
        <button
          className="app-bar__quick-btn"
          title="Voorkeuren"
          dangerouslySetInnerHTML={{ __html: iconSettingsSmall }}
        />
        <div className="app-bar__quick-separator" />
        {/* Brand */}
        <div className="app-bar__brand">
          <span className="app-bar__brand-open">Open</span>
          <span className="app-bar__brand-aec">AEC</span>
          <span className="app-bar__brand-subtitle">BCF Platform</span>
        </div>
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
