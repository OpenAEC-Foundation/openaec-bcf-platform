import { useState } from 'react';
import { Send, Trash2 } from 'lucide-react';
import { comments as commentsApi } from '../api/client';
import type { Comment } from '../types/api';

interface Props {
  projectId: string;
  topicId: string;
  comments: Comment[];
  onUpdate: () => void;
}

export default function CommentThread({ projectId, topicId, comments, onUpdate }: Props) {
  const [text, setText] = useState('');
  const [sending, setSending] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!text.trim()) return;
    setSending(true);
    try {
      await commentsApi.create(projectId, topicId, { comment: text.trim() });
      setText('');
      onUpdate();
    } finally {
      setSending(false);
    }
  };

  const handleDelete = async (commentId: string) => {
    if (!confirm('Comment verwijderen?')) return;
    await commentsApi.delete(projectId, topicId, commentId);
    onUpdate();
  };

  return (
    <div className="space-y-3">
      {comments.map((c) => (
        <div key={c.guid} className="bg-white rounded-[--radius-md] border border-border p-4 group">
          <div className="flex items-start justify-between gap-2">
            <p className="text-sm whitespace-pre-wrap flex-1 leading-relaxed">{c.comment}</p>
            <button
              onClick={() => handleDelete(c.guid)}
              className="p-1 text-scaffold-gray/30 hover:text-error transition opacity-0 group-hover:opacity-100 shrink-0"
              title="Verwijderen"
            >
              <Trash2 size={12} />
            </button>
          </div>
          <p className="text-xs text-text-subtle mt-2">
            {new Date(c.created_at).toLocaleString('nl-NL')}
          </p>
        </div>
      ))}

      <form onSubmit={handleSubmit} className="flex gap-2">
        <input
          type="text"
          placeholder="Schrijf een comment..."
          value={text}
          onChange={(e) => setText(e.target.value)}
          className="flex-1 border-[1.5px] border-[#D6D3D1] rounded-[--radius-md] px-4 py-2.5 text-sm focus:outline-none focus:border-amber focus:shadow-[0_0_0_3px_rgba(217,119,6,0.15)]"
        />
        <button
          type="submit"
          disabled={sending || !text.trim()}
          className="bg-amber text-white p-2.5 rounded-[--radius-md] hover:bg-signal-orange transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed"
        >
          <Send size={16} />
        </button>
      </form>
    </div>
  );
}
