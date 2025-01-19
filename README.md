# Blog Search Service

博客搜索服务 - 基于 Rust + Tantivy 实现的全文搜索 API

## 项目结构

```
.
├── Cargo.toml              # 项目依赖配置
├── src/
│   ├── lib.rs             # 搜索引擎核心实现
│   ├── main.rs            # API 服务入口
│   └── bin/
│       └── indexer.rs     # 索引构建工具
├── api/                   # Vercel Serverless Functions
│   └── search.rs         # 搜索API实现
├── data/                 # 索引数据目录
│   └── search_index/    # 搜索索引文件
├── .github/             # GitHub Actions 配置
│   └── workflows/
│       └── index.yml   # 索引更新工作流
└── tests/              # 测试目录
    └── integration/   # 集成测试
```

## 核心文件说明

### src/lib.rs
搜索引擎核心实现，包含：
- `SearchEngine` 结构体：搜索引擎主体
- 文档索引功能
- 搜索查询实现
- 结果处理逻辑

### src/main.rs
API服务实现，包含：
- HTTP服务器配置
- CORS 中间件
- 路由处理
- 错误处理

### api/search.rs
Vercel Serverless Function 实现：
- 请求参数解析
- 搜索引擎调用
- 响应格式化

## API 接口

### 搜索接口
```
GET /api/search?q={query}
```

请求参数：
- `q`: 搜索关键词（必填）
- `page`: 页码（可选，默认1）
- `size`: 每页结果数（可选，默认10）

响应格式:
```json
{
    "results": [
        {
            "title": "文章标题",
            "path": "/blog/article-1",
            "excerpt": "文章摘要...",
            "tags": ["标签1", "标签2"]
        }
    ],
    "total": 100,
    "page": 1,
    "size": 10
}
```

## 部署说明

1. 克隆仓库
```bash
git clone https://github.com/your-username/blog-search-service.git
cd blog-search-service
```

2. 安装依赖
```bash
cargo build
```

3. 本地运行
```bash
cargo run
```

4. Vercel 部署
```bash
vercel deploy
```

## 开发指南

### 环境要求
- Rust 1.70+
- Node.js 18+ (用于Vercel CLI)
- Git

### 本地开发
1. 启动开发服务器
```bash
cargo watch -x run
```

2. 构建索引
```bash
cargo run --bin indexer
```

3. 运行测试
```bash
cargo test
```

## 配置说明

### 环境变量
- `BLOG_URL`: 博客网站URL
- `RUST_LOG`: 日志级别
- `INDEX_PATH`: 索引文件路径

### Vercel配置
见 `vercel.json` 文件

## TODO 待办事项

### 功能增强
- [ ] 添加中文分词支持
- [ ] 实现搜索结果高亮
- [ ] 添加搜索建议功能
- [ ] 支持高级搜索语法
- [ ] 实现相关文章推荐

### 性能优化
- [ ] 添加搜索结果缓存
- [ ] 优化索引更新策略
- [ ] 实现增量索引更新
- [ ] 添加性能监控

### 开发体验
- [ ] 完善开发文档
- [ ] 添加更多单元测试
- [ ] 设置CI/CD流水线
- [ ] 添加API文档生成

### 部署优化
- [ ] 设置自动备份
- [ ] 添加健康检查
- [ ] 实现多区域部署
- [ ] 优化资源使用

## 贡献指南

1. Fork 项目
2. 创建特性分支
3. 提交改动
4. 发起 Pull Request

## 许可证

MIT

## 作者

千虑者 