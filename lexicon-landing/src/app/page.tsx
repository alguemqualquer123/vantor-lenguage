'use client';

import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { 
  Download, Terminal, Zap, Shield, Globe, Cpu, 
  ChevronRight, Github, Code, Box, Layers, 
  Monitor, Layout, CheckCircle2, Languages
} from 'lucide-react';
import Link from 'next/link';
import { useLanguage } from '@/hooks/useLanguage';

export default function LandingPage() {
  const { lang, setLang, t } = useLanguage();
  const githubUrl = process.env.NEXT_PUBLIC_GITHUB_URL || 'https://github.com/lexicon-team/lexicon';
  const docsUrl = process.env.NEXT_PUBLIC_DOCS_URL || '/docs';
  const installMsi = process.env.NEXT_PUBLIC_INSTALL_MSI || '#';

  return (
    <div className="min-h-screen bg-black text-white selection:bg-primary/30 overflow-x-hidden">
      {/* Background Glow */}
      <div className="fixed top-0 left-1/2 -translate-x-1/2 w-[1000px] h-[600px] bg-primary/10 rounded-full blur-[120px] -z-10 pointer-events-none" />

      {/* Navigation */}
      <motion.nav 
        initial={{ y: -100 }}
        animate={{ y: 0 }}
        className="fixed top-0 w-full z-50 glass border-b border-white/5"
      >
        <div className="max-w-7xl mx-auto px-6 h-16 flex justify-between items-center">
          <Link href="/" className="flex items-center space-x-3 group">
            <div className="w-9 h-9 bg-primary/10 rounded-xl flex items-center justify-center overflow-hidden group-hover:rotate-12 transition-transform">
              <img src="/logo.png" alt="Lexicon Logo" className="w-7 h-7 object-contain" />
            </div>
            <span className="text-xl font-black tracking-tighter code-font uppercase">LEXICON</span>
          </Link>
          
          <div className="hidden md:flex items-center space-x-8 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            <a href="#features" className="hover:text-primary transition-colors">{t.nav.features}</a>
            <a href="#templates" className="hover:text-primary transition-colors">{t.nav.templates}</a>
            <Link href={docsUrl} className="hover:text-primary transition-colors">{t.nav.docs}</Link>
            
            {/* Language Switcher */}
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

            <a href={installMsi} className="bg-primary hover:bg-primary/90 text-white px-5 py-2.5 rounded-full transition-all shadow-lg shadow-primary/10 flex items-center gap-2">
              <Download size={16} />
              <span>{t.nav.install}</span>
            </a>
          </div>
        </div>
      </motion.nav>

      {/* Hero Section */}
      <section className="relative pt-44 pb-32 px-6">
        <div className="max-w-7xl mx-auto text-center">
          <motion.div 
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="inline-flex items-center space-x-2 bg-white/5 border border-white/10 px-4 py-1.5 rounded-full mb-10"
          >
            <span className="bg-primary text-[10px] text-white px-2 py-0.5 rounded-full font-black uppercase tracking-widest">{t.hero.alpha}</span>
            <span className="text-muted-foreground text-xs font-bold uppercase tracking-widest">{t.hero.new_era}</span>
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
            <a href={installMsi} className="w-full sm:w-auto bg-primary hover:bg-primary/90 text-white text-lg font-black px-10 py-5 rounded-2xl flex items-center justify-center gap-3 transition-all transform hover:scale-105 shadow-2xl shadow-primary/30">
              <Download size={24} />
              <span>{t.hero.download}</span>
            </a>
            <a href={githubUrl} target="_blank" className="w-full sm:w-auto bg-white/5 hover:bg-white/10 text-white text-lg font-black px-10 py-5 rounded-2xl flex items-center justify-center gap-3 border border-white/10 transition-all">
              <Github size={24} />
              <span>{t.hero.repository}</span>
            </a>
          </motion.div>
          
          {/* Interactive Code Preview */}
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
                   <div className="text-[10px] text-muted-foreground font-black uppercase tracking-widest code-font">playground.lex</div>
                   <div className="w-12"></div>
                </div>
                <div className="p-8 text-left text-sm md:text-base leading-relaxed code-font overflow-x-auto">
                   <pre>
                     <code>
                       <span className="text-primary font-bold">import</span> core.net.Http;<br />
                       <span className="text-primary font-bold">import</span> core.json.Json;<br /><br />
                       <span className="text-muted-foreground italic opacity-60">@Test</span><br />
                       <span className="text-primary font-bold">fn</span> <span className="text-blue-400">test_api</span>() &#123;<br />
                       &nbsp;&nbsp;<span className="text-primary font-bold">let</span> response = Http::get(<span className="text-green-400">"/status"</span>);<br />
                       &nbsp;&nbsp;<span className="text-cyan-400 font-bold">assert</span> response.code == <span className="text-orange-400">200</span>;<br />
                       &#125;<br /><br />
                       <span className="text-primary font-bold">pub fn</span> <span className="text-blue-400">main</span>() <span className="text-primary font-bold">-&gt;</span> <span className="text-cyan-400 font-bold">void</span> &#123;<br />
                       &nbsp;&nbsp;<span className="text-primary font-bold">let</span> server = Http::serve(<span className="text-green-400">"0.0.0.0:8080"</span>);<br />
                       &nbsp;&nbsp;server.listen(); <span className="text-muted-foreground opacity-40">// {"|>"} hot reload active</span><br />
                       &#125;
                     </code>
                   </pre>
                </div>
             </div>
          </motion.div>
        </div>
      </section>

      {/* Features Section */}
      <section id="features" className="py-32 px-6">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-24">
            <h2 className="text-4xl md:text-6xl font-black mb-6 tracking-tight uppercase">{t.features.title}</h2>
            <p className="text-muted-foreground text-lg font-medium">{t.features.subtitle}</p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-10">
            <FeatureItem icon={<Cpu />} title={t.features.native.title} desc={t.features.native.desc} />
            <FeatureItem icon={<Globe />} title={t.features.wasm.title} desc={t.features.wasm.desc} />
            <FeatureItem icon={<Shield />} title={t.features.cloud.title} desc={t.features.cloud.desc} />
            <FeatureItem icon={<Zap />} title={t.features.pipes.title} desc={t.features.pipes.desc} />
          </div>
        </div>
      </section>

      {/* Templates Section */}
      <section id="templates" className="py-32 px-6 relative bg-white/[0.01]">
        <div className="max-w-7xl mx-auto">
          <div className="text-left mb-20">
            <h2 className="text-4xl md:text-6xl font-black mb-6 tracking-tight uppercase">
              {t.templates.title.split(' ')[0]} <br/>
              <span className="gradient-text">{t.templates.title.split(' ').slice(1).join(' ')}</span>
            </h2>
            <p className="text-muted-foreground text-lg max-w-xl font-medium">
              {lang === 'en' ? 'Start instantly with pre-configured models for any use case.' : 'Comece instantaneamente com modelos pré-configurados para qualquer caso de uso.'}
            </p>
          </div>
          
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <TemplateCard 
              icon={<Globe className="text-primary" />}
              title={t.templates.api}
              badge={lang === 'en' ? "RECOMMENDED" : "RECOMENDADO"}
              command="lex new api --edge"
              description={lang === 'en' ? "Optimized for low latency on cloud providers. Includes router, middleware and native JSON support." : "Otimizado para baixa latência em cloud providers. Inclui router, middleware e suporte a JSON nativo."}
            />
            <TemplateCard 
              icon={<Cpu className="text-primary" />}
              title={t.templates.plugin}
              badge={lang === 'en' ? "EXPERIMENTAL" : "EXPERIMENTAL"}
              command="lex new plugin --target wasm"
              description={lang === 'en' ? "Create extensible plugins that run in the browser or in serverless runtimes with native performance." : "Crie plugins extensíveis que rodam no navegador ou em runtimes serverless com performance nativa."}
            />
            <TemplateCard 
              icon={<Layers className="text-primary" />}
              title={t.templates.service}
              badge={lang === 'en' ? "STABLE" : "ESTÁVEL"}
              command="lex new service --grpc"
              description={lang === 'en' ? "Microservices architecture with gRPC and native integration for inter-process communication." : "Arquitetura de microsserviços com gRPC e integração nativa para comunicação entre processos."}
            />
          </div>
        </div>
      </section>

      {/* OS Support & Downloads */}
      <section className="py-32 px-6 bg-white/[0.02] border-y border-white/5">
        <div className="max-w-7xl mx-auto text-center">
          <h2 className="text-3xl md:text-5xl font-black mb-20 tracking-tight uppercase">
            {lang === 'en' ? 'Available for ' : 'Disponível para '} <br/>
            <span className="gradient-text">{lang === 'en' ? 'ALL SYSTEMS' : 'TODOS OS SISTEMAS'}</span>
          </h2>
          
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-6">
            <OSCard icon={<Monitor />} name="Windows" version="v0.1.0-alpha" ext=".msi" />
            <OSCard icon={<Box />} name="Linux" version="v0.1.0-alpha" ext=".tar.gz" />
            <OSCard icon={<Layout />} name="macOS" version="v0.1.0-alpha" ext=".pkg" />
            <OSCard icon={<Cpu />} name="WASM" version="v0.1.0-alpha" ext=".wasm" />
          </div>

          <div className="mt-20 glass p-8 rounded-[2rem] max-w-2xl mx-auto border border-white/10">
             <div className="flex items-center gap-4 text-left">
                <div className="w-12 h-12 rounded-full bg-white/10 flex items-center justify-center shrink-0">
                   <Terminal className="text-primary" />
                </div>
                <div>
                   <h4 className="font-black text-sm uppercase tracking-widest mb-1">
                      {lang === 'en' ? 'Installation via Terminal' : 'Instalação via Terminal'}
                   </h4>
                   <code className="text-xs text-muted-foreground code-font break-all">curl -fsSL https://get.lexicon.dev | sh</code>
                </div>
             </div>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-20 px-6 border-t border-white/5">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row justify-between items-center gap-10">
          <div className="flex items-center space-x-3 group">
            <div className="w-8 h-8 bg-primary/10 rounded-lg flex items-center justify-center overflow-hidden">
               <img src="/logo.png" alt="Lexicon Logo" className="w-6 h-6 object-contain" />
            </div>
            <span className="text-xl font-black tracking-tighter code-font uppercase">LEXICON</span>
          </div>
          
          <div className="flex flex-wrap justify-center gap-10 text-xs font-black uppercase tracking-widest text-muted-foreground">
             <a href="#" className="hover:text-primary transition-colors">Twitter</a>
             <a href="#" className="hover:text-primary transition-colors">Discord</a>
             <a href={githubUrl} target="_blank" className="hover:text-primary transition-colors">GitHub</a>
             <a href="#" className="hover:text-primary transition-colors">Cloud</a>
          </div>

          <div className="text-[10px] text-muted-foreground font-bold uppercase tracking-[0.2em]">
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
        {React.cloneElement(icon as React.ReactElement, { size: 28, className: "text-primary" })}
      </div>
      <h3 className="text-lg font-black uppercase tracking-tight mb-4">{title}</h3>
      <p className="text-muted-foreground text-sm font-medium leading-relaxed">{desc}</p>
    </div>
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
      <h3 className="text-xl font-black mb-3 uppercase tracking-tight">{title}</h3>
      <p className="text-muted-foreground text-sm font-medium leading-relaxed mb-6">{description}</p>
      <div className="bg-black/40 p-4 rounded-xl border border-white/5 group-hover:border-primary/20 transition-colors">
        <div className="flex items-center justify-between">
           <code className="text-xs text-primary font-bold code-font">{command}</code>
           <button className="text-muted-foreground hover:text-white transition-colors">
              <Code size={14} />
           </button>
        </div>
      </div>
    </div>
  );
}

function OSCard({ icon, name, version, ext }: { icon: React.ReactNode, name: string, version: string, ext: string }) {
  return (
    <div className="glass p-6 rounded-2xl border border-white/5 hover:bg-white/[0.03] transition-all cursor-pointer group">
      <div className="text-muted-foreground group-hover:text-primary transition-colors mb-4 flex justify-center">
        {React.cloneElement(icon as React.ReactElement, { size: 32 })}
      </div>
      <h4 className="font-black text-sm uppercase tracking-widest mb-1">{name}</h4>
      <div className="flex items-center justify-center gap-2 text-[10px] text-muted-foreground font-bold">
         <span>{version}</span>
         <span className="w-1 h-1 bg-white/20 rounded-full" />
         <span className="text-primary">{ext}</span>
      </div>
    </div>
  );
}
