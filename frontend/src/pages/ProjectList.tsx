import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { FolderOpen, Plus } from 'lucide-react';
import { projects } from '../api/client';
import type { Project } from '../types/api';

export default function ProjectList() {
  const [items, setItems] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [creating, setCreating] = useState(false);

  const load = () => {
    projects.list().then(setItems).finally(() => setLoading(false));
  };

  useEffect(load, []);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setCreating(true);
    try {
      await projects.create({ name: name.trim(), description: description.trim() || undefined });
      setName('');
      setDescription('');
      setShowCreate(false);
      load();
    } finally {
      setCreating(false);
    }
  };

  if (loading) {
    return <div className="text-text-muted py-12 text-center">Laden...</div>;
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Projecten</h1>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="flex items-center gap-1.5 bg-verdigris text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-verdigris-light transition"
        >
          <Plus size={16} /> Nieuw project
        </button>
      </div>

      {/* Create form */}
      {showCreate && (
        <form onSubmit={handleCreate} className="bg-white rounded-lg shadow-sm border border-surface-dark p-4 mb-6">
          <div className="grid gap-3">
            <input
              type="text"
              placeholder="Projectnaam"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="border border-surface-dark rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-verdigris/40"
              autoFocus
            />
            <input
              type="text"
              placeholder="Beschrijving (optioneel)"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="border border-surface-dark rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-verdigris/40"
            />
            <div className="flex gap-2 justify-end">
              <button type="button" onClick={() => setShowCreate(false)} className="px-3 py-1.5 text-sm text-text-muted hover:text-text transition">
                Annuleren
              </button>
              <button type="submit" disabled={creating || !name.trim()} className="bg-verdigris text-white px-4 py-1.5 rounded-lg text-sm font-medium hover:bg-verdigris-light transition disabled:opacity-50">
                {creating ? 'Aanmaken...' : 'Aanmaken'}
              </button>
            </div>
          </div>
        </form>
      )}

      {/* Project cards */}
      {items.length === 0 ? (
        <div className="text-center py-16 text-text-muted">
          <FolderOpen size={48} className="mx-auto mb-3 opacity-40" />
          <p className="text-lg mb-1">Geen projecten</p>
          <p className="text-sm">Maak een nieuw project aan of importeer een BCF bestand.</p>
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {items.map((p) => (
            <Link
              key={p.project_id}
              to={`/projects/${p.project_id}`}
              className="bg-white rounded-lg shadow-sm border border-surface-dark p-4 hover:shadow-md hover:border-verdigris/30 transition group"
            >
              <div className="flex items-start justify-between">
                <div>
                  <h3 className="font-semibold text-sm group-hover:text-verdigris transition">{p.name}</h3>
                  {p.description && (
                    <p className="text-xs text-text-muted mt-1 line-clamp-2">{p.description}</p>
                  )}
                </div>
                <FolderOpen size={18} className="text-text-muted/40 group-hover:text-verdigris/60 transition shrink-0" />
              </div>
              <p className="text-xs text-text-muted mt-3">
                {new Date(p.updated_at).toLocaleDateString('nl-NL')}
              </p>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
