import { useEffect, useState } from 'react';
import { Key, Plus, Trash2, Copy, Check } from 'lucide-react';
import { apiKeys } from '../api/client';
import type { ApiKey } from '../types/api';

interface Props {
  projectId: string;
}

export default function ApiKeyManager({ projectId }: Props) {
  const [keys, setKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [name, setName] = useState('');
  const [creating, setCreating] = useState(false);
  const [newKey, setNewKey] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const load = () => {
    apiKeys.list(projectId).then(setKeys).finally(() => setLoading(false));
  };

  useEffect(load, [projectId]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setCreating(true);
    try {
      const result = await apiKeys.create(projectId, { name: name.trim() });
      setNewKey(result.key);
      setName('');
      setShowCreate(false);
      load();
    } finally {
      setCreating(false);
    }
  };

  const handleDelete = async (keyId: string) => {
    if (!confirm('API key intrekken? Dit kan niet ongedaan worden gemaakt.')) return;
    await apiKeys.delete(projectId, keyId);
    load();
  };

  const handleCopy = () => {
    if (newKey) {
      navigator.clipboard.writeText(newKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="max-w-lg space-y-4">
      {/* New key revealed */}
      {newKey && (
        <div className="bg-yellow/10 border border-yellow rounded-lg p-4">
          <p className="text-sm font-medium mb-2">API Key aangemaakt — kopieer deze nu!</p>
          <p className="text-xs text-text-muted mb-2">Deze key wordt maar één keer getoond.</p>
          <div className="flex items-center gap-2">
            <code className="flex-1 text-xs bg-white border border-surface-dark rounded px-2 py-1.5 break-all">
              {newKey}
            </code>
            <button onClick={handleCopy} className="p-1.5 text-text-muted hover:text-verdigris transition" title="Kopiëren">
              {copied ? <Check size={16} className="text-verdigris" /> : <Copy size={16} />}
            </button>
          </div>
          <button onClick={() => setNewKey(null)} className="text-xs text-text-muted hover:text-text mt-2">
            Sluiten
          </button>
        </div>
      )}

      {/* Create form */}
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold flex items-center gap-1.5">
          <Key size={16} /> API Keys
        </h3>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="flex items-center gap-1 text-sm text-verdigris hover:text-verdigris-light transition"
        >
          <Plus size={14} /> Nieuwe key
        </button>
      </div>

      {showCreate && (
        <form onSubmit={handleCreate} className="bg-white rounded-lg shadow-sm border border-surface-dark p-4">
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="Key naam (bijv. Validator, CI/CD)"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="flex-1 border border-surface-dark rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-verdigris/40"
              autoFocus
            />
            <button type="submit" disabled={creating || !name.trim()} className="bg-verdigris text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-verdigris-light transition disabled:opacity-50">
              {creating ? '...' : 'Aanmaken'}
            </button>
          </div>
        </form>
      )}

      {/* Key list */}
      {loading ? (
        <p className="text-sm text-text-muted">Laden...</p>
      ) : keys.length === 0 ? (
        <p className="text-sm text-text-muted">Geen API keys. Maak er een aan voor service-to-service authenticatie.</p>
      ) : (
        <div className="space-y-2">
          {keys.map((k) => (
            <div key={k.id} className="bg-white rounded-lg border border-surface-dark px-4 py-3 flex items-center justify-between group">
              <div>
                <p className="text-sm font-medium">{k.name}</p>
                <p className="text-xs text-text-muted">
                  <code>{k.prefix}...</code>
                  {' · '}
                  {new Date(k.created_at).toLocaleDateString('nl-NL')}
                  {k.expires_at && (
                    <> · Verloopt {new Date(k.expires_at).toLocaleDateString('nl-NL')}</>
                  )}
                </p>
              </div>
              <button
                onClick={() => handleDelete(k.id)}
                className="p-1.5 text-text-muted/30 hover:text-peach transition opacity-0 group-hover:opacity-100"
                title="Intrekken"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
