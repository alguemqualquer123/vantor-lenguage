import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Lexicon SDK | A Linguagem para Cloud & Edge",
  description: "Construa APIs escaláveis com performance nativa, WebAssembly e deploy instantâneo. Conheça o Lexicon, a linguagem de programação da nova era.",
  keywords: ["Lexicon", "Programação", "SDK", "Cloud", "Edge", "WASM", "WebAssembly", "Rust", "Performance"],
  authors: [{ name: "Lexicon Team" }],
  openGraph: {
    title: "Lexicon SDK | A Linguagem para Cloud & Edge",
    description: "Sintaxe moderna, performance de baixo nível e deploy instantâneo.",
    url: "https://lexicon.dev",
    siteName: "Lexicon",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
      },
    ],
    locale: "pt_BR",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Lexicon SDK | A Linguagem para Cloud & Edge",
    description: "Construa APIs escaláveis com performance nativa.",
    images: ["/og-image.png"],
  },
  icons: {
    icon: "/favicon.ico",
    apple: "/apple-touch-icon.png",
  },
};

import { LanguageProvider } from "@/hooks/useLanguage";

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <LanguageProvider>
          {children}
        </LanguageProvider>
      </body>
    </html>
  );
}
