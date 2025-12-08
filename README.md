# xScanner

> **Multi-Blockchain Deposit Scanner with Real-time Notifications**

xScanner는 다중 블록체인을 모니터링하여 고객 주소로의 입금을 실시간으로 감지하고, AWS SQS를 통해 blockbit-back-custody에 알림을 전송하는 Rust 기반 스캐너 서비스입니다.

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-Proprietary-red.svg)]()

---

## 🌟 주요 기능

### Core Features
- ✅ **12개 블록체인 동시 모니터링**: Ethereum, Bitcoin, TRON, THETA, ICON, AION, ALGORAND, GXCHAIN, QUARK, TERRA, TEZOS, WAYKI
- ✅ **2-Stage Deposit Notification**:
  - **DEPOSIT_DETECTED** (1 confirmation) → 즉시 알림
  - **DEPOSIT_CONFIRMED** (충분한 confirmation) → 확정 알림
- ✅ **AWS SQS 실시간 알림**: blockbit-back-custody로 입금 이벤트 전송
- ✅ **중복 방지 로직**: 같은 입금에 대한 중복 알림 완벽 차단
- ✅ **고속 주소 매칭**: RocksDB 캐시로 ms 단위 조회
- ✅ **확장 가능한 아키텍처**: Fetcher-Analyzer 파이프라인

### Technical Highlights
- 🚀 **비동기 처리**: Tokio 기반 고성능 동시 블록 스캔
- 🔒 **안전한 Confirmation**: 체인별 맞춤 confirmation 수 (BTC: 3, ETH: 12, SOL: 40 등)
- 💾 **다중 저장소 지원**: PostgreSQL, RocksDB, Memory DB
- 📊 **상태 관리**: 재시작 시에도 마지막 처리 블록부터 자동 재개
- 🔧 **동적 설정**: TOML 기반 설정으로 재빌드 없이 체인 추가

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         xScanner                             │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐                │
│  │ Fetcher  │   │ Fetcher  │   │ Fetcher  │  ... (N개)     │
│  │  (ETH)   │   │  (BTC)   │   │  (TRON)  │                │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘                │
│       │              │              │                         │
│       └──────────────┴──────────────┘                        │
│                      │                                        │
│                      ▼ (mpsc channel)                        │
│              ┌───────────────┐                               │
│              │   Analyzer    │                               │
│              │ (매칭 & 처리)  │                               │
│              └───────┬───────┘                               │
│                      │                                        │
│       ┌──────────────┼──────────────┐                        │
│       │              │              │                         │
│       ▼              ▼              ▼                         │
│  ┌─────────┐   ┌─────────┐   ┌──────────┐                  │
│  │RocksDB  │   │  PG DB  │   │ AWS SQS  │                  │
│  │(cache)  │   │(persist)│   │(notify)  │                  │
│  └─────────┘   └─────────┘   └──────────┘                  │
└─────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
                        ┌──────────────────────┐
                        │ blockbit-back-custody │
                        └──────────────────────┘
```

**자세한 아키텍처**: [ARCHITECTURE.md](docs/ARCHITECTURE.md) 참조

---

## 🚀 Quick Start

### Prerequisites
- **Rust 1.70+** (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **PostgreSQL 14+** (또는 Memory DB 모드 사용)
- **RocksDB** (`brew install rocksdb` or `apt-get install librocksdb-dev`)
- **AWS Credentials** (SQS 사용 시)

### Installation

```bash
# 1. Clone repository
git clone https://github.com/your-org/xScanner.git
cd xScanner

# 2. Configure
cp config.toml.example config.toml
# Edit config.toml with your settings

# 3. Build
cargo build --release

# 4. Run
RUST_LOG=info ./target/release/xScaner
```

### Configuration Example

```toml
# config.toml
[blockchain.ethereum]
api = "https://mainnet.infura.io/v3/YOUR_API_KEY"
symbol = "eth"
start_block = 18000000
interval_secs = 12
required_confirmations = 12  # ETH standard

[blockchain.bitcoin]
api = "https://blockchain.info"
symbol = "btc"
start_block = 800000
interval_secs = 600
required_confirmations = 3   # BTC standard

[repository]
memory_db = false  # Set true for testing without PostgreSQL
postgresql_url = "postgres://user:pass@localhost/xscanner"
leveldb_path = "./customer_db"
customer_address_file = "./customer_addresses.txt"

[notification]
sqs_queue_url = "https://sqs.ap-northeast-2.amazonaws.com/123/deposit-events"
aws_region = "ap-northeast-2"
```

**배포 가이드**: [DEPLOYMENT.md](docs/DEPLOYMENT.md) 참조

---

## 📊 Supported Blockchains

| Blockchain | Symbol | Confirmations | Block Time | Status |
|------------|--------|--------------|------------|--------|
| Ethereum | ETH | 12 | ~12s | ✅ Fully Supported |
| Bitcoin | BTC | 3 | ~10m | ✅ Fully Supported |
| TRON | TRX | 19 | ~3s | ✅ Fully Supported |
| THETA | THETA | 12 | ~6s | ✅ Fully Supported |
| ICON | ICX | 1 | ~2s | ✅ Fully Supported |
| AION | AION | 12 | ~10s | ✅ Fully Supported |
| ALGORAND | ALGO | 12 | ~4.5s | ✅ Fully Supported |
| QUARK | QRK | 12 | ~30s | 🔶 Partial |
| GXCHAIN | GXC | - | - | 🔶 Placeholder |
| TERRA | LUNA | - | - | 🔶 Placeholder |
| TEZOS | XTZ | - | - | 🔶 Placeholder |
| WAYKI | WICC | - | - | 🔶 Placeholder |

**EVM 호환 체인 (추가 예정)**: Arbitrum, Optimism, Base, BNB Chain, Polygon, Avalanche

---

## 🔔 Deposit Notification Flow

### Stage 1: DEPOSIT_DETECTED (1 Confirmation)
```json
{
  "event": "DepositDetected",
  "customer_id": "customer_001",
  "address": "0x1234...abcd",
  "chain": "ETH",
  "tx_hash": "0xabcd...1234",
  "amount": "1000000000000000000",  // Wei
  "block_number": 18123456,
  "confirmations": 1
}
```

### Stage 2: DEPOSIT_CONFIRMED (12 Confirmations for ETH)
```json
{
  "event": "DepositConfirmed",
  "customer_id": "customer_001",
  "address": "0x1234...abcd",
  "chain": "ETH",
  "tx_hash": "0xabcd...1234",
  "amount": "1000000000000000000",
  "block_number": 18123456,
  "confirmations": 12
}
```

**SQS 메시지는 정확히 2번만 전송** (중복 방지 로직 적용)

---

## 🛠️ Development

### Project Structure
```
xScanner/
├── src/
│   ├── coin/            # 블록체인별 클라이언트 & 모델
│   │   ├── ethereum/
│   │   ├── bitcoin/
│   │   ├── tron/
│   │   └── ...
│   ├── fetcher/         # 블록 데이터 수집
│   │   ├── ethereum_fetcher.rs
│   │   ├── bitcoin_fetcher.rs
│   │   └── ...
│   ├── analyzer/        # 트랜잭션 분석 & 주소 매칭
│   ├── respository/     # 데이터 저장소 (trait-based)
│   │   ├── postgresql.rs
│   │   ├── rocksdb_repo.rs
│   │   └── memory.rs
│   ├── notification/    # AWS SQS 통합
│   ├── config.rs        # 설정 관리
│   └── main.rs
├── tests/               # Integration tests
├── config.toml          # Runtime configuration
├── ARCHITECTURE.md      # System design document
├── DEPLOYMENT.md        # Deployment guide
└── CHANGELOG.md         # Version history
```

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test ethereum_it
cargo test --test bitcoin_it

# All tests
cargo test
```

### Adding a New Blockchain

1. **Create coin module** (`src/coin/yourchain/`)
   ```rust
   // model.rs - Define block/transaction structures
   // client.rs - Implement RPC client
   // mod.rs - Export public interfaces
   ```

2. **Create fetcher** (`src/fetcher/yourchain_fetcher.rs`)
   ```rust
   impl Fetcher for YourChainFetcher {
       async fn fetch_next_block(&self, block_number: u64) -> Result<BlockData, AppError>;
   }
   ```

3. **Add to analyzer** (`src/analyzer/analyzer.rs`)
   ```rust
   BlockData::YourChain(block) => analyze_yourchain_block(...).await
   ```

4. **Update config** (`config.toml`)
   ```toml
   [blockchain.yourchain]
   api = "https://rpc.yourchain.io"
   symbol = "YRC"
   start_block = 1000000
   interval_secs = 5
   required_confirmations = 20
   ```

5. **Register in main** (`src/main.rs`)
   ```rust
   "yourchain" => {
       let client = Arc::new(YourChainClient::new(...));
       let fetcher = Arc::new(YourChainFetcher { client });
       spawn_fetcher(fetcher, sender_clone, start_block, interval_secs)
   }
   ```

---

## 📈 Performance

### Benchmarks (Single Instance)
- **Chains**: 5개 동시 모니터링 (ETH, BTC, TRON, THETA, ICON)
- **Throughput**: ~1000 blocks/min (체인별 상이)
- **Latency**:
  - Address lookup: <1ms (RocksDB cache)
  - DB write: ~10ms (PostgreSQL)
  - SQS send: ~50ms
- **Memory**: ~200MB (RocksDB cache 포함)

### Scalability Notes
- ⚠️ **Single instance only** (stateful service)
- Horizontal scaling 시 leader election 필요
- 권장: Primary/Standby 구성

---

## 🐛 Troubleshooting

### Common Issues

**1. "Failed to connect to PostgreSQL"**
```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Test connection
psql -h localhost -U scanner -d xscanner
```

**2. "SQS send failed: InvalidClientTokenId"**
```bash
# Verify AWS credentials
aws sts get-caller-identity

# Set credentials
export AWS_ACCESS_KEY_ID="your_key"
export AWS_SECRET_ACCESS_KEY="your_secret"
```

**3. "RocksDB open failed"**
```bash
# Check permissions
ls -la ./customer_db

# Recreate directory
rm -rf ./customer_db
mkdir ./customer_db
```

**4. "No deposits detected"**
- Verify `customer_addresses` table has data
- Check RocksDB cache loaded: 로그에서 "Loaded X customer addresses" 확인
- Ensure blockchain RPC endpoint is accessible

---

## 📚 Documentation

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design, data flow, components
- **[DEPLOYMENT.md](docs/DEPLOYMENT.md)** - Production deployment guide
- **[CHANGELOG.md](docs/CHANGELOG.md)** - Version history and migration guides
- **[API.md](./API.md)** - Internal API reference (if applicable)

---

## 🔐 Security

### Best Practices
- ✅ Use AWS IAM roles instead of access keys (when possible)
- ✅ Store API keys in environment variables or AWS Secrets Manager
- ✅ Restrict PostgreSQL access to localhost
- ✅ Use HTTPS for all RPC endpoints
- ✅ Regular dependency updates (`cargo update`)

### Reporting Security Issues
Please report security vulnerabilities to: **security@blockbit.com**

---

## 🤝 Contributing

This is a proprietary internal project. For BlockBit team members:

1. Create a feature branch (`git checkout -b feature/amazing-feature`)
2. Commit your changes (`git commit -m 'Add amazing feature'`)
3. Push to the branch (`git push origin feature/amazing-feature`)
4. Open a Pull Request

### Code Style
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add tests for new features

---

## 📝 License

**Proprietary - BlockBit Internal Use Only**

Copyright (c) 2024 BlockBit. All rights reserved.

---

## 👥 Authors

- **HAMA** - *Initial development* - [@hama](https://github.com/hama)

---

## 🙏 Acknowledgments

- [Tokio](https://tokio.rs/) - Async runtime
- [sqlx](https://github.com/launchbadge/sqlx) - PostgreSQL driver
- [RocksDB](https://rocksdb.org/) - Key-value storage
- [AWS SDK for Rust](https://github.com/awslabs/aws-sdk-rust) - SQS integration
- Blockchain RPC providers: Infura, Alchemy, Blockchain.info

---

## 📞 Support

- **Internal Slack**: `#xscanner-support`
- **Email**: dev-team@blockbit.com
- **Issues**: [GitHub Issues](https://github.com/your-org/xScanner/issues)

---

**Made with ❤️ by BlockBit Engineering Team**
