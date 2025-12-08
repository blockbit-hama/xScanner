# xScanner Deployment Guide

## Overview

xScanner는 다중 블록체인을 모니터링하여 고객 주소로의 입금을 실시간으로 감지하고, AWS SQS를 통해 blockbit-back-custody에 알림을 전송하는 서비스입니다.

## Prerequisites

### 1. System Requirements
- **OS**: Linux (Ubuntu 20.04+ 권장) or macOS
- **CPU**: 2 cores 이상
- **RAM**: 4GB 이상
- **Disk**: 20GB 이상 (블록체인 데이터 증가에 따라 조정)

### 2. Software Dependencies
- **Rust**: 1.70+ (stable)
- **PostgreSQL**: 14+ (또는 Memory DB 사용)
- **RocksDB**: librocksdb 7.0+ (system library)

### 3. AWS Credentials
- **AWS Access Key ID** / **Secret Access Key** (SQS 사용 시)
- **SQS Queue URL**: 입금 알림을 전송할 SQS 큐

### 4. Blockchain RPC Endpoints
- Ethereum: Infura, Alchemy 등 (무료 tier 가능)
- Bitcoin: Blockchain.info API (무료)
- 기타 체인별 RPC 엔드포인트

---

## Installation

### Step 1: Clone Repository

```bash
git clone https://github.com/your-org/xScanner.git
cd xScanner
```

### Step 2: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Verify installation
```

### Step 3: Install System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y \
    librocksdb-dev \
    libssl-dev \
    pkg-config \
    postgresql-client
```

**macOS:**
```bash
brew install rocksdb postgresql
```

### Step 4: Setup PostgreSQL (Optional)

Memory DB를 사용하지 않는 경우:

```bash
# PostgreSQL 설치 및 시작
sudo systemctl start postgresql

# Database 생성
sudo -u postgres psql -c "CREATE DATABASE xscanner;"
sudo -u postgres psql -c "CREATE USER scanner WITH PASSWORD 'your_password';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE xscanner TO scanner;"
```

---

## Configuration

### 1. Create `config.toml`

```toml
# Blockchain configurations
[blockchain.ethereum]
api = "https://mainnet.infura.io/v3/YOUR_INFURA_KEY"
symbol = "eth"
start_block = 18000000  # Starting block number
interval_secs = 12      # Scan interval (Ethereum block time ~12s)
required_confirmations = 12  # Number of confirmations before DEPOSIT_CONFIRMED

[blockchain.bitcoin]
api = "https://blockchain.info"
symbol = "btc"
start_block = 800000
interval_secs = 600     # Bitcoin block time ~10 minutes
required_confirmations = 3

# Repository (Database) configurations
[repository]
memory_db = false  # Set to true for testing without PostgreSQL
postgresql_url = "postgres://scanner:your_password@localhost:5432/xscanner"
leveldb_path = "./customer_db"  # RocksDB path for customer address cache
customer_address_file = "./customer_addresses.txt"  # Not used if PostgreSQL is configured

# Notification (AWS SQS) configurations
[notification]
sqs_queue_url = "https://sqs.ap-northeast-2.amazonaws.com/123456789/deposit-events"
aws_region = "ap-northeast-2"
```

### 2. Set AWS Credentials

**Option A: Environment Variables**
```bash
export AWS_ACCESS_KEY_ID="your_access_key"
export AWS_SECRET_ACCESS_KEY="your_secret_key"
export AWS_REGION="ap-northeast-2"
```

**Option B: AWS Credentials File**
```bash
aws configure
# Follow prompts to enter credentials
```

### 3. Prepare Customer Addresses

고객 주소는 PostgreSQL의 `customer_addresses` 테이블에 저장:

```sql
INSERT INTO customer_addresses (address, customer_id, chain_name)
VALUES
    ('0x1234...abcd', 'customer_001', 'ETH'),
    ('1A1zP1...', 'customer_002', 'BTC');
```

---

## Build & Run

### Development Mode

```bash
# Check for errors
cargo check

# Run with logging
RUST_LOG=info cargo run

# Run tests
cargo test
```

### Production Build

```bash
# Build optimized binary
cargo build --release

# Binary location
./target/release/xScaner
```

### Run as Service

**systemd service file** (`/etc/systemd/system/xscanner.service`):

```ini
[Unit]
Description=xScanner - Multi-Blockchain Scanner
After=network.target postgresql.service

[Service]
Type=simple
User=scanner
WorkingDirectory=/opt/xscanner
Environment="RUST_LOG=info"
Environment="AWS_ACCESS_KEY_ID=your_key"
Environment="AWS_SECRET_ACCESS_KEY=your_secret"
ExecStart=/opt/xscanner/target/release/xScaner
Restart=on-failure
RestartSec=10s

[Install]
WantedBy=multi-user.target
```

**Start service:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable xscanner
sudo systemctl start xscanner
sudo systemctl status xscanner
```

---

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY .. .

# Install system dependencies
RUN apt-get update && apt-get install -y librocksdb-dev

# Build release binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    librocksdb7.8 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/xScaner /app/
COPY ../config.toml /app/

CMD ["./xScaner"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  xscanner:
    build: .
    container_name: xscanner
    restart: unless-stopped
    environment:
      - RUST_LOG=info
      - AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}
      - AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}
      - AWS_REGION=ap-northeast-2
    volumes:
      - ./config.toml:/app/config.toml:ro
      - ./customer_db:/app/customer_db
    depends_on:
      - postgres

  postgres:
    image: postgres:15-alpine
    container_name: xscanner-db
    restart: unless-stopped
    environment:
      POSTGRES_DB: xscanner
      POSTGRES_USER: scanner
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

volumes:
  postgres_data:
```

**Run with Docker:**
```bash
docker-compose up -d
docker-compose logs -f xscanner
```

---

## Monitoring

### Logs

**systemd logs:**
```bash
journalctl -u xscanner -f
```

**Docker logs:**
```bash
docker logs -f xscanner
```

### Key Log Messages

```
[INFO] Application starting...
[INFO] Configuration loaded.
[INFO] Using PostgreSQLRepository (memory_db = false)
[INFO] Opened RocksDB for customer address caching.
[INFO] Found 5 blockchain(s) to monitor
[INFO] ETH last processed block: 18123456, starting from block 18123457
[INFO] SQS Notifier initialized: https://sqs...
[INFO] [DEPOSIT_DETECTED] ✅ SQS notification sent
[INFO] [DEPOSIT_CONFIRMED] ✅ SQS notification sent
```

### Health Checks

PostgreSQL 사용 시:
```sql
-- Check last processed blocks
SELECT * FROM blockchain_state;

-- Check recent deposits
SELECT * FROM deposit_events ORDER BY created_at DESC LIMIT 10;

-- Check pending deposits (not yet confirmed)
SELECT * FROM deposit_events WHERE confirmed = FALSE;
```

---

## Scaling & Performance

### Single Instance Recommendations
- **Chains**: 최대 5-10개 동시 모니터링
- **Database Connection Pool**: 5-10 connections (default: 5)
- **RocksDB Cache**: 512MB-1GB

### Multi-Instance Deployment (Not Recommended)
⚠️ **주의**: xScanner는 stateful 서비스입니다.
- `last_processed_block`을 로컬 DB에 저장
- 여러 인스턴스 실행 시 블록 중복 처리 가능
- **권장**: Primary/Standby 구성 (leader election 필요)

---

## Troubleshooting

### 1. "Failed to connect to PostgreSQL"
```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Verify connection
psql -h localhost -U scanner -d xscanner
```

### 2. "SQS send failed"
```bash
# Verify AWS credentials
aws sts get-caller-identity

# Check SQS queue exists
aws sqs get-queue-attributes --queue-url YOUR_QUEUE_URL
```

### 3. "RocksDB open failed"
```bash
# Check directory permissions
ls -la ./customer_db

# Verify librocksdb is installed
ldconfig -p | grep rocksdb
```

### 4. "No customer addresses loaded"
- PostgreSQL 사용: `customer_addresses` 테이블에 데이터 삽입 확인
- Memory DB 사용: 현재는 하드코딩된 주소 없음 (개발 필요)

---

## Security Considerations

### 1. API Keys
- 환경 변수 또는 AWS Secrets Manager 사용
- `config.toml`에 API 키 하드코딩 금지

### 2. Database Access
- PostgreSQL: 강력한 비밀번호 사용
- 외부 접근 제한 (localhost only)

### 3. AWS Credentials
- IAM 역할 사용 (EC2/ECS에서 실행 시)
- Least privilege principle: SQS SendMessage만 허용

### 4. Network
- RPC 엔드포인트: HTTPS 사용
- Rate limiting 고려 (Infura 등 무료 tier 제한)

---

## Backup & Recovery

### Database Backup
```bash
# PostgreSQL backup
pg_dump -U scanner xscanner > backup_$(date +%Y%m%d).sql

# Restore
psql -U scanner xscanner < backup_20241208.sql
```

### RocksDB Backup
```bash
# Simple file copy (service stopped)
sudo systemctl stop xscanner
cp -r ./customer_db ./customer_db.backup
sudo systemctl start xscanner
```

---

## Upgrade Procedure

1. **Pull latest code**
   ```bash
   git pull origin main
   ```

2. **Backup database**
   ```bash
   pg_dump -U scanner xscanner > backup_before_upgrade.sql
   ```

3. **Build new version**
   ```bash
   cargo build --release
   ```

4. **Stop service**
   ```bash
   sudo systemctl stop xscanner
   ```

5. **Run database migrations** (if any)
   ```sql
   -- Example: Add new columns
   ALTER TABLE deposit_events ADD COLUMN status VARCHAR(20) DEFAULT 'PENDING';
   ```

6. **Start service**
   ```bash
   sudo systemctl start xscanner
   ```

7. **Verify logs**
   ```bash
   journalctl -u xscanner -f
   ```

---

## Support

- **Issues**: https://github.com/your-org/xScanner/issues
- **Docs**: `ARCHITECTURE.md` for system design
- **Contact**: dev-team@blockbit.com
