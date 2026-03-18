# ZeroClaw API 接口文档

## 项目概述

ZeroClaw 是一个基于 Rust 的自主代理运行时系统，采用特质驱动架构，支持模块化扩展。项目核心设计理念是"零开销、零妥协"，支持在低资源硬件上运行。

## 核心架构模块

### 1. Agent 模块 (`src/agent/agent.rs`)
- **Agent**: 核心代理类，管理对话历史、工具调用和模型交互
- **AgentBuilder**: 构建器模式创建 Agent 实例
- **主要功能**:
  - `turn()`: 处理单轮对话
  - `run_single()`: 运行单次对话
  - `run_interactive()`: 交互式对话模式
  - `from_config()`: 从配置创建 Agent

### 2. Provider 接口 (`src/providers/traits.rs`)
- **Provider trait**: 大模型提供者抽象接口
- **核心数据结构**:
  - `ChatMessage`: 聊天消息结构体
  - `ChatRequest`: 聊天请求结构体
  - `ChatResponse`: 聊天响应结构体
  - `ToolCall`: 工具调用结构体
  - `TokenUsage`: Token 使用统计
- **主要方法**:
  - `chat_with_system()`: 带系统提示的聊天
  - `chat()`: 结构化聊天 API
  - `chat_with_tools()`: 带工具调用的聊天
  - `supports_native_tools()`: 是否支持原生工具调用
  - `warmup()`: 预热 HTTP 连接

### 3. 具体 Provider 实现

#### OpenAI Provider (`src/providers/openai.rs`)
- **支持特性**: 原生工具调用、温度调整、推理内容传递
- **特殊处理**: 针对特定模型（如 o1, o3, gpt-5 系列）强制 temperature=1.0
- **API 端点**: `https://api.openai.com/v1/chat/completions`

#### Anthropic Provider (`src/providers/anthropic.rs`)
- **支持特性**: 原生工具调用、视觉输入、提示缓存
- **认证方式**: API Key 或 OAuth Token
- **缓存策略**: 大系统提示自动缓存
- **API 端点**: `https://api.anthropic.com/v1/messages`

### 4. Gateway API (`src/gateway/api.rs`)
- **认证方式**: Bearer Token (配对机制)
- **主要端点**:

#### 系统状态端点
- `GET /api/status`: 系统状态概览
- `GET /api/health`: 健康检查
- `GET /api/metrics`: Prometheus 指标

#### 配置管理
- `GET /api/config`: 获取配置（敏感字段已屏蔽）
- `PUT /api/config`: 更新配置（支持 TOML 格式）

#### 工具管理
- `GET /api/tools`: 列出注册的工具规格

#### 定时任务管理
- `GET /api/cron`: 列出定时任务
- `POST /api/cron`: 添加新定时任务
- `DELETE /api/cron/{id}`: 删除定时任务
- `GET /api/cron/{id}/runs`: 查看任务运行记录

#### 集成管理
- `GET /api/integrations`: 列出所有集成状态
- `GET /api/integrations/settings`: 获取集成设置

#### 内存管理
- `GET /api/memory`: 列出或搜索内存条目
- `POST /api/memory`: 存储内存条目
- `DELETE /api/memory/{key}`: 删除内存条目

#### 成本管理
- `GET /api/cost`: 成本摘要

#### 诊断工具
- `POST /api/doctor`: 运行系统诊断

#### 会话管理
- `GET /api/sessions`: 列出网关会话
- `DELETE /api/sessions/{id}`: 删除会话

### 5. Webhook 端点 (`src/gateway/mod.rs`)
- **主要端点**:
  - `POST /webhook`: 通用 Webhook 端点
  - `GET/POST /whatsapp`: WhatsApp Webhook
  - `POST /linq`: Linq Webhook (iMessage/RCS/SMS)
  - `GET/POST /wati`: WATI Webhook
  - `POST /nextcloud-talk`: Nextcloud Talk Webhook

### 6. 配对管理
- `POST /pair`: 使用一次性代码配对
- `POST /api/pairing/initiate`: 发起配对
- `POST /api/pair`: 提交配对
- `GET /api/devices`: 列出设备
- `DELETE /api/devices/{id}`: 撤销设备
- `POST /api/devices/{id}/token/rotate`: 轮换令牌

### 7. Channel 接口 (`src/channels/traits.rs`)
- **Channel trait**: 消息通道抽象接口
- **核心数据结构**:
  - `ChannelMessage`: 通道消息结构体
  - `SendMessage`: 发送消息结构体
- **主要方法**:
  - `send()`: 发送消息
  - `listen()`: 监听消息
  - `health_check()`: 健康检查
  - `start_typing()`: 开始输入指示
  - `stop_typing()`: 停止输入指示

### 8. Tool 接口 (`src/tools/traits.rs`)
- **Tool trait**: 工具抽象接口
- **核心数据结构**:
  - `ToolResult`: 工具执行结果
  - `ToolSpec`: 工具规格描述
- **主要方法**:
  - `name()`: 工具名称
  - `description()`: 工具描述
  - `parameters_schema()`: 参数 JSON Schema
  - `execute()`: 执行工具

### 9. Memory 接口 (`src/memory/traits.rs`)
- **Memory trait**: 内存存储抽象接口
- **核心数据结构**:
  - `MemoryEntry`: 内存条目
  - `MemoryCategory`: 内存分类（Core, Daily, Conversation, Custom）
- **主要方法**:
  - `store()`: 存储内存
  - `recall()`: 召回内存
  - `get()`: 获取特定内存
  - `list()`: 列出内存
  - `forget()`: 删除内存

## API 认证机制

### 1. 配对系统
- **配对流程**: 使用一次性代码获取 Bearer Token
- **令牌存储**: 加密存储在配置文件中
- **速率限制**: 防止暴力破解

### 2. Webhook 认证
- **Bearer Token**: 标准 HTTP Bearer 认证
- **Webhook Secret**: 可选的 X-Webhook-Secret 头验证
- **平台特定签名**: WhatsApp、Linq、Nextcloud Talk 支持签名验证

### 3. 速率限制
- **滑动窗口算法**: 防止滥用
- **独立限制**: 配对和 Webhook 端点独立限制
- **客户端识别**: 基于 IP 地址或 X-Forwarded-For 头

## 数据模型

### 1. 聊天消息流
```
ChatMessage → ChatRequest → Provider → ChatResponse → ToolCall → ToolResult
```

### 2. 工具调用流程
```
Agent → Provider (with tools) → ToolCall → Tool.execute() → ToolResult → Agent
```

### 3. 内存存储结构
```
MemoryEntry {
    id: String,
    key: String,
    content: String,
    category: MemoryCategory,
    timestamp: String,
    session_id: Option<String>,
    score: Option<f64>
}
```

## 配置系统

### 1. 配置文件格式
- **格式**: TOML
- **位置**: `config.toml`
- **敏感字段**: API Key、令牌等敏感信息自动屏蔽

### 2. 配置更新流程
1. 客户端发送 TOML 配置
2. 服务端恢复屏蔽的敏感字段
3. 验证配置有效性
4. 保存到磁盘
5. 更新内存配置

## 扩展点

### 1. Provider 扩展
- 实现 `Provider` trait
- 注册到 provider factory
- 支持自定义 API 端点

### 2. Channel 扩展
- 实现 `Channel` trait
- 配置通道参数
- 支持 Webhook 或长连接

### 3. Tool 扩展
- 实现 `Tool` trait
- 定义参数 Schema
- 注册到工具注册表

### 4. Memory 扩展
- 实现 `Memory` trait
- 支持不同存储后端（SQLite、向量数据库等）

## 安全特性

### 1. 输入验证
- 请求体大小限制（64KB）
- 请求超时（30秒）
- JSON 解析验证

### 2. 输出过滤
- 敏感错误信息过滤
- API Key 屏蔽
- 安全头设置

### 3. 访问控制
- 本地管理端点限制
- 公共绑定保护
- 配对要求配置

## 性能特性

### 1. 内存优化
- 零拷贝设计
- 高效序列化
- 最小化分配

### 2. 并发处理
- 异步 I/O
- 并行工具执行
- 连接池复用

### 3. 缓存策略
- 响应缓存
- 提示缓存
- 连接预热

## 部署选项

### 1. 单机部署
- 直接运行二进制
- 系统服务安装
- Docker 容器

### 2. 网络部署
- 网关模式
- 隧道支持（Cloudflare、ngrok）
- 负载均衡

### 3. 边缘部署
- 低资源硬件支持
- ARM 架构优化
- 最小依赖

## 监控和运维

### 1. 健康检查
- `/health` 端点
- 组件健康状态
- 依赖服务检查

### 2. 指标收集
- Prometheus 格式指标
- 请求延迟统计
- Token 使用统计

### 3. 日志记录
- 结构化日志
- 不同日志级别
- 上下文跟踪

## 故障排除

### 1. 常见问题
- 配对失败
- Webhook 验证失败
- 内存不足

### 2. 诊断工具
- `zeroclaw doctor` 命令
- 通道健康检查
- 配置验证

### 3. 恢复流程
- 配置备份
- 会话恢复
- 故障转移

---

## 总结

ZeroClaw 提供了一个完整、模块化的自主代理运行时系统，具有以下特点：

1. **模块化设计**: 所有核心组件通过 trait 抽象，支持热插拔
2. **安全第一**: 多重认证机制、输入验证、敏感信息保护
3. **高性能**: Rust 原生性能、异步处理、内存优化
4. **可扩展**: 易于添加新的 Provider、Channel、Tool 和 Memory 后端
5. **运维友好**: 完整的监控、诊断和恢复工具

API 设计遵循 RESTful 原则，提供清晰的端点结构和一致的错误处理，适合构建企业级 AI 代理应用。