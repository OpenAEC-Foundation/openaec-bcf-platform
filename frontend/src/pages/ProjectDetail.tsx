import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { ArrowLeft, Download, Upload, Key } from 'lucide-react';
import { projects, topics as topicsApi, bcf } from '../api/client';
import type { Project, Topic, ImportSummary } from '../types/api';
import TopicList from '../components/TopicList';
import CreateTopic from '../components/CreateTopic';
import ApiKeyManager from '../components/ApiKeyManager';

type Tab = 'topics' | 'import' | 'keys';

export default function ProjectDetail() {
  const { projectId } = useParams<{ projectId: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [topicsList, setTopics] = useState<Topic[]>([]);
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState<Tab>('topics');
  const [importing, setImporting] = useState(false);
  const [importResult, setImportResult] = useState<ImportSummary | null>(null);

  const load = async () => {
    if (!projectId) return;
    try {
      const [p, t] = await Promise.all([
        projects.get(projectId),
        topicsApi.list(projectId),
      ]);
      setProject(p);
      setTopics(t);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { load(); }, [projectId]);

  const handleImport = async (file: File) => {
    if (!projectId) return;
    setImporting(true);
    setImportResult(null);
    try {
      const result = await bcf.importZip(projectId, file);
      setImportResult(result);
      load();
    } finally {
      setImporting(false);
    }
  };

  if (loading) {
    return <div className="text-text-muted py-12 text-center">Laden...</div>;
  }

  if (!project || !projectId) {
    return <div className="text-text-muted py-12 text-center">Project niet gevonden</div>;
  }

  return (
    <div>
      {/* Header */}
      <div className="flex items-center gap-3 mb-1">
        <Link to="/" className="text-text-muted hover:text-text transition">
          <ArrowLeft size={18} />
        </Link>
        <h1 className="text-2xl font-bold">{project.name}</h1>
      </div>
      {project.description && (
        <p className="text-sm text-text-muted mb-4 ml-8">{project.description}</p>
      )}

      {/* Tabs */}
      <div className="flex items-center gap-1 border-b border-surface-dark mb-6 ml-8">
        <TabButton active={tab === 'topics'} onClick={() => setTab('topics')}>
          Issues ({topicsList.length})
        </TabButton>
        <TabButton active={tab === 'import'} onClick={() => setTab('import')}>
          <Upload size={14} /> Import / Export
        </TabButton>
        <TabButton active={tab === 'keys'} onClick={() => setTab('keys')}>
          <Key size={14} /> API Keys
        </TabButton>
      </div>

      {/* Tab content */}
      <div className="ml-8">
        {tab === 'topics' && (
          <div>
            <CreateTopic projectId={projectId} onCreated={load} />
            <TopicList projectId={projectId} topics={topicsList} onDelete={load} />
          </div>
        )}

        {tab === 'import' && (
          <div className="max-w-lg space-y-4">
            {/* Import */}
            <div className="bg-white rounded-lg shadow-sm border border-surface-dark p-5">
              <h3 className="font-semibold text-sm mb-3 flex items-center gap-2">
                <Upload size={16} /> BCF Importeren
              </h3>
              <label className="block border-2 border-dashed border-surface-dark rounded-lg p-8 text-center cursor-pointer hover:border-verdigris/40 transition">
                <input
                  type="file"
                  accept=".bcfzip,.bcf"
                  className="hidden"
                  onChange={(e) => {
                    const f = e.target.files?.[0];
                    if (f) handleImport(f);
                  }}
                  disabled={importing}
                />
                <Upload size={32} className="mx-auto mb-2 text-text-muted/40" />
                <p className="text-sm text-text-muted">
                  {importing ? 'Importeren...' : 'Klik of sleep een .bcfzip bestand'}
                </p>
              </label>
              {importResult && (
                <div className="mt-3 bg-verdigris/10 text-verdigris rounded-lg p-3 text-sm">
                  Geïmporteerd: {importResult.topics_imported} issues, {importResult.comments_imported} comments, {importResult.viewpoints_imported} viewpoints
                </div>
              )}
            </div>

            {/* Export */}
            <div className="bg-white rounded-lg shadow-sm border border-surface-dark p-5">
              <h3 className="font-semibold text-sm mb-3 flex items-center gap-2">
                <Download size={16} /> BCF Exporteren
              </h3>
              <a
                href={bcf.exportUrl(projectId)}
                className="inline-flex items-center gap-1.5 bg-violet text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-violet-light transition"
              >
                <Download size={16} /> Download .bcfzip
              </a>
            </div>
          </div>
        )}

        {tab === 'keys' && <ApiKeyManager projectId={projectId} />}
      </div>
    </div>
  );
}

function TabButton({ active, onClick, children }: {
  active: boolean; onClick: () => void; children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-1.5 px-4 py-2.5 text-sm font-medium border-b-2 transition -mb-px ${
        active
          ? 'border-verdigris text-verdigris'
          : 'border-transparent text-text-muted hover:text-text hover:border-surface-dark'
      }`}
    >
      {children}
    </button>
  );
}
