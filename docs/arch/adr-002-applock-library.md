# ADR-002: app.lock 동시성 제어 라이브러리 선정

- **상태**: Proposed
- **날짜**: 2026-05-19
- **결정자**: SmartHB 개발팀 (Sprint 1 T6)

## Context (배경)

PRD §5.3 / §8.1 은 양 PC(Windows 교습소 + macOS 자택) 시점 분리 사용을 강제하기 위해 클라우드 동기화 폴더에 `app.lock` 파일 + heartbeat 메커니즘 도입을 의무로 규정한다. 두 디바이스가 동시에 SmartHB 를 실행하면 SQLCipher DB 파일이 동시 쓰기로 손상될 수 있으므로 OS 수준 / 파일 시스템 수준 락이 필수다.

### 제약 사항

- 양 OS (Windows 10/11 + macOS 12+) 지원 (PRD §5.1)
- 클라우드 동기화 폴더(MYBOX / iCloud / Dropbox) 호환 — 네이티브 OS 락은 클라우드 폴더에서 동작 불확실
- 60초 heartbeat + 5분 미갱신 강제 점유 정책 (PRD §5.3)
- JSON 메타데이터(device_id, last_heartbeat) 포함 — 단순 파일 존재 확인만으로 부족
- 단일 사용자(원장 1인) — 멀티 사용자 시나리오 미고려

---

## 1단계: Weighted Decision Matrix

| 기준 | 가중치 | 선택지 A | A | 선택지 B | B | 선택지 C | C |
|------|--------|---------|---|---------|---|---------|---|
| 빌드 단순성 (양 OS 통합) | 0.25 | fs2 단일 crate, 양 OS 자동 | **5** | file-lock 단일 crate 이나 활성도 낮음 | **3** | OS native API 분기(Win LockFileEx + macOS flock) — cfg 분기 코드 필요 | **2** |
| 양 OS 호환성 | 0.20 | Linux/Win/macOS 모두 advisory locking 지원 | **5** | Win/macOS 지원, Linux 일부 미흡 | **4** | 분기 코드로 정밀 제어 | **5** |
| API 단순성 | 0.15 | `file.lock_exclusive()`/`file.unlock()` — 직관적 | **5** | 비슷한 수준 | **4** | unsafe FFI 호출 + 핸들 관리 | **2** |
| 유지보수성 (crate 활성도) | 0.15 | fs2 v0.4 안정, 광범위 채택 | **4** | file-lock 최근 업데이트 적음 | **2** | 표준 라이브러리만 사용 | **5** |
| 보안성 (정밀 제어) | 0.10 | advisory lock 만 — 클라우드 동기화 폴더에서 OS 락은 어쨌든 신뢰 어려움 | **4** | 동일 | **4** | 모든 OS 보안 attribute 가능 | **5** |
| 성능 | 0.15 | 락 acquire/release ~μs | **4** | 동일 | **4** | 동일 | **5** |
| **총점** |  |  | **4.60** |  | **3.55** |  | **3.55** |

- A 총점 = 5×0.25 + 5×0.20 + 5×0.15 + 4×0.15 + 4×0.10 + 4×0.15 = 1.25 + 1.00 + 0.75 + 0.60 + 0.40 + 0.60 = **4.60**
- B 총점 = 3×0.25 + 4×0.20 + 4×0.15 + 2×0.15 + 4×0.10 + 4×0.15 = 0.75 + 0.80 + 0.60 + 0.30 + 0.40 + 0.60 = **3.45**
- C 총점 = 2×0.25 + 5×0.20 + 2×0.15 + 5×0.15 + 5×0.10 + 5×0.15 = 0.50 + 1.00 + 0.30 + 0.75 + 0.50 + 0.75 = **3.80**
- A 우세 명확 (B/C 대비 0.8~1.15 큰 폭)

---

## 2단계: SWOT + Trade-off

### 선택지 A: `fs2` advisory locking + 자체 heartbeat

- **Strengths**
  1. 양 OS 단일 API — `file.try_lock_exclusive()`, `file.unlock()` 호출만
  2. 광범위 채택 (rust 생태계 표준), 안정판
  3. JSON 메타데이터(device_id + heartbeat)를 락 파일 본문에 자유 저장
- **Weaknesses**
  1. Advisory only — OS 가 강제하지 않음 (다른 프로그램이 무시 가능). 단, SmartHB 자체가 락 규칙을 준수하면 충분
  2. 클라우드 동기화 지연 시 락 충돌 가능 — 60초 heartbeat 임계로 완화
- **Opportunities**
  1. heartbeat 메커니즘이 자체 구현이라 5분 정책 외 정책 변경 자유
  2. Linux 보조 환경(개발자) 지원 보너스
- **Threats**
  1. fs2 crate 자체 deprecate 가능성 (낮음, 활발 유지)
  2. 클라우드 동기화 race condition — heartbeat 갱신 vs 다른 디바이스 동시 acquire 시도

### 선택지 B: `file-lock` crate

- **Strengths**
  1. fs2 와 유사한 API
- **Weaknesses**
  1. 최근 업데이트 적음
  2. fs2 대비 community 채택 작음
- **Opportunities**
  1. 거의 없음
- **Threats**
  1. 유지보수 중단 위험

### 선택지 C: OS native API 분기

- **Strengths**
  1. OS 보안 attribute 정밀 제어 가능
  2. 표준 라이브러리만 — 외부 의존성 0
- **Weaknesses**
  1. Win `LockFileEx` + macOS `flock` 분기 코드 + unsafe FFI
  2. 신규 개발자 학습 비용 큼
  3. 코드 양 약 3배 증가
- **Opportunities**
  1. 특수 보안 요구 발생 시 즉시 대응
- **Threats**
  1. OS 메이저 업데이트 시 동작 변경 검증 부담

### Trade-off

| 선택 시 | 개선 (↑) | 저하 (↓) |
|---------|----------|----------|
| **A 선택** | 양 OS 빌드 자동화, 단순 API, 유지보수성 | OS 강제력 없음 (advisory) |
| B 선택 | (A 와 거의 동일하나 활성도 낮음) | 유지보수 중단 위험 |
| C 선택 | OS 보안 attribute 정밀 제어 | 코드 복잡도, unsafe, 빌드 매트릭스 |

### Risk

| 리스크 | 관련 | 영향도 | 완화 |
|--------|------|--------|------|
| 클라우드 동기화 지연으로 양 PC race condition | A 공통 | 중간 | 60초 heartbeat + 5분 강제 점유 — 동기화 지연 5분 초과 시 사용자 결정 |
| advisory lock 의 OS 비강제성 | A | 낮음 | SmartHB 자체가 락 규칙 준수 — 단일 사용자라 충돌 가능성 제한적 |
| fs2 deprecate | A | 낮음 | crate 활발 유지. 발생 시 B/C 로 마이그레이션 (분기 코드 1회 작성) |
| 클라우드 폴더의 OS 락 미지원 (네이버 MYBOX 케이스) | A | 중간 | fs2 의 락 자체가 동작 안 해도 heartbeat 파일 내용 검사로 충분 |

---

## 3단계: Decision

**선택지 A — `fs2` advisory locking + 자체 heartbeat** 채택.

> 1단계 총점: A=4.60, B=3.45, C=3.80 → A 우세 (B 대비 1.15, C 대비 0.80 차이)
> 핵심 Trade-off: A 채택으로 OS 보안 attribute 정밀 제어를 일부 포기하는 대신, 양 OS 빌드 자동화 / 단순 API / 유지보수성을 얻는다. 단일 사용자 데스크톱 앱이라 advisory 수준 락 + heartbeat 정책 조합으로 충분.

### 구체 적용 방안

1. **Cargo.toml 추가**:
   ```toml
   fs2 = "0.4"
   uuid = { version = "1", features = ["v4"] }
   chrono = { version = "0.4", features = ["serde"] }
   ```

2. **락 파일 구조**:
   ```rust
   #[derive(Serialize, Deserialize)]
   struct LockInfo {
       device_id: Uuid,           // 앱별 1회 OsRng v4 생성
       last_heartbeat: DateTime<Utc>,
   }
   ```

3. **상태 enum**:
   ```rust
   #[derive(Serialize)]
   #[serde(rename_all = "kebab-case", tag = "kind")]
   enum LockStatus {
       Free,                                           // 락 파일 없음
       OwnedBySelf,                                    // 우리가 점유
       OwnedByOther { stale: bool,                     // 5분 미갱신 시 stale=true
                      last_heartbeat_seconds_ago: i64 },
   }
   ```

4. **IPC 함수**:
   - `acquire_lock(force: bool)`: 락 파일 생성/덮어쓰기. `force=true` 일 때만 점유 중인 락 덮어쓰기 (5분 stale 검증은 호출자가 사전 수행)
   - `release_lock()`: 우리가 점유 중일 때만 파일 삭제 (다른 디바이스 락 보호)
   - `check_lock_status()`: 위 enum 반환

5. **heartbeat 갱신**: T6 에서는 IPC 만 구현. background task 통합은 T10 (시작 시퀀스).

6. **락 파일 위치**: T6 임시로 `./SmartHB-data/app.lock` (dev). T9 (마법사 + 클라우드 폴더 통합) 시점에 `<클라우드폴더>/smarthb/app.lock` 으로 이전.

---

## Consequences

### 긍정적 영향

- 양 OS 빌드 자동화 — fs2 단일 crate
- 신규 개발자 온보딩 단순 (file lock API 표준 이해로 충분)
- heartbeat 정책 변경 자유 (60초/5분 정책은 상수, 추후 사용자 설정 옵션 도입 가능)
- 클라우드 동기화 폴더에서 OS 락 미지원이어도 락 파일 내용 검사로 보완

### 부정적 영향 / 주의사항

- Advisory only — 외부 프로그램이 SmartHB 락 규칙 무시 가능 (수용 가능, 단일 사용자 데스크톱)
- 클라우드 동기화 지연 시 한 디바이스의 heartbeat 갱신이 다른 디바이스에 늦게 전달 — 5분 임계로 완화하나 race window 존재
- 비정상 종료(전원 차단) 시 락 파일 잔류 — 5분 후 강제 점유로 회복

### 후속 액션

- **T10 (시작 시퀀스)**: heartbeat 백그라운드 task (`tokio::spawn` + 60초 interval)
- **T9 (마법사)**: 락 파일 정식 경로 (클라우드 폴더 하위) 이전
- **UAT**: 양 PC 시점 분리 사용 시나리오 + 강제 점유 흐름 사용자 검증
- **장기**: PRD §10.3 Q? — 사용자가 락 임계 시간(5분)을 설정에서 조정할 수 있는지 검토
