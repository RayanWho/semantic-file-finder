#!/usr/bin/env python3
"""
FAISS → usearch 迁移脚本
将现有的 FAISS 索引迁移到 usearch 格式
"""

import sys
import json
import pickle
from pathlib import Path
from typing import Optional

try:
    import faiss
    import numpy as np
    HAS_FAISS = True
except ImportError:
    HAS_FAISS = False
    print("Warning: faiss not installed. Cannot migrate existing index.")

try:
    import usearch.index as ui
    HAS_USEARCH = True
except ImportError:
    HAS_USEARCH = False
    print("Error: usearch not installed. Install with: pip install usearch")
    sys.exit(1)


def migrate_index(faiss_index_dir: str, usearch_index_dir: Optional[str] = None):
    """
    迁移 FAISS 索引到 usearch
    
    Args:
        faiss_index_dir: FAISS 索引目录
        usearch_index_dir: usearch 索引目录（默认与 FAISS 相同）
    """
    if not HAS_FAISS:
        print("❌ FAISS not available. Cannot migrate.")
        return
    
    if not HAS_USEARCH:
        print("❌ usearch not available. Install first.")
        return
    
    faiss_dir = Path(faiss_index_dir)
    usearch_dir = Path(usearch_index_dir) if usearch_index_dir else faiss_dir
    
    # 检查 FAISS 索引
    faiss_index_path = faiss_dir / "faiss.index"
    metadata_path = faiss_dir / "metadata.pkl"
    
    if not faiss_index_path.exists():
        print(f"❌ FAISS index not found at {faiss_index_path}")
        return
    
    if not metadata_path.exists():
        print(f"❌ Metadata not found at {metadata_path}")
        return
    
    print(f"📂 FAISS index: {faiss_index_path}")
    print(f"📂 Metadata: {metadata_path}")
    print(f"📂 Target: {usearch_dir}")
    
    # 加载 FAISS 索引
    print("\n⏳ Loading FAISS index...")
    faiss_index = faiss.read_index(str(faiss_index_path))
    
    with open(metadata_path, 'rb') as f:
        metadata = pickle.load(f)
        file_metadata = metadata["file_metadata"]
        file_paths = metadata["file_paths"]
        dimension = metadata.get("dimension", faiss_index.d)
    
    print(f"✅ Loaded {len(file_paths)} vectors (dim={dimension})")
    
    # 创建 usearch 索引
    print("\n⏳ Creating usearch index...")
    usearch_index = ui.Index(
        ndim=dimension,
        metric='cos',
        connectivity=16,
        expansion=64
    )
    
    # 从 FAISS 提取向量
    print("\n⏳ Extracting vectors from FAISS...")
    
    # FAISS IVF 索引需要特殊处理
    if hasattr(faiss_index, 'reconstruct'):
        # 可以直接重构向量
        vectors = []
        for i in range(len(file_paths)):
            try:
                vec = faiss_index.reconstruct(i)
                vectors.append(vec)
            except Exception as e:
                print(f"⚠️  Failed to reconstruct vector {i}: {e}")
                vectors.append(np.zeros(dimension, dtype=np.float32))
    else:
        # 对于其他索引，需要原始向量（无法迁移）
        print("❌ Cannot extract vectors from this FAISS index type.")
        print("   Please re-index all files with usearch.")
        return
    
    print(f"✅ Extracted {len(vectors)} vectors")
    
    # 添加到 usearch
    print("\n⏳ Adding vectors to usearch...")
    for i, (vec, path) in enumerate(zip(vectors, file_paths)):
        usearch_index.add(i, vec)
        if (i + 1) % 1000 == 0:
            print(f"   Added {i + 1}/{len(vectors)} vectors...")
    
    print(f"✅ Added {len(vectors)} vectors to usearch")
    
    # 保存 usearch 索引
    print("\n⏳ Saving usearch index...")
    usearch_index_path = usearch_dir / "usearch.index"
    usearch_index.save(str(usearch_index_path))
    
    # 保存元数据（更新 backend 标记）
    new_metadata_path = usearch_dir / "metadata.pkl"
    with open(new_metadata_path, 'wb') as f:
        pickle.dump({
            "file_metadata": file_metadata,
            "file_paths": file_paths,
            "dimension": dimension,
            "backend": "usearch",
            "migrated_from": "faiss"
        }, f)
    
    print(f"✅ usearch index saved to {usearch_index_path}")
    
    # 备份旧的 FAISS 索引
    backup_dir = faiss_dir / "faiss_backup"
    backup_dir.mkdir(exist_ok=True)
    
    print(f"\n📦 Backing up FAISS index to {backup_dir}...")
    faiss_index_path.rename(backup_dir / "faiss.index")
    metadata_path.rename(backup_dir / "metadata.pkl")
    
    print("\n✅ Migration complete!")
    print(f"\n📊 Statistics:")
    print(f"   - Indexed files: {len(file_paths)}")
    print(f"   - Dimension: {dimension}")
    print(f"   - Backend: usearch")
    print(f"   - FAISS backup: {backup_dir}")
    
    # 验证
    print("\n🔍 Verifying usearch index...")
    test_index = ui.Index.restore(str(usearch_index_path))
    print(f"   - Loaded: {test_index.size()} vectors")
    
    if len(file_paths) == test_index.size():
        print("✅ Verification passed!")
    else:
        print("⚠️  Verification mismatch!")


def main():
    """主函数"""
    if len(sys.argv) < 2:
        print("Usage: python migrate_to_usearch.py <index_dir> [usearch_index_dir]")
        print("\nExample:")
        print("  python migrate_to_usearch.py index")
        print("  python migrate_to_usearch.py index usearch_index")
        sys.exit(1)
    
    faiss_dir = sys.argv[1]
    usearch_dir = sys.argv[2] if len(sys.argv) > 2 else None
    
    migrate_index(faiss_dir, usearch_dir)


if __name__ == "__main__":
    main()
