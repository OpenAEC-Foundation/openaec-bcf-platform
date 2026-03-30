import { useState } from 'react';
import { Plus, ChevronDown, ChevronUp } from 'lucide-react';
import { topics } from '../api/client';

interface Props {
  projectId: string;
  onCreated: () => void;
}

const inputClass = "w-full border border-border rounded-[--radius-md] px-4 py-3 text-sm bg-concrete text-text placeholder:text-text-subtle focus:outline-none focus:border-amber focus:shadow-[0_0_0_3px_rgba(217,119,6,0.15)]";
const selectClass = "w-full border border-border rounded-[--radius-md] px-4 py-2.5 text-sm bg-concrete text-text focus:outline-none focus:border-amber focus:shadow-[0_0_0_3px_rgba(217,119,6,0.15)]";

export default function CreateTopic({ projectId, onCreated }: Props) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [topicType, setTopicType] = useState('');
  const [status, setStatus] = useState('Open');
  const [priority, setPriority] = useState('Normal');
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [creating, setCreating] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    setCreating(true);
    try {
      await topics.create(projectId, {
        title: title.trim(),
        description: description.trim() || undefined,
        topic_type: topicType || undefined,
        topic_status: status,
        priority,
      });
      setTitle('');
      setDescription('');
      setTopicType('');
      setStatus('Open');
      setPriority('Normal');
      setOpen(false);
      setShowAdvanced(false);
      onCreated();
    } finally {
      setCreating(false);
    }
  };

  if (!open) {
    return (
      <button
        onClick={() => setOpen(true)}
        className="flex items-center gap-1.5 bg-amber text-deep-forge px-4 py-2.5 rounded-[--radius-md] text-sm font-semibold hover:bg-signal-orange transition-all duration-150 mb-4"
      >
        <Plus size={16} /> Nieuw issue
      </button>
    );
  }

  return (
    <form onSubmit={handleSubmit} className="bg-deep-forge rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-5 mb-4">
      <div className="grid gap-3">
        <input type="text" placeholder="Issue titel" value={title} onChange={(e) => setTitle(e.target.value)} className={inputClass} autoFocus />
        <textarea placeholder="Beschrijving (optioneel)" value={description} onChange={(e) => setDescription(e.target.value)} rows={3} className={inputClass + " resize-y"} />

        <button type="button" onClick={() => setShowAdvanced(!showAdvanced)} className="flex items-center gap-1 text-xs text-text-muted hover:text-text transition w-fit">
          {showAdvanced ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
          {showAdvanced ? 'Minder opties' : 'Meer opties'}
        </button>

        {showAdvanced && (
          <div className="grid gap-3 sm:grid-cols-3">
            <div>
              <label className="block text-xs font-medium text-text-muted mb-1">Type</label>
              <input type="text" value={topicType} onChange={(e) => setTopicType(e.target.value)} placeholder="bijv. Clash, Request" className={inputClass} />
            </div>
            <div>
              <label className="block text-xs font-medium text-text-muted mb-1">Status</label>
              <select value={status} onChange={(e) => setStatus(e.target.value)} className={selectClass}>
                <option>Open</option>
                <option>Active</option>
                <option>Closed</option>
                <option>ReOpened</option>
              </select>
            </div>
            <div>
              <label className="block text-xs font-medium text-text-muted mb-1">Prioriteit</label>
              <select value={priority} onChange={(e) => setPriority(e.target.value)} className={selectClass}>
                <option>Critical</option>
                <option>High</option>
                <option>Normal</option>
                <option>Low</option>
              </select>
            </div>
          </div>
        )}

        <div className="flex gap-2 justify-end">
          <button type="button" onClick={() => setOpen(false)} className="px-4 py-2.5 text-sm text-text-muted hover:text-text transition">
            Annuleren
          </button>
          <button type="submit" disabled={creating || !title.trim()} className="bg-amber text-deep-forge px-6 py-2.5 rounded-[--radius-md] text-sm font-semibold hover:bg-signal-orange transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed">
            {creating ? 'Aanmaken...' : 'Aanmaken'}
          </button>
        </div>
      </div>
    </form>
  );
}
