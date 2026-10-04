import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import { VERSION } from "@/lib/releases";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

const title = `Lexicon v${VERSION} | Um binário com hot reload, GUI wgpu e 89 pacotes std`;
const description = `Lexicon v${VERSION}: toolchain completo num binário de 9,4 MB — lex run --watch, lex check/vet, lex mod, motor gráfico wgpu e 89 pacotes de stdlib escritos em Lex. Baixe o SDK completo ou só a linguagem.`;

export const metadata: Metadata = {
  metadataBase: new URL("https://lexicon.dev"),
  title,
  description,
  keywords: ["Lexicon", "Lex", "Linguagem de programação", "SDK", "Rust", "wGPU", "Hot Reload", "VS Code", "SQLite", "std"],
  authors: [{ name: "Lexicon Team" }],
  openGraph: {
    title,
    description,
    url: "https://lexicon.dev",
    siteName: "Lexicon",
    images: [
      {
        url: "/og-image.png",
        width: 1200,
        height: 630,
        alt: "Lexicon — axolote roxo mascote",
      },
    ],
    locale: "pt_BR",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title,
    description,
    images: ["/og-image.png"],
  },
  icons: {
    icon: [
      { url: "/favicon.ico", sizes: "16x16", type: "image/x-icon" },
      { url: "/mascote_lex_lang_32.png", sizes: "32x32", type: "image/png" },
    ],
    apple: "/mascote_lex_lang_180.png",
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
