import { Link, Outlet, useLocation } from 'react-router-dom';
import { FolderOpen, LogIn, LogOut, Menu, X } from 'lucide-react';
import { useState } from 'react';
import { useAuth } from '../context/AuthContext';

export default function Layout() {
  const { user, login, logout } = useAuth();
  const location = useLocation();
  const [menuOpen, setMenuOpen] = useState(false);

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="bg-violet text-white shadow-lg">
        <div className="max-w-7xl mx-auto px-4 h-14 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <button
              className="md:hidden p-1"
              onClick={() => setMenuOpen(!menuOpen)}
            >
              {menuOpen ? <X size={20} /> : <Menu size={20} />}
            </button>
            <Link to="/" className="font-bold text-lg tracking-tight flex items-center gap-2">
              <span className="text-verdigris">BCF</span> Platform
            </Link>
            <nav className="hidden md:flex items-center gap-1 ml-6">
              <NavLink to="/" current={location.pathname} label="Projects" icon={<FolderOpen size={16} />} />
            </nav>
          </div>
          <div className="flex items-center gap-3">
            {user ? (
              <div className="flex items-center gap-3">
                <span className="text-sm text-white/80 hidden sm:block">{user.name}</span>
                <button
                  onClick={logout}
                  className="flex items-center gap-1 text-sm text-white/70 hover:text-white transition"
                >
                  <LogOut size={16} />
                </button>
              </div>
            ) : (
              <button
                onClick={login}
                className="flex items-center gap-1.5 text-sm bg-verdigris/20 hover:bg-verdigris/30 px-3 py-1.5 rounded transition"
              >
                <LogIn size={16} /> Login
              </button>
            )}
          </div>
        </div>

        {/* Mobile menu */}
        {menuOpen && (
          <nav className="md:hidden border-t border-white/10 px-4 py-2">
            <NavLink to="/" current={location.pathname} label="Projects" icon={<FolderOpen size={16} />} onClick={() => setMenuOpen(false)} />
          </nav>
        )}
      </header>

      {/* Content */}
      <main className="flex-1 max-w-7xl mx-auto w-full px-4 py-6">
        <Outlet />
      </main>
    </div>
  );
}

function NavLink({ to, current, label, icon, onClick }: {
  to: string; current: string; label: string; icon: React.ReactNode; onClick?: () => void;
}) {
  const active = current === to || (to !== '/' && current.startsWith(to));
  return (
    <Link
      to={to}
      onClick={onClick}
      className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-sm transition ${
        active ? 'bg-white/15 text-white' : 'text-white/70 hover:text-white hover:bg-white/10'
      }`}
    >
      {icon} {label}
    </Link>
  );
}
