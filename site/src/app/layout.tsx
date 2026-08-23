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
  metadataBase: new URL("https://github.com/Starz099/oto"),
  title: "Oto | Keyboard-first desktop audio mixer overlay",
  description: "A lightweight, keyboard-first desktop audio mixer overlay for Windows, featuring app-specific volume control, zero-latency global push-to-talk, and Discord integration.",
  icons: {
    icon: "/favicon.ico",
  },
  openGraph: {
    title: "Oto | Keyboard-First Desktop Audio Mixer Overlay",
    description: "A lightweight, keyboard-first desktop audio mixer overlay for Windows, featuring app-specific volume control, zero-latency global push-to-talk, and Discord integration.",
    url: "https://github.com/Starz099/oto",
    siteName: "Oto Mixer",
    images: [
      {
        url: "/assets/oto.png",
        width: 1200,
        height: 630,
        alt: "Oto - Keyboard-First Desktop Audio Mixer Overlay",
      },
    ],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Oto | Keyboard-First Desktop Audio Mixer Overlay",
    description: "A lightweight, keyboard-first desktop audio mixer overlay for Windows, featuring app-specific volume control, zero-latency global push-to-talk, and Discord integration.",
    images: ["/assets/oto.png"],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="scroll-smooth" suppressHydrationWarning>
      <body className={`${inter.variable} ${jetbrainsMono.variable} ${coolvetica.variable} font-sans antialiased bg-oto-dark text-oto-white selection:bg-oto-pink selection:text-oto-dark`} suppressHydrationWarning>
        {children}
      </body>
    </html>
  );
}
