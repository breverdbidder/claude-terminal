/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // Legacy aliases (map to elevation system)
        'bg-primary': 'var(--elevation-0)',
        'bg-secondary': 'var(--elevation-1)',
        'bg-elevated': 'var(--elevation-2)',
        'bg-surface': 'var(--elevation-3)',
        // Elevation system (5 levels)
        'elevation-0': 'var(--elevation-0)',
        'elevation-1': 'var(--elevation-1)',
        'elevation-2': 'var(--elevation-2)',
        'elevation-3': 'var(--elevation-3)',
        'elevation-4': 'var(--elevation-4)',
        // Accent colors (desaturated for dark-mode comfort)
        'accent-primary': '#4B8DF8',
        'accent-secondary': '#3A73D4',
        'accent-glow': 'rgba(75, 141, 248, 0.15)',
        // Borders
        'border': '#232323',
        'border-light': '#2E2E2E',
        'border-focus': 'rgba(75, 141, 248, 0.4)',
        // Text
        'text-primary': '#E2E2E2',
        'text-secondary': '#787878',
        'text-tertiary': '#4F4F4F',
        // Semantic (desaturated)
        'success': '#3EC978',
        'warning': '#E5AD2B',
        'error': '#E04545',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
      boxShadow: {
        'glow-sm': '0 0 8px rgba(75, 141, 248, 0.12)',
        'glow-md': '0 0 16px rgba(75, 141, 248, 0.18)',
        'elevation-2': '0 2px 8px rgba(0, 0, 0, 0.3)',
        'elevation-3': '0 4px 16px rgba(0, 0, 0, 0.4)',
        'elevation-4': '0 8px 32px rgba(0, 0, 0, 0.5)',
      },
    },
  },
  plugins: [],
}
