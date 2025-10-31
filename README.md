# xScanner

xScanner는 다중 블록체인 데이터를 스캔하고 특정 정보를 수집하는 Rust로 만들어진 프로젝트입니다.

## 주요 기능

* **다중 블록체인 지원:** 이더리움, 비트코인, TRON, THETA, ICON 등 여러 블록체인을 동시에 모니터링할 수 있습니다.
* **동적 블록체인 추가:** 설정 파일만 수정하면 새로운 블록체인을 쉽게 추가할 수 있습니다.
* **사용자 정의 가능한 스캔 간격:** `config.toml` 파일을 통해 각 블록체인별 스캔 간격을 설정할 수 있습니다.
* **시작 블록 설정:** 스캔을 시작할 특정 블록 번호를 설정하여 과거 데이터부터 분석할 수 있습니다.
* **데이터 저장소 지원:** 수집된 데이터를 PostgreSQL 데이터베이스와 LevelDB 데이터베이스에 저장할 수 있습니다.
* **고객 주소 관리:** 특정 고객 주소 목록을 파일에서 읽어와 관리할 수 있습니다 (LevelDB 활용).
* **모듈화된 구조:** 코드는 `coin`, `fetcher`, `repository` 모듈로 구성되어 있어 유지보수 및 확장이 용이합니다.
* **비동기 처리:** Tokio를 사용한 고성능 비동기 블록 처리
* **통합 테스트:** 프로젝트의 주요 기능을 검증하는 통합 테스트를 포함하고 있습니다.

## 시작하기

### Prerequisites

* Rust (최신 안정화 버전 권장)
* Cargo (Rust 패키지 관리자)
* PostgreSQL (데이터베이스 사용 시)

### 빌드 및 실행

1.  **저장소 클론:**
    ```bash
    git clone <your_repository_url>
    cd xScanner
    ```

2.  **설정 파일 구성:** `config.toml` 파일을 필요에 맞게 수정합니다. 특히 블록체인 API 키 및 PostgreSQL 연결 URL을 정확하게 설정해야 합니다.

3.  **빌드:**
    ```bash
    cargo build --release
    ```

4.  **실행:**
    ```bash
    cargo run --release
    ```

## 설정

프로젝트의 설정은 `config.toml` 파일을 통해 관리합니다.

### `[blockchain.ethereum]`

* `api`: 이더리움 API 엔드포인트 URL을 설정합니다 (예: Infura, Alchemy).
* `symbol`: 이더리움 심볼 ("eth").
* `start_block`: 스캔을 시작할 이더리움 블록 번호를 설정합니다.
* `interval_secs`: 이더리움 데이터를 스캔할 간격 (초)을 설정합니다.

### `[blockchain.bitcoin]`

* `api`: 비트코인 API 기본 URL을 설정합니다 (예: Blockchain.info). 클라이언트 코드에서 필요한 경로를 조정할 수 있습니다.
* `symbol`: 비트코인 심볼 ("btc").
* `start_block`: 스캔을 시작할 비트코인 블록 번호를 설정합니다.
* `interval_secs`: 비트코인 데이터를 스캔할 간격 (초)을 설정합니다.

### `[repository]`

* `postgresql_url`: PostgreSQL 데이터베이스 연결 URL을 설정합니다.
* `leveldb_path`: LevelDB 데이터베이스를 저장할 경로를 설정합니다 (예: `./customer_db`).
* `customer_address_file`: 고객 주소 목록이 있는 파일 경로를 설정합니다 (예: `./customer_addresses.txt`).

## 데이터베이스

xScanner는 데이터를 저장하기 위해 두 가지 데이터베이스를 사용합니다.

* **PostgreSQL:** `sqlx` 크레이트를 사용하여 관계형 데이터를 저장하는 데 사용될 수 있습니다. 연결 URL은 `config.toml` 파일에서 설정합니다.
* **LevelDB:** `leveldb` 크레이트를 사용하여 키-값 형태의 데이터를 저장하는 데 사용됩니다. 주로 고객 주소와 같은 정보를 저장하는 데 활용될 수 있으며, 경로를 `config.toml` 파일에서 설정합니다.

## 지원 블록체인

현재 다음 블록체인을 지원합니다:

* **Ethereum (ETH)**: Infura, Alchemy 등 RPC 엔드포인트
* **Bitcoin (BTC)**: Blockchain.info API
* **TRON**: TronGrid API
* **THETA**: THETA RPC 엔드포인트
* **ICON**: ICON RPC 엔드포인트

### 새로운 블록체인 추가하기

새로운 블록체인을 추가하려면:

1. `src/coin/` 디렉토리에 새 모듈 생성 (`model.rs`, `client.rs`, `mod.rs`)
2. `src/fetcher/` 디렉토리에 새 fetcher 구현
3. `src/types.rs`의 `BlockData` enum에 새 블록 타입 추가
4. `src/analyzer/analyzer.rs`에 주소 추출 로직 추가
5. `src/main.rs`의 매칭 로직에 새 블록체인 추가
6. `config.toml`에 새 블록체인 설정 추가

## 테스트

프로젝트에는 주요 기능을 검증하기 위한 통합 테스트가 포함되어 있습니다. 테스트를 실행하려면 다음 명령어를 사용하세요.

```bash
cargo test --tests
```