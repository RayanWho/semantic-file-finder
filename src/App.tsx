import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { SearchBox } from './components/SearchBox';
import { ResultList } from './components/ResultList';
import { IndexStatus } from './components/IndexStatus';
import { ConfigPanel, AppConfig } from './components/ConfigPanel';
import { WelcomePanel } from './components/WelcomePanel';
import './App.css';

const defaultConfig: AppConfig = {
  default_directory: null,
  excluded_dirs: ['.git', 'node_modules', '.DS_Store', '__pycache__', 'target'],
  top_k: 10,
  threshold: 0.5,
  file_types: [],
};

function App() {
  const [activeTab, setActiveTab] = useState<'search' | 'config'>('search');
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [config, setConfig] = React.useState<AppConfig>(defaultConfig);
  const [configLoading, setConfigLoading] = React.useState(true);
  const [configSaving, setConfigSaving] = React.useState(false);
  const [statusRefreshKey, setStatusRefreshKey] = React.useState(0);
  const [isIndexing, setIsIndexing] = React.useState(false);
  const [indexedFiles, setIndexedFiles] = React.useState(0);

  React.useEffect(() => {
    const loadConfig = async () => {
      try {
        const result = await invoke('get_config');
        setConfig(result as AppConfig);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load config');
      } finally {
        setConfigLoading(false);
      }
    };

    void loadConfig();
  }, []);

  React.useEffect(() => {
    const loadStatus = async () => {
      try {
        const result = await invoke<{ is_indexing: boolean; indexed_files: number }>('get_index_status');
        setIsIndexing(Boolean(result.is_indexing));
        setIndexedFiles(result.indexed_files ?? 0);
      } catch (err) {
        console.error('Failed to load index status', err);
      }
    };

    void loadStatus();
    const interval = setInterval(loadStatus, 5000);
    return () => clearInterval(interval);
  }, [statusRefreshKey]);

  const handleSearch = async (searchQuery: string) => {
    setLoading(true);
    setError(null);
    setNotice(null);
    setQuery(searchQuery);

    try {
      const response = await invoke<{ results: any[] }>('search_files', {
        request: {
          query: searchQuery,
          directory: config.default_directory,
          file_types: config.file_types,
          top_k: config.top_k,
          threshold: config.threshold,
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
      setNotice('路径已复制到剪贴板');
    } catch (err) {
      console.error('Failed to copy path:', err);
    }
  };

  const handleSelectDirectory = async () => {
    try {
      const selected = await invoke<string | null>('select_directory');
      if (selected) {
        setConfig((current) => ({ ...current, default_directory: selected }));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to select directory');
    }
  };

  const handleSaveConfig = async () => {
    setConfigSaving(true);
    setError(null);
    setNotice(null);
    try {
      await invoke('update_config', { config });
      setNotice('配置已保存');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save config');
    } finally {
      setConfigSaving(false);
    }
  };

  const handleStartIndexing = async () => {
    if (!config.default_directory) {
      setError('请先选择一个默认搜索目录');
      setActiveTab('config');
      return;
    }

    setError(null);
    setNotice(`开始为 ${config.default_directory} 建立索引`);
    try {
      await invoke('update_config', { config });
      await invoke('start_indexing', { directory: config.default_directory });
      setIsIndexing(true);
      setIndexedFiles(0);
      setStatusRefreshKey((key) => key + 1);
      setActiveTab('search');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start indexing');
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

            {(!config.default_directory || (indexedFiles === 0 && !isIndexing)) && (
              <WelcomePanel
                hasDirectory={Boolean(config.default_directory)}
                indexedFiles={indexedFiles}
                isIndexing={isIndexing}
                onOpenSettings={() => setActiveTab('config')}
                onStartIndexing={handleStartIndexing}
              />
            )}

            {notice && (
              <div className="notice-message">
                {notice}
              </div>
            )}

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

            <IndexStatus
              refreshKey={statusRefreshKey}
              onStartIndexing={handleStartIndexing}
            />
          </div>
        ) : (
          <ConfigPanel
            config={config}
            loading={configLoading}
            saving={configSaving}
            indexing={isIndexing}
            onConfigChange={setConfig}
            onSave={handleSaveConfig}
            onSelectDirectory={handleSelectDirectory}
            onStartIndexing={handleStartIndexing}
          />
        )}
      </main>

      <footer className="footer">
        <p>Semantic File Finder v0.1.0 | Powered by bge-m3 + USearch</p>
      </footer>
    </div>
  );
}

export default App;
