import { useEffect, useState, useCallback } from 'react';
import { useParams, Link } from 'react-router-dom';
import {
  ChevronRight, Download, Upload, Key, Users,
  BarChart3, CircleDot, CheckCircle2, Clock, AlertTriangle,
  MapPin, Image as ImageIcon,
} from 'lucide-react';
import {
  projects, topics as topicsApi, bcf, stats as statsApi,
  projectImage,
} from '../api/client';
import type { Project, Topic, ImportSummary, ProjectStats } from '../types/api';
import TopicList from '../components/TopicList';
import CreateTopic from '../components/CreateTopic';
import ApiKeyManager from '../components/ApiKeyManager';
import MemberManager from '../components/MemberManager';
import { StatusBadge } from '../components/StatusBadge';
import { viewpoints as viewpointsApi } from '../api/client';

type Tab = 'dashboard' | 'topics' | 'import' | 'keys' | 'members';

export default function ProjectDetail() {
  const { projectId } = useParams<{ projectId: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [topicsList, setTopics] = useState<Topic[]>([]);
  const [dashStats, setStats] = useState<ProjectStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [tab, setTab] = useState<Tab>('dashboard');
  const [importing, setImporting] = useState(false);
  const [importResult, setImportResult] = useState<ImportSummary | null>(null);

  // Filters
  const [filterStatus, setFilterStatus] = useState<string>('');
  const [filterPriority, setFilterPriority] = useState<string>('');

  const load = useCallback(async () => {
    if (!projectId) return;
    try {
      const [p, t, s] = await Promise.all([
        projects.get(projectId),
        topicsApi.list(projectId),
        statsApi.get(projectId),
      ]);
      setProject(p);
      setTopics(t);
      setStats(s);
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { load(); }, [load]);

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

  const handleImageUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !projectId) return;
    try {
      const updated = await projectImage.upload(projectId, file);
      setProject(updated);
    } catch { /* handled by API client */ }
  };

  // Filtered topics
  const filtered = topicsList.filter((t) => {
    if (filterStatus && t.topic_status !== filterStatus) return false;
    if (filterPriority && t.priority !== filterPriority) return false;
    return true;
  });

  if (loading) return <div className="text-text-muted py-12 text-center">Laden...</div>;
  if (!project || !projectId) return <div className="text-text-muted py-12 text-center">Project niet gevonden</div>;

  return (
    <div>
      {/* Breadcrumb + header */}
      <div className="flex items-center gap-1.5 mb-1 text-sm text-text-muted">
        <Link to="/" className="hover:text-amber transition">Projecten</Link>
        <ChevronRight size={14} />
        <span className="text-text font-medium">{project.name}</span>
      </div>
      <div className="flex items-start justify-between mb-1">
        <div>
          <h1 className="text-2xl">{project.name}</h1>
          {project.location && (
            <p className="flex items-center gap-1 text-sm text-text-muted mt-0.5">
              <MapPin size={14} /> {project.location}
            </p>
          )}
          {project.description && (
            <p className="text-sm text-text-muted mt-1">{project.description}</p>
          )}
        </div>
        {/* Image upload */}
        <label className="shrink-0 w-20 h-20 rounded-[--radius-md] bg-concrete border border-border flex items-center justify-center overflow-hidden cursor-pointer hover:border-amber transition">
          <input type="file" accept="image/*" className="hidden" onChange={handleImageUpload} />
          {project.image_url ? (
            <img src={project.image_url} alt="" className="w-full h-full object-cover" />
          ) : (
            <ImageIcon size={24} className="text-text-subtle opacity-40" />
          )}
        </label>
      </div>

      {/* Tabs */}
      <div className="flex items-center gap-1 border-b border-border mb-6 mt-4">
        <TabButton active={tab === 'dashboard'} onClick={() => setTab('dashboard')}>
          <BarChart3 size={14} /> Dashboard
        </TabButton>
        <TabButton active={tab === 'topics'} onClick={() => setTab('topics')}>
          Issues ({topicsList.length})
        </TabButton>
        <TabButton active={tab === 'import'} onClick={() => setTab('import')}>
          <Upload size={14} /> Import / Export
        </TabButton>
        <TabButton active={tab === 'keys'} onClick={() => setTab('keys')}>
          <Key size={14} /> API Keys
        </TabButton>
        <TabButton active={tab === 'members'} onClick={() => setTab('members')}>
          <Users size={14} /> Leden
        </TabButton>
      </div>

      <div>
        {/* === DASHBOARD === */}
        {tab === 'dashboard' && dashStats && (
          <div>
            {/* KPI Cards */}
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4 mb-6">
              <KpiCard label="Totaal" value={dashStats.total} icon={<CircleDot size={20} />} color="text-text" />
              <KpiCard label="Open" value={dashStats.open} icon={<AlertTriangle size={20} />} color="text-info" />
              <KpiCard label="In bewerking" value={dashStats.in_progress} icon={<Clock size={20} />} color="text-amber" />
              <KpiCard label="Gesloten" value={dashStats.closed} icon={<CheckCircle2 size={20} />} color="text-success" />
            </div>

            {/* Priority breakdown */}
            {dashStats.by_priority.length > 0 && (
              <div className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-5 mb-6">
                <h3 className="font-heading font-bold text-sm mb-3">Issues per prioriteit</h3>
                <div className="space-y-2">
                  {dashStats.by_priority.map((bp) => (
                    <div key={bp.priority} className="flex items-center gap-3">
                      <StatusBadge value={bp.priority} type="priority" />
                      <div className="flex-1 h-5 bg-concrete rounded overflow-hidden">
                        <div
                          className="h-full bg-amber rounded transition-all"
                          style={{
                            width: `${dashStats.total > 0 ? (bp.count / dashStats.total) * 100 : 0}%`,
                          }}
                        />
                      </div>
                      <span className="text-sm font-semibold w-8 text-right">{bp.count}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Issue grid with filters */}
            <div className="mb-4">
              <div className="flex items-center gap-3 mb-3">
                <h3 className="font-heading font-bold text-sm">Alle issues</h3>
                <select
                  value={filterStatus}
                  onChange={(e) => setFilterStatus(e.target.value)}
                  className="text-xs border border-border rounded px-2 py-1 bg-concrete text-text"
                >
                  <option value="">Alle statussen</option>
                  <option value="Open">Open</option>
                  <option value="Active">In bewerking</option>
                  <option value="Closed">Gesloten</option>
                </select>
                <select
                  value={filterPriority}
                  onChange={(e) => setFilterPriority(e.target.value)}
                  className="text-xs border border-border rounded px-2 py-1 bg-concrete text-text"
                >
                  <option value="">Alle prioriteiten</option>
                  <option value="Critical">Kritiek</option>
                  <option value="Normal">Normaal</option>
                  <option value="Minor">Laag</option>
                </select>
              </div>

              {filtered.length === 0 ? (
                <p className="text-sm text-text-muted py-4">Geen issues gevonden</p>
              ) : (
                <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                  {filtered.map((t) => (
                    <IssueCard key={t.guid} topic={t} projectId={projectId} />
                  ))}
                </div>
              )}
            </div>
          </div>
        )}

        {/* === TOPICS LIST === */}
        {tab === 'topics' && (
          <div>
            <CreateTopic projectId={projectId} onCreated={load} />
            <TopicList projectId={projectId} topics={topicsList} onDelete={load} />
          </div>
        )}

        {/* === IMPORT/EXPORT === */}
        {tab === 'import' && (
          <div className="max-w-lg space-y-4">
            <div className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-5">
              <h3 className="font-heading font-bold text-sm mb-3 flex items-center gap-2">
                <Upload size={16} /> BCF Importeren
              </h3>
              <label className="block border-2 border-dashed border-border rounded-[--radius-md] p-8 text-center cursor-pointer hover:border-amber/40 transition">
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
                <Upload size={32} className="mx-auto mb-2 text-text-subtle opacity-40" />
                <p className="text-sm text-text-muted">
                  {importing ? 'Importeren...' : 'Klik of sleep een .bcfzip bestand'}
                </p>
              </label>
              {importResult && (
                <div className="mt-3 border-l-4 border-success rounded-[--radius-md] p-3 text-sm" style={{ background: 'var(--oaec-success-soft, rgba(22,163,74,0.15))', color: 'var(--oaec-success, #16A34A)' }}>
                  Geimporteerd: {importResult.topics_imported} issues, {importResult.comments_imported} comments, {importResult.viewpoints_imported} viewpoints
                </div>
              )}
            </div>
            <div className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-5">
              <h3 className="font-heading font-bold text-sm mb-3 flex items-center gap-2">
                <Download size={16} /> BCF Exporteren
              </h3>
              <a
                href={bcf.exportUrl(projectId)}
                className="inline-flex items-center gap-1.5 bg-amber text-deep-forge px-4 py-2.5 rounded-[--radius-md] text-sm font-semibold hover:bg-signal-orange transition-all duration-150"
              >
                <Download size={16} /> Download .bcfzip
              </a>
            </div>
          </div>
        )}

        {/* === API KEYS === */}
        {tab === 'keys' && <ApiKeyManager projectId={projectId} />}

        {/* === MEMBERS === */}
        {tab === 'members' && <MemberManager projectId={projectId} />}
      </div>
    </div>
  );
}

// --- Sub-components ---

function TabButton({ active, onClick, children }: {
  active: boolean; onClick: () => void; children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-1.5 px-4 py-2.5 text-sm font-semibold border-b-2 transition -mb-px ${
        active
          ? 'border-amber text-amber'
          : 'border-transparent text-text-muted hover:text-text hover:border-border'
      }`}
    >
      {children}
    </button>
  );
}

function KpiCard({ label, value, icon, color }: {
  label: string; value: number; icon: React.ReactNode; color: string;
}) {
  return (
    <div className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-4 flex items-center gap-3">
      <div className={`${color} opacity-60`}>{icon}</div>
      <div>
        <div className="text-2xl font-bold">{value}</div>
        <div className="text-xs text-text-muted">{label}</div>
      </div>
    </div>
  );
}

function IssueCard({ topic, projectId }: { topic: Topic; projectId: string }) {
  const [snapshotUrl, setSnapshotUrl] = useState<string | null>(null);

  // Try to load first viewpoint snapshot
  useEffect(() => {
    viewpointsApi
      .list(projectId, topic.guid)
      .then((vps) => {
        if (vps.length > 0 && vps[0].snapshot_url) {
          setSnapshotUrl(vps[0].snapshot_url);
        }
      })
      .catch(() => {});
  }, [projectId, topic.guid]);

  return (
    <Link
      to={`/projects/${projectId}/topics/${topic.guid}`}
      className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border overflow-hidden hover:shadow-[--shadow-md] hover:border-border-hover transition group"
    >
      <div className="aspect-video bg-concrete flex items-center justify-center overflow-hidden">
        {snapshotUrl ? (
          <img src={snapshotUrl} alt="" className="w-full h-full object-cover" />
        ) : (
          <CircleDot size={28} className="text-text-subtle opacity-40" />
        )}
      </div>
      <div className="p-3">
        <p className="text-xs font-semibold group-hover:text-amber transition line-clamp-2">
          {topic.title}
        </p>
        <div className="flex items-center gap-1.5 mt-1.5">
          <StatusBadge value={topic.topic_status} type="status" />
          <StatusBadge value={topic.priority} type="priority" />
        </div>
      </div>
    </Link>
  );
}
