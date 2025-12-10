# Backend 연동 테스트 가이드

## 목표

Backend에서 계정을 생성하면 SQS를 통해 xScanner로 전달되고 RocksDB에 저장되는지 확인합니다.

## 사전 준비

### 1. xScanner 설정 확인

`config.toml`:
```toml
[blockchain.ethereum]
api = "https://sepolia.infura.io/v3/51d1d5bfaeab44fc87d77cf298d7c591"
symbol = "eth"
start_block = 9801775
interval_secs = 5
required_confirmations = 12

[customer_sync]
sqs_queue_url = "https://sqs.ap-northeast-2.amazonaws.com/YOUR_ACCOUNT_ID/customer-address-updates"
aws_region = "ap-northeast-2"
batch_size = 100
flush_interval_secs = 5
cache_file_path = "./customer_addresses_cache.json"
```

### 2. AWS Credentials 설정

```bash
export AWS_ACCESS_KEY_ID="your_access_key"
export AWS_SECRET_ACCESS_KEY="your_secret_key"
export AWS_REGION="ap-northeast-2"
```

또는 `~/.aws/credentials` 파일 사용

## 테스트 시나리오

### Phase 1: xScanner 시작 및 캐시 로드

```bash
cd /Users/hama/work/blockbit/blockbit/xScanner
cargo run
```

**예상 로그**:
```
[INFO] Application starting...
[INFO] Using MemoryRepository (memory_db = true)
[INFO] Opened RocksDB for customer address caching.
[INFO] [CustomerSync] Starting customer address sync service (batch_size: 100, flush_interval: 5s)
[INFO] [CustomerSync] Loading customer addresses from cache file: ./customer_addresses_cache.json
[INFO] ✅ [CustomerSync] Loaded 2 customer addresses from file
[INFO] [CustomerSync] SQS Consumer started, queue: https://sqs...
[INFO] [ConfirmationChecker] Starting with check_interval: 30s
[INFO] Ethereum scanner from block 9801775
```

**확인 사항**:
- ✅ RocksDB 캐시 파일에서 2개 주소 로드
- ✅ SQS Consumer 시작
- ✅ ConfirmationChecker 시작
- ✅ Ethereum 스캔 시작

### Phase 2: RocksDB 내용 확인

**별도 터미널**:
```bash
cargo run --example check_rocksdb -- ./dummy_db
```

**예상 출력**:
```
=== RocksDB Contents Check ===
Path: ./dummy_db

✅ Successfully opened RocksDB

=== Customer Addresses ===
Key: ethereum:0x0c32a378c0c5fa39710c140a8d1c7c21af3eebf2
Value: {"wallet_id":"cmiskt6ny0002pjgilw6ll2eo","account_id":"cmiskwus90006pjgi5309mjgn"}

Key: ethereum:0xe5d62c4a9ece7f3dcbfd07729f874f473d03185c
Value: {"wallet_id":"cmiskt6ny0002pjgilw6ll2eo","account_id":"cmiskwyeu0008pjgixssu8241"}

📊 Total customer addresses: 2
```

### Phase 3: Backend에서 계정 생성

Backend (TypeScript):
```typescript
// custody-wallet.service.ts

async createVirtualAccount(walletId: string, userId: string) {
  const address = await this.generateAddress(walletId);

  const account = await this.accountRepository.save({
    id: generateUUID(),
    walletId: walletId,
    address: address,
    chain: "sepolia",
    derivationPath: "m/0/2", // 다음 인덱스
  });

  // ⭐ SQS 메시지 발송
  await this.sqsClient.send(
    new SendMessageCommand({
      QueueUrl: process.env.SQS_CUSTOMER_ADDRESS_QUEUE_URL,
      MessageBody: JSON.stringify({
        event: "CustomerAddressAdded",
        address: address,
        chain: "ethereum", // "sepolia" 아님, "ethereum"으로 통일
        wallet_id: walletId,
        account_id: account.id,
        timestamp: new Date().toISOString(),
      }),
    })
  );

  console.log(`✅ Created virtual account: ${address}`);
  return account;
}
```

### Phase 4: xScanner에서 SQS 메시지 수신 확인

**xScanner 로그**:
```
[INFO] [CustomerSync] Received 1 SQS messages
[INFO] [CustomerSync] Buffered: 0xNEW_ADDRESS (chain: ethereum, wallet: cmiskt6ny0002pjgilw6ll2eo, account: Some("new_account_id")) | Buffer size: 1/100
[INFO] [CustomerSync] Flush interval reached, flushing 1 items...
[INFO] ✅ [CustomerSync] Flushed 1 monitored addresses to RocksDB cache
```

**확인 사항**:
- ✅ SQS 메시지 수신
- ✅ 버퍼에 추가
- ✅ RocksDB에 저장 (5초 후 또는 100개 도달 시)

### Phase 5: RocksDB 업데이트 확인

```bash
cargo run --example check_rocksdb -- ./dummy_db
```

**예상 출력** (3개로 증가):
```
=== Customer Addresses ===
Key: ethereum:0x0c32a378c0c5fa39710c140a8d1c7c21af3eebf2
Value: {"wallet_id":"cmiskt6ny0002pjgilw6ll2eo","account_id":"cmiskwus90006pjgi5309mjgn"}

Key: ethereum:0xe5d62c4a9ece7f3dcbfd07729f874f473d03185c
Value: {"wallet_id":"cmiskt6ny0002pjgilw6ll2eo","account_id":"cmiskwyeu0008pjgixssu8241"}

Key: ethereum:0x_new_address_here
Value: {"wallet_id":"cmiskt6ny0002pjgilw6ll2eo","account_id":"new_account_id"}

📊 Total customer addresses: 3
```

### Phase 6: 입금 테스트

새로 생성된 주소로 Sepolia ETH 전송:

```
From: Sepolia Faucet or existing wallet
To: 0xNEW_ADDRESS (Backend에서 방금 생성한 주소)
Amount: 0.01 ETH
```

**xScanner 로그**:
```
[INFO] [Analyzer] ✅ Deposit detected for account new_account_id
[INFO] [Analyzer] Address: 0xNEW_ADDRESS
[INFO] [Analyzer] Amount: 0.01 ETH
[INFO] [Analyzer] Tx: 0xabcd1234...
[INFO] [Analyzer] Block: 9801850
[INFO] [SQS] DEPOSIT_DETECTED sent

... (2.5분 후) ...

[INFO] [ConfirmationChecker] ✅ Deposit 0xabcd1234... reached 12 confirmations
[INFO] [SQS] DEPOSIT_CONFIRMED sent
```

## 테스트 체크리스트

### 초기 설정
- [ ] config.toml 설정 완료
- [ ] AWS credentials 설정
- [ ] customer_addresses_cache.json 생성 (초기 2개 주소)
- [ ] SQS Queue 생성 (customer-address-updates)

### xScanner 시작
- [ ] xScanner 실행 (`cargo run`)
- [ ] 캐시 파일에서 2개 주소 로드 확인
- [ ] SQS Consumer 시작 확인
- [ ] RocksDB에 2개 주소 저장 확인

### Backend 연동
- [ ] Backend에서 새 계정 생성
- [ ] SQS 메시지 발송 코드 작동 확인
- [ ] xScanner에서 SQS 메시지 수신 확인
- [ ] RocksDB에 새 주소 추가 확인 (3개로 증가)

### 입금 테스트
- [ ] 새 주소로 Sepolia ETH 전송
- [ ] DEPOSIT_DETECTED 로그 확인
- [ ] DEPOSIT_CONFIRMED 로그 확인 (12 블록 후)

## 트러블슈팅

### 문제 1: SQS 메시지가 수신되지 않음

**원인**:
- SQS Queue URL 잘못됨
- AWS credentials 없음
- Queue에 메시지가 안 들어감

**해결**:
```bash
# ElasticMQ (로컬 개발 환경)
# blockbit-custody의 docker-compose.yml에서 ElasticMQ 사용 중

# AWS CLI로 Queue 확인 (프로덕션)
aws sqs get-queue-attributes \
  --queue-url "https://sqs.ap-northeast-2.amazonaws.com/YOUR_ACCOUNT_ID/customer-address-updates" \
  --attribute-names All

# 수동으로 테스트 메시지 발송
aws sqs send-message \
  --queue-url "https://sqs..." \
  --message-body '{
    "event": "CustomerAddressAdded",
    "address": "0xTEST123",
    "chain": "ethereum",
    "wallet_id": "test",
    "account_id": "test",
    "timestamp": "2025-12-09T10:00:00Z"
  }'
```

### 문제 2: 메시지는 수신되지만 파싱 에러

**로그**:
```
[ERROR] [CustomerSync] Failed to parse SQS message: ...
```

**원인**: JSON 형식 잘못됨

**확인**:
- `event` 필드가 `"CustomerAddressAdded"`인지 (대소문자 정확히)
- `chain` 필드가 `"ethereum"` (소문자)
- `wallet_id`, `account_id` 필드 존재하는지

### 문제 3: RocksDB에 저장 안 됨

**로그**:
```
[ERROR] ❌ [CustomerSync] Failed to flush batch to RocksDB: ...
```

**원인**: RocksDB 경로 권한 문제

**해결**:
```bash
# 권한 확인
ls -la ./dummy_db

# 권한 수정
chmod -R 755 ./dummy_db

# 또는 새로 생성
rm -rf ./dummy_db
mkdir ./dummy_db
```

### 문제 4: 입금이 감지되지 않음

**원인**: RocksDB 키 형식 잘못됨

**확인**:
```bash
cargo run --example check_rocksdb -- ./dummy_db
```

**올바른 키 형식**:
- `ethereum:0x0c32a378...` (소문자, 콜론으로 구분)
- ❌ `ETHEREUM:0x0c32a378...`
- ❌ `ethereum_0x0c32a378...`

## Backend SQS 발송 코드 체크포인트

```typescript
// ✅ 올바른 예제
{
  "event": "CustomerAddressAdded",    // ✅ 정확한 이벤트 이름
  "address": "0x...",                 // ✅ 주소
  "chain": "ethereum",                // ✅ 소문자, 통일된 체인 이름
  "wallet_id": "uuid...",             // ✅ wallet_id (snake_case)
  "account_id": "uuid...",            // ✅ account_id (snake_case)
  "timestamp": "2025-12-09T10:00:00Z" // ✅ ISO 8601
}

// ❌ 잘못된 예제
{
  "event": "AddressAdded",            // ❌ 이벤트 이름 다름
  "address": "0x...",
  "chain": "ETHEREUM",                // ❌ 대문자
  "walletId": "uuid...",              // ❌ camelCase
  "accountId": "uuid...",             // ❌ camelCase
  "timestamp": "2025-12-09"           // ❌ 날짜만
}
```

## 성공 기준

- ✅ xScanner 시작 시 캐시 파일에서 주소 로드
- ✅ Backend에서 계정 생성 시 SQS 메시지 발송
- ✅ xScanner에서 SQS 메시지 수신
- ✅ RocksDB에 새 주소 추가
- ✅ 새 주소로 입금 시 DEPOSIT_DETECTED 발송
- ✅ 12 confirmations 후 DEPOSIT_CONFIRMED 발송

## 참고 문서

- `docs/SQS_MESSAGE_FORMAT.md` - SQS 메시지 형식 상세
- `docs/TESTING_SEPOLIA.md` - Sepolia 테스트 가이드
- `docs/ARCHITECTURE.md` - 전체 아키텍처 설명
