interface RibbonTabProps {
  label: string;
  active?: boolean;
  isFileTab?: boolean;
  onClick: () => void;
}

export default function RibbonTab({
  label,
  active = false,
  isFileTab = false,
  onClick,
}: RibbonTabProps) {
  const className = [
    'ribbon__tab',
    isFileTab && 'ribbon__tab--file',
    active && !isFileTab && 'ribbon__tab--active',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <button className={className} onClick={onClick}>
      {label}
    </button>
  );
}
