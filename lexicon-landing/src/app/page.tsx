'use client';

import React, { useState } from 'react';
import { motion } from 'framer-motion';
import {
  Download, Terminal, Zap, ShieldCheck, Globe, Cpu,
  ChevronRight, Github, Code, Box, Layers,
  Monitor, Apple, Check, Copy, Package, Boxes,
  FileArchive, ExternalLink, Flame, Blocks
} from 'lucide-react';
import Link from 'next/link';
import { useLanguage } from '@/hooks/useLanguage';
import { VERSION, TAG, links, flavors, installCommandPs1, installCommandSh } from '@/lib/releases';

export default function LandingPage() {
  const { lang, setLang, t } = useLanguage();
  const d = t.download;

  return (
    <div className="min-h-screen bg-black text-white selection:bg-primary/30 overflow-x-hidden">
      <div className="fixed top-0 left-1/2 -translate-x-1/2 w-[1000px] h-[600px] bg-primary/10 rounded-full blur-[120px] -z-10 pointer-events-none" />

      <motion.nav
        initial={{ y: -100 }}
        animate={{ y: 0 }}
        className="fixed top-0 w-full z-50 glass border-b border-white/5"
      >
        <div className="max-w-7xl mx-auto px-6 h-16 flex justify-between items-center">
          <Link href="/" className="flex items-center space-x-3 group">
            <div className="w-9 h-9 bg-primary/10 rounded-xl flex items-center justify-center overflow-hidden group-hover:rotate-12 transition-transform">
              <img src="/mascote_lex_lang_64.png" alt="Lexicon" className="w-8 h-8 object-contain" />
            </div>
            <span className="text-xl font-black tracking-tighter code-font uppercase">LEXICON</span>
          </Link>

          <div className="hidden md:flex items-center space-x-8 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            <a href="#features" className="hover:text-primary transition-colors">{t.nav.features}</a>
            <a href="#downloads" className="hover:text-primary transition-colors">{t.nav.download}</a>
            <a href="#templates" className="hover:text-primary transition-colors">{t.nav.templates}</a>
            <Link href="/docs" className="hover:text-primary transition-colors">{t.nav.docs}</Link>

            <div className="flex items-center gap-2 bg-white/5 border border-white/10 p-1 rounded-full">
              <button
                onClick={() => setLang('en')}
                className={`px-3 py-1 rounded-full text-[10px] font-bold transition-all ${lang === 'en' ? 'bg-primary text-white' : 'hover:text-white'}`}
              >
                EN
              </button>
              <button
                onClick={() => setLang('pt')}
                className={`px-3 py-1 rounded-full text-[10px] font-bold transition-all ${lang === 'pt' ? 'bg-primary text-white' : 'hover:text-white'}`}
              >
                PT
              </button>
            </div>

            <a href={links.sdkZip} className="bg-primary hover:bg-primary/90 text-white px-5 py-2.5 rounded-full transition-all shadow-lg shadow-primary/10 flex items-center gap-2">
              <Download size={16} />
              <span>{t.nav.download}</span>
            </a>
          </div>

          {/* Mobile: só o essencial — docs e download */}
          <div className="flex md:hidden items-center gap-3">
            <Link href="/docs" className="text-[10px] font-black uppercase tracking-widest text-muted-foreground">
              {t.nav.docs}
            </Link>
            <a href="#downloads" className="text-[10px] font-black uppercase tracking-widest text-muted-foreground">
              {t.nav.features}
            </a>
            <a href={links.sdkZip} className="bg-primary px-4 py-2 rounded-full text-[10px] font-black uppercase tracking-widest flex items-center gap-2">
              <Download size={14} />
              <span>{t.nav.download}</span>
            </a>
          </div>
        </div>
      </motion.nav>

      {/* Hero */}
      <section className="relative pt-44 pb-32 px-6">
        <div className="max-w-7xl mx-auto text-center">
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="inline-flex items-center space-x-2 bg-white/5 border border-white/10 px-4 py-1.5 rounded-full mb-10"
          >
            <span className="bg-primary text-[10px] text-white px-2 py-0.5 rounded-full font-black uppercase tracking-widest">{t.hero.version}</span>
            <span className="text-muted-foreground text-xs font-bold uppercase tracking-widest">{t.hero.tagline}</span>
            <ChevronRight size={14} className="text-muted-foreground" />
          </motion.div>

          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="text-6xl md:text-8xl font-black tracking-tightest mb-8 leading-[0.9]"
          >
            {t.hero.title_part1} <br />
            <span className="gradient-text">{t.hero.title_part2}</span>
          </motion.h1>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 }}
            className="text-xl text-muted-foreground max-w-2xl mx-auto mb-12 font-medium leading-relaxed"
          >
            {t.hero.description}
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="flex flex-col sm:flex-row items-center justify-center gap-5"
          >
            <a href={links.sdkZip} className="w-full sm:w-auto bg-primary hover:bg-primary/90 text-white text-lg font-black px-8 py-5 rounded-2xl flex items-center justify-center gap-3 transition-all transform hover:scale-105 shadow-2xl shadow-primary/30">
              <Download size={22} />
              <span>{t.hero.download_sdk}</span>
            </a>
            <a href={links.langZip} className="w-full sm:w-auto bg-white/5 hover:bg-white/10 text-white text-lg font-black px-8 py-5 rounded-2xl flex items-center justify-center gap-3 border border-white/10 transition-all">
              <FileArchive size={22} />
              <span>{t.hero.download_lang}</span>
            </a>
            <a href={links.repo} target="_blank" className="text-muted-foreground hover:text-white text-sm font-black uppercase tracking-widest flex items-center gap-2 transition-colors">
              <Github size={18} />
              <span>{t.hero.repository}</span>
            </a>
          </motion.div>

          <motion.div
            initial={{ opacity: 0, y: 40 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.4 }}
            className="mt-24 max-w-4xl mx-auto relative group animate-float"
          >
            <div className="absolute -inset-1 bg-gradient-to-r from-primary to-orange-400 rounded-[2rem] blur-2xl opacity-20 group-hover:opacity-40 transition duration-1000"></div>
            <div className="relative glass rounded-[2rem] overflow-hidden shadow-3xl border border-white/10">
              <div className="flex items-center justify-between px-6 py-4 border-b border-white/5 bg-white/5">
                <div className="flex items-center space-x-2">
                  <div className="w-3 h-3 rounded-full bg-red-500/50"></div>
                  <div className="w-3 h-3 rounded-full bg-yellow-500/50"></div>
                  <div className="w-3 h-3 rounded-full bg-green-500/50"></div>
                </div>
                <div className="text-[10px] text-muted-foreground font-black uppercase tracking-widest code-font">hero.lex · lex run hero.lex</div>
                <div className="w-12"></div>
              </div>
              <div className="p-8 text-left text-sm md:text-base leading-relaxed code-font overflow-x-auto">
                <pre>
                  <code>
                    <span className="text-primary font-bold">import</span> std::strings;{'\n'}
                    <span className="text-primary font-bold">import</span> std::crypto::sha256;{'\n\n'}
                    <span className="text-primary font-bold">struct</span> <span className="text-cyan-400">User</span> {'{'} id: i64, name: String {'}'}{'\n\n'}
                    <span className="text-primary font-bold">pub fn</span> <span className="text-blue-400">main</span>() <span className="text-primary font-bold">-&gt;</span> <span className="text-cyan-400 font-bold">void</span> {'{'}{'\n'}
                    &nbsp;&nbsp;<span className="text-primary font-bold">let</span> axo = <span className="text-cyan-400">User</span> {'{'} id: <span className="text-orange-400">1</span>, name: <span className="text-green-400">"axolote"</span> {'}'};{'\n'}
                    &nbsp;&nbsp;Console::writeLine(<span className="text-green-400">"#"</span> + axo.id + <span className="text-green-400">" "</span> + axo.name);{'\n\n'}
                    &nbsp;&nbsp;<span className="text-muted-foreground opacity-40">// pipe into a qualified function</span>{'\n'}
                    &nbsp;&nbsp;<span className="text-primary font-bold">let</span> parts = <span className="text-green-400">"a b c"</span> <span className="text-primary font-bold">|&gt;</span> strings::Fields;{'\n'}
                    &nbsp;&nbsp;Console::writeLine(parts <span className="text-primary font-bold">|&gt;</span> strings::Join(<span className="text-green-400">"-"</span>));{'\n'}
                    &nbsp;&nbsp;Console::writeLine(sha256::Sum(<span className="text-green-400">"lexicon"</span>));{'\n'}
                    {'}'}{'\n\n'}
                    <span className="text-muted-foreground opacity-50">$ lex run hero.lex</span>{'\n'}
                    <span className="text-green-400">#1 axolote</span>{'\n'}
                    <span className="text-green-400">a-b-c</span>{'\n'}
                    <span className="text-green-400">239aec50c94b6ea398dadb908b531bbe124c08fbbe756ce60cd59c37cfd719f2</span>
                  </code>
                </pre>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* Features */}
      <section id="features" className="py-32 px-6">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-24">
            <h2 className="text-4xl md:text-6xl font-black mb-6 tracking-tight uppercase">{t.features.title}</h2>
            <p className="text-muted-foreground text-lg font-medium">{t.features.subtitle}</p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-10">
            <FeatureItem icon={<Flame />} title={t.features.hotreload.title} desc={t.features.hotreload.desc} />
            <FeatureItem icon={<Blocks />} title={t.features.extension.title} desc={t.features.extension.desc} />
            <FeatureItem icon={<ShieldCheck />} title={t.features.diagnostics.title} desc={t.features.diagnostics.desc} />
            <FeatureItem icon={<Cpu />} title={t.features.engine.title} desc={t.features.engine.desc} />
          </div>
        </div>
      </section>

      {/* Downloads */}
      <section id="downloads" className="py-32 px-6 relative bg-white/[0.01] border-y border-white/5">
        <div className="max-w-7xl mx-auto">
          <div className="flex flex-col md:flex-row md:items-end justify-between gap-8 mb-16">
            <div>
              <h2 className="text-4xl md:text-6xl font-black mb-4 tracking-tight uppercase">
                {d.title}{' '}
                <span className="gradient-text">{d.from_github}</span>
              </h2>
              <p className="text-muted-foreground text-lg font-medium">{d.subtitle}</p>
            </div>
            <img
              src="/mascote_lex_lang_128.png"
              alt="Axolote Lex"
              className="w-24 h-24 object-contain animate-float hidden md:block"
            />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <FlavorCard
              icon={<Boxes />}
              title={d.sdk.title}
              file={flavors[0].file}
              zip={d.sdk.size_zip}
              installed={d.sdk.size_installed}
              blurb={d.sdk.blurb}
              items={d.contains_sdk.items}
              href={links.sdkZip}
              label={d.button}
              primary
            />
            <FlavorCard
              icon={<Package />}
              title={d.lang.title}
              file={flavors[1].file}
              zip={d.lang.size_zip}
              installed={d.lang.size_installed}
              blurb={d.lang.blurb}
              items={d.contains_lang.items}
              href={links.langZip}
              label={d.button}
            />
          </div>

          {/* Installer scripts */}
          <div className="mt-8 grid grid-cols-1 lg:grid-cols-2 gap-8">
            <div className="glass p-8 rounded-[2rem] border border-white/5">
              <div className="flex items-center gap-3 mb-2">
                <Monitor size={18} className="text-primary" />
                <h3 className="text-lg font-black uppercase tracking-tight">{d.installer_windows}</h3>
              </div>
              <CommandBlock command={installCommandPs1} label={d.installer_title} />
            </div>
            <div className="glass p-8 rounded-[2rem] border border-white/5">
              <div className="flex items-center gap-3 mb-2">
                <Apple size={18} className="text-primary" />
                <h3 className="text-lg font-black uppercase tracking-tight">{d.installer_posix}</h3>
              </div>
              <CommandBlock command={installCommandSh} label={d.installer_title} />
            </div>
          </div>
          <p className="mt-6 text-muted-foreground text-sm font-medium max-w-3xl">{d.installer_note}</p>

          {/* Platforms */}
          <div className="mt-16">
            <h3 className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] mb-6 border-l-2 border-primary/20 pl-3">
              {d.os_title}
            </h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
              <PlatformCard icon={<Monitor />} name="Windows 10/11 x64" status="BINÁRIO PRONTO" detail={`${flavors[0].zip} · ${flavors[0].installed}`} href={links.sdkZip} />
              <PlatformCard icon={<Terminal />} name="Linux x64 / arm64" status="INSTALL.SH" detail="compila do fonte com cargo" href={links.installSh} />
              <PlatformCard icon={<Apple />} name="macOS" status="INSTALL.SH" detail="compila do fonte com cargo" href={links.installSh} />
              <PlatformCard icon={<Boxes />} name="WASM" status="EM BREVE" detail="lex new plugin --target wasm" href={`${links.repo}/blob/${TAG}/Docs/FEATURE_MATRIX.md`} />
            </div>
            <p className="mt-6 text-muted-foreground text-sm font-medium">{d.os_note}</p>
          </div>

          {/* Manual + source */}
          <div className="mt-16 grid grid-cols-1 lg:grid-cols-3 gap-8">
            <div className="glass p-8 rounded-[2rem] border border-white/5">
              <h4 className="font-black text-sm uppercase tracking-widest mb-5">{d.manual.title}</h4>
              <ol className="space-y-3 text-sm text-muted-foreground font-medium">
                {d.manual.steps.map((step: string, i: number) => (
                  <li key={i} className="flex gap-3">
                    <span className="text-primary font-black shrink-0">{i + 1}.</span>
                    <span>{step}</span>
                  </li>
                ))}
              </ol>
              <p className="mt-6 text-[10px] font-black uppercase tracking-widest text-primary/70">{d.no_admin}</p>
            </div>

            <div className="glass p-8 rounded-[2rem] border border-white/5">
              <h4 className="font-black text-sm uppercase tracking-widest mb-2">{d.source_title}</h4>
              <p className="text-muted-foreground text-xs font-medium mb-5">{d.source_note}</p>
              <div className="bg-black/40 p-4 rounded-xl border border-white/5 code-font text-xs leading-loose overflow-x-auto">
                <div><span className="text-muted-foreground"># 1. clone + build do perfil dist</span></div>
                <div>git clone https://github.com/{links.repo.replace('https://github.com/', '')}.git</div>
                <div>cargo build --profile dist -p lexicon-cli</div>
                <div><span className="text-muted-foreground"># 2. PATH + SDK em ~/.lexicon</span></div>
                <div>./target/dist/lex install</div>
              </div>
            </div>

            <div className="glass p-8 rounded-[2rem] border border-white/5 flex flex-col justify-between gap-6">
              <div>
                <h4 className="font-black text-sm uppercase tracking-widest mb-2">v{VERSION}</h4>
                <p className="text-muted-foreground text-xs font-medium">
                  {lang === 'en'
                    ? 'Every asset ships with a SHA-256. Verify before you run it.'
                    : 'Cada asset sai com SHA-256. Confira antes de rodar.'}
                </p>
              </div>
              <div className="space-y-3">
                <p className="text-[10px] font-black uppercase tracking-widest code-font break-all">sha256sum {flavors[0].file}</p>
                <a href={links.checksums} target="_blank" className="flex items-center justify-between text-xs font-black uppercase tracking-widest bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl px-4 py-3 transition-colors">
                  <span>{d.checksums}</span><ExternalLink size={14} />
                </a>
                <a href={links.extensionVsix} className="flex items-center justify-between text-xs font-black uppercase tracking-widest bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl px-4 py-3 transition-colors">
                  <span>{d.extension}</span><Download size={14} />
                </a>
                <a href={links.releases} target="_blank" className="flex items-center justify-between text-xs font-black uppercase tracking-widest bg-primary/10 hover:bg-primary/20 border border-primary/20 text-primary rounded-xl px-4 py-3 transition-colors">
                  <span>{d.all_releases}</span><ExternalLink size={14} />
                </a>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Templates */}
      <section id="templates" className="py-32 px-6">
        <div className="max-w-7xl mx-auto">
          <div className="text-left mb-20">
            <h2 className="text-4xl md:text-6xl font-black mb-6 tracking-tight uppercase">
              {t.templates.title.split(' ')[0]} <br />
              <span className="gradient-text">{t.templates.title.split(' ').slice(1).join(' ')}</span>
            </h2>
            <p className="text-muted-foreground text-lg max-w-xl font-medium">
              {t.templates.subtitle}
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
            <TemplateCard
              icon={<Globe className="text-primary" />}
              title={t.templates.api}
              badge="lex new api"
              command="lex new meu-api -t api"
              description={t.templates.api_desc}
            />
            <TemplateCard
              icon={<Cpu className="text-primary" />}
              title={t.templates.gui}
              badge="wgpu"
              command="lex new minha-janela -t gui"
              description={t.templates.gui_desc}
            />
            <TemplateCard
              icon={<Box className="text-primary" />}
              title={t.templates.plugin}
              badge="SCAFFOLD"
              command="lex new meu-plugin -t plugin"
              description={t.templates.plugin_desc}
            />
            <TemplateCard
              icon={<Layers className="text-primary" />}
              title={t.templates.service}
              badge="SCAFFOLD"
              command="lex new meu-servico -t service"
              description={t.templates.service_desc}
            />
          </div>

          <div className="mt-12 glass p-8 rounded-[2rem] border border-white/5">
            <div className="flex items-center gap-3 mb-2">
              <Terminal size={18} className="text-primary" />
              <h3 className="text-lg font-black uppercase tracking-tight">{t.templates.quickstart_title}</h3>
            </div>
            <p className="text-muted-foreground text-sm font-medium mb-6">{t.templates.quickstart_desc}</p>
            <div className="bg-black/40 p-6 rounded-xl border border-white/5 code-font text-sm leading-loose overflow-x-auto">
              <div><span className="text-muted-foreground"># hot reload — salvou, reiniciou</span></div>
              <div><span className="text-primary font-bold">lex run --watch</span> <span className="text-green-400">examples/pong.lex</span></div>
              <div><span className="text-muted-foreground"># gate de correção antes de rodar</span></div>
              <div><span className="text-primary font-bold">lex vet</span> <span className="text-green-400">examples/fizzbuzz.lex</span></div>
              <div><span className="text-muted-foreground"># API real: axum HTTP + SQLite (.env dev)</span></div>
              <div><span className="text-primary font-bold">powershell -ExecutionPolicy Bypass -File</span> <span className="text-green-400">demo-api\run-dev.ps1</span></div>
              <div><span className="text-primary font-bold">curl</span> <span className="text-green-400">http://localhost:3000/users</span></div>
            </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-20 px-6 border-t border-white/5">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row justify-between items-center gap-10">
          <div className="flex items-center space-x-3 group">
            <div className="w-8 h-8 bg-primary/10 rounded-lg flex items-center justify-center overflow-hidden">
              <img src="/mascote_lex_lang_64.png" alt="Lexicon" className="w-7 h-7 object-contain" />
            </div>
            <span className="text-xl font-black tracking-tighter code-font uppercase">LEXICON</span>
          </div>

          <div className="flex flex-wrap justify-center gap-10 text-xs font-black uppercase tracking-widest text-muted-foreground">
            <a href={links.repo} target="_blank" className="hover:text-primary transition-colors">GitHub</a>
            <a href={links.releases} target="_blank" className="hover:text-primary transition-colors">{lang === 'en' ? 'Releases' : 'Releases'}</a>
            <a href={links.sdkZip} className="hover:text-primary transition-colors">{t.nav.download}</a>
            <Link href="/docs" className="hover:text-primary transition-colors">{t.nav.docs}</Link>
          </div>

          <div className="text-[10px] text-muted-foreground font-bold uppercase tracking-[0.2em] text-center md:text-right">
            <span className="inline-block bg-primary/10 border border-primary/20 text-primary px-2 py-0.5 rounded-full mb-2">v{VERSION}</span><br />
            © 2026 LEXICON TEAM. {lang === 'en' ? 'MIT LICENSE' : 'LICENÇA MIT'}.
          </div>
        </div>
      </footer>
    </div>
  );
}

function FeatureItem({ icon, title, desc }: { icon: React.ReactNode, title: string, desc: string }) {
  return (
    <div className="text-center group">
      <div className="w-16 h-16 bg-white/5 rounded-[2rem] flex items-center justify-center mx-auto mb-8 group-hover:bg-primary/10 group-hover:rotate-12 transition-all duration-500 border border-white/5 group-hover:border-primary/20">
        {React.cloneElement(icon as React.ReactElement<{ size?: number; className?: string }>, { size: 28, className: "text-primary" })}
      </div>
      <h3 className="text-lg font-black uppercase tracking-tight mb-4">{title}</h3>
      <p className="text-muted-foreground text-sm font-medium leading-relaxed">{desc}</p>
    </div>
  );
}

function FlavorCard({
  icon, title, file, zip, installed, blurb, items, href, label, primary = false,
}: {
  icon: React.ReactNode,
  title: string,
  file: string,
  zip: string,
  installed: string,
  blurb: string,
  items: string[],
  href: string,
  label: string,
  primary?: boolean,
}) {
  return (
    <div className={`glass p-8 rounded-[2rem] border transition-all ${primary ? 'border-primary/30 bg-primary/[0.03]' : 'border-white/5 hover:border-primary/30'}`}>
      <div className="flex items-start justify-between mb-6">
        <div className={`w-12 h-12 rounded-2xl flex items-center justify-center ${primary ? 'bg-primary/15 text-primary' : 'bg-white/5 text-primary/70'}`}>
          {React.cloneElement(icon as React.ReactElement<{ size?: number }>, { size: 24 })}
        </div>
        <div className="text-right">
          <p className="text-[10px] font-black uppercase tracking-widest text-muted-foreground">{zip}</p>
          <p className="text-[10px] font-black uppercase tracking-widest text-primary/70">{installed}</p>
        </div>
      </div>

      <h3 className="text-xl font-black mb-3 uppercase tracking-tight">{title}</h3>
      <p className="text-muted-foreground text-sm font-medium leading-relaxed mb-5">{blurb}</p>

      <ul className="space-y-2 mb-6">
        {items.map((item: string) => (
          <li key={item} className="flex gap-2 text-xs font-medium text-muted-foreground">
            <Check size={14} className="text-primary shrink-0 mt-0.5" />
            <span>{item}</span>
          </li>
        ))}
      </ul>

      <a
        href={href}
        className={`w-full flex items-center justify-center gap-3 rounded-2xl px-6 py-4 text-sm font-black uppercase tracking-widest transition-all ${
          primary
            ? 'bg-primary hover:bg-primary/90 text-white shadow-xl shadow-primary/20'
            : 'bg-white/5 hover:bg-white/10 text-white border border-white/10'
        }`}
      >
        <Download size={18} />
        <span>{label}</span>
      </a>
      <p className="mt-3 text-[10px] text-muted-foreground font-bold code-font break-all">{file}</p>
    </div>
  );
}

function CommandBlock({ command, label }: { command: string, label: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(command);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div>
      <p className="text-[10px] font-black uppercase tracking-widest text-muted-foreground mb-3">{label}</p>
      <div className="bg-black/40 p-4 rounded-xl border border-white/5 flex items-start gap-3">
        <code className="text-xs text-orange-400 code-font break-all flex-1">{command}</code>
        <button onClick={copy} className="text-muted-foreground hover:text-white transition-colors shrink-0" aria-label="copiar">
          {copied ? <Check size={14} className="text-primary" /> : <Copy size={14} />}
        </button>
      </div>
    </div>
  );
}

function PlatformCard({ icon, name, status, detail, href }: { icon: React.ReactNode, name: string, status: string, detail: string, href: string }) {
  const ready = status === 'BINÁRIO PRONTO';
  return (
    <a
      href={href}
      target="_blank"
      className={`glass p-6 rounded-2xl border transition-all group ${ready ? 'border-primary/30 hover:bg-primary/[0.06]' : 'border-white/5 hover:bg-white/[0.03]'}`}
    >
      <div className={`mb-4 flex justify-center ${ready ? 'text-primary' : 'text-muted-foreground group-hover:text-primary'} transition-colors`}>
        {React.cloneElement(icon as React.ReactElement<{ size?: number }>, { size: 32 })}
      </div>
      <h4 className="font-black text-sm uppercase tracking-widest mb-1 text-center">{name}</h4>
      <p className={`text-[10px] font-black uppercase tracking-widest text-center ${ready ? 'text-primary' : 'text-muted-foreground'}`}>{status}</p>
      <p className="text-[10px] text-muted-foreground font-bold text-center mt-2">{detail}</p>
    </a>
  );
}

function TemplateCard({ icon, title, badge, command, description }: { icon: React.ReactNode, title: string, badge: string, command: string, description: string }) {
  return (
    <div className="glass p-8 rounded-[2rem] border border-white/5 hover:border-primary/30 transition-all group">
      <div className="flex justify-between items-start mb-6">
        <div className="w-12 h-12 bg-white/5 rounded-2xl flex items-center justify-center group-hover:bg-primary/10 group-hover:scale-110 transition-all duration-500">
          {icon}
        </div>
        <span className="text-[9px] font-black bg-white/10 px-2 py-1 rounded-full tracking-widest text-muted-foreground">{badge}</span>
      </div>
      <h3 className="text-lg font-black mb-3 uppercase tracking-tight">{title}</h3>
      <p className="text-muted-foreground text-xs font-medium leading-relaxed mb-6">{description}</p>
      <div className="bg-black/40 p-4 rounded-xl border border-white/5 group-hover:border-primary/20 transition-colors">
        <div className="flex items-center justify-between gap-3">
          <code className="text-[11px] text-primary font-bold code-font break-all">{command}</code>
          <Code size={14} className="text-muted-foreground shrink-0" />
        </div>
      </div>
    </div>
  );
}
