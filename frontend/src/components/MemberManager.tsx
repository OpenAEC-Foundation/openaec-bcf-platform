import { useState, useEffect, useCallback } from 'react';
import { UserPlus, Trash2, Search } from 'lucide-react';
import { members as membersApi, users as usersApi } from '../api/client';
import type { Member, User } from '../types/api';

interface MemberManagerProps {
  projectId: string;
}

const ROLES = ['owner', 'admin', 'member', 'viewer'] as const;
const ROLE_LABELS: Record<string, string> = {
  owner: 'Eigenaar',
  admin: 'Beheerder',
  member: 'Lid',
  viewer: 'Kijker',
};

const inputClass =
  'border-[1.5px] border-[#D6D3D1] rounded-[--radius-md] px-3 py-2 text-sm focus:outline-none focus:border-amber focus:shadow-[0_0_0_3px_rgba(217,119,6,0.15)]';

export default function MemberManager({ projectId }: MemberManagerProps) {
  const [membersList, setMembers] = useState<Member[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);

  // Add member state
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<User[]>([]);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [newRole, setNewRole] = useState('member');

  // Create local user state
  const [showCreateUser, setShowCreateUser] = useState(false);
  const [newEmail, setNewEmail] = useState('');
  const [newName, setNewName] = useState('');
  const [newPassword, setNewPassword] = useState('');

  const loadMembers = useCallback(async () => {
    try {
      const list = await membersApi.list(projectId);
      setMembers(list);
    } catch {
      // handle silently
    } finally {
      setLoading(false);
    }
  }, [projectId]);

  useEffect(() => { loadMembers(); }, [loadMembers]);

  // Search users
  useEffect(() => {
    if (searchQuery.length < 2) {
      setSearchResults([]);
      return;
    }
    const timer = setTimeout(async () => {
      try {
        const results = await usersApi.search(searchQuery);
        // Filter out already-members
        const memberIds = new Set(membersList.map((m) => m.user_id));
        setSearchResults(results.filter((u) => !memberIds.has(u.user_id)));
      } catch {
        setSearchResults([]);
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery, membersList]);

  const handleAddMember = async () => {
    if (!selectedUser) return;
    try {
      await membersApi.add(projectId, { user_id: selectedUser.user_id, role: newRole });
      setSelectedUser(null);
      setSearchQuery('');
      setShowAdd(false);
      loadMembers();
    } catch { /* handled */ }
  };

  const handleRoleChange = async (userId: string, role: string) => {
    try {
      await membersApi.updateRole(projectId, userId, role);
      loadMembers();
    } catch { /* handled */ }
  };

  const handleRemove = async (userId: string, name: string) => {
    if (!confirm(`${name} verwijderen uit dit project?`)) return;
    try {
      await membersApi.remove(projectId, userId);
      loadMembers();
    } catch { /* handled */ }
  };

  const handleCreateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newEmail.trim() || !newName.trim() || newPassword.length < 8) return;
    try {
      const user = await usersApi.create({
        email: newEmail.trim(),
        name: newName.trim(),
        password: newPassword,
      });
      // Auto-select the new user
      setSelectedUser(user);
      setShowCreateUser(false);
      setNewEmail('');
      setNewName('');
      setNewPassword('');
    } catch { /* handled */ }
  };

  if (loading) return <div className="text-text-muted py-8 text-center">Laden...</div>;

  return (
    <div className="max-w-xl">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-heading font-bold text-sm">Projectleden ({membersList.length})</h3>
        <button
          onClick={() => setShowAdd(!showAdd)}
          className="flex items-center gap-1.5 bg-amber text-white px-3 py-2 rounded-[--radius-md] text-xs font-semibold hover:bg-signal-orange transition"
        >
          <UserPlus size={14} /> Lid toevoegen
        </button>
      </div>

      {/* Add member form */}
      {showAdd && (
        <div className="bg-white rounded-[--radius-lg] shadow-[--shadow-sm] border border-border p-4 mb-4 space-y-3">
          {/* User search */}
          {!selectedUser ? (
            <div>
              <div className="relative">
                <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted" />
                <input
                  type="text"
                  placeholder="Zoek gebruiker op naam of email..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className={`${inputClass} w-full pl-8`}
                  autoFocus
                />
              </div>
              {searchResults.length > 0 && (
                <div className="mt-1 border border-border rounded-[--radius-md] max-h-40 overflow-y-auto">
                  {searchResults.map((u) => (
                    <button
                      key={u.user_id}
                      onClick={() => setSelectedUser(u)}
                      className="w-full text-left px-3 py-2 text-sm hover:bg-[#F5F5F4] transition flex justify-between"
                    >
                      <span className="font-medium">{u.name}</span>
                      <span className="text-text-muted">{u.email}</span>
                    </button>
                  ))}
                </div>
              )}
              <button
                onClick={() => setShowCreateUser(!showCreateUser)}
                className="text-xs text-amber hover:underline mt-2"
              >
                Nieuwe gebruiker aanmaken
              </button>
            </div>
          ) : (
            <div className="flex items-center justify-between bg-[#F5F5F4] rounded px-3 py-2">
              <div>
                <span className="text-sm font-medium">{selectedUser.name}</span>
                <span className="text-xs text-text-muted ml-2">{selectedUser.email}</span>
              </div>
              <button onClick={() => setSelectedUser(null)} className="text-xs text-text-muted hover:text-text">
                wijzig
              </button>
            </div>
          )}

          {/* Create user inline form */}
          {showCreateUser && !selectedUser && (
            <form onSubmit={handleCreateUser} className="border border-border rounded-[--radius-md] p-3 space-y-2">
              <input type="text" placeholder="Naam" value={newName} onChange={(e) => setNewName(e.target.value)} className={`${inputClass} w-full`} />
              <input type="email" placeholder="Email" value={newEmail} onChange={(e) => setNewEmail(e.target.value)} className={`${inputClass} w-full`} />
              <input type="password" placeholder="Wachtwoord (min. 8 tekens)" value={newPassword} onChange={(e) => setNewPassword(e.target.value)} className={`${inputClass} w-full`} />
              <button type="submit" className="bg-amber text-white px-4 py-2 rounded-[--radius-md] text-xs font-semibold hover:bg-signal-orange transition w-full">
                Gebruiker aanmaken
              </button>
            </form>
          )}

          {/* Role + submit */}
          {selectedUser && (
            <div className="flex items-center gap-2">
              <select value={newRole} onChange={(e) => setNewRole(e.target.value)} className={`${inputClass} flex-1`}>
                {ROLES.map((r) => (
                  <option key={r} value={r}>{ROLE_LABELS[r]}</option>
                ))}
              </select>
              <button
                onClick={handleAddMember}
                className="bg-amber text-white px-4 py-2 rounded-[--radius-md] text-sm font-semibold hover:bg-signal-orange transition"
              >
                Toevoegen
              </button>
            </div>
          )}
        </div>
      )}

      {/* Members list */}
      {membersList.length === 0 ? (
        <p className="text-sm text-text-muted py-4">Geen leden toegevoegd aan dit project.</p>
      ) : (
        <div className="bg-white rounded-[--radius-lg] shadow-[--shadow-sm] border border-border overflow-hidden">
          {membersList.map((m) => (
            <div key={m.user_id} className="flex items-center justify-between px-4 py-3 border-b border-border last:border-b-0 hover:bg-[#FAFAF9] transition group">
              <div>
                <span className="text-sm font-medium">{m.name}</span>
                <span className="text-xs text-text-muted ml-2">{m.email}</span>
              </div>
              <div className="flex items-center gap-2">
                <select
                  value={m.role}
                  onChange={(e) => handleRoleChange(m.user_id, e.target.value)}
                  className="text-xs border border-border rounded px-2 py-1 bg-transparent"
                >
                  {ROLES.map((r) => (
                    <option key={r} value={r}>{ROLE_LABELS[r]}</option>
                  ))}
                </select>
                <button
                  onClick={() => handleRemove(m.user_id, m.name)}
                  className="text-text-muted hover:text-red-600 opacity-0 group-hover:opacity-100 transition"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
