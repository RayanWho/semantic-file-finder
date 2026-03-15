import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface IndexStatusData {
  is_indexing: boolean;
  indexed_files: number;
  last_update: string | null;
  index_size_mb: number;
}

export function IndexStatus() {
  const [status, setStatus] = useState<IndexStatusData | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchStatus = async () => {
    try {
      const result = await invoke('get_index_status');
      setStatus(result as IndexStatusData);
    } catch (err) {
      console.error('Failed to fetch index status:', err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    
    // 每 30 秒刷新一次状态
    const interval = setInterval(fetchStatus, 30000);
    return () => clearInterval(interval);
  }, []);

  const handleRefresh = async () => {
    setLoading(true);
    await fetchStatus();
  };

  if (loading) {
    return (
      <div className="index-status">
        <p>加载索引状态...</p>
      </div>
    );
  }

  return (
    <div className="index-status">
      <div className="status-header">
        <h3>📊 索引状态</h3>
        <button className="action-button" onClick={handleRefresh}>
          🔄 刷新
        </button>
      </div>
      
      <div className="status-grid">
        <div className="status-item">
          <span className="status-label">已索引文件</span>
          <span className="status-value">
            {status?.indexed_files || 0}
          </span>
        </div>
        
        <div className="status-item">
          <span className="status-label">索引大小</span>
          <span className="status-value">
            {status?.index_size_mb.toFixed(2) || 0} MB
          </span>
        </div>
        
        <div className="status-item">
          <span className="status-label">最后更新</span>
          <span className="status-value">
            {status?.last_update ? new Date(status.last_update).toLocaleString() : '从未'}
          </span>
        </div>
        
        <div className="status-item">
          <span className="status-label">状态</span>
          <span className={`status-value ${status?.is_indexing ? 'indexing' : 'ready'}`}>
            {status?.is_indexing ? '⏳ 索引中' : '✅ 就绪'}
          </span>
        </div>
      </div>
      
      <div className="status-actions">
        <button className="search-button" onClick={handleRefresh}>
          🔄 更新索引
        </button>
      </div>
    </div>
  );
}
