import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { SearchBox } from './components/SearchBox';
import { ResultList } from './components/ResultList';
import { IndexStatus } from './components/IndexStatus';
import { ConfigPanel } from './components/ConfigPanel';
import './App.css';

function App() {
  const [activeTab, setActiveTab] = useState<'search' | 'config'>('search');
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = async (searchQuery: string) => {
    setLoading(true);
    setError(null);
    setQuery(searchQuery);

    try {
      const response = await invoke('search_files', {
        request: {
          query: searchQuery,
          top_k: 10,
          threshold: 0.5,
        },
      });
      setResults(response.results || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
    } finally {
      setLoading(false);
    }
  };

  const handleOpenFile = async (path: string) => {
    try {
      await invoke('open_file', { path });
    } catch (err) {
      console.error('Failed to open file:', err);
    }
  };

  const handleCopyPath = async (path: string) => {
    try {
      await invoke('copy_path', { path });
      // TODO: 显示 toast 提示
    } catch (err) {
      console.error('Failed to copy path:', err);
    }
  };

  return (
    <div className="app">
      <header className="header">
        <h1>🔍 Semantic File Finder</h1>
        <nav className="nav">
          <button
            className={activeTab === 'search' ? 'active' : ''}
            onClick={() => setActiveTab('search')}
          >
            搜索
          </button>
          <button
            className={activeTab === 'config' ? 'active' : ''}
            onClick={() => setActiveTab('config')}
          >
            设置
          </button>
        </nav>
      </header>

      <main className="main">
        {activeTab === 'search' ? (
          <div className="search-page">
            <SearchBox
              onSearch={handleSearch}
              loading={loading}
              initialValue={query}
            />

            {error && (
              <div className="error-message">
                ❌ {error}
              </div>
            )}

            {loading ? (
              <div className="loading">
                <div className="spinner"></div>
                <p>搜索中...</p>
              </div>
            ) : (
              <ResultList
                results={results}
                onOpenFile={handleOpenFile}
                onCopyPath={handleCopyPath}
              />
            )}

            <IndexStatus />
          </div>
        ) : (
          <ConfigPanel />
        )}
      </main>

      <footer className="footer">
        <p>Semantic File Finder v0.1.0 | Powered by bge-m3 + FAISS</p>
      </footer>
    </div>
  );
}

export default App;
