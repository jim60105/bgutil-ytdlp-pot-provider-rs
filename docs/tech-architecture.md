# 技術架構文件

## 專案概覽

BgUtils POT Provider Rust 版本是一個高效能、跨平台的 YouTube POT (Proof-of-Origin Token) 生成服務，旨在幫助 yt-dlp 使用者繞過 YouTube 的機器人檢查限制。

### 設計目標

1. **高效能**：相較於 TypeScript 版本，提供更快的 token 生成速度
2. **低資源消耗**：最佳化記憶體使用和 CPU 效率
3. **跨平台支援**：原生支援 Linux、Windows、macOS
4. **易於部署**：單一二進位文件，無額外執行時期依賴
5. **向後相容**：與現有 TypeScript 版本的 API 完全相容

### 核心功能

- **POT Token 生成**：實作完整的 BotGuard 挑戰解決流程
- **HTTP 伺服器模式**：提供 RESTful API 服務
- **腳本模式**：支援單次 token 生成
- **快取機制**：智慧型 token 快取以提升效能
- **錯誤處理**：完整的錯誤分類和恢復機制
- **設定管理**：彈性的設定檔案和命令列選項

## 整體架構

### 系統架構圖

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│                 │    │                  │    │                 │
│     yt-dlp      │───▶│  Python Plugin   │───▶│  Rust Provider  │
│                 │    │                  │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                │                        ▼
                                │               ┌─────────────────┐
                                │               │                 │
                                └──────────────▶│ HTTP API Server │
                                                │                 │
                                                └─────────────────┘
                                                         │
                                                         ▼
                                               ┌─────────────────┐
                                               │                 │
                                               │ Session Manager │
                                               │                 │
                                               └─────────────────┘
                                                         │
                                                         ▼
                                               ┌─────────────────┐
                                               │                 │
                                               │ BotGuard Engine │
                                               │                 │
                                               └─────────────────┘
```

### 架構層級

1. **應用層 (Application Layer)**
   - HTTP 伺服器 (`src/server/`)
   - CLI 介面 (`src/bin/`)
   - 設定管理 (`src/config/`)

2. **業務邏輯層 (Business Logic Layer)**
   - Session 管理 (`src/session/manager.rs`)
   - POT Token 生成邏輯 (`src/session/webpo_minter.rs`)
   - BotGuard 整合 (`src/session/botguard.rs`)

3. **基礎設施層 (Infrastructure Layer)**
   - 網路通訊 (`src/session/network.rs`)
   - 錯誤處理 (`src/error/`)
   - 類型定義 (`src/types/`)

### 資料流程

1. **請求接收**：HTTP API 或 CLI 接收 POT 請求
2. **Session 檢查**：檢查現有快取的有效 token
3. **BotGuard 挑戰**：若需要新 token，執行 BotGuard 挑戰
4. **Token 生成**：處理挑戰回應並生成 POT token
5. **快取儲存**：將新 token 儲存至快取
6. **回應傳送**：返回 token 給請求者

## 核心模組設計

### Session Manager (`src/session/manager.rs`)

Session Manager 是整個系統的核心，負責協調所有 POT token 相關操作。

**主要職責**：
- 管理 token 生命週期
- 協調不同類型的 token 請求 (GVS, Player, Subs)
- 實作快取策略
- 處理並行請求

**關鍵介面**：
```rust
impl SessionManager {
    /// 生成或取得快取的 POT token
    pub async fn get_pot_token(&self, request: PotRequest) -> Result<PotResponse, SessionError>;
    
    /// 清除快取的 token
    pub async fn invalidate_cache(&self, content_binding: &str) -> Result<(), SessionError>;
    
    /// 取得 session 統計資訊
    pub fn get_statistics(&self) -> SessionStatistics;
}
```

### BotGuard Engine (`src/session/botguard.rs`)

負責與 YouTube BotGuard 系統互動，執行 JavaScript 挑戰。

**技術實作**：
- 使用 `deno_core` JavaScript 引擎
- 實作 BotGuard challenge 解析
- 支援多種挑戰類型

**關鍵功能**：
```rust
impl BotGuardEngine {
    /// 執行 BotGuard 挑戰
    pub async fn solve_challenge(&self, challenge: Challenge) -> Result<Solution, BotGuardError>;
    
    /// 初始化 JavaScript 執行環境
    pub fn initialize_runtime(&mut self) -> Result<(), BotGuardError>;
}
```

### Network Layer (`src/session/network.rs`)

處理所有外部 HTTP 通訊，包括 Proxy 支援。

**功能特色**：
- 支援 HTTP/HTTPS/SOCKS 代理
- 連接重試機制
- 請求超時控制
- User-Agent 管理

### Configuration System (`src/config/`)

彈性的配置系統，支援檔案、環境變數、命令列參數。

**配置層級**（由高到低優先級）：
1. 命令列參數
2. 環境變數
3. 設定檔案
4. 預設值

### Error Handling (`src/error/`)

完整的錯誤分類和處理機制：

```rust
#[derive(Debug, thiserror::Error)]
pub enum PotProviderError {
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    
    #[error("BotGuard error: {0}")]
    BotGuard(#[from] BotGuardError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}
```

## 依賴庫選擇

### HTTP 伺服器框架

**選擇**：`axum` v0.8.4  
**理由**：
- 高效能異步 HTTP 框架
- 基於 `tokio` 生態系統
- 優秀的類型安全性
- 豐富的中間件支援

**配套依賴**：
- `tower` - 服務抽象層
- `tower-http` - HTTP 中間件（CORS、追蹤）

### 異步執行時期

**選擇**：`tokio` v1.0（full features）  
**理由**：
- Rust 異步程式設計標準
- 完整的異步 I/O 支援
- 豐富的工具生態系統

### CLI 框架

**選擇**：`clap` v4.5（derive features）  
**理由**：
- 強大的命令列解析能力
- 自動產生說明文件
- 支援子命令和參數驗證

### HTTP 客戶端

**選擇**：`reqwest` v0.12（json, stream, rustls-tls features）  
**理由**：
- 功能完整的 HTTP 客戶端
- 支援 rustls（純 Rust TLS 實作）
- 內建 JSON 序列化支援

### JavaScript 引擎

**選擇**：`deno_core` v0.240  
**理由**：
- 高效能 V8 JavaScript 引擎綁定
- 安全的沙盒執行環境
- 針對 BotGuard 挑戰最佳化

### 序列化

**選擇**：`serde` + `serde_json`  
**理由**：
- Rust 序列化標準
- 高效能 JSON 處理
- 豐富的派生巨集支援

### 錯誤處理

**選擇**：`thiserror` + `anyhow`  
**理由**：
- `thiserror`：定義結構化錯誤類型
- `anyhow`：動態錯誤處理和錯誤鏈追蹤

### 日誌系統

**選擇**：`tracing` + `tracing-subscriber`  
**理由**：
- 結構化日誌記錄
- 異步友善設計
- 豐富的訂閱者和過濾器

### 其他工具庫

- **時間處理**：`chrono` - 完整的日期時間處理
- **設定解析**：`toml` - TOML 格式設定檔案支援
- **目錄管理**：`dirs` - 跨平台目錄路徑取得
- **URL 解析**：`url` - URL 解析和驗證
- **Base64**：`base64` - Base64 編碼解碼

## 錯誤處理策略

### 錯誤分類架構

系統採用分層錯誤處理架構，每個模組定義專屬錯誤類型：

```rust
// 頂層錯誤類型
pub enum PotProviderError {
    Session(SessionError),
    BotGuard(BotGuardError),
    Network(NetworkError),
    Config(ConfigError),
    Validation(ValidationError),
}

// Session 層錯誤
pub enum SessionError {
    TokenExpired,
    CacheCorrupted,
    ConcurrencyLimitExceeded,
    InvalidRequest,
}

// BotGuard 層錯誤
pub enum BotGuardError {
    ChallengeParsingFailed,
    JavaScriptExecutionFailed,
    SolutionInvalid,
    TimeoutExceeded,
}
```

### 錯誤恢復機制

**1. 重試策略**
- 網路錯誤：指數退避重試（最多 3 次）
- BotGuard 挑戰失敗：立即重試一次，然後清除快取
- Token 過期：自動重新生成

**2. 降級服務**
- BotGuard 服務不可用時，返回快取 token（如果可用）
- 網路問題時，延長 token TTL

**3. 錯誤上報**
- 結構化錯誤日誌
- 錯誤統計和監控指標
- 使用者友善的錯誤訊息

### 錯誤處理最佳實務

**1. 快速失敗原則**
```rust
// 輸入驗證立即失敗
pub fn validate_request(request: &PotRequest) -> Result<(), ValidationError> {
    if request.visitor_data.is_empty() {
        return Err(ValidationError::MissingVisitorData);
    }
    // ... 其他驗證
    Ok(())
}
```

**2. 錯誤上下文保留**
```rust
// 使用 anyhow 保留錯誤鏈
pub async fn generate_token(request: PotRequest) -> Result<PotResponse, PotProviderError> {
    let challenge = fetch_challenge(&request)
        .await
        .context("Failed to fetch BotGuard challenge")?;
    
    let solution = solve_challenge(challenge)
        .await
        .context("Failed to solve BotGuard challenge")?;
    
    // ...
}
```

**3. 非同步錯誤處理**
```rust
// 使用 tokio::select! 處理超時
pub async fn generate_with_timeout(request: PotRequest) -> Result<PotResponse, SessionError> {
    tokio::select! {
        result = generate_token_internal(request) => result,
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            Err(SessionError::TimeoutExceeded)
        }
    }
}
```

## 效能考量

### 效能目標

**響應時間目標**：
- 快取命中：< 10ms
- 新 token 生成：< 2 秒
- BotGuard 挑戰解決：< 1.5 秒

**併發處理能力**：
- 同時處理請求：100+ 並行請求
- 記憶體使用：< 50MB（正常運作）
- CPU 使用：< 30%（單核心，正常負載）

### 最佳化策略

**1. Token 快取機制**
```rust
pub struct TokenCache {
    // 使用 DashMap 實現無鎖並行存取
    cache: DashMap<String, CachedToken>,
    // TTL 管理
    expiry_queue: Arc<tokio::sync::Mutex<VecDeque<ExpiryEntry>>>,
}

impl TokenCache {
    // O(1) 快取查找
    pub fn get(&self, key: &str) -> Option<PotResponse> {
        // 實作細節...
    }
    
    // 背景清理過期項目
    async fn cleanup_expired(&self) {
        // 實作細節...
    }
}
```

**2. 連接池化**
```rust
// 使用 reqwest 的內建連接池
pub struct NetworkClient {
    client: reqwest::Client,
}

impl NetworkClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }
}
```

**3. JavaScript 引擎最佳化**
```rust
// 複用 JavaScript 執行時期
pub struct BotGuardEngine {
    runtime: tokio::sync::Mutex<JsRuntime>,
}

impl BotGuardEngine {
    // 預編譯常用的 JavaScript 函式
    pub async fn precompile_functions(&mut self) -> Result<(), BotGuardError> {
        // 實作細節...
    }
}
```

### 記憶體管理

**1. 智慧型快取大小控制**
```rust
pub struct CacheConfig {
    max_entries: usize,           // 最大快取項目數
    max_memory_usage: usize,      // 最大記憶體使用（位元組）
    cleanup_interval: Duration,   // 清理間隔
}
```

**2. 零拷貝最佳化**
```rust
// 使用 Bytes 實現零拷貝
pub struct ResponseBuffer {
    data: bytes::Bytes,
}

// 使用 Arc 分享昂貴物件
pub struct SharedConfig {
    settings: Arc<Settings>,
}
```

### 並行處理

**1. 無鎖資料結構**
- 使用 `DashMap` 實現並行安全的快取
- 使用 `Arc<AtomicU64>` 實現統計計數器

**2. 異步工作負載平衡**
```rust
// 使用 tokio::spawn 處理 CPU 密集任務
pub async fn solve_challenge_async(challenge: Challenge) -> Result<Solution, BotGuardError> {
    let solution = tokio::task::spawn_blocking(move || {
        // CPU 密集的 BotGuard 解算
        solve_challenge_sync(challenge)
    }).await??;
    
    Ok(solution)
}
```

### 效能監控

**1. 內建指標收集**
```rust
pub struct PerformanceMetrics {
    pub cache_hit_rate: f64,
    pub average_response_time: Duration,
    pub active_sessions: u64,
    pub total_requests: u64,
    pub error_rate: f64,
}
```

**2. 效能分析工具整合**
- 支援 `tracing` 效能追蹤
- 內建記憶體使用統計
- HTTP 響應時間監控

## 測試策略

### 測試金字塔架構

**1. 單元測試（80%）**
- 每個模組的核心功能測試
- 錯誤處理路徑測試
- 邊界條件測試

**2. 整合測試（15%）**
- 模組間互動測試
- API 端點測試
- 設定載入測試

**3. 端到端測試（5%）**
- 完整工作流程測試
- 與 yt-dlp 整合測試
- 效能基準測試

### 測試覆蓋率目標

**目前狀態**：87.38% 覆蓋率，185 個測試通過  
**目標**：維持 85%+ 覆蓋率，重點關注：
- 錯誤處理路徑
- 邊界條件
- 並行安全性

### 測試工具和框架

**1. 核心測試框架**
```toml
[dev-dependencies]
tokio-test = "0.4"      # 異步測試支援
tower-test = "0.4"      # HTTP 服務測試
mockito = "1.7"         # HTTP 模擬
tempfile = "3.0"        # 臨時檔案測試
```

**2. 契約測試**
```rust
// 確保與 TypeScript 版本 API 相容性
#[cfg(test)]
mod contract_tests {
    use super::*;
    
    #[test]
    fn test_pot_response_schema_compatibility() {
        // 驗證回應格式與 TypeScript 版本相同
    }
    
    #[test]
    fn test_error_response_format() {
        // 驗證錯誤回應格式相容性
    }
}
```

**3. 效能測試**
```rust
// 注意：不實作真實的效能基準測試
// 因為核心功能涉及外部 API 呼叫
#[cfg(test)]
mod performance_tests {
    #[test]
    fn test_cache_performance() {
        // 只測試快取效能，不進行真實 API 呼叫
    }
}
```

### CI/CD 測試整合

**1. 自動化測試流程**
```yaml
# .github/workflows/build-test-audit-coverage.yml
- name: Run tests
  run: cargo nextest run --all-features
  
- name: Generate coverage report
  run: cargo llvm-cov --all-features --workspace --html
  
- name: Quality check
  run: ./scripts/quality_check.sh
```

**2. 多平台測試**
- Linux x86_64：主要測試平台
- Windows x86_64：相容性測試
- macOS（Intel/ARM）：相容性測試

### 測試最佳實務

**1. 測試資料工廠**
```rust
pub struct TestDataFactory;

impl TestDataFactory {
    pub fn create_pot_request() -> PotRequest {
        PotRequest::new()
            .with_visitor_data("test_visitor_123")
            .with_content_binding("test_video_456")
    }
    
    pub fn create_mock_challenge() -> Challenge {
        // 建立模擬 BotGuard 挑戰
    }
}
```

**2. 隔離測試環境**
```rust
#[cfg(test)]
mod test_helpers {
    use tempfile::TempDir;
    
    pub fn create_test_config() -> (Settings, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let mut settings = Settings::default();
        settings.cache.directory = temp_dir.path().to_path_buf();
        (settings, temp_dir)
    }
}
```

**3. 異步測試模式**
```rust
#[tokio::test]
async fn test_concurrent_token_generation() {
    let manager = SessionManager::new(test_config()).await;
    
    // 測試並行請求處理
    let futures: Vec<_> = (0..10)
        .map(|i| {
            let manager = manager.clone();
            tokio::spawn(async move {
                manager.get_pot_token(create_test_request(i)).await
            })
        })
        .collect();
    
    let results = futures::future::join_all(futures).await;
    // 驗證所有請求都成功完成
}
```

## 部署和發佈

### 1. 編譯目標
```toml
# 支援多平台編譯
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 2. CI/CD Pipeline
- **GitHub Actions** - 自動化建構、測試、覆蓋率檢查
- **Cross-compilation** - 支援 Linux、Windows、macOS
- **自動發佈** - 自動生成 GitHub Releases 和 crates.io 發佈

### 3. 發佈策略
- **GitHub Releases** - 預編譯二進位文件
- **crates.io** - Rust 套件發佈
- **安裝腳本** - 自動化安裝腳本 (`scripts/install.sh`)
- **Shell 完成** - 自動生成 bash/zsh/fish 完成腳本

## 品質保證

### 1. 程式碼品質
- **rustfmt** - 程式碼格式化
- **clippy** - 靜態分析和 linting
- **rustdoc** - 文件品質檢查
- **audit** - 安全漏洞掃描

### 2. 測試覆蓋率
- **llvm-cov** - 程式碼覆蓋率分析
- **codecov** - 覆蓋率報告和追蹤
- **並行測試** - 測試穩定性驗證

### 3. 程式碼品質和文件品質檢查
- **檢查腳本** - `scripts/quality_check.sh`
- **內連結驗證** - 確保文件連結有效
- **API 文件** - 完整的 rustdoc 文件

## 系統需求

### 最低需求
- **作業系統**: Linux (x86_64), Windows (x86_64), macOS (x86_64, ARM64)
- **記憶體**: 建議 4GB 以上
- **硬碟空間**: 100MB （不含快取和臨時檔案）

### 外部依賴

#### 執行時期依賴

**無額外執行時期依賴**：
- Rust 編譯為靜態連結的二進位文件
- 內嵌 JavaScript 引擎（deno_core）
- 使用 rustls 純 Rust TLS 實作

#### 網路依賴

**YouTube BotGuard API**：
- **用途**：取得 BotGuard 挑戰和驗證解答
- **端點**：`https://www.youtube.com/youtubei/v1/player`
- **認證**：無需 API 金鑰，使用模擬瀏覽器標頭

**Innertube API**：
- **用途**：YouTube 內部 API，用於取得 visitor data
- **端點**：`https://www.youtube.com/youtubei/v1/browse`
- **限制**：需要適當的 User-Agent 和 Client 資訊

#### 系統需求詳細規格

**最低硬體需求**：
- **CPU**：x86_64 或 ARM64 架構
- **記憶體**：512MB 可用記憶體
- **儲存空間**：50MB（二進位文件 + 快取）
- **網路**：穩定的網際網路連接

**支援的作業系統**：
- **Linux**：glibc 2.17+ (CentOS 7+, Ubuntu 16.04+)
- **Windows**：Windows 10+ (x86_64)
- **macOS**：macOS 10.15+ (Intel), macOS 11+ (Apple Silicon)

#### 可選依賴

**Proxy 支援**：
- HTTP/HTTPS Proxy
- SOCKS4/SOCKS5 Proxy
- 認證代理支援

**TLS 憑證**：
- 自動使用系統根憑證存放區
- 支援自訂 CA 憑證（通過環境變數）
