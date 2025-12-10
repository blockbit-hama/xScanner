# Rust 튜토리얼 - xScanner로 배우는 실전 Rust

## 📚 Overview

이 문서는 xScanner 프로젝트의 실제 코드를 통해 Rust를 학습하는 튜토리얼입니다.
블록체인 스캐너라는 실용적인 프로젝트로 Rust의 핵심 개념들을 배웁니다.

## 🏗️ xScanner 전체 구조

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│  - tokio runtime 초기화                                      │
│  - mpsc channel 생성                                         │
│  - Arc로 공유 자원 관리                                      │
│  - 여러 async task spawn                                     │
└─────────────────┬───────────────────────────────────────────┘
                  │
        ┌─────────┼─────────┬──────────────┐
        │         │         │              │
        ▼         ▼         ▼              ▼
   ┌─────────┐ ┌──────────┐ ┌────────┐ ┌─────────────┐
   │ Fetcher │ │ Analyzer │ │ Repo   │ │ Confirmation│
   │         │ │          │ │        │ │ Checker     │
   └─────────┘ └──────────┘ └────────┘ └─────────────┘

   각 컴포넌트는 독립적인 async task로 실행
   mpsc channel로 메시지 전달
   Arc<T>로 상태 공유
```

---

## 1. 비동기 프로그래밍: `async/await` + `tokio`

### 📍 위치: `src/main.rs` - Entry Point

```rust
#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Application 초기화...

    // 여러 async task를 동시에 실행
    let fetcher_handle = tokio::spawn(async move {
        fetcher.run().await
    });

    let analyzer_handle = tokio::spawn(async move {
        analyzer.run().await
    });

    // 모든 task가 끝날 때까지 대기
    fetcher_handle.await?;
    analyzer_handle.await?;

    Ok(())
}
```

### 📖 Rust 문법 설명

#### 1.1 `#[tokio::main]` 매크로

```rust
// 이 매크로가 없으면:
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        // async 코드...
    });
}

// 매크로 사용 시:
#[tokio::main]
async fn main() {
    // async 코드를 바로 작성!
}
```

- **역할**: `async fn main()`을 가능하게 만듦
- **내부 동작**: Tokio runtime을 자동으로 생성하고 `block_on()` 호출

#### 1.2 `async/await`

```rust
// async fn: Future를 반환하는 함수
async fn fetch_data() -> Result<String, Error> {
    let response = http_client.get("https://...").await?;
    //                                           ^^^^
    //                                           Future를 완료될 때까지 대기
    Ok(response)
}

// await 없이 호출하면 Future만 반환 (실행 안 됨!)
let future = fetch_data(); // 아직 실행 안 함
let result = future.await; // 여기서 실행됨
```

**핵심 개념**:
- `async fn`은 즉시 실행되지 않고 `Future`를 반환
- `.await`를 만나야 실제로 실행됨
- `.await` 중에 다른 task가 실행 가능 (비선점형 멀티태스킹)

#### 1.3 `tokio::spawn`

```rust
// 새로운 async task 생성 (OS 스레드 아님!)
let handle = tokio::spawn(async move {
    //                     ^^^^^^^^^
    //                     클로저 안으로 소유권 이동
    loop {
        // 독립적으로 실행되는 작업
        process_data().await;
    }
});

// JoinHandle을 통해 결과 받기
let result = handle.await?;
```

**중요**:
- `spawn`된 task는 독립적으로 실행
- `move` 키워드로 클로저 안으로 소유권 이동 (ownership 이동)
- 반환값: `JoinHandle<T>` (결과를 받을 수 있음)

---

## 2. 소유권과 공유: `Arc<T>`

### 📍 위치: `src/main.rs` - 공유 상태 관리

```rust
use std::sync::Arc;

// Repository를 여러 task가 공유해야 함
let repository = Arc::new(RepositoryWrapper::new(/* ... */));
//               ^^^^^^^^
//               Atomic Reference Counting: 스레드 안전한 참조 카운팅

// Clone으로 참조 카운트 증가 (데이터 복사 아님!)
let repo_clone1 = repository.clone(); // count: 1 -> 2
let repo_clone2 = repository.clone(); // count: 2 -> 3

// 각 task에서 사용
tokio::spawn(async move {
    repo_clone1.save_data().await; // repo_clone1 소유권 이동
});

tokio::spawn(async move {
    repo_clone2.get_data().await;  // repo_clone2 소유권 이동
});

// 모든 Arc가 drop되면 데이터도 drop
```

### 📖 Rust 문법 설명

#### 2.1 Ownership (소유권)

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // s1의 소유권이 s2로 이동

    // println!("{}", s1); // ❌ 컴파일 에러! s1은 더 이상 유효하지 않음
    println!("{}", s2); // ✅ OK
}
```

**규칙**:
1. 각 값은 정확히 하나의 소유자(owner)를 가짐
2. 소유자가 스코프를 벗어나면 값은 drop됨
3. 소유권은 이동(move)하거나 빌려줄(borrow) 수 있음

#### 2.2 `Arc<T>` vs `Rc<T>`

```rust
use std::rc::Rc;
use std::sync::Arc;

// Rc: Reference Counting (싱글 스레드)
let rc_data = Rc::new(vec![1, 2, 3]);
let rc_clone = rc_data.clone(); // 참조 카운트 증가

// Arc: Atomic Reference Counting (멀티 스레드 안전)
let arc_data = Arc::new(vec![1, 2, 3]);
let arc_clone = arc_data.clone(); // 원자적 참조 카운트 증가

// Arc는 스레드 간 공유 가능
std::thread::spawn(move || {
    println!("{:?}", arc_clone); // ✅ OK
});

// Rc는 스레드 간 공유 불가
// std::thread::spawn(move || {
//     println!("{:?}", rc_clone); // ❌ 컴파일 에러!
// });
```

#### 2.3 실전 예시: xScanner의 공유 패턴

```rust
// src/main.rs 실제 코드 (간소화)
let repository = Arc::new(RepositoryWrapper::new(/* ... */));
let sqs_notifier = Arc::new(SqsNotifier::new(/* ... */));

// Fetcher에게 전달
let repo_for_fetcher = repository.clone();
tokio::spawn(async move {
    run_fetcher(repo_for_fetcher).await
});

// Analyzer에게 전달
let repo_for_analyzer = repository.clone();
let sqs_for_analyzer = sqs_notifier.clone();
tokio::spawn(async move {
    run_analyzer(repo_for_analyzer, sqs_for_analyzer).await
});

// ConfirmationChecker에게 전달
let repo_for_checker = repository.clone();
let sqs_for_checker = sqs_notifier.clone();
tokio::spawn(async move {
    run_confirmation_checker(repo_for_checker, sqs_for_checker).await
});

// 모든 clone은 같은 데이터를 가리킴!
```

---

## 3. 메시지 전달: `mpsc::channel`

### 📍 위치: `src/main.rs` - Fetcher와 Analyzer 연결

```rust
use tokio::sync::mpsc;

// Channel 생성 (capacity: 128)
let (sender, receiver) = mpsc::channel::<BlockData>(128);
//   ^^^^^^  ^^^^^^^^                   ^^^^^^^^^   ^^^
//   보내는 쪽  받는 쪽                    메시지 타입   버퍼 크기

// Fetcher: Sender를 사용해서 블록 데이터 전송
tokio::spawn(async move {
    loop {
        let block = fetch_block().await?;
        sender.send(block).await?; // 버퍼가 가득 차면 대기
        //     ^^^^
        //     async send (백프레셔 지원)
    }
});

// Analyzer: Receiver로 블록 데이터 수신
tokio::spawn(async move {
    while let Some(block) = receiver.recv().await {
        //        ^^^^                      ^^^^
        //        Option<T>                 async recv
        analyze_block(block).await?;
    }
    // sender가 모두 drop되면 None 반환 → 루프 종료
});
```

### 📖 Rust 문법 설명

#### 3.1 `mpsc` (Multiple Producer, Single Consumer)

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel::<i32>(10);
//       ^^^
//       mutable: recv()가 &mut self를 요구

// 여러 sender (clone 가능)
let tx1 = tx.clone();
let tx2 = tx.clone();

tokio::spawn(async move {
    tx1.send(1).await.unwrap();
});

tokio::spawn(async move {
    tx2.send(2).await.unwrap();
});

// 단일 receiver (clone 불가)
tokio::spawn(async move {
    while let Some(value) = rx.recv().await {
        println!("Received: {}", value);
    }
});
```

**핵심**:
- `Sender`: Clone 가능 (여러 producer)
- `Receiver`: Clone 불가 (단일 consumer)
- 버퍼가 가득 차면 `send()` 대기 (백프레셔)

#### 3.2 `while let` 패턴 매칭

```rust
// while let: Option/Result를 계속 처리
while let Some(value) = receiver.recv().await {
//        ^^^^^^^^^^^   ^^^^^^^^^^^^^^^^^^^^^
//        패턴          표현식
    println!("Got: {}", value);
}

// 위 코드는 다음과 같음:
loop {
    match receiver.recv().await {
        Some(value) => println!("Got: {}", value),
        None => break, // sender가 모두 drop됨
    }
}
```

#### 3.3 실전 예시: BlockData 전달

```rust
// src/main.rs 실제 코드
#[derive(Debug, Clone)]
pub struct BlockData {
    pub chain: String,
    pub block_number: u64,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
}

// Channel 생성
let (block_sender, block_receiver) = mpsc::channel::<BlockData>(128);

// Fetcher → Analyzer 메시지 흐름
// [Fetcher] --BlockData--> [Channel Buffer(128)] --BlockData--> [Analyzer]
```

---

## 4. Trait: 다형성과 추상화

### 📍 위치: `src/repository/trait.rs` - Repository 추상화

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Repository: Send + Sync {
//                    ^^^^   ^^^^
//                    스레드로 이동 가능   스레드 간 공유 가능

    async fn save_deposit_event(
        &self, // immutable borrow
        address: &str,
        wallet_id: &str,
        account_id: Option<&str>,
        chain_name: &str,
        tx_hash: &str,
        block_number: u64,
        amount: &str,
        amount_decimal: Option<rust_decimal::Decimal>,
    ) -> Result<(), AppError>;

    async fn get_last_processed_block(
        &self,
        chain_name: &str,
    ) -> Result<u64, AppError>;
}
```

### 📖 Rust 문법 설명

#### 4.1 Trait 정의와 구현

```rust
// Trait 정의: 인터페이스 선언
trait Drawable {
    fn draw(&self);

    // Default 구현 제공 가능
    fn area(&self) -> f64 {
        0.0
    }
}

// Trait 구현
struct Circle {
    radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }

    fn area(&self) -> f64 {
        3.14 * self.radius * self.radius
    }
}

// 사용
fn render(shape: &impl Drawable) {
//                ^^^^^^^^^^^^
//                "Drawable을 구현한 어떤 타입"
    shape.draw();
}

let circle = Circle { radius: 5.0 };
render(&circle);
```

#### 4.2 `#[async_trait]` 매크로

```rust
// async_trait 없이는 trait에서 async fn 사용 불가
// trait Repository {
//     async fn save(&self) -> Result<(), Error>; // ❌ 컴파일 에러!
// }

// async_trait 사용:
#[async_trait]
trait Repository {
    async fn save(&self) -> Result<(), Error>; // ✅ OK!
}

// 내부적으로 다음과 같이 변환:
// fn save(&self) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send + '_>>;
```

#### 4.3 Trait Bounds: `Send + Sync`

```rust
#[async_trait]
pub trait Repository: Send + Sync {
//                    ^^^^   ^^^^
//                    │      └─ 여러 스레드에서 동시에 접근 가능
//                    └─ 스레드 간 이동 가능
}

// Send: 소유권을 스레드 간 이동 가능
// Sync: 여러 스레드에서 &T로 접근 가능

// 예시:
fn must_be_sendable<T: Send>(value: T) {
    std::thread::spawn(move || {
        // T를 다른 스레드로 이동 가능
    });
}

fn must_be_shareable<T: Sync>(value: &T) {
    std::thread::spawn(move || {
        // &T를 다른 스레드에서 사용 가능
    });
}
```

#### 4.4 실전 예시: 3가지 Repository 구현

```rust
// PostgreSQL 구현
pub struct PostgreSQLRepository {
    pool: PgPool,
}

#[async_trait]
impl Repository for PostgreSQLRepository {
    async fn save_deposit_event(&self, /* ... */) -> Result<(), AppError> {
        sqlx::query!(/* SQL */)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

// Memory 구현
pub struct MemoryRepository {
    deposits: Arc<Mutex<Vec<DepositEvent>>>,
}

#[async_trait]
impl Repository for MemoryRepository {
    async fn save_deposit_event(&self, /* ... */) -> Result<(), AppError> {
        let mut deposits = self.deposits.lock().unwrap();
        deposits.push(event);
        Ok(())
    }
}

// 사용: 런타임에 선택
let repo: Box<dyn Repository> = if use_postgres {
    Box::new(PostgreSQLRepository::new())
} else {
    Box::new(MemoryRepository::new())
};

repo.save_deposit_event(/* ... */).await?;
```

---

## 5. 에러 처리: `Result<T, E>` + `?` 연산자

### 📍 위치: `src/analyzer/analyzer.rs` - Deposit 처리

```rust
async fn process_deposit(
    repository: &Arc<RepositoryWrapper>,
    chain_name: &str,
    deposit: DepositInfo,
    current_block: u64,
    required_confirmations: u64,
    sqs_notifier: Option<&SqsNotifier>,
    #[cfg(feature = "rocksdb-backend")]
    kv_db: Option<&KeyValueDB>,
) -> Result<(), String> {
//   ^^^^^^^^^^^^^^^^^^^
//   성공: (), 실패: String 에러 메시지

    // 1. Calculate confirmations
    let confirmations = current_block.saturating_sub(deposit.block_number) + 1;

    // 2. Get metadata from RocksDB
    #[cfg(feature = "rocksdb-backend")]
    let (wallet_id, account_id) = if let Some(db) = kv_db {
        match get_address_metadata_from_rocksdb(db, &deposit.address, chain_name) {
            Ok(metadata) => (metadata.wallet_id, metadata.account_id),
            Err(e) => {
                error!("[Analyzer] Failed to get metadata: {}", e);
                return Err(format!("Address metadata not found: {}", e));
                //     ^^^
                //     Early return on error
            }
        }
    } else {
        return Err("RocksDB not available".to_string());
    };

    // 3. Check if deposit already exists
    let deposit_exists = repository
        .deposit_exists(&deposit.tx_hash)
        .await
        .map_err(|e| format!("Failed to check deposit existence: {}", e))?;
        //                                                               ^
        //                                                               ? 연산자

    if deposit_exists {
        let is_confirmed = repository
            .is_deposit_confirmed(&deposit.tx_hash)
            .await?; // 에러 발생 시 자동으로 return

        if is_confirmed {
            info!("[Analyzer] Deposit {} already confirmed, skipping", deposit.tx_hash);
            return Ok(());
        }
    }

    // 4. Save to database
    repository.save_deposit_event(
        &deposit.address,
        &wallet_id,
        account_id.as_deref(),
        chain_name,
        &deposit.tx_hash,
        deposit.block_number,
        &deposit.amount,
        deposit.amount_decimal,
    )
    .await
    .map_err(|e| format!("Failed to save deposit event: {}", e))?;

    // 5. Send SQS notification
    if let Some(notifier) = sqs_notifier {
        notifier.send_deposit_detected(
            deposit.address.clone(),
            wallet_id,
            account_id,
            chain_name.to_uppercase(),
            deposit.tx_hash.clone(),
            deposit.amount.clone(),
            deposit.block_number,
            confirmations,
        )
        .await
        .map_err(|e| format!("Failed to send SQS: {}", e))?;
    }

    Ok(())
}
```

### 📖 Rust 문법 설명

#### 5.1 `Result<T, E>` 타입

```rust
// Result는 enum (두 가지 상태 중 하나)
enum Result<T, E> {
    Ok(T),  // 성공: 값 T
    Err(E), // 실패: 에러 E
}

// 사용 예시
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// 호출
match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e),
}
```

#### 5.2 `?` 연산자 (에러 전파)

```rust
// ? 없이:
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e), // 에러 발생 시 즉시 return
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e),
    }
}

// ? 사용:
fn read_file(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?; // Err이면 자동 return
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}
```

**핵심**:
- `?`는 `Err`를 만나면 즉시 함수에서 return
- `Ok`면 내부 값을 unwrap해서 계속 진행
- 함수 반환 타입이 `Result<T, E>`여야 사용 가능

#### 5.3 `.map_err()` - 에러 타입 변환

```rust
async fn process() -> Result<(), String> {
    // sqlx는 sqlx::Error 반환, 우리는 String 필요
    repository
        .save_data()
        .await // Result<(), sqlx::Error>
        .map_err(|e| format!("DB error: {}", e))?;
        // ^^^^^^^^
        // sqlx::Error -> String 변환

    Ok(())
}
```

#### 5.4 실전 예시: 다층 에러 처리

```rust
// src/analyzer/analyzer.rs 실제 패턴
pub async fn run_analyzer(
    receiver: mpsc::Receiver<BlockData>,
    repository: Arc<RepositoryWrapper>,
    sqs_notifier: Option<Arc<SqsNotifier>>,
) {
    while let Some(block) = receiver.recv().await {
        // 에러 발생해도 analyzer는 계속 실행
        if let Err(e) = process_block(&block, &repository, &sqs_notifier).await {
            error!("[Analyzer] Error processing block {}: {}", block.block_number, e);
            // 로그만 남기고 다음 블록 처리
        }
    }
}

async fn process_block(/* ... */) -> Result<(), String> {
    // 여러 단계에서 에러 발생 가능
    let deposits = extract_deposits()?; // ?로 에러 전파

    for deposit in deposits {
        process_deposit(deposit).await?; // ?로 에러 전파
    }

    Ok(())
}
```

---

## 6. Option 타입: null 대신 안전한 처리

### 📍 위치: `src/analyzer/chains/evm.rs` - Address 확인

```rust
pub async fn analyze_ethereum_block(
    block: EthereumBlock,
    repository: &Arc<RepositoryWrapper>,
    #[cfg(feature = "rocksdb-backend")]
    kv_db: Option<&KeyValueDB>,
    //    ^^^^^^
    //    RocksDB가 있을 수도, 없을 수도
) -> Result<Vec<DepositInfo>, String> {

    let mut deposits = Vec::new();

    for tx in transactions {
        // to 주소가 있을 수도, 없을 수도 (contract creation)
        if let Some(to_address) = tx.to {
            //      ^^^^^^^^^^^^^^^^^
            //      패턴 매칭으로 Some인 경우만 처리

            // RocksDB로 주소 확인
            #[cfg(feature = "rocksdb-backend")]
            let is_monitored = if let Some(db) = kv_db {
                is_monitored_address_in_rocksdb(db, &to_address, chain_name)
                    .unwrap_or(false)
                    // ^^^^^^^^^^^
                    // None/Err이면 false 사용
            } else {
                false
            };

            if is_monitored {
                // account_id는 Option (Omnibus는 None)
                let account_id: Option<String> = metadata.account_id;

                deposits.push(DepositInfo {
                    address: to_address,
                    wallet_id: metadata.wallet_id,
                    account_id, // Option<String>
                    tx_hash: tx.hash,
                    amount: tx.value,
                    amount_decimal: parse_amount(&tx.value).ok(),
                    //                                      ^^^
                    //                                      Result -> Option 변환
                    block_number,
                });
            }
        }
    }

    Ok(deposits)
}
```

### 📖 Rust 문법 설명

#### 6.1 `Option<T>` 타입

```rust
// Option은 enum (값이 있거나 없거나)
enum Option<T> {
    Some(T), // 값이 있음
    None,    // 값이 없음 (null 대신)
}

// 사용 예시
fn find_user(id: u32) -> Option<User> {
    if id == 1 {
        Some(User { name: "Alice".to_string() })
    } else {
        None // 찾지 못함
    }
}
```

#### 6.2 `if let` - Option 처리

```rust
let maybe_value: Option<i32> = Some(42);

// if let: Some인 경우만 처리
if let Some(value) = maybe_value {
    println!("Got value: {}", value);
} else {
    println!("No value");
}

// match로도 가능:
match maybe_value {
    Some(value) => println!("Got: {}", value),
    None => println!("None"),
}
```

#### 6.3 `.unwrap_or()` - 기본값 제공

```rust
let maybe_name: Option<String> = None;

// unwrap_or: None이면 기본값 사용
let name = maybe_name.unwrap_or("Unknown".to_string());
println!("Name: {}", name); // "Unknown"

// unwrap_or_else: None이면 클로저 실행
let name = maybe_name.unwrap_or_else(|| {
    // 복잡한 계산...
    "Default".to_string()
});
```

#### 6.4 `.ok()` - Result를 Option으로 변환

```rust
use std::num::ParseIntError;

fn parse_number(s: &str) -> Result<i32, ParseIntError> {
    s.parse()
}

// Result<T, E> -> Option<T>
let maybe_num: Option<i32> = parse_number("42").ok();
//                                              ^^^
//                                              Ok(42) -> Some(42)
//                                              Err(_) -> None

// 체이닝
let num = parse_number("not a number")
    .ok()              // Result -> Option
    .unwrap_or(0);     // None이면 0
```

#### 6.5 실전 예시: account_id 처리

```rust
// Omnibus address는 account_id가 None
struct AddressMetadata {
    wallet_id: String,
    account_id: Option<String>, // Omnibus는 None
}

// DB 저장 시 Option 처리
repository.save_deposit_event(
    &address,
    &wallet_id,
    account_id.as_deref(), // Option<String> -> Option<&str>
    //         ^^^^^^^^^
    //         Some("abc") -> Some("abc")
    //         None -> None
    chain_name,
    /* ... */
).await?;

// SQS 메시지
let message = json!({
    "wallet_id": wallet_id,
    "account_id": account_id, // Option은 자동으로 null로 serialize
});
```

---

## 7. 패턴 매칭: `match` 표현식

### 📍 위치: `src/main.rs` - Chain별 Fetcher 선택

```rust
// Spawn fetcher based on chain name
let handle = match chain_name.to_lowercase().as_str() {
    "ethereum" | "eth" | "sepolia" => {
        //       ^^^
        //       여러 패턴을 | 로 연결
        let client = Arc::new(EthereumClient::new(chain_config.api.clone()));
        let fetcher = Arc::new(EthereumFetcher { client });
        tokio::spawn(crate::fetcher::runner::run_fetcher(
            fetcher,
            sender_clone,
            start_block,
            interval_duration
        ))
    }
    "bitcoin" | "btc" => {
        let client = Arc::new(BitcoinClient::new(chain_config.api.clone()));
        let fetcher = Arc::new(BitcoinFetcher { client });
        tokio::spawn(crate::fetcher::runner::run_fetcher(
            fetcher,
            sender_clone,
            start_block,
            interval_duration
        ))
    }
    "tron" => {
        let client = Arc::new(TronClient::new(chain_config.api.clone()));
        let fetcher = Arc::new(TronFetcher { client });
        tokio::spawn(crate::fetcher::runner::run_fetcher(
            fetcher,
            sender_clone,
            start_block,
            interval_duration
        ))
    }
    unknown => {
        //^^^^^^
        // catch-all 패턴 (모든 다른 값)
        error!("Unsupported blockchain: {}", unknown);
        continue; // 다음 chain으로
    }
};
```

### 📖 Rust 문법 설명

#### 7.1 `match` 기본

```rust
let number = 42;

let result = match number {
    0 => "zero",           // 특정 값
    1 | 2 | 3 => "small", // 여러 값
    10..=20 => "medium",  // 범위 (inclusive)
    _ => "other",         // 와일드카드 (모든 경우)
};

println!("{}", result); // "other"
```

**핵심**:
- `match`는 **표현식** (값을 반환)
- 모든 가능한 경우를 처리해야 함 (exhaustive)
- `_`는 "나머지 모든 경우"

#### 7.2 Destructuring (구조 분해)

```rust
struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 10, y: 20 };

match point {
    Point { x: 0, y: 0 } => println!("Origin"),
    Point { x: 0, y } => println!("On Y axis at {}", y),
    Point { x, y: 0 } => println!("On X axis at {}", x),
    Point { x, y } => println!("At ({}, {})", x, y),
}
```

#### 7.3 `match` with `Option` & `Result`

```rust
// Option 매칭
let maybe_value: Option<i32> = Some(42);

match maybe_value {
    Some(x) if x > 0 => println!("Positive: {}", x),
    //       ^^^^^^^^
    //       match guard (추가 조건)
    Some(x) => println!("Non-positive: {}", x),
    None => println!("No value"),
}

// Result 매칭
let result: Result<i32, String> = Ok(42);

match result {
    Ok(value) => println!("Success: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}
```

#### 7.4 실전 예시: Confirmation Checker

```rust
// src/tasks/confirmation_checker.rs
loop {
    check_interval.tick().await;

    let pending_deposits = repository.get_pending_deposits().await?;

    for deposit in pending_deposits {
        let confirmations = current_block - deposit.block_number + 1;

        // Match로 분기 처리
        match confirmations.cmp(&required_confirmations) {
            std::cmp::Ordering::Less => {
                // 아직 confirmation 부족
                info!(
                    "[ConfirmationChecker] Deposit {} needs {} more confirmations",
                    deposit.tx_hash,
                    required_confirmations - confirmations
                );
            }
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
                // Required confirmations 도달!
                info!(
                    "[ConfirmationChecker] ✅ Deposit {} reached confirmations",
                    deposit.tx_hash
                );

                // DB 업데이트 + SQS 전송
                repository.update_deposit_confirmed(&deposit.tx_hash).await?;

                if let Some(notifier) = sqs_notifier.as_ref() {
                    notifier.send_deposit_confirmed(/* ... */).await?;
                }
            }
        }
    }
}
```

---

## 8. Lifetime (생명주기)

### 📍 위치: `src/repository/trait.rs` - 참조의 생명주기

```rust
#[async_trait]
pub trait Repository: Send + Sync {
    async fn save_deposit_event(
        &self,
        //^
        // &self의 생명주기는 컴파일러가 추론

        address: &str,
        //       ^
        // 암시적 생명주기: &'a str

        wallet_id: &str,
        account_id: Option<&str>,
        //                 ^
        // Option 안의 참조도 생명주기 존재

        /* ... */
    ) -> Result<(), AppError>;
}

// 명시적으로 작성하면:
async fn save_deposit_event<'a>(
//                          ^^^
//                          생명주기 매개변수
    &'a self,
    address: &'a str,
    wallet_id: &'a str,
    account_id: Option<&'a str>,
) -> Result<(), AppError>;
```

### 📖 Rust 문법 설명

#### 8.1 Lifetime 기초

```rust
// 문제: 참조가 유효한지 컴파일러가 확인 필요
fn longest(x: &str, y: &str) -> &str {
//                              ^
//                              어느 참조의 생명주기?
    if x.len() > y.len() { x } else { y }
}

// 해결: 명시적 생명주기
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
//        ^^^     ^^          ^^            ^^
//        1. 선언  2. x는 'a  3. y는 'a    4. 반환값도 'a
    if x.len() > y.len() { x } else { y }
}

// 의미: 반환된 참조는 x와 y 중 짧은 생명주기를 가짐
let s1 = String::from("long string");
let result;
{
    let s2 = String::from("short");
    result = longest(&s1, &s2); // s2의 생명주기가 짧음
}
// println!("{}", result); // ❌ s2가 drop되어 사용 불가!
```

#### 8.2 Struct의 Lifetime

```rust
struct ImportantExcerpt<'a> {
//                      ^^^
//                      struct가 참조를 포함하면 생명주기 필요
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
//   ^^^
//   impl도 생명주기 명시
    fn level(&self) -> i32 {
        3
    }

    fn announce_and_return_part(&self, announcement: &str) -> &str {
        //                                                     ^
        //                                                     &self의 생명주기
        println!("Attention: {}", announcement);
        self.part
    }
}

// 사용
let novel = String::from("Call me Ishmael...");
let first_sentence = novel.split('.').next().expect("Could not find '.'");
let i = ImportantExcerpt { part: first_sentence };
// i는 novel이 살아있는 동안만 유효
```

#### 8.3 Lifetime Elision (생략 규칙)

```rust
// 대부분의 경우 컴파일러가 추론 가능
fn first_word(s: &str) -> &str {
//             ^          ^
//             명시적 생명주기 없어도 OK
    s.split_whitespace().next().unwrap_or("")
}

// 컴파일러가 자동으로 추론:
fn first_word<'a>(s: &'a str) -> &'a str { /* ... */ }

// 규칙:
// 1. 각 참조 매개변수는 고유한 생명주기 받음
// 2. 매개변수가 하나면 반환값도 같은 생명주기
// 3. &self가 있으면 반환값은 self의 생명주기
```

#### 8.4 실전 예시: Repository 메서드

```rust
// src/repository/trait.rs 실제 코드
#[async_trait]
pub trait Repository: Send + Sync {
    // 생명주기 생략 가능 (rule 1, 3 적용)
    async fn is_monitored_address(
        &self,
        address: &str,
        chain_name: &str,
    ) -> Result<bool, AppError>;

    // 명시적으로 작성하면:
    async fn is_monitored_address<'a>(
        &'a self,
        address: &'a str,
        chain_name: &'a str,
    ) -> Result<bool, AppError>;

    // 의미: address와 chain_name은 self가 살아있는 동안 유효해야 함
}
```

---

## 9. Conditional Compilation: `#[cfg]`

### 📍 위치: `src/analyzer/analyzer.rs` - Feature 기반 컴파일

```rust
async fn process_deposit(
    repository: &Arc<RepositoryWrapper>,
    chain_name: &str,
    deposit: DepositInfo,
    current_block: u64,
    required_confirmations: u64,
    sqs_notifier: Option<&SqsNotifier>,

    // RocksDB feature가 활성화된 경우만 컴파일
    #[cfg(feature = "rocksdb-backend")]
    //^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    kv_db: Option<&KeyValueDB>,
) -> Result<(), String> {

    // RocksDB에서 메타데이터 가져오기
    #[cfg(feature = "rocksdb-backend")]
    let (wallet_id, account_id) = if let Some(db) = kv_db {
        match get_address_metadata_from_rocksdb(db, &deposit.address, chain_name) {
            Ok(metadata) => (metadata.wallet_id, metadata.account_id),
            Err(e) => return Err(format!("Metadata not found: {}", e)),
        }
    } else {
        return Err("RocksDB not available".to_string());
    };

    // RocksDB 없으면 PostgreSQL 사용
    #[cfg(not(feature = "rocksdb-backend"))]
    //^^^^^^^
    // 조건 반대: rocksdb-backend가 없으면
    let (wallet_id, account_id) = {
        match repository.get_address_metadata(&deposit.address, chain_name).await {
            Ok(Some(metadata)) => (metadata.wallet_id, metadata.account_id),
            Ok(None) => return Err("Address not found".to_string()),
            Err(e) => return Err(format!("Failed to get metadata: {}", e)),
        }
    };

    // ... 나머지 처리 ...
}
```

### 📖 Rust 문법 설명

#### 9.1 `#[cfg]` 기본

```rust
// 특정 OS에서만 컴파일
#[cfg(target_os = "linux")]
fn linux_only() {
    println!("This only runs on Linux");
}

#[cfg(target_os = "windows")]
fn windows_only() {
    println!("This only runs on Windows");
}

// Feature flag
#[cfg(feature = "experimental")]
fn experimental_feature() {
    println!("Experimental code");
}

// 조합
#[cfg(all(target_os = "linux", feature = "debug"))]
//    ^^^
//    모두 만족
fn linux_debug() { }

#[cfg(any(target_os = "linux", target_os = "macos"))]
//    ^^^
//    하나라도 만족
fn unix_like() { }

#[cfg(not(feature = "production"))]
//    ^^^
//    조건 반대
fn dev_only() { }
```

#### 9.2 Cargo.toml의 Features

```toml
# Cargo.toml
[features]
default = ["rocksdb-backend"]  # 기본 활성화
rocksdb-backend = ["rocksdb"]
postgres-only = []

[dependencies]
rocksdb = { version = "0.22", optional = true }
#                             ^^^^^^^^
#                             feature 활성화 시에만 컴파일
sqlx = "0.7"
```

#### 9.3 빌드 명령어

```bash
# 기본 (default features)
cargo build

# 특정 feature 활성화
cargo build --features rocksdb-backend

# 모든 feature
cargo build --all-features

# feature 없이
cargo build --no-default-features

# 여러 features
cargo build --features "rocksdb-backend,experimental"
```

#### 9.4 실전 예시: Repository 선택

```rust
// src/main.rs
let repository = {
    #[cfg(feature = "rocksdb-backend")]
    {
        info!("Using RocksDB + PostgreSQL repository");
        Arc::new(RepositoryWrapper::new(
            PostgreSQLRepository::new(pool).await?,
            Some(kv_db),
        ))
    }

    #[cfg(not(feature = "rocksdb-backend"))]
    {
        info!("Using PostgreSQL only");
        Arc::new(RepositoryWrapper::new(
            PostgreSQLRepository::new(pool).await?,
            None,
        ))
    }
};

// 각 빌드에서 하나의 코드만 컴파일됨!
```

---

## 10. 주기적 작업: `tokio::time::interval`

### 📍 위치: `src/tasks/confirmation_checker.rs` - 30초마다 체크

```rust
use tokio::time::{interval, Duration};

pub async fn run_confirmation_checker(
    repository: Arc<RepositoryWrapper>,
    chain_configs: HashMap<String, ChainConfig>,
    sqs_notifier: Option<Arc<SqsNotifier>>,
    config: ConfirmationCheckerConfig,
) {
    if !config.enabled {
        info!("[ConfirmationChecker] Disabled by configuration");
        return;
    }

    info!(
        "[ConfirmationChecker] Starting with check_interval: {}s",
        config.check_interval_secs
    );

    // Interval 생성 (30초마다 tick)
    let mut check_interval = interval(Duration::from_secs(config.check_interval_secs));
    //                       ^^^^^^^^
    //                       tokio의 interval 타입

    loop {
        // 다음 tick까지 대기
        check_interval.tick().await;
        //             ^^^^
        //             첫 호출은 즉시 반환, 이후는 interval마다

        // Pending deposits 체크
        if let Err(e) = check_pending_deposits(
            &repository,
            &chain_configs,
            sqs_notifier.as_deref(),
        ).await {
            error!("[ConfirmationChecker] Error: {}", e);
            // 에러 발생해도 계속 실행
        }
    }
}

async fn check_pending_deposits(
    repository: &Arc<RepositoryWrapper>,
    chain_configs: &HashMap<String, ChainConfig>,
    sqs_notifier: Option<&SqsNotifier>,
) -> Result<(), String> {
    // 1. 미확정 입금 조회
    let pending_deposits = repository
        .get_pending_deposits()
        .await
        .map_err(|e| format!("Failed to get pending deposits: {}", e))?;

    if pending_deposits.is_empty() {
        info!("[ConfirmationChecker] No pending deposits to check");
        return Ok(());
    }

    info!("[ConfirmationChecker] Checking {} pending deposits", pending_deposits.len());

    // 2. 각 입금 확인
    for deposit in pending_deposits {
        let required_confirmations = chain_configs
            .get(&deposit.chain_name.to_uppercase())
            .or_else(|| chain_configs.get(&deposit.chain_name.to_lowercase()))
            .map(|c| c.required_confirmations)
            .unwrap_or(6);

        // 현재 블록 조회
        let current_block = match repository.get_last_processed_block(&deposit.chain_name).await {
            Ok(block) => block,
            Err(e) => {
                error!("[ConfirmationChecker] Failed to get block for {}: {}", deposit.chain_name, e);
                continue;
            }
        };

        // Confirmation 계산
        let confirmations = current_block.saturating_sub(deposit.block_number) + 1;
        //                                ^^^^^^^^^^^^^^
        //                                underflow 방지 (0 미만이면 0)

        info!(
            "[ConfirmationChecker] Deposit {} on {} - confirmations: {}/{}",
            deposit.tx_hash, deposit.chain_name, confirmations, required_confirmations
        );

        // 3. Required confirmations 도달 체크
        if confirmations >= required_confirmations {
            // 중복 체크
            let is_confirmed = repository
                .is_deposit_confirmed(&deposit.tx_hash)
                .await
                .map_err(|e| format!("Failed to check confirmation: {}", e))?;

            if is_confirmed {
                warn!("[ConfirmationChecker] Deposit {} already confirmed", deposit.tx_hash);
                continue;
            }

            info!(
                "[ConfirmationChecker] ✅ Deposit {} reached {} confirmations",
                deposit.tx_hash, confirmations
            );

            // DB 업데이트
            repository
                .update_deposit_confirmed(&deposit.tx_hash)
                .await
                .map_err(|e| format!("Failed to update: {}", e))?;

            // SQS 전송
            if let Some(notifier) = sqs_notifier {
                if let Err(e) = notifier.send_deposit_confirmed(
                    deposit.address,
                    deposit.wallet_id,
                    deposit.account_id,
                    deposit.chain_name.to_uppercase(),
                    deposit.tx_hash.clone(),
                    deposit.amount,
                    deposit.block_number,
                    confirmations,
                ).await {
                    error!("[ConfirmationChecker] Failed to send SQS: {}", e);
                } else {
                    info!("[ConfirmationChecker] ✅ SQS DEPOSIT_CONFIRMED sent");
                }
            }
        } else {
            info!(
                "[ConfirmationChecker] Deposit {} needs {} more confirmations",
                deposit.tx_hash, required_confirmations - confirmations
            );
        }
    }

    Ok(())
}
```

### 📖 Rust 문법 설명

#### 10.1 `tokio::time::interval`

```rust
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() {
    // 2초마다 tick
    let mut ticker = interval(Duration::from_secs(2));

    for i in 0..5 {
        ticker.tick().await; // 첫 호출은 즉시, 이후는 2초마다
        println!("Tick {}", i);
    }
}

// 출력:
// Tick 0  (즉시)
// Tick 1  (2초 후)
// Tick 2  (4초 후)
// Tick 3  (6초 후)
// Tick 4  (8초 후)
```

**주의사항**:
```rust
let mut ticker = interval(Duration::from_secs(5));

loop {
    ticker.tick().await;

    // 처리 시간이 5초보다 길면?
    expensive_operation().await; // 10초 걸림

    // 다음 tick은 즉시 발생! (drift 방지)
}
```

#### 10.2 `.saturating_sub()` - Overflow 방지

```rust
let a: u64 = 5;
let b: u64 = 10;

// 일반 뺄셈: panic!
// let result = a - b; // ❌ panic: attempt to subtract with overflow

// saturating_sub: 0으로 clamp
let result = a.saturating_sub(b); // ✅ result = 0
println!("{}", result); // 0

// 다른 saturating 연산들
let x: u8 = 250;
let y = x.saturating_add(10);  // 255 (u8 max)
let z = x.saturating_mul(2);   // 255 (u8 max)
```

#### 10.3 `.is_empty()` vs `.len() == 0`

```rust
let deposits: Vec<Deposit> = vec![];

// 두 방법 모두 가능
if deposits.is_empty() {      // ✅ 관용적
    println!("No deposits");
}

if deposits.len() == 0 {       // ✅ 동일한 의미
    println!("No deposits");
}

// is_empty()를 선호하는 이유:
// - 의도가 명확
// - 일부 타입은 len()보다 is_empty()가 더 빠름
//   (예: LinkedList는 is_empty()가 O(1), len()은 O(n))
```

#### 10.4 실전 예시: Fetcher의 주기적 블록 가져오기

```rust
// src/fetcher/runner.rs
pub async fn run_fetcher<F: Fetcher + 'static>(
    fetcher: Arc<F>,
    sender: mpsc::Sender<BlockData>,
    start_block: u64,
    interval_duration: Duration,
) {
    let mut current_block_number = start_block;

    // Interval ticker 생성
    let mut tick = interval(interval_duration);
    //                       ^^^^^^^^^^^^^^^^^
    //                       8초 (Sepolia)

    loop {
        // 다음 tick 대기
        tick.tick().await;

        info!(
            "[{} Fetcher] 블록 #{} 가져오는 중...",
            fetcher.chain_name(),
            current_block_number
        );

        // 블록 fetch 시도
        match fetcher.fetch_block_by_number(current_block_number).await {
            Ok(block_data) => {
                info!("[{} Fetcher] ✅ 블록 #{} 가져오기 성공!",
                    fetcher.chain_name(), current_block_number);

                // Analyzer로 전송
                if let Err(e) = sender.send(block_data).await {
                    error!("[{} Fetcher] Failed to send block: {}",
                        fetcher.chain_name(), e);
                    break;
                }

                current_block_number += 1; // 다음 블록으로
            }
            Err(e) => {
                warn!(
                    "[{} Fetcher] Failed to fetch block {}: {}, retrying in {:?}",
                    fetcher.chain_name(),
                    current_block_number,
                    e,
                    interval_duration / 2
                );

                // 실패 시 절반 시간 대기 후 재시도
                tokio::time::sleep(interval_duration / 2).await;
                // 블록 번호는 증가시키지 않음 (재시도)
            }
        }
    }
}
```

---

## 11. 종합 예제: 전체 흐름 이해

### 📍 위치: 전체 시스템 - 입금 감지부터 알림까지

```rust
// ===== 1. main.rs: 시스템 초기화 =====
#[tokio::main]
async fn main() -> Result<(), AppError> {
    // 1-1. 공유 자원 생성 (Arc로 래핑)
    let repository = Arc::new(RepositoryWrapper::new(/* ... */));
    let sqs_notifier = Arc::new(SqsNotifier::new(/* ... */));

    // 1-2. Channel 생성 (Fetcher → Analyzer)
    let (block_sender, block_receiver) = mpsc::channel::<BlockData>(128);

    // 1-3. Fetcher spawn
    let repo_clone = repository.clone();
    tokio::spawn(async move {
        run_fetcher(/* ... */, repo_clone, block_sender).await
    });

    // 1-4. Analyzer spawn
    let repo_clone = repository.clone();
    let sqs_clone = sqs_notifier.clone();
    tokio::spawn(async move {
        run_analyzer(block_receiver, repo_clone, sqs_clone).await
    });

    // 1-5. ConfirmationChecker spawn
    let repo_clone = repository.clone();
    let sqs_clone = sqs_notifier.clone();
    tokio::spawn(async move {
        run_confirmation_checker(repo_clone, sqs_clone).await
    });

    // 1-6. Ctrl+C 대기
    tokio::signal::ctrl_c().await?;
    Ok(())
}

// ===== 2. fetcher/runner.rs: 블록 가져오기 =====
async fn run_fetcher(
    fetcher: Arc<impl Fetcher>,
    repository: Arc<RepositoryWrapper>,
    sender: mpsc::Sender<BlockData>,
) {
    let mut current_block = repository
        .get_last_processed_block("SEPOLIA")
        .await
        .unwrap_or(9801775);

    let mut tick = interval(Duration::from_secs(8));

    loop {
        tick.tick().await; // 8초마다

        // 블록 가져오기
        match fetcher.fetch_block_by_number(current_block).await {
            Ok(block_data) => {
                // Analyzer로 전송
                sender.send(block_data).await.unwrap();
                current_block += 1;
            }
            Err(e) => {
                error!("Fetch error: {}", e);
                tokio::time::sleep(Duration::from_secs(4)).await;
            }
        }
    }
}

// ===== 3. analyzer/analyzer.rs: 트랜잭션 분석 =====
async fn run_analyzer(
    mut receiver: mpsc::Receiver<BlockData>,
    repository: Arc<RepositoryWrapper>,
    sqs_notifier: Option<Arc<SqsNotifier>>,
) {
    while let Some(block) = receiver.recv().await {
        info!("[Analyzer] 블록 데이터 수신!");

        // EVM 체인 분석
        match analyze_ethereum_block(block, &repository).await {
            Ok(deposits) => {
                info!("[Analyzer] found {} deposits", deposits.len());

                // 각 입금 처리
                for deposit in deposits {
                    if let Err(e) = process_deposit(
                        &repository,
                        "SEPOLIA",
                        deposit,
                        block.block_number,
                        6, // required_confirmations
                        sqs_notifier.as_deref(),
                    ).await {
                        error!("[Analyzer] Error: {}", e);
                    }
                }

                // 블록 번호 업데이트
                repository
                    .update_last_processed_block("SEPOLIA", block.block_number)
                    .await
                    .unwrap();
            }
            Err(e) => error!("[Analyzer] Analysis error: {}", e),
        }
    }
}

// ===== 4. analyzer/analyzer.rs: 입금 처리 =====
async fn process_deposit(
    repository: &Arc<RepositoryWrapper>,
    chain_name: &str,
    deposit: DepositInfo,
    current_block: u64,
    required_confirmations: u64,
    sqs_notifier: Option<&SqsNotifier>,
) -> Result<(), String> {
    // 4-1. Confirmation 계산
    let confirmations = current_block - deposit.block_number + 1;

    // 4-2. 중복 체크
    if repository.deposit_exists(&deposit.tx_hash).await? {
        let is_confirmed = repository
            .is_deposit_confirmed(&deposit.tx_hash)
            .await?;

        if is_confirmed {
            info!("Deposit {} already confirmed, skipping", deposit.tx_hash);
            return Ok(());
        }
    }

    // 4-3. DB 저장
    repository.save_deposit_event(
        &deposit.address,
        &deposit.wallet_id,
        deposit.account_id.as_deref(),
        chain_name,
        &deposit.tx_hash,
        deposit.block_number,
        &deposit.amount,
        deposit.amount_decimal,
    ).await?;

    // 4-4. Stage 1: DEPOSIT_DETECTED 전송
    if let Some(notifier) = sqs_notifier {
        notifier.send_deposit_detected(
            deposit.address,
            deposit.wallet_id,
            deposit.account_id,
            chain_name.to_string(),
            deposit.tx_hash,
            deposit.amount,
            deposit.block_number,
            confirmations,
        ).await?;

        info!("[DEPOSIT_DETECTED] ✅ SQS notification sent");
    }

    Ok(())
}

// ===== 5. tasks/confirmation_checker.rs: Confirmation 체크 =====
async fn run_confirmation_checker(
    repository: Arc<RepositoryWrapper>,
    sqs_notifier: Option<Arc<SqsNotifier>>,
) {
    let mut tick = interval(Duration::from_secs(30));

    loop {
        tick.tick().await; // 30초마다

        // 미확정 입금 조회
        let pending = repository.get_pending_deposits().await.unwrap();

        for deposit in pending {
            let current_block = repository
                .get_last_processed_block(&deposit.chain_name)
                .await
                .unwrap();

            let confirmations = current_block - deposit.block_number + 1;

            // Required confirmations 도달?
            if confirmations >= 6 {
                info!(
                    "[ConfirmationChecker] ✅ Deposit {} reached {} confirmations",
                    deposit.tx_hash, confirmations
                );

                // DB 업데이트
                repository
                    .update_deposit_confirmed(&deposit.tx_hash)
                    .await
                    .unwrap();

                // Stage 2: DEPOSIT_CONFIRMED 전송
                if let Some(notifier) = sqs_notifier.as_ref() {
                    notifier.send_deposit_confirmed(
                        deposit.address,
                        deposit.wallet_id,
                        deposit.account_id,
                        deposit.chain_name,
                        deposit.tx_hash,
                        deposit.amount,
                        deposit.block_number,
                        confirmations,
                    ).await.unwrap();

                    info!("[ConfirmationChecker] ✅ SQS DEPOSIT_CONFIRMED sent");
                }
            }
        }
    }
}
```

### 🔄 전체 데이터 흐름

```
1. Blockchain (Sepolia)
   ↓ (RPC call, 8초마다)
2. Fetcher (run_fetcher)
   ↓ (mpsc channel)
3. Analyzer (run_analyzer)
   ├─ EVM 블록 분석 (analyze_ethereum_block)
   ├─ RocksDB 주소 확인 (is_monitored_address)
   └─ 입금 처리 (process_deposit)
       ├─ PostgreSQL 저장 (save_deposit_event)
       └─ SQS 전송 (DEPOSIT_DETECTED)

4. ConfirmationChecker (run_confirmation_checker, 30초마다)
   ├─ PostgreSQL 조회 (get_pending_deposits)
   ├─ Confirmation 계산
   ├─ PostgreSQL 업데이트 (update_deposit_confirmed)
   └─ SQS 전송 (DEPOSIT_CONFIRMED)
```

---

## 12. 학습 팁

### 🎯 Rust 개념 우선순위

1. **필수 (먼저 마스터)**:
   - Ownership & Borrowing
   - `Result<T, E>` & `Option<T>`
   - `match` 표현식
   - `async/await`

2. **중요 (실전에서 자주 사용)**:
   - `Arc<T>` & `Rc<T>`
   - Trait & Trait Objects
   - Error handling (`?` operator)
   - Lifetimes

3. **고급 (필요할 때 학습)**:
   - `Pin` & `Unpin`
   - Unsafe Rust
   - Macros
   - Interior Mutability (`RefCell`, `Mutex`)

### 📚 추천 학습 순서

1. **The Rust Book** 읽기: https://doc.rust-lang.org/book/
2. **Rustlings** 연습: https://github.com/rust-lang/rustlings
3. **xScanner 코드 읽기** (이 문서 활용)
4. **작은 기능 추가해보기**:
   - 새로운 blockchain 추가
   - 새로운 notification 타입 추가
   - 테스트 작성

### 🔍 디버깅 팁

```rust
// 1. dbg! 매크로
let value = 42;
dbg!(value); // [src/main.rs:10] value = 42

// 2. #[derive(Debug)]
#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

let user = User { name: "Alice".to_string(), age: 30 };
println!("{:?}", user); // User { name: "Alice", age: 30 }
println!("{:#?}", user); // Pretty-print

// 3. unwrap 대신 expect
let value = some_option.expect("This should never be None");
// panic 시 메시지 출력

// 4. RUST_LOG 환경변수
// RUST_LOG=debug cargo run
// RUST_LOG=info cargo run
```

### 💡 자주 하는 실수

```rust
// ❌ 실수 1: String vs &str 혼동
fn process(s: String) { } // 소유권 이동
fn process(s: &str) { }   // 빌림 (더 flexible)

// ❌ 실수 2: await 빼먹기
let result = async_fn(); // Future만 반환, 실행 안 됨!
let result = async_fn().await; // ✅

// ❌ 실수 3: move 없이 closure
let data = vec![1, 2, 3];
tokio::spawn(async {
    println!("{:?}", data); // ❌ 컴파일 에러!
});
tokio::spawn(async move {
    println!("{:?}", data); // ✅
});

// ❌ 실수 4: ? in non-Result fn
fn main() {
    let file = File::open("file.txt")?; // ❌ main은 Result 반환 안 함
}

fn main() -> Result<(), std::io::Error> {
    let file = File::open("file.txt")?; // ✅
    Ok(())
}
```

---

## 📖 참고 자료

- **The Rust Programming Language**: https://doc.rust-lang.org/book/
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/
- **Tokio Tutorial**: https://tokio.rs/tokio/tutorial
- **async-trait**: https://docs.rs/async-trait/
- **sqlx**: https://docs.rs/sqlx/

이 튜토리얼은 xScanner 프로젝트의 실제 코드를 기반으로 작성되었습니다.
더 자세한 내용은 각 파일의 주석과 문서를 참고하세요! 🦀
