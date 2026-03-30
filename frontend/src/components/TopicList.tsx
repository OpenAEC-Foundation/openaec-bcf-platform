import { Link } from 'react-router-dom';
import { Trash2, AlertCircle } from 'lucide-react';
import { topics as topicsApi } from '../api/client';
import type { Topic } from '../types/api';
import { StatusBadge } from './StatusBadge';

interface Props {
  projectId: string;
  topics: Topic[];
  onDelete: () => void;
}

export default function TopicList({ projectId, topics, onDelete }: Props) {
  const handleDelete = async (topicId: string) => {
    if (!confirm('Issue verwijderen?')) return;
    await topicsApi.delete(projectId, topicId);
    onDelete();
  };

  if (topics.length === 0) {
    return (
      <div className="text-center py-12 text-text-muted">
        <AlertCircle size={40} className="mx-auto mb-2 opacity-40" />
        <p className="text-sm">Geen issues. Maak er een aan of importeer een BCF bestand.</p>
      </div>
    );
  }

  return (
    <div className="space-y-1">
      {topics.map((t) => (
        <div
          key={t.guid}
          className="flex items-center gap-3 bg-deep-forge rounded-[--radius-md] border border-border px-4 py-3 hover:shadow-[--shadow-sm] hover:border-border-hover transition group"
        >
          <Link to={`/projects/${projectId}/topics/${t.guid}`} className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-0.5">
              <span className="font-medium text-sm truncate group-hover:text-amber transition">
                {t.title}
              </span>
            </div>
            <div className="flex items-center gap-2">
              <StatusBadge value={t.topic_status} type="status" />
              <StatusBadge value={t.priority} type="priority" />
              {t.topic_type && <span className="text-xs text-text-subtle">{t.topic_type}</span>}
              <span className="text-xs text-text-subtle ml-auto">
                {new Date(t.updated_at).toLocaleDateString('nl-NL')}
              </span>
            </div>
          </Link>
          <button
            onClick={() => handleDelete(t.guid)}
            className="p-1.5 text-text-subtle hover:text-error transition opacity-0 group-hover:opacity-100"
            title="Verwijderen"
          >
            <Trash2 size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}
