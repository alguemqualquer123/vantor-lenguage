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
  title: "Lexicon SDK v0.2.0 | Hot Reload, Super Extension & Pre-Run Diagnostics",
  description: "Lexicon v0.2.0: hot-reload supervisor (lex run --watch), VS Code super extension 1.1.x, lex check + lex vet with E-code diagnostics, and a real axum HTTP + SQLite demo API.",
  keywords: ["Lexicon", "Programação", "SDK", "v0.2.0", "Hot Reload", "VS Code", "WASM", "WebAssembly", "Rust", "Performance", "SQLite"],
  authors: [{ name: "Lexicon Team" }],
  openGraph: {
    title: "Lexicon SDK v0.2.0 | Hot Reload, Super Extension & Pre-Run Diagnostics",
    description: "Save → auto-restart. Syntax gate with line:col errors. Real HTTP + SQLite demo.",
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
    title: "Lexicon SDK v0.2.0 | Hot Reload, Super Extension & Pre-Run Diagnostics",
    description: "Save → auto-restart. Syntax gate with line:col errors. Real HTTP + SQLite demo.",
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
