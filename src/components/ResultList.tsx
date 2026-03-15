import React from 'react';

interface SearchResult {
  path: string;
  score: number;
  summary: string;
  file_type: string;
  size: number;
  modified: string;
}

interface ResultListProps {
  results: SearchResult[];
  onOpenFile: (path: string) => void;
  onCopyPath: (path: string) => void;
}

export function ResultList({ results, onOpenFile, onCopyPath }: ResultListProps) {
  if (results.length === 0) {
    return (
      <div className="empty-state">
        <div className="empty-state-icon">📁</div>
        <p>暂无结果</p>
        <p>尝试搜索其他关键词</p>
      </div>
    );
  }

  return (
    <div className="result-list">
      {results.map((result, index) => (
        <div key={index} className="result-item">
          <div className="result-header">
            <code className="result-path">{result.path}</code>
            <span className="result-score">
              {(result.score * 100).toFixed(1)}% 匹配
            </span>
          </div>
          
          {result.summary && (
            <p className="result-summary">
              {result.summary.length > 300 
                ? result.summary.slice(0, 300) + '...' 
                : result.summary}
            </p>
          )}
          
          <div className="result-meta">
            <span>📄 {result.file_type || '未知'}</span>
            <span>💾 {(result.size / 1024).toFixed(1)} KB</span>
            <span>🕐 {result.modified || '未知'}</span>
          </div>
          
          <div className="result-actions">
            <button
              className="action-button"
              onClick={() => onOpenFile(result.path)}
            >
              📂 打开
            </button>
            <button
              className="action-button"
              onClick={() => onCopyPath(result.path)}
            >
              📋 复制路径
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}
