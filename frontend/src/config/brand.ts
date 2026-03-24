export interface BrandColors {
  primary: string;
  secondary: string;
  headerBg: string;
  headerText: string;
  accentSoft: string;
  surface: string;
  gold: string;
  orange: string;
}

export interface BrandConfig {
  namePrefix: string;
  nameAccent: string;
  product: string;
  tagline: string;
  colors: BrandColors;
}

export const brand: BrandConfig = {
  namePrefix: "Open",
  nameAccent: "AEC",
  product: "BCF Platform",
  tagline: "Issue management voor BIM-projecten",
  colors: {
    primary: "#D97706",
    secondary: "#EA580C",
    headerBg: "#36363E",
    headerText: "#FAFAF9",
    accentSoft: "rgba(217,119,6,0.08)",
    surface: "#36363E",
    gold: "#F59E0B",
    orange: "#EA580C",
  },
};
