import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { ChevronRight, MessageSquare, Eye, Tag, Calendar, User, Hash } from 'lucide-react';
import { topics as topicsApi, comments as commentsApi, viewpoints as vpApi } from '../api/client';
import type { Topic, Comment, Viewpoint } from '../types/api';
import { StatusBadge } from '../components/StatusBadge';
import CommentThread from '../components/CommentThread';
import ViewpointCard from '../components/ViewpointCard';

export default function TopicDetail() {
  const { projectId, topicId } = useParams<{ projectId: string; topicId: string }>();
  const [topic, setTopic] = useState<Topic | null>(null);
  const [commentsList, setComments] = useState<Comment[]>([]);
  const [vpList, setViewpoints] = useState<Viewpoint[]>([]);
  const [loading, setLoading] = useState(true);

  const load = async () => {
    if (!projectId || !topicId) return;
    try {
      const [t, c, v] = await Promise.all([
        topicsApi.get(projectId, topicId),
        commentsApi.list(projectId, topicId),
        vpApi.list(projectId, topicId),
      ]);
      setTopic(t);
      setComments(c);
      setViewpoints(v);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { load(); }, [projectId, topicId]);

  if (loading) return <div className="text-text-muted py-12 text-center">Laden...</div>;
  if (!topic || !projectId || !topicId) return <div className="text-text-muted py-12 text-center">Issue niet gevonden</div>;

  // First viewpoint snapshot as hero image
  const heroSnapshot = vpList.length > 0 ? vpList[0].snapshot_url : null;

  return (
    <div>
      {/* Breadcrumb */}
      <div className="flex items-center gap-1.5 mb-1 text-sm text-text-muted">
        <Link to="/" className="hover:text-amber transition">Projecten</Link>
        <ChevronRight size={14} />
        <Link to={`/projects/${projectId}`} className="hover:text-amber transition">Project</Link>
        <ChevronRight size={14} />
        <span className="text-text font-medium truncate max-w-[200px]">{topic.title}</span>
      </div>

      {/* Hero snapshot */}
      {heroSnapshot && (
        <div className="mb-4 rounded-[--radius-lg] overflow-hidden border border-border bg-concrete">
          <img
            src={heroSnapshot}
            alt={topic.title}
            className="w-full max-h-[400px] object-contain"
          />
        </div>
      )}

      {/* Title + badges */}
      <h1 className="text-2xl mb-1">{topic.title}</h1>
      <div className="flex flex-wrap items-center gap-2 mb-6">
        <StatusBadge value={topic.topic_status} type="status" />
        <StatusBadge value={topic.priority} type="priority" />
        {topic.topic_type && (
          <span className="text-[0.7rem] font-semibold uppercase tracking-wider px-[0.6em] py-[0.2em] rounded-full" style={{ background: 'var(--oaec-warning-soft, rgba(245,158,11,0.15))', color: 'var(--oaec-warning, #F59E0B)' }}>
            {topic.topic_type}
          </span>
        )}
        {topic.labels.map((l) => (
          <span key={l} className="text-xs font-medium px-3 py-1 rounded-full border border-border text-text-muted">
            <Tag size={10} className="inline mr-1" />{l}
          </span>
        ))}
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Left column: description + comments */}
        <div className="lg:col-span-2 space-y-6">
          {topic.description && (
            <div className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-5">
              <h2 className="text-xs font-bold text-text-muted uppercase tracking-wider mb-2">Beschrijving</h2>
              <p className="text-sm whitespace-pre-wrap leading-relaxed text-text-muted">{topic.description}</p>
            </div>
          )}

          <div>
            <h2 className="text-sm font-heading font-bold mb-3 flex items-center gap-1.5">
              <MessageSquare size={16} /> Comments ({commentsList.length})
            </h2>
            <CommentThread
              projectId={projectId}
              topicId={topicId}
              comments={commentsList}
              onUpdate={load}
            />
          </div>
        </div>

        {/* Right column: metadata + viewpoints */}
        <div className="space-y-4">
          {/* All BCF metadata */}
          <div className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-4 space-y-3 text-xs">
            <h3 className="font-bold text-text-muted uppercase tracking-wider text-[0.7rem]">Issue details</h3>
            <DetailRow label="Status" value={topic.topic_status} />
            <DetailRow label="Prioriteit" value={topic.priority} />
            {topic.topic_type && <DetailRow label="Type" value={topic.topic_type} />}
            {topic.stage && <DetailRow label="Fase" value={topic.stage} />}
            {topic.index !== null && (
              <DetailRow label="Index" value={String(topic.index)} icon={<Hash size={12} />} />
            )}
            {topic.assigned_to && (
              <DetailRow label="Toegewezen aan" value={topic.assigned_to} icon={<User size={12} />} />
            )}
            {topic.due_date && (
              <DetailRow
                label="Deadline"
                value={new Date(topic.due_date).toLocaleDateString('nl-NL')}
                icon={<Calendar size={12} />}
              />
            )}
            {topic.labels.length > 0 && (
              <DetailRow label="Labels" value={topic.labels.join(', ')} icon={<Tag size={12} />} />
            )}
          </div>

          {/* Timestamps */}
          <div className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-4 space-y-2 text-xs">
            <h3 className="font-bold text-text-muted uppercase tracking-wider text-[0.7rem]">Tijdstempels</h3>
            <DetailRow label="Aangemaakt" value={new Date(topic.created_at).toLocaleString('nl-NL')} />
            <DetailRow label="Bijgewerkt" value={new Date(topic.updated_at).toLocaleString('nl-NL')} />
            {topic.creation_author && <DetailRow label="Auteur" value={topic.creation_author} />}
          </div>

          {/* Viewpoints */}
          <div>
            <h2 className="text-sm font-heading font-bold mb-3 flex items-center gap-1.5">
              <Eye size={16} /> Viewpoints ({vpList.length})
            </h2>
            {vpList.length === 0 ? (
              <p className="text-xs text-text-subtle">Geen viewpoints</p>
            ) : (
              <div className="space-y-3">
                {vpList.map((vp) => (
                  <ViewpointCard key={vp.guid} viewpoint={vp} projectId={projectId} topicId={topicId} />
                ))}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function DetailRow({ label, value, icon }: { label: string; value: string; icon?: React.ReactNode }) {
  return (
    <div className="flex justify-between items-center">
      <span className="text-text-subtle flex items-center gap-1">
        {icon}
        {label}
      </span>
      <span className="font-medium text-right max-w-[60%] truncate">{value}</span>
    </div>
  );
}
