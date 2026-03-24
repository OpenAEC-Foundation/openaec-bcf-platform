const statusColors: Record<string, string> = {
  Open: 'bg-verdigris/15 text-verdigris',
  Active: 'bg-yellow/20 text-yellow-800',
  Closed: 'bg-surface-dark text-text-muted',
  ReOpened: 'bg-peach/15 text-peach',
};

const priorityColors: Record<string, string> = {
  Critical: 'bg-peach/15 text-peach',
  High: 'bg-magenta/15 text-magenta',
  Normal: 'bg-surface-dark text-text-muted',
  Low: 'bg-verdigris/15 text-verdigris',
};

export function StatusBadge({ value, type = 'status' }: { value: string; type?: 'status' | 'priority' }) {
  const colors = type === 'priority' ? priorityColors : statusColors;
  const cls = colors[value] ?? 'bg-surface-dark text-text-muted';
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${cls}`}>
      {value}
    </span>
  );
}
