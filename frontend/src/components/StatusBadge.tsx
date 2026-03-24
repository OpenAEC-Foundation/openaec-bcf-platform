const statusColors: Record<string, string> = {
  Open: 'bg-[#DBEAFE] text-[#1E40AF]',
  Active: 'bg-[#FEF3C7] text-[#92400E]',
  Closed: 'bg-[#F4F4F5] text-[#3F3F46]',
  ReOpened: 'bg-[#FEE2E2] text-[#991B1B]',
};

const priorityColors: Record<string, string> = {
  Critical: 'bg-[#FEE2E2] text-[#991B1B]',
  High: 'bg-[#FEF3C7] text-[#92400E]',
  Normal: 'bg-[#F4F4F5] text-[#3F3F46]',
  Low: 'bg-[#DCFCE7] text-[#166534]',
};

export function StatusBadge({ value, type = 'status' }: { value: string; type?: 'status' | 'priority' }) {
  const colors = type === 'priority' ? priorityColors : statusColors;
  const cls = colors[value] ?? 'bg-[#F4F4F5] text-[#3F3F46]';
  return (
    <span className={`inline-flex items-center px-[0.6em] py-[0.2em] rounded-full text-[0.7rem] font-semibold uppercase tracking-wider ${cls}`}>
      {value}
    </span>
  );
}
