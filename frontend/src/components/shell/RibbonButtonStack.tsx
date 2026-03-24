import type { ReactNode } from 'react';

interface RibbonButtonStackProps {
  children: ReactNode;
}

export default function RibbonButtonStack({ children }: RibbonButtonStackProps) {
  return <div className="ribbon-btn-stack">{children}</div>;
}
