import type { ReactNode } from 'react';

interface RibbonGroupProps {
  label: string;
  children: ReactNode;
}

export default function RibbonGroup({ label, children }: RibbonGroupProps) {
  return (
    <div className="ribbon-group">
      <div className="ribbon-group__items">
        {children}
      </div>
      <div className="ribbon-group__label">{label}</div>
    </div>
  );
}
