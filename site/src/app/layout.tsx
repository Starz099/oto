import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";
import localFont from "next/font/local";
import "./globals.css";

const inter = Inter({
  subsets: ["latin"],
  variable: "--font-sans",
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
});

const coolvetica = localFont({
  src: [
    {
      path: "../../public/fonts/Coolvetica Rg.otf",
      weight: "400",
      style: "normal",
    },
    {
      path: "../../public/fonts/Coolvetica Rg.otf",
      weight: "700",
      style: "normal",
    },
    {
      path: "../../public/fonts/Coolvetica Rg.otf",
      weight: "800",
      style: "normal",
    },
    {
      path: "../../public/fonts/Coolvetica Rg It.otf",
      weight: "400",
      style: "italic",
    },
  ],
  variable: "--font-coolvetica",
});

export const metadata: Metadata = {
  title: "Oto | Keyboard-first desktop audio mixer overlay",
  description: "A lightweight, keyboard-first desktop audio mixer overlay for Windows, featuring app-specific volume control, zero-latency global push-to-talk, and Discord integration.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="scroll-smooth">
      <body className={`${inter.variable} ${jetbrainsMono.variable} ${coolvetica.variable} font-sans antialiased bg-oto-dark text-oto-white selection:bg-oto-pink selection:text-oto-dark`}>
        {children}
      </body>
    </html>
  );
}
