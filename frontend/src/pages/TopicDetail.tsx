import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { ChevronRight, MessageSquare, Eye } from 'lucide-react';
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

  return (
    <div>
      <div className="flex items-center gap-1.5 mb-1 text-sm text-text-muted">
        <Link to="/" className="hover:text-amber transition">Projecten</Link>
        <ChevronRight size={14} />
        <Link to={`/projects/${projectId}`} className="hover:text-amber transition">Project</Link>
        <ChevronRight size={14} />
        <span className="text-text font-medium truncate max-w-[200px]">{topic.title}</span>
      </div>
      <h1 className="text-2xl mb-1">{topic.title}</h1>

      <div className="flex flex-wrap items-center gap-2 mb-6">
        <StatusBadge value={topic.topic_status} type="status" />
        <StatusBadge value={topic.priority} type="priority" />
        {topic.topic_type && (
          <span className="text-[0.7rem] font-semibold uppercase tracking-wider bg-[#FEF3C7] text-[#92400E] px-[0.6em] py-[0.2em] rounded-full">{topic.topic_type}</span>
        )}
        {topic.labels.map((l) => (
          <span key={l} className="text-xs font-medium px-3 py-1 rounded-full border border-border text-text-muted">{l}</span>
        ))}
        {topic.due_date && (
          <span className="text-xs text-text-subtle">
            Deadline: {new Date(topic.due_date).toLocaleDateString('nl-NL')}
          </span>
        )}
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <div className="lg:col-span-2 space-y-6">
          {topic.description && (
            <div className="bg-white rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-5">
              <p className="text-sm whitespace-pre-wrap leading-relaxed">{topic.description}</p>
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

          <div className="mt-6 bg-white rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-4 space-y-2 text-xs">
            <DetailRow label="Aangemaakt" value={new Date(topic.created_at).toLocaleString('nl-NL')} />
            <DetailRow label="Bijgewerkt" value={new Date(topic.updated_at).toLocaleString('nl-NL')} />
            {topic.stage && <DetailRow label="Fase" value={topic.stage} />}
            {topic.index !== null && <DetailRow label="Index" value={String(topic.index)} />}
          </div>
        </div>
      </div>
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex justify-between">
      <span className="text-text-subtle">{label}</span>
      <span className="font-medium">{value}</span>
    </div>
  );
}
