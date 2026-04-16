import { useState, useEffect, useCallback, useRef } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useAuth } from '../../context/AuthContext';
import { projects as projectsApi, bcf } from '../../api/client';
import type { Project } from '../../types/api';
import AppBar from './AppBar';
import Ribbon from './Ribbon';
import StatusBar from './StatusBar';
import Backstage from './Backstage';

interface AppShellProps {
  children: React.ReactNode;
}

export default function AppShell({ children }: AppShellProps) {
  const { user, login, logout } = useAuth();
  const navigate = useNavigate();
  const params = useParams<{ projectId?: string }>();
  const [backstageOpen, setBackstageOpen] = useState(false);
  const [projectCount, setProjectCount] = useState<number | undefined>(undefined);
  const [currentProject, setCurrentProject] = useState<Project | undefined>(undefined);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Load project count for status bar + current project
  useEffect(() => {
    projectsApi.list().then((list: Project[]) => {
      setProjectCount(list.length);
      if (params.projectId) {
        const proj = list.find((p) => p.project_id === params.projectId);
        setCurrentProject(proj);
      }
    }).catch(() => {});
  }, [params.projectId]);

  const handleNewProject = useCallback(() => {
    // Navigate to home and let ProjectList handle creation
    navigate('/');
  }, [navigate]);

  const handleImportBcf = useCallback(() => {
    fileInputRef.current?.click();
  }, []);

  const handleFileSelected = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (!file || !params.projectId) return;
      try {
        await bcf.importZip(params.projectId, file);
        // Reload current page
        window.location.reload();
      } catch {
        // Error is logged by API client
      }
      // Reset the input so the same file can be re-selected
      e.target.value = '';
    },
    [params.projectId],
  );

  const handleExportBcf = useCallback(() => {
    if (params.projectId) {
      window.open(bcf.exportUrl(params.projectId), '_blank');
    }
  }, [params.projectId]);

  // Global keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape → close backstage
      if (e.key === 'Escape' && backstageOpen) {
        e.preventDefault();
        setBackstageOpen(false);
        return;
      }
      // Ctrl+I → import BCF
      if (e.ctrlKey && e.key === 'i') {
        e.preventDefault();
        handleImportBcf();
        return;
      }
      // Ctrl+E → export BCF
      if (e.ctrlKey && e.key === 'e') {
        e.preventDefault();
        handleExportBcf();
        return;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [backstageOpen, handleImportBcf, handleExportBcf]);

  return (
    <div className="flex h-screen flex-col" data-theme="light">
      <AppBar user={user} onLogin={login} onLogout={logout} />
      <Ribbon
        onFileTabClick={() => setBackstageOpen(true)}
        onNewProject={handleNewProject}
        onImportBcf={handleImportBcf}
        onExportBcf={handleExportBcf}
      />
      <div
        className="flex-1 overflow-auto"
        style={{ background: 'var(--theme-content-bg)' }}
      >
        <div className="max-w-7xl mx-auto w-full px-4 py-6">
          {children}
        </div>
      </div>
      <StatusBar projectCount={projectCount} />
      <Backstage
        open={backstageOpen}
        onClose={() => setBackstageOpen(false)}
        onNavigate={navigate}
        currentProject={currentProject}
        activeProjectId={params.projectId}
        onNewProject={handleNewProject}
        onSetProject={setCurrentProject}
        onSetActiveProjectId={(id) => navigate(`/projects/${id}`)}
        onReset={() => setCurrentProject(undefined)}
      />
      {/* Hidden file input for BCF import */}
      <input
        ref={fileInputRef}
        type="file"
        accept=".bcfzip,.bcf"
        style={{ display: 'none' }}
        onChange={handleFileSelected}
      />
    </div>
  );
}
