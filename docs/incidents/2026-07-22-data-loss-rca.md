# 사고 분석 (RCA) — 프로덕션 데이터 전면 소실 + 유휴 후 저장 오류

- **작성일**: 2026-07-22
- **사고 발생 환경**: 학원 Windows PC, 앱 버전 **v1.3** (DB 스키마 maxV=310)
- **영향**: 원생 등 전체 도메인 데이터가 앱에서 0건으로 표시됨 (전면 소실로 체감)
- **데이터 복구**: ✅ 완료 (원생 31명 등 1,238행, 손실 없음)
- **근본 원인 수정**: ⬜ 예정 (Sprint 23, v1.5)

> 본 문서는 코드 정밀 검증(5개 영역 병렬 조사) + 실제 클라우드 폴더 포렌식으로 확정한 결과다. 재발방지 스프린트(A+B)의 입력 문서.

---

## 1. 요약 (TL;DR)

두 증상은 **같은 뿌리**에서 나왔다: **라이브 SQLite/SQLCipher DB를 클라우드 동기화 폴더(MYBOX)에 열어둔 채 사용**하는 아키텍처.

- **증상 ① 전면 소실**: 클라우드 파일이 유휴/강제종료로 일시 부재(dehydration)일 때, 앱 시작의 `create_if_missing(true)`가 **가드 없이 올바른 경로에 빈 DB를 날조**하고 마이그레이션+시드를 돌려 부팅. 무결성 검사가 빈 DB를 "정상"으로 판정해 자동복원도 미발동.
- **증상 ② 유휴 후 저장 오류(재시작하면 정상)**: startup PRAGMA(특히 `PRAGMA key`)가 **커넥션이 아니라 풀에 1회만** 적용됨. 유휴 중 파일 핸들이 stale/교체되어 커넥션이 죽으면, sqlx가 여는 새 "맨 커넥션"에는 키가 없어 이후 모든 저장이 `NOTADB`로 실패. 재시작(=initialize 재실행)만이 복구.

**삭제 버그는 아니다** — 전수 감사 결과 우발적 전체 wipe IPC/CASCADE는 없다. 원인은 "삭제"가 아니라 "빈 DB 날조 + 정상 오판"이다.

---

## 2. 사고 경위 (원장님 증언)

1. 앱을 장시간 사용하지 않고 띄워둠 → 여러 번 종료 반복
2. (이전부터) 장시간 방치 후 저장 시 **가끔 오류** → 껐다 켜면 정상이라 무시하고 사용
3. 재사용 시 6자리 PIN 로그인+엔터에서 **오류** → 창 종료 후 재로그인 성공
4. 수납 입력 후 종료 → 재로그인하니 **원생정보 등 전체 데이터 소실**

---

## 3. 포렌식 증거 (복구한 클라우드 폴더)

| 파일 | 크기 | 원생 | 도메인 총행 | 시드(pm/sc/sf) | maxV | 판정 |
|------|:---:|:---:|:---:|:---:|:---:|------|
| 라이브 app.db (사고 후) | 229,376 | 0 | 0 | 5 / **6** / 4 | 310 | **fresh 빈 DB** |
| backup/exit/app_20260722_074410.db (7/22 16:44) | 499,712 | 31 | 1238 | 5 / **8** / 4 | 310 | **정상 (복원 대상)** |
| backup/daily/app_20260628_011655.db (최초) | 229,376 | 0 | 0 | 5 / 6 / 4 | 307 | 최초 fresh |

**결정적 판별:**
- 빈 DB는 **시드 있음 + 도메인 0행** → 삭제가 아니라 마이그레이션+시드로 **새로 만들어진 DB**.
- `schedule_codes` 정상=8(원장 추가분 포함) vs 빈 DB=6(시드 기본값) → 사용자 데이터가 없는 초기 시드 상태.
- **salt.bin mtime = 6/28 (원본)** → salt/키 재생성 없었음 (set_password 미실행).
- 빈 DB가 **올바른 클라우드 경로**에 **원본 키로 암호화** → config-fallback 경로(`./SmartHB-data`)가 아님.
- **restore_rollback/ 비어 있음** → integrity auto_restore 미실행.
- salt.bin, PIN(000000)으로 모든 백업 복호화 성공 (무결성 ok).

---

## 4. 근본 원인 (확정)

### 증상 ① — 전면 소실 인과 시퀀스
1. 장시간 유휴 + 강제종료 반복으로 MYBOX가 `app.db`를 방출(dehydrate)하거나 미다운로드 placeholder/0바이트 상태로 둠.
2. 앱 시작 → config 정상(salt·경로 정상)이라 `data_root`는 올바른 클라우드 폴더로 해석.
3. `db.rs:89` `build_pool`의 **`create_if_missing(true)`** 가 그 경로에 **빈 DB를 새로 생성**, `create_dir_all`로 폴더까지 재생성(`db.rs:81`).
4. 마이그레이션 V001~V310 + 시드 적용 → 229KB 빈 DB. 신규 DB라 `has_pending_migrations=false`(`db.rs:120`) → **마이그레이션 직전 백업도 스킵**.
5. `integrity.rs:309-314` startup quick_check가 부재/빈 DB를 **`Ok`(정상)** 로 fail-soft 판정 → `startup.rs:175`의 auto_restore **미진입**.
6. 원장님이 빈 화면 상태로 수납 입력 후 종료 → `exit_hook`(`startup.rs:325`)이 **빈 DB를 exit 백업**으로 기록(7/22 19:12).

**핵심 결함:** `build_pool` 앞단에 **"setup 완료됐고 셋업 흔적(salt.bin)이 있는데 app.db가 없으면 새로 만들지 말고 중단"** 이라는 가드가 전혀 없다.

### 증상 ② — 유휴 후 저장 오류 인과
- `db.rs:91-98`: `PRAGMA key`·`busy_timeout`·`journal_mode=WAL`·`foreign_keys` 등이 **풀에 명령형 1회** 적용되고, sqlx `after_connect` 훅에 없음.
- 유휴 중 클라우드가 파일 핸들 stale/교체 → 단일 커넥션 사망 → sqlx가 여는 **새 "맨 커넥션"에 키 없음** → 모든 실쿼리 `NOTADB` 실패. `SELECT 1` 핑은 페이지 미접근이라 통과 → sqlx가 깨진 커넥션을 계속 내어줌 → **앱 내 재시도 불가, 재시작만 복구**.
- `startup.rs:294` 2시간 백그라운드 체크포인트가 유휴 후 첫 쿼리로서 커넥션을 먼저 깨뜨려, 사용자의 첫 저장이 바로 실패하는 타이밍 유발.

---

## 5. 검증된 결함 목록 (심각도순)

### 🔴 Critical (이번 사고 직접 원인)
| # | 위치 | 결함 |
|---|------|------|
| C1 | `db.rs:89` | `create_if_missing(true)` + "setup 완료인데 DB 부재" 가드 전무 → 빈 DB 날조 |
| C2 | `integrity.rs:309-314` | 부재/빈 DB를 `Ok`로 fail-soft → auto_restore 미발동, 빈 DB 라이브 승격 |
| C3 | `db.rs:91-98` | 커넥션 재연결 시 `PRAGMA key`+pragma 유실 → 유휴 후 세션 영구 저장 실패(증상 ②) |

### 🟠 High (재발/증폭 경로)
| # | 위치 | 결함 |
|---|------|------|
| H1 | `integrity.rs:209-237` | 복원 시 stale `-wal`/`-shm` 미처리 → 복원 직후 재손상 위험(리허설엔 방어 있음) |
| H2 | `backup.rs:271,423,215,64` | 빈 DB 무검증 백업 + 시각 기반 rotation → open-close 5회면 정상 exit 백업 전멸 |
| H3 | `integrity.rs:242,258` | auto_restore가 exit 계층만 스캔 → exit 전멸 시 daily/weekly 폴백 없음 |
| H4 | `integrity.rs:178,277` | 복원 소스 행수/신선도 무검증 → 빈 백업으로 정상 데이터 덮을 수 있음 |
| H5 | `startup.rs:294` | 2시간 체크포인트가 유휴 후 커넥션을 먼저 깨뜨림 |

### 🟡 Medium (별개 잠재 위험 — 이번 사고와 무관하나 수정 필요)
| # | 위치 | 결함 |
|---|------|------|
| M1 | `paths.rs:122-137` vs `setup.rs:72-100` | config 처리 불일치: startup은 무음 fallback(상대경로 `./SmartHB-data`), setup은 손상감지+백업. 손상 시 `setup_completed=false`로 마법사 강제(`page.tsx:59`)하여 salt.bin(진짜 SSOT) 무시 |
| M2 | `auth.rs:626` | `set_password`가 기존 salt 존재 검사 없이 새 salt 생성 → 마법사에서 빈/다른 폴더·새 PIN 시 복호화 불가 (프론트 가드로 same-folder는 구제) |
| M3 | `lock.rs:35,76,155,327` | device.id 손상 시 자기 락을 "타 PC"로 오판 → 최대 24h 로그인 차단 ("로그인 오류"의 유력 후보) |
| M4 | `lock.rs:38-39,155` | heartbeat 제거(Sprint 17)로 STALE 임계가 "활동"이 아닌 "시작 시각" 기준 |
| M5 | `paths.rs:9`, `lock.rs:192` | sync(외부 파일 변경 감지) 모듈은 주석만 있고 **미구현** — 클라우드 파일 교체 감지 로직 부재 |
| B | `auth.rs:336-356,659-695` | (문제 B) DB 키가 PC별 키체인에만 저장·미동기화 + 2번째 PC 키 채택 경로 없음 → 신규 PC 로그인 불가 |

### 참고
- `synchronous` 미설정 = **FULL**(NORMAL 아님) → 단일 PC 전원손실 torn 위험은 낮음. 단 클라우드 파일 교체는 fsync 무관.
- **우발적 전체 wipe IPC/CASCADE 없음** (전수 감사 완료). CASCADE 2건(`student_schedules→students`, `payments→bills`) 모두 안전. 마이그레이션 111의 `DELETE FROM payments`는 fresh DB에서만 실행.

---

## 6. 데이터 복구 기록 (완료)

- 앱과 동일한 키 유도(PBKDF2-HMAC-SHA256, 60만회) + 동일 SQLCipher(rusqlite 0.32/libsqlite3-sys 0.30)로 오프라인 복구 도구 제작.
- salt.bin + PIN(000000)으로 폴더 내 전체 `.db` 복호화·행수 검사 → 최신 정상본 **`backup/exit/app_20260722_074410.db`**(원생 31, 총 1238행) 식별.
- MYBOX 동기화 일시중지 → 라이브 `app.db`를 원자적 교체(원본 보존) → 복호화 재검증(원생 31 확인) 완료.
- v1.3(V310) → v1.4(V311/V312) 마이그레이션 테스트 통과: 데이터 무손실, makeup_allocations 백필 정상, makeup_done 1건→absent(의도된 부분보강 정정).

---

## 7. 재발방지 범위 (Sprint 23, v1.5)

### A — 데이터 안전 (필수)
- **A1 (근본)**: 라이브 DB를 클라우드 폴더 밖(로컬)으로 이전, 클라우드엔 스냅샷/백업만 동기화. 일회성 데이터 이전(재설정) 흐름 포함.
- **A2**: `create_if_missing` 가드 — 셋업 흔적(salt.bin) 있는데 app.db 부재 시 생성 금지·중단+안내(동기화 대기).
- **A3**: `after_connect` 훅으로 매 커넥션 `PRAGMA key`+pragma 재적용 (증상 ② 자기회복화).
- **A4**: 복원 시 WAL 사이드카 원자 처리 + 복원 소스 행수/신선도 검증 + exit→daily→weekly 폴백.
- **A5**: 백업 빈/열세 소스 거부 + 마지막 정상 백업 축출 금지 rotation.
- **A6**: config startup/setup 손상처리 통일(무음 fallback 제거, salt.bin을 SSOT로) + `set_password` 기존 salt 하드 가드.

### B — 2번째 PC 로그인 (권장)
- **B1**: 신규 PC에서 `PBKDF2(PIN, salt)` 로 키 재유도 → DB로 검증 → 성공 시 로컬 키체인에 채택. (salt 재생성 금지)
- **B2**: device.id 손상 시 락 오판 방지, STALE 판정을 활동 기준으로 보정.

> A1(데이터 이전)이 마이그레이션/재설정을 동반하면, 배포 전 복구 복사본으로 마이그레이션 테스트를 재실행한다.
