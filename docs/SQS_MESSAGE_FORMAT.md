# SQS Message Format for Customer Address Sync

## Overview

Backend에서 xScanner로 고객 주소를 전달하기 위한 SQS 메시지 형식입니다.

## Queue Configuration

**Queue Name**: `customer-address-updates`

**Config (xScanner)**:
```toml
[customer_sync]
sqs_queue_url = "https://sqs.ap-northeast-2.amazonaws.com/YOUR_ACCOUNT_ID/customer-address-updates"
aws_region = "ap-northeast-2"
batch_size = 100
flush_interval_secs = 5
cache_file_path = "./customer_addresses_cache.json"
```

## Message Format

### Event: CustomerAddressAdded

Backend에서 새 고객 주소가 생성될 때 보내는 메시지:

```json
{
  "event": "CustomerAddressAdded",
  "address": "0x0C32A378c0c5FA39710C140A8d1C7c21Af3EeBf2",
  "chain": "ethereum",
  "wallet_id": "cmiskt6ny0002pjgilw6ll2eo",
  "account_id": "cmiskwus90006pjgi5309mjgn",
  "timestamp": "2025-12-09T10:30:00.000Z"
}
```

**필드 설명**:
- `event`: **"CustomerAddressAdded"** (고정값)
- `address`: 모니터링할 블록체인 주소
- `chain`: 블록체인 이름 (`"ethereum"`, `"bitcoin"`, `"tron"` 등)
- `wallet_id`: Custody Wallet UUID
- `account_id`: Virtual Account UUID (Omnibus인 경우 `null`)
- `timestamp`: ISO 8601 형식의 타임스탬프

### Omnibus (Master) Address Example

```json
{
  "event": "CustomerAddressAdded",
  "address": "0x1234567890123456789012345678901234567890",
  "chain": "ethereum",
  "wallet_id": "cmiskt6ny0002pjgilw6ll2eo",
  "account_id": null,
  "timestamp": "2025-12-09T10:30:00.000Z"
}
```

**account_id가 null인 경우**: Omnibus (Master) 주소로 인식됩니다.

## Backend Implementation Example (TypeScript)

### 1. SQS Client 초기화

```typescript
import { SQSClient, SendMessageCommand } from "@aws-sdk/client-sqs";

const sqsClient = new SQSClient({
  region: "ap-northeast-2",
});

const CUSTOMER_ADDRESS_QUEUE_URL =
  "https://sqs.ap-northeast-2.amazonaws.com/YOUR_ACCOUNT_ID/customer-address-updates";
```

### 2. 주소 생성 시 SQS 메시지 발송

```typescript
async function notifyAddressAdded(
  address: string,
  chain: string,
  walletId: string,
  accountId: string | null
) {
  const message = {
    event: "CustomerAddressAdded",
    address: address,
    chain: chain.toLowerCase(), // "ethereum", "bitcoin", etc.
    wallet_id: walletId,
    account_id: accountId, // null for Omnibus
    timestamp: new Date().toISOString(),
  };

  try {
    await sqsClient.send(
      new SendMessageCommand({
        QueueUrl: CUSTOMER_ADDRESS_QUEUE_URL,
        MessageBody: JSON.stringify(message),
      })
    );

    console.log(`✅ SQS notification sent for address: ${address}`);
  } catch (error) {
    console.error(`❌ Failed to send SQS notification:`, error);
    throw error;
  }
}
```

### 3. Virtual Account 생성 시 호출

```typescript
// custody-wallet.service.ts

async createVirtualAccount(walletId: string, userId: string) {
  // 1. Generate address
  const address = await this.generateAddress(walletId);

  // 2. Save to database
  const account = await this.accountRepository.save({
    id: generateUUID(),
    walletId: walletId,
    address: address,
    chain: "sepolia", // or "ethereum"
    userId: userId,
    derivationPath: "m/0/0",
  });

  // 3. Notify xScanner via SQS
  await notifyAddressAdded(
    address,
    "ethereum", // xScanner expects normalized chain name
    walletId,
    account.id // Virtual Account ID
  );

  return account;
}
```

### 4. Omnibus Address 생성 시 호출

```typescript
async createOmnibusAddress(walletId: string) {
  // 1. Generate master address
  const address = await this.generateMasterAddress(walletId);

  // 2. Save to database
  const wallet = await this.walletRepository.update(walletId, {
    omnibusAddress: address,
  });

  // 3. Notify xScanner via SQS (account_id = null for Omnibus)
  await notifyAddressAdded(
    address,
    "ethereum",
    walletId,
    null // Omnibus has no account_id
  );

  return wallet;
}
```

## xScanner Processing Flow

```
Backend: Customer Address Created
   ↓
Backend: INSERT INTO customer_addresses
   ↓
Backend: SQS.sendMessage("CustomerAddressAdded")
   ↓
xScanner: SQS Consumer receives message
   ↓
xScanner: Buffer message (batch_size: 100 or 5 seconds)
   ↓
xScanner: Flush batch to RocksDB
   Key: "ethereum:0x0c32a378..."
   Value: {"wallet_id": "...", "account_id": "..."}
   ↓
xScanner: Address now monitored in block scanning
```

## Testing SQS Integration

### Manual Test Message (AWS CLI)

```bash
aws sqs send-message \
  --queue-url "https://sqs.ap-northeast-2.amazonaws.com/YOUR_ACCOUNT_ID/customer-address-updates" \
  --message-body '{
    "event": "CustomerAddressAdded",
    "address": "0x0C32A378c0c5FA39710C140A8d1C7c21Af3EeBf2",
    "chain": "ethereum",
    "wallet_id": "test_wallet_001",
    "account_id": "test_account_001",
    "timestamp": "2025-12-09T10:30:00.000Z"
  }' \
  --region ap-northeast-2
```

### Expected xScanner Logs

```
[INFO] [CustomerSync] SQS Consumer started, queue: https://sqs...
[INFO] [CustomerSync] Received 1 SQS messages
[INFO] [CustomerSync] Buffered: 0x0C32A378c0c5FA39710C140A8d1C7c21Af3EeBf2 (chain: ethereum, wallet: test_wallet_001, account: Some("test_account_001")) | Buffer size: 1/100
[INFO] [CustomerSync] Flush interval reached, flushing 1 items...
[INFO] ✅ [CustomerSync] Flushed 1 monitored addresses to RocksDB cache
```

## Troubleshooting

### Issue 1: Messages not received

**Check**:
1. SQS Queue URL 정확한지 확인
2. AWS credentials 설정 확인
3. Queue에 메시지가 쌓여있는지 AWS Console에서 확인

### Issue 2: Messages received but not processed

**Check**:
1. Event type이 `"CustomerAddressAdded"`인지 확인 (대소문자 정확히)
2. JSON 형식이 올바른지 확인
3. xScanner 로그에서 파싱 에러 확인

### Issue 3: RocksDB에 저장 안 됨

**Check**:
1. `rocksdb` feature가 활성화되어 있는지 확인
2. RocksDB 경로 권한 확인
3. xScanner 로그에서 flush 에러 확인

## Cache File Format

xScanner 시작 시 로드할 초기 주소 목록 (downtime 대비):

**File**: `customer_addresses_cache.json`

```json
[
  {
    "address": "0x0C32A378c0c5FA39710C140A8d1C7c21Af3EeBf2",
    "chain": "ethereum",
    "wallet_id": "cmiskt6ny0002pjgilw6ll2eo",
    "account_id": "cmiskwus90006pjgi5309mjgn"
  },
  {
    "address": "0xe5d62C4A9eCE7F3Dcbfd07729F874F473D03185C",
    "chain": "ethereum",
    "wallet_id": "cmiskt6ny0002pjgilw6ll2eo",
    "account_id": "cmiskwyeu0008pjgixssu8241"
  }
]
```

**주의**: 배열 형식입니다 (객체가 아님)

## Performance Considerations

- **Batch Size**: 100개 단위로 RocksDB에 쓰기 (설정 가능)
- **Flush Interval**: 5초마다 버퍼 비우기 (작은 배치도 처리)
- **Long Polling**: SQS 20초 대기 (빈 응답 감소)
- **Max Messages**: 한 번에 최대 10개 메시지 수신

## Security

- AWS IAM Role로 SQS 접근 권한 관리
- 메시지 삭제는 처리 후에만 수행 (at-least-once delivery)
- 에러 발생 시 DLQ (Dead Letter Queue)로 이동 (권장)
