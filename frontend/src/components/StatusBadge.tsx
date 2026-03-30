interface BadgeStyle {
  background: string;
  color: string;
}

const statusStyles: Record<string, BadgeStyle> = {
  Open: { background: 'rgba(96, 165, 250, 0.15)', color: '#60A5FA' },
  Active: { background: 'rgba(217, 119, 6, 0.15)', color: '#D97706' },
  Closed: { background: 'rgba(250, 250, 249, 0.08)', color: 'rgba(250, 250, 249, 0.5)' },
  ReOpened: { background: 'rgba(220, 38, 38, 0.15)', color: '#f87171' },
};

const priorityStyles: Record<string, BadgeStyle> = {
  Critical: { background: 'rgba(220, 38, 38, 0.15)', color: '#f87171' },
  High: { background: 'rgba(217, 119, 6, 0.15)', color: '#D97706' },
  Normal: { background: 'rgba(250, 250, 249, 0.08)', color: 'rgba(250, 250, 249, 0.5)' },
  Low: { background: 'rgba(22, 163, 74, 0.15)', color: '#16A34A' },
};

const defaultStyle: BadgeStyle = {
  background: 'rgba(250, 250, 249, 0.08)',
  color: 'rgba(250, 250, 249, 0.5)',
};

export function StatusBadge({ value, type = 'status' }: { value: string; type?: 'status' | 'priority' }) {
  const styles = type === 'priority' ? priorityStyles : statusStyles;
  const style = styles[value] ?? defaultStyle;
  return (
    <span
      className="inline-flex items-center px-[0.6em] py-[0.2em] rounded-full text-[0.7rem] font-semibold uppercase tracking-wider"
      style={{ background: style.background, color: style.color }}
    >
      {value}
    </span>
  );
}
