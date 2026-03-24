interface RibbonButtonProps {
  icon: string;
  label: string;
  onClick?: () => void;
  active?: boolean;
  disabled?: boolean;
  small?: boolean;
}

export default function RibbonButton({
  icon,
  label,
  onClick,
  active = false,
  disabled = false,
  small = false,
}: RibbonButtonProps) {
  const className = [
    'ribbon-btn',
    small && 'ribbon-btn--small',
    active && 'ribbon-btn--active',
    disabled && 'ribbon-btn--disabled',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <button
      className={className}
      onClick={onClick}
      disabled={disabled}
      title={label}
    >
      <span
        className="ribbon-btn__icon"
        dangerouslySetInnerHTML={{ __html: icon }}
      />
      <span className="ribbon-btn__label">{label}</span>
    </button>
  );
}
