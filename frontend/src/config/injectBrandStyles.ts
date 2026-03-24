import { brand } from "./brand";

function hexToRgb(hex: string): string {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  if (!result) return "0 0 0";
  return `${parseInt(result[1], 16)} ${parseInt(result[2], 16)} ${parseInt(result[3], 16)}`;
}

export function injectBrandStyles(): void {
  const root = document.documentElement;
  const c = brand.colors;
  root.style.setProperty("--brand-primary", c.primary);
  root.style.setProperty("--brand-primary-rgb", hexToRgb(c.primary));
  root.style.setProperty("--brand-secondary", c.secondary);
  root.style.setProperty("--brand-secondary-rgb", hexToRgb(c.secondary));
  root.style.setProperty("--brand-header-bg", c.headerBg);
  root.style.setProperty("--brand-header-text", c.headerText);
  root.style.setProperty("--brand-gold", c.gold);
  root.style.setProperty("--brand-orange", c.orange);
}
