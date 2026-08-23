/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        oto: {
          dark: '#0F0F12',
          panel: '#16161A',
          selection: '#231E23',
          pink: '#F8C8DC',
          white: '#EBEBF0',
          slate: '#9696A0',
          rose: '#FFB6C1',
        }
      },
      fontFamily: {
        display: ['var(--font-coolvetica)', 'sans-serif'],
        mono: ['var(--font-mono)', 'JetBrains Mono', 'Fira Code', 'monospace'],
        sans: ['var(--font-sans)', 'Inter', 'system-ui', 'sans-serif'],
      }
    },
  },
  plugins: [],
};
