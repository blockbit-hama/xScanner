# xScanner

xScanner는 이더리움과 비트코인 블록체인 데이터를 스캔하고 특정 정보를 수집하는 Rust로 만들어진 프로젝트입니다.

## 주요 기능

* **이더리움 및 비트코인 블록체인 데이터 스캔:** Infura 및 Blockchain.info API를 사용하여 실시간 또는 특정 블록부터 데이터를 가져옵니다.
* **사용자 정의 가능한 스캔 간격:** `config.toml` 파일을 통해 각 블록체인별 스캔 간격을 설정할 수 있습니다.
* **시작 블록 설정:** 스캔을 시작할 특정 블록 번호를 설정하여 과거 데이터부터 분석할 수 있습니다.
* **데이터 저장소 지원:** 수집된 데이터를 PostgreSQL 데이터베이스와 LevelDB 데이터베이스에 저장할 수 있습니다.
* **고객 주소 관리:** 특정 고객 주소 목록을 파일에서 읽어와 관리할 수 있습니다 (LevelDB 활용 추정).
* **모듈화된 구조:** 코드는 `coin`, `Workspaceer`, `repository` 모듈로 구성되어 있어 유지보수 및 확장이 용이합니다.
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

## 테스트

프로젝트에는 주요 기능을 검증하기 위한 통합 테스트가 포함되어 있습니다. 테스트를 실행하려면 다음 명령어를 사용하세요.

```bash
cargo test --tests