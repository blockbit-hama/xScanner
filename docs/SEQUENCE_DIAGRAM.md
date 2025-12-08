# xScanner Sequence Diagrams

## 1. 주소 동기화 플로우 (Address Synchronization)

Backend에서 새 주소가 추가되면 xScanner가 실시간으로 동기화합니다.

```mermaid
sequenceDiagram
    participant Backend as Backend (blockbit-back-custody)
    participant SQS as AWS SQS Queue
    participant Sync as xScanner<br/>CustomerAddressSync
    participant RocksDB as RocksDB Cache
    participant File as customer_addresses<br/>_cache.json

    Note over Backend,File: === 신규 주소 추가 시나리오 ===

    Backend->>Backend: 고객 Virtual Account 생성<br/>(wallet_id, account_id)
    Backend->>SQS: AddressAdded 메시지 발송<br/>{address, wallet_id, account_id, chain}

    Note over SQS: SQS Queue에 메시지 저장

    Sync->>SQS: Long Polling (20초)
    SQS-->>Sync: AddressAdded 메시지 수신

    Sync->>Sync: 배치 버퍼에 추가<br/>(100개 or 5초 간격)

    Note over Sync: 배치 조건 충족 시

    Sync->>RocksDB: batch_add_monitored_addresses()<br/>Key: "eth:0x123..."<br/>Value: {"wallet_id": "...", "account_id": "..."}

    RocksDB-->>Sync: ✅ 저장 완료

    Note over Backend,File: === 재시작 대비 (Downtime Recovery) ===

    Backend->>File: 주기적으로 전체 주소 목록<br/>JSON 파일로 export

    Note over Sync: xScanner 재시작 시

    Sync->>File: 파일에서 주소 목록 로드
    File-->>Sync: 전체 주소 목록 반환
    Sync->>RocksDB: 대량 배치 삽입
```

## 2. 입금 감지 플로우 (Deposit Detection)

블록체인에서 입금을 감지하고 2단계로 알림을 보내는 과정입니다.

```mermaid
sequenceDiagram
    participant Chain as Blockchain<br/>(Ethereum/Bitcoin/TRON...)
    participant Fetcher as Fetcher
    participant Channel as mpsc Channel
    participant Analyzer as Analyzer
    participant RocksDB as RocksDB Cache
    participant Repo as Repository<br/>(PostgreSQL)
    participant SQS as AWS SQS Queue
    participant Backend as Backend

    Note over Chain,Backend: === 블록 스캔 루프 ===

    loop Every interval_secs (e.g., 5초)
        Fetcher->>Repo: get_last_processed_block("ETH")
        Repo-->>Fetcher: 마지막 블록 번호 (예: 1000)

        Fetcher->>Chain: RPC Call: getBlockByNumber(1001)
        Chain-->>Fetcher: 블록 데이터 + 트랜잭션 목록

        Fetcher->>Channel: send(BlockData)
    end

    Note over Channel,Analyzer: === 트랜잭션 분석 ===

    Channel->>Analyzer: BlockData 수신

    loop 블록 내 모든 트랜잭션
        Analyzer->>Analyzer: to_address 추출

        Analyzer->>RocksDB: get_address_metadata(to_address, "ETH")

        alt 주소가 관리 대상인 경우
            RocksDB-->>Analyzer: Some(wallet_id, account_id)

            Note over Analyzer: ✅ 입금 감지!

            Analyzer->>Analyzer: DepositInfo 생성<br/>(address, tx_hash, amount, block_number)

            Analyzer->>Analyzer: process_deposit() 호출
        else 관리 대상 아님
            RocksDB-->>Analyzer: None
            Note over Analyzer: 무시
        end
    end

    Analyzer->>Repo: update_last_processed_block("ETH", 1001)
```

## 3. 입금 처리 및 Confirmation 플로우 (2-Stage Deposit Notification)

입금 감지 후 1차 알림(DEPOSIT_DETECTED)과 2차 확정 알림(DEPOSIT_CONFIRMED)을 보내는 과정입니다.

```mermaid
sequenceDiagram
    participant Analyzer as Analyzer<br/>process_deposit()
    participant RocksDB as RocksDB Cache
    participant Repo as Repository<br/>(PostgreSQL)
    participant SQS as AWS SQS
    participant Backend as Backend

    Note over Analyzer,Backend: === 입금 처리 시작 ===

    Analyzer->>Analyzer: 현재 블록 confirmations 계산<br/>= current_block - deposit_block + 1

    Analyzer->>RocksDB: get_address_metadata(address, chain)
    RocksDB-->>Analyzer: (wallet_id, account_id)

    Note over Analyzer: wallet_id와 account_id 획득

    Analyzer->>Repo: deposit_exists(tx_hash, chain)?

    alt 신규 입금 (처음 발견)
        Repo-->>Analyzer: false (존재하지 않음)

        Note over Analyzer: === STAGE 1: DEPOSIT_DETECTED (1 confirmation) ===

        alt confirmations == 1
            Analyzer->>Repo: save_deposit_event(<br/>address, wallet_id, account_id,<br/>chain, tx_hash, block_number,<br/>amount, amount_decimal<br/>)

            Note over Repo: INSERT INTO deposit_events<br/>confirmed = false

            Repo-->>Analyzer: ✅ DB 저장 완료

            Analyzer->>SQS: send_deposit_detected(<br/>address, wallet_id, account_id,<br/>chain, tx_hash, amount, block_number<br/>)

            Note over SQS: {<br/>  "event": "DepositDetected",<br/>  "address": "0x123...",<br/>  "wallet_id": "wallet_uuid",<br/>  "account_id": "account_uuid",<br/>  "chain": "ETH",<br/>  "tx_hash": "0xabc...",<br/>  "amount": "1000000000000000000",<br/>  "block_number": 1001,<br/>  "confirmations": 1<br/>}

            SQS-->>Backend: 메시지 전달

            Backend->>Backend: 입금 감지 알림 처리<br/>(사용자에게 알림 등)

            Note over Analyzer: ⏳ 추가 confirmation 대기...
        end

    else 기존 입금 (이미 DB에 존재)
        Repo-->>Analyzer: true (이미 존재함)

        Note over Analyzer: === STAGE 2: DEPOSIT_CONFIRMED (required_confirmations 도달) ===

        alt confirmations >= required_confirmations (예: 12)
            Analyzer->>Repo: is_deposit_confirmed(tx_hash)?

            alt 아직 확정되지 않음
                Repo-->>Analyzer: false (confirmed = false)

                Analyzer->>Repo: update_deposit_confirmed(tx_hash)

                Note over Repo: UPDATE deposit_events<br/>SET confirmed = true<br/>WHERE tx_hash = ...

                Repo-->>Analyzer: ✅ 확정 상태 업데이트

                Analyzer->>SQS: send_deposit_confirmed(<br/>address, wallet_id, account_id,<br/>chain, tx_hash, amount,<br/>block_number, confirmations<br/>)

                Note over SQS: {<br/>  "event": "DepositConfirmed",<br/>  "address": "0x123...",<br/>  "wallet_id": "wallet_uuid",<br/>  "account_id": "account_uuid",<br/>  "chain": "ETH",<br/>  "tx_hash": "0xabc...",<br/>  "amount": "1000000000000000000",<br/>  "block_number": 1001,<br/>  "confirmations": 12<br/>}

                SQS-->>Backend: 메시지 전달

                Backend->>Backend: 입금 확정 처리<br/>(잔액 업데이트, Sweep 준비 등)

            else 이미 확정됨
                Repo-->>Analyzer: true (confirmed = true)
                Note over Analyzer: 중복 알림 방지 - 무시
            end
        else 아직 confirmation 부족
            Note over Analyzer: confirmations < required_confirmations<br/>⏳ 계속 대기...
        end
    end
```

## 4. 전체 시스템 플로우 (Complete System Flow)

모든 컴포넌트가 어떻게 상호작용하는지 전체 플로우입니다.

```mermaid
sequenceDiagram
    participant User as 은행 고객
    participant Backend as Backend<br/>(blockbit-back-custody)
    participant SQS_Addr as SQS Queue<br/>(Address Sync)
    participant Sync as CustomerAddressSync
    participant RocksDB as RocksDB Cache
    participant Chain as Blockchain
    participant Fetcher as Fetcher (12개 체인)
    participant Channel as mpsc Channel
    participant Analyzer as Analyzer
    participant PG as PostgreSQL
    participant SQS_Dep as SQS Queue<br/>(Deposit Events)

    Note over User,SQS_Dep: === 1. 고객 가입 및 Virtual Account 생성 ===

    User->>Backend: 회원 가입
    Backend->>Backend: Custody Wallet 생성<br/>+ Virtual Account 할당
    Backend->>PG: INSERT INTO customer_addresses<br/>(address, wallet_id, account_id, chain)
    Backend->>SQS_Addr: AddressAdded 메시지 발송

    Sync->>SQS_Addr: Long Polling
    SQS_Addr-->>Sync: 메시지 수신
    Sync->>RocksDB: 주소 등록<br/>Key: "eth:0x123..."<br/>Value: {"wallet_id": "...", "account_id": "..."}

    Note over User,SQS_Dep: === 2. 고객이 Virtual Account로 입금 ===

    User->>Chain: ETH 전송<br/>→ Virtual Account 주소

    Note over Chain: 트랜잭션 블록에 포함

    Note over User,SQS_Dep: === 3. xScanner가 블록 스캔 ===

    loop Every 5초
        Fetcher->>PG: get_last_processed_block("ETH")
        PG-->>Fetcher: block_number: 1000

        Fetcher->>Chain: getBlockByNumber(1001)
        Chain-->>Fetcher: Block + Transactions

        Fetcher->>Channel: send(BlockData::Ethereum)
    end

    Channel->>Analyzer: BlockData 수신

    loop 블록 내 모든 트랜잭션
        Analyzer->>RocksDB: get_address_metadata(to_address)

        alt 관리 대상 주소
            RocksDB-->>Analyzer: Some(wallet_id, account_id)

            Note over Analyzer: ✅ 고객 입금 감지!

            Analyzer->>Analyzer: process_deposit()

            Note over Analyzer,SQS_Dep: === 4-1. STAGE 1: 첫 confirmation ===

            Analyzer->>PG: deposit_exists()?
            PG-->>Analyzer: false

            Analyzer->>PG: save_deposit_event(<br/>address, wallet_id, account_id,<br/>tx_hash, amount<br/>)

            Analyzer->>SQS_Dep: DepositDetected 메시지<br/>(confirmations: 1)

            SQS_Dep-->>Backend: 메시지 전달
            Backend->>User: "입금 감지" 알림 📱

            Note over Analyzer,SQS_Dep: === 4-2. STAGE 2: Required confirmations 도달 ===

            Note over Fetcher,Analyzer: ... 11블록 후 (ETH 기준 12 confirmations) ...

            Analyzer->>Analyzer: confirmations 계산: 12
            Analyzer->>PG: deposit_exists()?
            PG-->>Analyzer: true (이미 존재)

            Analyzer->>PG: is_deposit_confirmed()?
            PG-->>Analyzer: false

            Analyzer->>PG: update_deposit_confirmed()

            Analyzer->>SQS_Dep: DepositConfirmed 메시지<br/>(confirmations: 12)

            SQS_Dep-->>Backend: 메시지 전달

            Backend->>Chain: 블록체인 직접 조회<br/>(최종 검증)
            Chain-->>Backend: 잔액 확인

            Backend->>PG: UPDATE customer_balances
            Backend->>User: "입금 확정" 알림 📱

            Backend->>Backend: Sweep 작업 스케줄링<br/>(Virtual Account → Omnibus)

        else 관리 대상 아님
            RocksDB-->>Analyzer: None
            Note over Analyzer: 무시
        end
    end

    Analyzer->>PG: update_last_processed_block("ETH", 1001)
```

## 5. Omnibus (Master) Address 입금 플로우

Master Address로의 직접 입금 처리입니다.

```mermaid
sequenceDiagram
    participant Admin as 관리자/운영팀
    participant UI as Backend UI
    participant Chain as Blockchain
    participant Fetcher as Fetcher
    participant Analyzer as Analyzer
    participant RocksDB as RocksDB
    participant SQS as SQS Queue
    participant Backend as Backend

    Note over Admin,Backend: === Omnibus Address 직접 입금 시나리오 ===

    Admin->>UI: "입금" 버튼 클릭
    UI->>Admin: Omnibus Address 표시<br/>(0xMASTER...)

    Admin->>Chain: ETH 전송<br/>→ Omnibus Address

    Note over Chain: 트랜잭션 블록에 포함

    Note over Admin,Backend: === xScanner가 감지 ===

    Fetcher->>Chain: getBlockByNumber()
    Chain-->>Fetcher: Block + Transactions

    Fetcher->>Analyzer: BlockData 전달

    Analyzer->>RocksDB: get_address_metadata(<br/>Omnibus Address, "ETH"<br/>)

    RocksDB-->>Analyzer: {<br/>  wallet_id: "wallet_uuid",<br/>  account_id: null  ← Master 표시<br/>}

    Note over Analyzer: ✅ Omnibus Address 입금 감지!

    Analyzer->>SQS: DepositDetected 메시지<br/>{<br/>  address: "0xMASTER...",<br/>  wallet_id: "wallet_uuid",<br/>  account_id: null,  ← Master 식별<br/>  chain: "ETH",<br/>  ...<br/>}

    SQS-->>Backend: 메시지 전달

    Backend->>Backend: account_id == null 확인<br/>→ Omnibus 입금으로 인식

    Backend->>Admin: "마스터 지갑 입금 감지" 알림 📊

    Note over Backend: Omnibus는 자동 Sweep 대상 아님<br/>(이미 집금 계좌이므로)
```

## 6. 중복 방지 메커니즘 (Duplicate Prevention)

같은 트랜잭션을 여러 번 알림하지 않도록 하는 메커니즘입니다.

```mermaid
sequenceDiagram
    participant Analyzer as Analyzer
    participant Repo as Repository

    Note over Analyzer,Repo: === 시나리오 1: 신규 입금 (1 confirmation) ===

    Analyzer->>Analyzer: confirmations = 1
    Analyzer->>Repo: deposit_exists(tx_hash)?
    Repo-->>Analyzer: false

    Note over Analyzer: ✅ 신규 입금 → STAGE 1 처리

    Analyzer->>Repo: save_deposit_event()<br/>(confirmed = false)
    Analyzer->>Analyzer: send_deposit_detected()

    Note over Analyzer,Repo: === 시나리오 2: 동일 블록 재스캔 (1 confirmation) ===

    Note over Analyzer: 스캐너 재시작 등으로<br/>동일 블록 재처리 시

    Analyzer->>Analyzer: confirmations = 1
    Analyzer->>Repo: deposit_exists(tx_hash)?
    Repo-->>Analyzer: true (이미 존재)

    Note over Analyzer: ⚠️ 이미 DB에 존재<br/>confirmations < required_confirmations<br/>→ 아무것도 안 함 (중복 방지)

    Note over Analyzer,Repo: === 시나리오 3: Confirmation 진행 중 (2~11 confirmations) ===

    Analyzer->>Analyzer: confirmations = 5
    Analyzer->>Repo: deposit_exists(tx_hash)?
    Repo-->>Analyzer: true

    Note over Analyzer: confirmations < required_confirmations (12)<br/>→ 대기 (아무것도 안 함)

    Note over Analyzer,Repo: === 시나리오 4: Required confirmations 도달 (12 confirmations) ===

    Analyzer->>Analyzer: confirmations = 12
    Analyzer->>Repo: deposit_exists(tx_hash)?
    Repo-->>Analyzer: true

    Analyzer->>Repo: is_deposit_confirmed(tx_hash)?

    alt 아직 확정되지 않음
        Repo-->>Analyzer: false

        Note over Analyzer: ✅ STAGE 2 처리

        Analyzer->>Repo: update_deposit_confirmed()
        Analyzer->>Analyzer: send_deposit_confirmed()

    else 이미 확정됨
        Repo-->>Analyzer: true

        Note over Analyzer: ⚠️ 이미 확정됨<br/>→ 중복 알림 방지
    end

    Note over Analyzer,Repo: === 시나리오 5: Confirmation 진행 중 재스캔 (13+ confirmations) ===

    Analyzer->>Analyzer: confirmations = 15
    Analyzer->>Repo: deposit_exists(tx_hash)?
    Repo-->>Analyzer: true

    Analyzer->>Repo: is_deposit_confirmed(tx_hash)?
    Repo-->>Analyzer: true (confirmed = true)

    Note over Analyzer: ⚠️ 이미 확정됨<br/>→ 무시 (중복 방지)
```

## 핵심 개념 정리

### 1. Two-Stage Notification
- **STAGE 1 (DEPOSIT_DETECTED)**: confirmations == 1
  - 첫 confirmation 시 즉시 알림
  - 빠른 사용자 피드백
  - DB에 `confirmed = false`로 저장

- **STAGE 2 (DEPOSIT_CONFIRMED)**: confirmations >= required_confirmations
  - 필요한 confirmation 수 도달 시 확정 알림
  - `confirmed = true`로 업데이트
  - Backend에서 실제 잔액 처리 시작

### 2. Duplicate Prevention (중복 방지)
- `deposit_exists()`: 이미 DB에 있는지 확인
- `is_deposit_confirmed()`: 이미 확정됐는지 확인
- UNIQUE constraint: (chain_name, tx_hash)

### 3. Address Metadata
- **wallet_id**: Custody Wallet 식별자 (필수)
- **account_id**: Virtual Account ID
  - 있으면: Virtual Account (고객 주소)
  - null이면: Omnibus Address (Master 주소)

### 4. RocksDB Cache
- **Key**: `chain_name:address` (소문자 정규화)
- **Value**: `{"wallet_id": "...", "account_id": "..." or null}`
- **목적**: 빠른 주소 조회 (O(1))

### 5. Required Confirmations (체인별)
- Ethereum: 12 blocks
- Bitcoin: 6 blocks
- TRON: 19 blocks
- 설정 가능 (`config.toml`)
