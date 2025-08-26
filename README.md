# MEV Simple Training

Rust와 Artemis 프레임워크를 사용한 MEV(Maximal Extractable Value) 프론트러닝 봇 구현 프로젝트입니다.

## 프로젝트 구조

```
mev-simple-training/
├── winner-project/          # Foundry 프로젝트 (Winner 컨트랙트)
│   ├── src/Winner.sol       # 타겟 컨트랙트
│   └── script/DeployWinner.s.sol  # 배포 스크립트
├── artemis/                 # MEV 봇 프레임워크
│   ├── bin/winner_bot/      # 프론트러닝 봇 실행 바이너리
│   └── crates/strategies/winner_snipe/  # 프론트러닝 전략
└── README.md               # 이 파일
```

## 사전 요구사항

### 필수 도구 설치

1. **Rust 설치**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Foundry 설치**
   ```bash
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```

## 환경 구축 및 실행

### 1단계: 로컬 테스트넷 시작

5초 블록타임으로 Anvil을 실행합니다:

```bash
anvil --block-time 5
```

**중요:** 기본 설정으로 anvil을 실행하면 트랜잭션이 즉시 블록에 포함되어 봇이 경쟁할 시간이 없습니다. `--block-time 5` 옵션으로 5초마다 블록을 생성하여 mempool에서 트랜잭션들이 경쟁할 수 있게 합니다.

### 2단계: Winner 컨트랙트 배포

새 터미널에서:

```bash
cd winner-project
forge script script/DeployWinner.s.sol \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --broadcast
```

배포 완료 후 출력되는 Contract Address를 복사해둡니다.

### 3단계: 봇 설정

배포된 컨트랙트 주소를 봇에 설정합니다:

```bash
cd ../artemis
```

`bin/winner_bot/src/main.rs` 파일에서 `target` 변수를 배포된 컨트랙트 주소로 변경:

```rust
let target: Address = "0x[배포된_컨트랙트_주소]".parse()?;
```

### 4단계: 봇 빌드 및 실행

```bash
cargo build -p winner_bot
cargo run -p winner_bot
```

봇이 시작되면 다음과 같은 로그가 출력됩니다:

```
Starting Winner Bot!
Target contract: 0x[컨트랙트_주소]
Bot address: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
Setting up engine...
Starting engine...
Listening for transactions...
```

### 5단계: 프론트러닝 테스트

새 터미널에서 테스트 트랜잭션을 전송합니다:

```bash
cd winner-project
cast send 0x[배포된_컨트랙트_주소] "setWinner()" \
  --private-key 0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356 \
  --rpc-url http://127.0.0.1:8545 \
  --gas-price 1000000000
```

### 6단계: 결과 확인

프론트러닝이 성공했는지 확인:

```bash
cast call 0x[배포된_컨트랙트_주소] "winner()" --rpc-url http://127.0.0.1:8545
```

봇의 주소(`0x70997970C51812dc3A010C7d01b50e0d17dc79C8`)가 출력되면 프론트러닝 성공입니다!

## 동작 원리

### Winner 컨트랙트

```solidity
contract Winner {
    address public winner;
    
    function setWinner() external {
        if(winner != address(0))
            revert("already set winner");
        winner = msg.sender;
    }
}
```

- `setWinner()` 함수를 첫 번째로 호출한 주소가 `winner`로 설정됩니다
- 한 번 설정되면 다른 호출은 실패합니다

### MEV 봇 동작

1. **트랜잭션 감지**: WebSocket을 통해 mempool의 pending 트랜잭션 모니터링
2. **타겟 식별**: `setWinner()` 함수 호출을 식별 (selector: `0xed05084e`)
3. **가스 경쟁**: 원본 트랜잭션보다 20% 높은 가스 가격으로 동일한 함수 호출
4. **프론트러닝**: 더 높은 가스 덕분에 봇의 트랜잭션이 먼저 처리됨

### 성공 로그 예시

```
Processing transaction: 0xabcd1234...
Transaction to: Some(0x[컨트랙트_주소])
Transaction targets our contract!
DETECTED setWinner() call! Preparing front-run...
Original gas price: 1000000000 wei
Boosted gas price: 1200000000 wei
Front-run transaction sent! Hash: 0xefgh5678...
```

## 핵심 파일 설명

### Winner.sol
- 테스트용 타겟 컨트랙트
- 첫 번째 `setWinner()` 호출자를 winner로 설정

### winner_snipe/src/lib.rs
- 프론트러닝 전략 구현
- 트랜잭션 감지, 가스 경쟁, 직접 전송 로직

### winner_bot/src/main.rs
- 봇 실행 바이너리
- Artemis 엔진 설정 및 구성 요소 연결

## 주의사항

1. **블록 타임**: 반드시 `anvil --block-time 5`로 실행해야 프론트러닝 가능
2. **가스 설정**: 테스트 시 낮은 가스 가격 사용 (봇이 20% 더 높게 설정)
3. **계정 잔액**: 봇 계정(`0x70997970...`)에 충분한 ETH 확보 필요

## 확장 가능성

이 기본 프레임워크를 바탕으로 다음과 같은 고급 MEV 전략 구현 가능:

- DEX 차익거래 봇
- NFT 스나이핑
- 청산 봇
- 샌드위치 공격 감지 및 방어
