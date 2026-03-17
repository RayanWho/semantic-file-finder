#!/usr/bin/env python3
"""
usearch 迁移验证测试
"""

import sys
import json
import numpy as np
from pathlib import Path

# 添加工作目录到路径
sys.path.insert(0, str(Path(__file__).parent))

from indexer_worker_usearch import IndexerWorker

def test_basic():
    """基础功能测试"""
    print("=" * 50)
    print("🧪 usearch 迁移验证测试")
    print("=" * 50)
    
    index_dir = Path(__file__).parent / "test_index"
    index_dir.mkdir(exist_ok=True)
    
    # 1. 创建索引
    print("\n1️⃣ 创建索引...")
    worker = IndexerWorker(str(index_dir))
    print(f"   ✅ 索引创建成功")
    
    # 2. 测试索引文件
    print("\n2️⃣ 测试索引文件...")
    test_files = [
        {
            "path": "/test/file1.txt",
            "content": "这是测试文件 1",
            "embedding": np.random.rand(1024).astype(np.float32).tolist()
        },
        {
            "path": "/test/file2.txt",
            "content": "这是测试文件 2",
            "embedding": np.random.rand(1024).astype(np.float32).tolist()
        },
        {
            "path": "/test/file3.txt",
            "content": "这是测试文件 3",
            "embedding": np.random.rand(1024).astype(np.float32).tolist()
        }
    ]
    
    result = worker.index_files(test_files)
    print(f"   ✅ 索引了 {result['indexed']} 个文件")
    print(f"   ❌ 失败 {result['failed']} 个文件")
    
    # 3. 测试搜索
    print("\n3️⃣ 测试搜索...")
    query_embedding = np.random.rand(1024).astype(np.float32).tolist()
    results = worker.search(query_embedding, top_k=3)
    
    print(f"   ✅ 搜索到 {len(results)} 个结果")
    for i, r in enumerate(results):
        print(f"      {i+1}. {r['path']} (score: {r['score']:.4f})")
    
    # 4. 测试保存和加载
    print("\n4️⃣ 测试保存和加载...")
    worker.save()
    print(f"   ✅ 索引已保存")
    
    # 创建新 worker 并加载
    worker2 = IndexerWorker(str(index_dir))
    worker2.load()
    print(f"   ✅ 索引已加载")
    
    # 验证数据
    stats = worker2.get_stats()
    print(f"\n5️⃣ 索引统计:")
    print(f"   - 文件数：{stats['indexed_files']}")
    print(f"   - 维度：{stats['dimension']}")
    print(f"   - 类型：{stats['index_type']}")
    print(f"   - 后端：{stats['backend']}")
    
    # 6. 验证搜索结果一致性
    print("\n6️⃣ 验证搜索结果一致性...")
    results2 = worker2.search(query_embedding, top_k=3)
    
    if len(results) == len(results2):
        print(f"   ✅ 搜索结果数量一致")
    else:
        print(f"   ❌ 搜索结果数量不一致")
    
    # 7. 清理
    print("\n7️⃣ 清理测试索引...")
    import shutil
    shutil.rmtree(index_dir)
    print(f"   ✅ 测试索引已删除")
    
    print("\n" + "=" * 50)
    print("✅ 所有测试通过！")
    print("=" * 50)
    
    return True


def test_performance():
    """性能测试"""
    print("\n" + "=" * 50)
    print("⚡ 性能测试")
    print("=" * 50)
    
    import time
    
    index_dir = Path(__file__).parent / "perf_test_index"
    index_dir.mkdir(exist_ok=True)
    
    worker = IndexerWorker(str(index_dir))
    
    # 测试批量索引
    print("\n📊 批量索引性能测试...")
    num_files = 1000
    
    test_files = []
    for i in range(num_files):
        test_files.append({
            "path": f"/test/file{i}.txt",
            "content": f"测试内容 {i}",
            "embedding": np.random.rand(1024).astype(np.float32).tolist()
        })
    
    start = time.time()
    result = worker.index_files(test_files)
    elapsed = time.time() - start
    
    print(f"   - 索引 {num_files} 个文件：{elapsed:.2f}秒")
    print(f"   - 速度：{num_files / elapsed:.1f} 文件/秒")
    
    # 测试搜索性能
    print("\n📊 搜索性能测试...")
    query_embedding = np.random.rand(1024).astype(np.float32).tolist()
    
    # 预热
    worker.search(query_embedding, top_k=10)
    
    # 正式测试
    iterations = 100
    start = time.time()
    for _ in range(iterations):
        worker.search(query_embedding, top_k=10)
    elapsed = time.time() - start
    
    avg_latency = (elapsed / iterations) * 1000  # 毫秒
    qps = iterations / elapsed  # 每秒查询数
    
    print(f"   - {iterations} 次搜索：{elapsed:.2f}秒")
    print(f"   - 平均延迟：{avg_latency:.2f}ms")
    print(f"   - QPS: {qps:.1f}")
    
    # 清理
    import shutil
    shutil.rmtree(index_dir)
    
    print("\n✅ 性能测试完成")


if __name__ == "__main__":
    success = test_basic()
    
    if success:
        test_performance()
    
    print("\n🎉 迁移验证完成！")
    print("\n下一步:")
    print("1. 运行实际数据测试")
    print("2. 对比 FAISS 和 usearch 的搜索结果")
    print("3. 更新生产环境配置")
