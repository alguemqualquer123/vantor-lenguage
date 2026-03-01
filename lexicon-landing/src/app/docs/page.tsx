'use client';

import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { 
  ChevronRight, Terminal, Book, Code, Rocket, Zap, 
  Shield, Search, FileText, Settings, AlertCircle, 
  HelpCircle, User, Cpu 
} from 'lucide-react';

import { useLanguage } from '@/hooks/useLanguage';

export default function DocsPage() {
  const { lang, setLang, t } = useLanguage();
  const [activeSection, setActiveSection] = useState('manual_usuario');
  const [searchQuery, setSearchQuery] = useState('');
  const [markdownContent, setMarkdownContent] = useState('');
  const [isLoading, setIsLoading] = useState(true);

  const menuSections = [
    {
      title: t.docs.menu.user_guide,
      items: [
        { id: 'manual_usuario', name: t.docs.menu.manual, icon: <User size={18} /> },
        { id: 'guia_instalacao', name: t.docs.menu.installation, icon: <Settings size={18} /> },
      ]
    },
    {
      title: t.docs.menu.tech_docs,
      items: [
        { id: 'documentacao_tecnica', name: t.docs.menu.architecture, icon: <Cpu size={18} /> },
        { id: 'troubleshooting_glossario', name: t.docs.menu.troubleshooting, icon: <AlertCircle size={18} /> },
      ]
    }
  ];

  useEffect(() => {
    async function fetchDocs() {
      setIsLoading(true);
      try {
        // Se o idioma for inglês, buscamos na pasta /docs/en/
        const path = lang === 'en' ? `/docs/en/${activeSection}.md` : `/docs/${activeSection}.md`;
        const response = await fetch(path);
        if (response.ok) {
          const text = await response.text();
          setMarkdownContent(text);
        } else {
          setMarkdownContent(`# ${t.docs.error_loading}`);
        }
      } catch (error) {
        setMarkdownContent(`# ${t.docs.error_generic}`);
      }
      setIsLoading(false);
    }
    fetchDocs();
  }, [activeSection, lang, t]);

  const filteredSections = menuSections.map(section => ({
    ...section,
    items: section.items.filter(item => 
      item.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      section.title.toLowerCase().includes(searchQuery.toLowerCase())
    )
  })).filter(section => section.items.length > 0);

  return (
    <div className="min-h-screen bg-black text-white selection:bg-primary/30">
      <div className="fixed top-0 right-0 w-[500px] h-[500px] bg-primary/5 rounded-full blur-[120px] -z-10 pointer-events-none" />

      <nav className="border-b border-white/5 bg-black/80 backdrop-blur-md sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-6 h-16 flex items-center justify-between gap-8">
          <Link href="/" className="flex items-center space-x-3 shrink-0 group">
            <div className="w-8 h-8 bg-primary/10 rounded-lg flex items-center justify-center overflow-hidden group-hover:rotate-12 transition-transform">
              <img src="/logo.png" alt="Lexicon Logo" className="w-6 h-6 object-contain" />
            </div>
            <span className="font-black tracking-tighter code-font text-lg uppercase">{t.docs.title}</span>
          </Link>

          <div className="flex-1 max-w-md relative hidden sm:block">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 text-muted-foreground" size={16} />
            <input 
              type="text" 
              placeholder={t.docs.search_placeholder}
              className="w-full bg-white/5 border border-white/10 rounded-full py-2.5 pl-12 pr-4 text-sm focus:outline-none focus:border-primary/50 transition-all"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>

          <div className="flex items-center gap-6 text-[10px] font-black uppercase tracking-[0.2em]">
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
            <Link href="/" className="text-muted-foreground hover:text-white transition-colors">{t.docs.portal}</Link>
            <a href={process.env.NEXT_PUBLIC_GITHUB_URL || 'https://github.com/lexicon-team/lexicon'} target="_blank" className="text-muted-foreground hover:text-white transition-colors">{t.docs.github}</a>
          </div>
        </div>
      </nav>

      <div className="max-w-7xl mx-auto px-6 py-12 flex flex-col md:flex-row gap-16">
        <aside className="w-full md:w-64 space-y-12 shrink-0">
          {filteredSections.length > 0 ? filteredSections.map((section) => (
            <div key={section.title} className="animate-in fade-in slide-in-from-left-4 duration-500">
              <h3 className="text-[10px] font-black text-muted-foreground uppercase tracking-[0.2em] mb-6 border-l-2 border-primary/20 pl-3">
                {section.title}
              </h3>
              <ul className="space-y-3">
                {section.items.map((item) => (
                  <li key={item.id}>
                    <button 
                      onClick={() => setActiveSection(item.id)}
                      className={`flex items-center space-x-3 text-sm font-bold w-full text-left px-4 py-3 rounded-2xl transition-all ${
                        activeSection === item.id 
                        ? 'bg-primary text-white shadow-xl shadow-primary/20 scale-[1.02]' 
                        : 'text-muted-foreground hover:text-white hover:bg-white/5'
                      }`}
                    >
                      <div className={activeSection === item.id ? 'text-white' : 'text-primary/60'}>
                        {item.icon}
                      </div>
                      <span className="tracking-tight">{item.name}</span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )) : (
            <div className="text-center py-10 opacity-30">
               <AlertCircle size={32} className="mx-auto mb-3" />
               <p className="text-[10px] font-black uppercase tracking-widest">{t.docs.no_results}</p>
            </div>
          )}
        </aside>

        <main className="flex-1 max-w-4xl pb-32 min-h-[60vh]">
          {isLoading ? (
            <div className="flex items-center gap-3 text-muted-foreground animate-pulse font-bold uppercase tracking-widest text-xs">
              <Zap size={16} className="animate-spin" /> {t.docs.loading}
            </div>
          ) : (
            <div className="animate-in fade-in slide-in-from-bottom-6 duration-700 markdown-content">
              <ReactMarkdown 
                remarkPlugins={[remarkGfm]}
                components={{
                  h1: ({node, ...props}) => <h1 className="text-4xl font-black uppercase tracking-tight mb-8 gradient-text" {...props} />,
                  h2: ({node, ...props}) => <h2 className="text-2xl font-bold mt-12 mb-6 border-b border-white/5 pb-2" {...props} />,
                  h3: ({node, ...props}) => <h3 className="text-xl font-bold mt-8 mb-4 text-primary" {...props} />,
                  p: ({node, ...props}) => <p className="text-muted-foreground leading-relaxed mb-6 font-medium" {...props} />,
                  code: ({node, inline, ...props}: any) => 
                    inline ? 
                    <code className="bg-white/10 px-1.5 py-0.5 rounded text-orange-400 font-bold" {...props} /> :
                    <div className="bg-black/40 border border-white/5 p-6 rounded-2xl font-mono text-sm mb-8 overflow-x-auto orange-glow">
                      <code className="text-orange-400" {...props} />
                    </div>,
                  ul: ({node, ...props}) => <ul className="list-disc list-inside space-y-3 mb-8 text-muted-foreground font-medium" {...props} />,
                  li: ({node, ...props}) => <li className="marker:text-primary" {...props} />,
                  strong: ({node, ...props}) => <strong className="text-white font-black" {...props} />,
                  blockquote: ({node, ...props}) => <blockquote className="border-l-4 border-primary/30 pl-6 italic my-8 text-muted-foreground" {...props} />
                }}
              >
                {markdownContent}
              </ReactMarkdown>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
