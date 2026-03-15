import React, { useState, FormEvent } from 'react';

interface SearchBoxProps {
  onSearch: (query: string) => void;
  loading: boolean;
  initialValue?: string;
}

export function SearchBox({ onSearch, loading, initialValue = '' }: SearchBoxProps) {
  const [query, setQuery] = useState(initialValue);

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (query.trim() && !loading) {
      onSearch(query.trim());
    }
  };

  return (
    <div className="search-box">
      <form onSubmit={handleSubmit}>
        <div className="search-input-wrapper">
          <input
            type="text"
            className="search-input"
            placeholder="描述你要找的文件内容，例如：'用户登录相关的代码文件'"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            disabled={loading}
          />
          <button
            type="submit"
            className="search-button"
            disabled={loading || !query.trim()}
          >
            {loading ? '搜索中...' : '🔍 搜索'}
          </button>
        </div>
      </form>
    </div>
  );
}
