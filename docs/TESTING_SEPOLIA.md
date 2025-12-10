# Sepolia Testnet 테스트 가이드

## 🧪 테스트 환경 설정

### 1. Config 설정 확인

`config.toml`이 Sepolia로 설정되어 있는지 확인:

```toml
[blockchain.ethereum]
api = "https://sepolia.infura.io/v3/51d1d5bfaeab44fc87d77cf298d7c591"
symbol = "eth"
start_block = 9801775  # Current Sepolia block
interval_secs = 5
required_confirmations = 12  # 빠른 테스트를 위해 3으로 줄일 수 있음
```

### 2. 모니터링할 주소 등록

#### 방법 A: customer_addresses_cache.json 사용 (추천)

파일 위치: `./customer_addresses_cache.json`

```json
{
  "ethereum": [
    {
      "address": "0xYourSepoliaTestAddress",
      "wallet_id": "test_wallet_001",
      "account_id": "test_account_001",
      "chain": "ethereum"
    }
  ]
}
```

**필드 설명**:
- `address`: 모니터링할 Sepolia 주소
- `wallet_id`: Custody Wallet ID (테스트용 임의 값)
- `account_id`: Virtual Account ID (Omnibus는 `null`, 일반 계정은 문자열)
- `chain`: 체인 이름 (`"ethereum"` 고정)

#### 방법 B: RocksDB에 직접 추가 (고급)

xScanner 실행 후 RocksDB에 수동으로 키-값 추가:

```
Key: "ethereum:0xyouraddress"
Value: {"wallet_id": "test_wallet_001", "account_id": "test_account_001"}
```

### 3. 테스트용 Sepolia ETH 받기

Sepolia Faucet에서 테스트용 ETH 받기:
- https://sepoliafaucet.com/
- https://www.alchemy.com/faucets/ethereum-sepolia

### 4. xScanner 실행

```bash
cargo run
```

**실행 로그 확인**:
```
[INFO] Application starting...
[INFO] Using MemoryRepository (memory_db = true)
[INFO] Starting customer address sync service...
[INFO] [ConfirmationChecker] Starting with check_interval: 30s
[INFO] Ethereum scanner from block 9801775
```

### 5. 테스트 입금 보내기

모니터링 중인 주소로 Sepolia ETH 전송:

```
From: Your Sepolia wallet
To: 0xYourSepoliaTestAddress (customer_addresses_cache.json에 등록한 주소)
Amount: 0.01 ETH (소량)
```

### 6. 예상 로그 출력

#### Stage 1: DEPOSIT_DETECTED (1 confirmation)

```
[INFO] [Analyzer] ✅ Deposit detected for customer test_account_001
[INFO] [Analyzer] Address: 0xYourSepoliaTestAddress
[INFO] [Analyzer] Amount: 0.01 ETH
[INFO] [Analyzer] Tx: 0xabcd1234...
[INFO] [Analyzer] Block: 9801850
[INFO] [SQS] DEPOSIT_DETECTED sent
```

#### Stage 2: DEPOSIT_CONFIRMED (12 confirmations)

약 2.5분 후 (Sepolia는 ~12초/블록):

```
[INFO] [ConfirmationChecker] Checking 1 pending deposits
[INFO] [ConfirmationChecker] Deposit 0xabcd1234... on ethereum - confirmations: 12/12
[INFO] [ConfirmationChecker] ✅ Deposit 0xabcd1234... reached 12 confirmations, sending DEPOSIT_CONFIRMED
[INFO] [ConfirmationChecker] ✅ SQS DEPOSIT_CONFIRMED sent for 0xabcd1234...
```

## 🔧 트러블슈팅

### 문제 1: 입금이 감지되지 않음

**원인**: 주소가 RocksDB 캐시에 없음

**해결**:
1. `customer_addresses_cache.json` 파일 확인
2. xScanner 재시작 (캐시 파일 로드)
3. 로그에서 "loaded N addresses" 확인

### 문제 2: DEPOSIT_CONFIRMED가 발송되지 않음

**원인**: confirmation_checker가 비활성화됨

**해결**:
```toml
[confirmation_checker]
enabled = true
check_interval_secs = 30
```

### 문제 3: 블록 스캔이 느림

**원인**: start_block이 너무 과거

**해결**:
```bash
# 현재 Sepolia 블록 확인
curl -X POST https://sepolia.infura.io/v3/YOUR_API_KEY \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# config.toml의 start_block을 현재 블록으로 업데이트
```

## 📊 테스트 시나리오

### 시나리오 1: 단일 입금 테스트

1. 주소 1개 등록
2. 0.01 ETH 전송
3. DEPOSIT_DETECTED 확인 (즉시)
4. DEPOSIT_CONFIRMED 확인 (~2.5분 후)

### 시나리오 2: 다중 입금 테스트

1. 주소 3개 등록
2. 각 주소에 0.01 ETH 전송 (동일 블록)
3. 3개의 DEPOSIT_DETECTED 확인
4. 3개의 DEPOSIT_CONFIRMED 확인

### 시나리오 3: Confirmation 임계값 변경 테스트

```toml
required_confirmations = 3  # 12에서 3으로 변경
```

- 확정 시간: ~2.5분 → ~36초

## 🎯 성공 기준

- ✅ Sepolia 블록 스캔 시작
- ✅ 입금 트랜잭션 감지 (1 confirmation)
- ✅ DB에 `confirmed=FALSE` 저장
- ✅ SQS DEPOSIT_DETECTED 발송
- ✅ 12 confirmations 후 `confirmed=TRUE` 업데이트
- ✅ SQS DEPOSIT_CONFIRMED 발송

## 📝 테스트 체크리스트

- [ ] config.toml Sepolia 설정 완료
- [ ] customer_addresses_cache.json 주소 등록
- [ ] Sepolia Faucet에서 ETH 수령
- [ ] xScanner 실행 확인
- [ ] 테스트 입금 전송
- [ ] DEPOSIT_DETECTED 로그 확인
- [ ] DEPOSIT_CONFIRMED 로그 확인 (12 블록 후)
- [ ] DB에서 confirmed=TRUE 확인

## 🔗 유용한 링크

- Sepolia Etherscan: https://sepolia.etherscan.io/
- Sepolia Faucet: https://sepoliafaucet.com/
- Infura Sepolia Endpoint: https://sepolia.infura.io/v3/YOUR_API_KEY
