import { useState, useEffect, useCallback } from 'react';
import {
  cloudStatus,
  cloudListProjects,
  cloudListFiles,
  cloudSaveBcf,
  cloudDownloadFile,
  cloudDeleteFile,
} from '../../api/cloudApi';
import type { CloudProject, CloudFile, CloudStatus } from '../../types/api';
import { iconCloudUpload, iconCloudDownload } from './icons';

interface CloudPanelProps {
  projectId?: string;
  projectName?: string;
  onImportFromCloud?: (file: File) => void;
  onClose: () => void;
}

type Mode = 'idle' | 'save' | 'open';

export default function CloudPanel({
  projectId,
  projectName,
  onImportFromCloud,
  onClose,
}: CloudPanelProps) {
  const [status, setStatus] = useState<CloudStatus | null>(null);
  const [projects, setProjects] = useState<CloudProject[]>([]);
  const [files, setFiles] = useState<CloudFile[]>([]);
  const [selectedProject, setSelectedProject] = useState<string | null>(null);
  const [mode, setMode] = useState<Mode>('idle');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [confirmOverwrite, setConfirmOverwrite] = useState<string | null>(null);

  // Load cloud status on mount
  useEffect(() => {
    cloudStatus()
      .then(setStatus)
      .catch(() => setStatus({ enabled: false, connected: false }));
  }, []);

  // Load projects when cloud is connected
  useEffect(() => {
    if (status?.connected) {
      cloudListProjects()
        .then(setProjects)
        .catch(() => setError('Kan projecten niet laden'));
    }
  }, [status?.connected]);

  // Load files when a project is selected
  useEffect(() => {
    if (selectedProject) {
      setFiles([]);
      cloudListFiles(selectedProject)
        .then(setFiles)
        .catch(() => {});
    }
  }, [selectedProject]);

  const handleSave = useCallback(async () => {
    if (!selectedProject || !projectId) return;

    // Check if file already exists
    const filename = `${(projectName || 'project').replace(/ /g, '_').toLowerCase()}.bcfzip`;
    const exists = files.some((f) => f.name === filename);

    if (exists && !confirmOverwrite) {
      setConfirmOverwrite(filename);
      return;
    }

    setLoading(true);
    setError(null);
    setConfirmOverwrite(null);

    try {
      const result = await cloudSaveBcf(selectedProject, projectId);
      setSuccess(`Opgeslagen als ${result.filename}`);
      // Refresh file list
      const updated = await cloudListFiles(selectedProject);
      setFiles(updated);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Opslaan mislukt');
    } finally {
      setLoading(false);
    }
  }, [selectedProject, projectId, projectName, files, confirmOverwrite]);

  const handleOpen = useCallback(
    async (filename: string) => {
      if (!selectedProject) return;
      setLoading(true);
      setError(null);

      try {
        const blob = await cloudDownloadFile(selectedProject, filename);
        const file = new File([blob], filename, { type: 'application/zip' });
        onImportFromCloud?.(file);
        onClose();
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Downloaden mislukt');
      } finally {
        setLoading(false);
      }
    },
    [selectedProject, onImportFromCloud, onClose],
  );

  const handleDelete = useCallback(
    async (filename: string) => {
      if (!selectedProject || !confirm(`${filename} verwijderen?`)) return;
      setLoading(true);
      try {
        await cloudDeleteFile(selectedProject, filename);
        setFiles((prev) => prev.filter((f) => f.name !== filename));
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Verwijderen mislukt');
      } finally {
        setLoading(false);
      }
    },
    [selectedProject],
  );

  // Not configured
  if (status && !status.enabled) {
    return (
      <div className="cloud-panel">
        <h2 className="cloud-panel__title">Cloud opslag</h2>
        <p className="cloud-panel__muted">
          Nextcloud cloud opslag is niet geconfigureerd. Stel de NEXTCLOUD_URL,
          NEXTCLOUD_SERVICE_USER en NEXTCLOUD_SERVICE_PASS omgevingsvariabelen in.
        </p>
      </div>
    );
  }

  // Not connected
  if (status && !status.connected) {
    return (
      <div className="cloud-panel">
        <h2 className="cloud-panel__title">Cloud opslag</h2>
        <p className="cloud-panel__error">
          Nextcloud is niet bereikbaar. Controleer de verbinding.
        </p>
      </div>
    );
  }

  return (
    <div className="cloud-panel">
      <h2 className="cloud-panel__title">Cloud opslag</h2>

      {/* Mode selector */}
      <div className="cloud-panel__modes">
        <button
          className={`cloud-panel__mode-btn ${mode === 'save' ? 'cloud-panel__mode-btn--active' : ''}`}
          onClick={() => { setMode('save'); setSuccess(null); setError(null); }}
          disabled={!projectId}
          title={!projectId ? 'Open eerst een project' : undefined}
        >
          <span className="cloud-panel__mode-icon" dangerouslySetInnerHTML={{ __html: iconCloudUpload }} />
          Opslaan naar cloud
        </button>
        <button
          className={`cloud-panel__mode-btn ${mode === 'open' ? 'cloud-panel__mode-btn--active' : ''}`}
          onClick={() => { setMode('open'); setSuccess(null); setError(null); }}
          disabled={!projectId}
          title={!projectId ? 'Open eerst een project' : undefined}
        >
          <span className="cloud-panel__mode-icon" dangerouslySetInnerHTML={{ __html: iconCloudDownload }} />
          Openen van cloud
        </button>
      </div>

      {mode !== 'idle' && (
        <>
          {/* Project selector */}
          <div className="cloud-panel__section">
            <label className="cloud-panel__label">Nextcloud project</label>
            <select
              className="cloud-panel__select"
              value={selectedProject || ''}
              onChange={(e) => setSelectedProject(e.target.value || null)}
            >
              <option value="">Kies een project...</option>
              {projects.map((p) => (
                <option key={p.name} value={p.name}>
                  {p.name}
                </option>
              ))}
            </select>
          </div>

          {/* File list */}
          {selectedProject && (
            <div className="cloud-panel__section">
              <label className="cloud-panel__label">
                Bestanden in {selectedProject}/issues/
              </label>
              {files.length === 0 ? (
                <p className="cloud-panel__muted">Geen bestanden gevonden</p>
              ) : (
                <div className="cloud-panel__file-list">
                  {files.map((f) => (
                    <div key={f.name} className="cloud-panel__file">
                      <div className="cloud-panel__file-info">
                        <span className="cloud-panel__file-name">{f.name}</span>
                        <span className="cloud-panel__file-meta">
                          {formatSize(f.size)}
                          {f.last_modified && ` \u00B7 ${f.last_modified}`}
                        </span>
                      </div>
                      <div className="cloud-panel__file-actions">
                        {mode === 'open' && (
                          <button
                            className="cloud-panel__btn cloud-panel__btn--primary"
                            onClick={() => handleOpen(f.name)}
                            disabled={loading}
                          >
                            Importeren
                          </button>
                        )}
                        <button
                          className="cloud-panel__btn cloud-panel__btn--danger"
                          onClick={() => handleDelete(f.name)}
                          disabled={loading}
                        >
                          Verwijderen
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Save button */}
          {mode === 'save' && selectedProject && (
            <div className="cloud-panel__section">
              {confirmOverwrite && (
                <p className="cloud-panel__warning">
                  {confirmOverwrite} bestaat al. Klik nogmaals om te overschrijven.
                </p>
              )}
              <button
                className="cloud-panel__btn cloud-panel__btn--primary cloud-panel__btn--lg"
                onClick={handleSave}
                disabled={loading}
              >
                {loading ? 'Bezig met opslaan...' : confirmOverwrite ? 'Overschrijven' : 'BCF opslaan naar Nextcloud'}
              </button>
            </div>
          )}
        </>
      )}

      {/* Feedback */}
      {error && <p className="cloud-panel__error">{error}</p>}
      {success && <p className="cloud-panel__success">{success}</p>}
    </div>
  );
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
