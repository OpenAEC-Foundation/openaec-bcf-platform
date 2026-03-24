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
      {/* Header — deep-forge navbar with amber gradient strip */}
      <header className="bg-deep-forge text-white">
        {/* Amber gradient accent strip */}
        <div className="h-[3px]" style={{ background: 'linear-gradient(90deg, #D97706 0%, #F59E0B 40%, #EA580C 100%)' }} />
        <div className="max-w-7xl mx-auto px-4 h-14 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <button
              className="md:hidden p-1"
              onClick={() => setMenuOpen(!menuOpen)}
            >
              {menuOpen ? <X size={20} /> : <Menu size={20} />}
            </button>
            <Link to="/" className="font-heading font-bold text-lg tracking-tight flex items-center gap-0.5">
              <span className="text-white">Open</span>
              <span className="text-amber">AEC</span>
              <span className="text-scaffold-gray font-body font-normal text-sm ml-2">BCF Platform</span>
            </Link>
            <nav className="hidden md:flex items-center gap-1 ml-6">
              <NavLink to="/" current={location.pathname} label="Projects" icon={<FolderOpen size={16} />} />
            </nav>
          </div>
          <div className="flex items-center gap-3">
            {user ? (
              <div className="flex items-center gap-3">
                <span className="text-sm text-scaffold-gray hidden sm:block">{user.name}</span>
                <button
                  onClick={logout}
                  className="flex items-center gap-1 text-sm text-scaffold-gray hover:text-white transition"
                >
                  <LogOut size={16} />
                </button>
              </div>
            ) : (
              <button
                onClick={login}
                className="flex items-center gap-1.5 text-sm bg-amber text-white px-3 py-1.5 rounded-[--radius-md] font-semibold hover:bg-signal-orange transition-all duration-150"
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
      className={`flex items-center gap-1.5 px-3 py-1.5 rounded-[--radius-sm] text-sm font-medium transition ${
        active ? 'bg-amber text-white' : 'text-scaffold-gray hover:text-white hover:bg-white/10'
      }`}
    >
      {icon} {label}
    </Link>
  );
}
