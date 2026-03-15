import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AppConfig {
  default_directory: string | null;
  excluded_dirs: string[];
  top_k: number;
  threshold: number;
  file_types: string[];
}

export function ConfigPanel() {
  const [config, setConfig] = useState<AppConfig>({
    default_directory: null,
    excluded_dirs: ['.git', 'node_modules', '.DS_Store'],
    top_k: 10,
    threshold: 0.5,
    file_types: [],
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const result = await invoke('get_config');
      setConfig(result as AppConfig);
    } catch (err) {
      console.error('Failed to load config:', err);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    setSaving(true);
    try {
      await invoke('update_config', { config });
      alert('✅ 配置已保存');
    } catch (err) {
      alert('❌ 保存失败：' + (err as Error).message);
    } finally {
      setSaving(false);
    }
  };

  const handleSelectDirectory = async () => {
    try {
      // TODO: 使用 Tauri dialog 插件
      const dir = prompt('请输入默认搜索目录:');
      if (dir) {
        setConfig({ ...config, default_directory: dir });
      }
    } catch (err) {
      console.error('Failed to select directory:', err);
    }
  };

  if (loading) {
    return <div className="config-panel">加载配置...</div>;
  }

  return (
    <div className="config-panel">
      <h2>⚙️ 配置</h2>
      
      <div className="config-section">
        <h3>搜索设置</h3>
        
        <div className="config-item">
          <label>默认搜索目录</label>
          <div className="config-input-group">
            <input
              type="text"
              value={config.default_directory || ''}
              onChange={(e) => setConfig({ ...config, default_directory: e.target.value })}
              placeholder="留空使用当前目录"
            />
            <button className="action-button" onClick={handleSelectDirectory}>
              📁 选择
            </button>
          </div>
        </div>
        
        <div className="config-item">
          <label>返回结果数量 (Top K)</label>
          <input
            type="number"
            min="1"
            max="100"
            value={config.top_k}
            onChange={(e) => setConfig({ ...config, top_k: parseInt(e.target.value) || 10 })}
          />
        </div>
        
        <div className="config-item">
          <label>匹配阈值 ({config.threshold})</label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            value={config.threshold}
            onChange={(e) => setConfig({ ...config, threshold: parseFloat(e.target.value) })}
          />
          <small>低于此阈值的文件将不会显示</small>
        </div>
      </div>
      
      <div className="config-section">
        <h3>排除目录</h3>
        <div className="config-item">
          <textarea
            value={config.excluded_dirs.join('\n')}
            onChange={(e) => setConfig({ 
              ...config, 
              excluded_dirs: e.target.value.split('\n').filter(d => d.trim())
            })}
            placeholder="每行一个目录名"
            rows={5}
          />
          <small>这些目录将不会被索引</small>
        </div>
      </div>
      
      <div className="config-section">
        <h3>文件类型</h3>
        <div className="config-item">
          <textarea
            value={config.file_types.join('\n')}
            onChange={(e) => setConfig({ 
              ...config, 
              file_types: e.target.value.split('\n').filter(t => t.trim())
            })}
            placeholder=".md, .py, .js (留空表示不限制)"
            rows={3}
          />
          <small>只搜索指定类型的文件，留空表示不限制</small>
        </div>
      </div>
      
      <div className="config-actions">
        <button 
          className="search-button" 
          onClick={saveConfig}
          disabled={saving}
        >
          {saving ? '保存中...' : '💾 保存配置'}
        </button>
      </div>
    </div>
  );
}
