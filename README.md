# Blog Search Service

博客搜索服务 - 基于 Rust + Tantivy 实现的全文搜索 API

## 技术栈

- **后端框架**: Axum (异步 Web 框架)
- **搜索引擎**: Tantivy (Rust 实现的全文搜索引擎)
- **部署平台**: Vercel (Serverless 平台)
- **前端**: 原生 JavaScript + CSS Variables
- **构建工具**: Cargo (Rust 包管理器)

## 核心组件

### 搜索引擎核心 (src/lib.rs)
- **Tantivy**: 全文搜索引擎实现
  - Schema 定义
  - 文档索引
  - 查询解析
  - 结果排序
- **Serde**: JSON 序列化/反序列化
- **Thiserror**: 错误处理
- **Tokio**: 异步运行时

### API 服务 (src/main.rs)
- **Axum**: Web 框架
  - 路由处理
  - 中间件集成
  - 错误处理
- **Tower-http**: HTTP 中间件
  - CORS 支持
  - 请求追踪
  - 静态文件服务
- **Tracing**: 日志记录

### Serverless API (api/search.rs)
- **Vercel Runtime**: Serverless 函数支持
- **URL**: 查询参数解析
- **Headers**: CORS 配置
- **Error Handling**: 错误处理和响应

### 索引工具 (src/bin/indexer.rs)
- **Walkdir**: 文件系统遍历
- **TOML**: Front Matter 解析
- **Anyhow**: 错误处理
- **PathBuf**: 路径处理

### 前端界面 (static/test.html)
- **CSS Variables**: 主题定制
- **Flexbox**: 布局
- **Fetch API**: 数据请求
- **DOM API**: 动态内容
- **RegExp**: 搜索结果高亮

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
├── static/               # 静态资源
│   ├── test.html        # 搜索界面
│   └── images/          # 图片资源
├── content/             # 博客内容
│   └── blog/           # Markdown 文章
├── data/               # 数据目录
│   └── search_index/  # 搜索索引
└── scripts/           # 构建脚本
    └── build.sh      # Vercel 构建脚本
```

## 功能特性

### 搜索功能
- [x] 全文搜索
- [x] 标题匹配
- [x] 标签过滤
- [x] 结果高亮
- [x] 相关度排序
- [ ] 中文分词
- [ ] 搜索建议

### 用户界面
- [x] 响应式设计
- [x] 暗色模式
- [x] 实时搜索
- [x] 结果预览
- [ ] 加载动画
- [ ] 错误提示

### 部署特性
- [x] Serverless 部署
- [x] 静态资源托管
- [x] CORS 支持
- [ ] 缓存优化
- [ ] CDN 加速

## API 文档

### 搜索接口
```http
GET /api/search?q={query}&page={page}&size={size}
```

#### 请求参数
| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| q | string | 是 | 搜索关键词 |
| page | number | 否 | 页码(默认1) |
| size | number | 否 | 每页结果数(默认10) |

#### 响应格式
```typescript
interface SearchResponse {
    results: Array<{
        title: string;      // 文章标题
        path: string;       // 文章路径
        excerpt: string;    // 文章摘要
        tags: string[];     // 文章标签
    }>;
    total: number;         // 总结果数
    page: number;          // 当前页码
    size: number;          // 每页结果数
}
```

## 开发指南

### 环境配置
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Node.js
nvm install 18

# 安装 Vercel CLI
npm i -g vercel
```

### 本地开发
```bash
# 安装依赖
cargo build

# 运行索引器
cargo run --bin indexer

# 启动开发服务器
cargo run --bin server

# 运行测试
cargo test
```

### Vercel 部署
```bash
# 登录 Vercel
vercel login

# 部署项目
vercel --prod
```

## 性能优化

### 已实现
- [x] 异步请求处理
- [x] 静态文件缓存
- [x] 搜索结果限制
- [x] 内存映射索引

### 计划中
- [ ] 查询缓存
- [ ] 增量索引更新
- [ ] 压缩传输
- [ ] 预渲染结果

## 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

## 许可证

MIT

## 作者

千虑者 