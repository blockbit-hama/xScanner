# Changelog

All notable changes to xScanner will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **2-Stage Deposit Notification System**
  - Stage 1 (DEPOSIT_DETECTED): 1 confirmation 시 즉시 알림
  - Stage 2 (DEPOSIT_CONFIRMED): 충분한 confirmation 후 확정 알림
- **Confirmation-based Processing**: 체인별 `required_confirmations` 설정 추가
- **AWS SQS Integration**: blockbit-back-custody로 실시간 입금 알림 전송
- **Duplicate Prevention Logic**: 중복 알림 방지를 위한 DB 체크 로직
  - `deposit_exists()`: 입금 존재 여부 확인
  - `is_deposit_confirmed()`: 이미 confirmed된 입금 체크
  - `update_deposit_confirmed()`: confirmed 상태 업데이트
- **Multi-chain Support Expansion**: 7개 블록체인 추가
  - AION, ALGORAND, GXCHAIN, QUARK, TERRA, TEZOS, WAYKI
- **Configuration Enhancement**:
  - `required_confirmations` 체인별 설정 가능
  - `[notification]` 섹션 추가 (SQS 설정)

### Changed
- **Repository Layer Refactoring**: trait-based 아키텍처로 개선
  - PostgreSQL, Memory, RocksDB 모두 통합 인터페이스 제공
- **Analyzer Logic Optimization**: 중복 처리 방지 로직 추가
- **Configuration Structure**: 동적 체인 추가를 위한 `chains` HashMap 구조

### Fixed
- 중복 SQS 메시지 전송 문제 해결
- 같은 블록 재분석 시 중복 입금 이벤트 생성 문제 해결
- confirmation 수가 증가할 때마다 CONFIRMED 알림 중복 전송 문제 해결

### Security
- AWS credentials 환경 변수 지원
- PostgreSQL connection pooling (기본 5 connections)

---

## [0.1.0] - 2024-11-20

### Added
- **Initial Release**
- **Core Features**:
  - Multi-blockchain scanning (Ethereum, Bitcoin, TRON, THETA, ICON)
  - PostgreSQL database integration
  - RocksDB customer address caching
  - Memory database for testing
  - Asynchronous block processing with Tokio
- **Repository Pattern**: MemoryRepository, PostgreSQLRepository 구현
- **Fetcher Pattern**: 블록체인별 데이터 수집 모듈
- **Analyzer Pattern**: 트랜잭션 분석 및 고객 주소 매칭
- **Configuration**: TOML-based 설정 파일
- **Testing**: Integration tests for Ethereum and Bitcoin

### Architecture
- Fetcher → Analyzer 파이프라인 (mpsc 채널)
- LevelDB/RocksDB 고속 주소 캐싱
- PostgreSQL 데이터 영속성
- 체인별 독립적인 fetcher 실행

---

## Migration Guide

### Upgrading from 0.1.0 to Unreleased

#### 1. Configuration File Updates

**Before (0.1.0):**
```toml
[blockchain.ethereum]
api = "https://mainnet.infura.io/v3/YOUR_KEY"
symbol = "eth"
start_block = 18000000
interval_secs = 12
```

**After (Unreleased):**
```toml
[blockchain.ethereum]
api = "https://mainnet.infura.io/v3/YOUR_KEY"
symbol = "eth"
start_block = 18000000
interval_secs = 12
required_confirmations = 12  # ← 추가

[notification]  # ← 새 섹션
sqs_queue_url = "https://sqs.ap-northeast-2.amazonaws.com/YOUR_ACCOUNT/deposit-events"
aws_region = "ap-northeast-2"
```

#### 2. Database Schema Updates (Optional)

기존 `confirmed` boolean 컬럼은 그대로 사용 가능합니다.
향후 개선을 위해 다음 컬럼 추가를 권장합니다:

```sql
-- Optional: Add new columns for better tracking
ALTER TABLE deposit_events
ADD COLUMN status VARCHAR(20) DEFAULT 'PENDING',
ADD COLUMN confirmations INT DEFAULT 0,
ADD COLUMN detected_at TIMESTAMP DEFAULT NOW(),
ADD COLUMN confirmed_at TIMESTAMP;

-- Migrate existing data
UPDATE deposit_events SET status = 'CONFIRMED' WHERE confirmed = true;
UPDATE deposit_events SET status = 'PENDING' WHERE confirmed = false;
```

#### 3. AWS Credentials Setup

SQS 알림을 사용하려면 AWS credentials 설정 필요:

```bash
export AWS_ACCESS_KEY_ID="your_access_key"
export AWS_SECRET_ACCESS_KEY="your_secret_key"
export AWS_REGION="ap-northeast-2"
```

또는 `~/.aws/credentials` 파일 사용:
```
[default]
aws_access_key_id = your_access_key
aws_secret_access_key = your_secret_key
```

#### 4. Breaking Changes

**None** - 이번 업데이트는 backward compatible합니다.
- SQS 설정이 없으면 알림 없이 로컬 DB 저장만 수행
- `required_confirmations` 기본값: 12 (설정 안 하면 자동 적용)

---

## Roadmap

### v0.2.0 (Planned)
- [ ] Pending Deposits Monitoring (백그라운드 confirmation checker)
- [ ] Webhook support (SQS 대신 HTTP webhook)
- [ ] Prometheus metrics endpoint
- [ ] Admin API (health check, 통계 조회)

### v0.3.0 (Future)
- [ ] Reorg handling (chain reorganization 감지 및 복구)
- [ ] Multi-region deployment support
- [ ] WebSocket RPC support (폴링 대신 실시간 구독)
- [ ] Event sourcing & replay functionality

---

## Contributors

- **HAMA** - Initial development and architecture
- Community contributors - See GitHub contributors page

---

## License

Proprietary - BlockBit Internal Use Only
