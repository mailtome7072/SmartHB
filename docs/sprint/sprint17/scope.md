---
Sprint: 17  |  Date: 2026-06-30  |  Session: #1 (완료)
---

## 이번 세션에서 수정할 파일
| 파일 | 수정 횟수 | 비고 |
|------|---------|------|
| `src-tauri/src/commands/setup.rs` | [1회] | T1: WAL 체크포인트 에러 처리 강화 |
| `src-tauri/src/commands/backup.rs` | [1회] | T2: 백업 atomic write (tmp+rename) |
| `src-tauri/src/startup.rs` | [2회] | T3: 자동 복원 재검증 + T4: hourly 간격 + T5: heartbeat 제거, simplify |
| `src-tauri/src/commands/integrity.rs` | [1회] | T3: 재검증 로직 |
| `src-tauri/src/commands/lock.rs` | [1회] | T5: heartbeat 제거 |
| `src-tauri/src/commands/sync.rs` | [1회] | T6: 모듈 삭제 |
| `src-tauri/src/lib.rs` | [1회] | T6: sync 등록 제거 |
| `src/lib/tauri/index.ts` | [1회] | T6: checkSyncStatus 삭제 |
| `src/components/layout/app-shell.tsx` | [1회] | T6: polling 제거 |
| `src/components/layout/top-bar.tsx` | [1회] | T6: 동기화 표시 UI 제거 |

## 수정하지 않을 파일 (Forbidden Areas 포함)
- [x] `.github/workflows/` — CI/CD 파이프라인 (hook이 차단)
- [x] `SETUP.sh` — 초기화 스크립트 (hook이 차단)
- [x] DB 마이그레이션 — Sprint 17은 마이그레이션 없음
- [x] 새 의존성 추가 — Sprint 17은 새 의존성 없음

## 완료 기준 (이번 세션)
- ✅ T1: WAL 체크포인트 실패 시 DB 폴더 변경 중단 + 사용자 오류 메시지 반환
- ✅ T2: 백업 파일 tmp 생성 → perform_backup_with_cipher(quick_check 포함) → rename 방식으로 변경
- ✅ T3: 자동 복원 후 quick_check 재검증, 실패 시 rollback 복원 시도 (최대 3회)
- ✅ T4: hourly 백업 간격 3600 → 7200초 변경
- ✅ T5: heartbeat 루프 제거, app.lock 시작/종료 시에만 생성/삭제
- ✅ T6: sync.rs IPC + 프론트엔드 polling + 동기화 표시 UI 완전 제거
- ✅ T7: cargo test(411) / clippy --all-targets / cipher check / pnpm lint + tsc + build 전수 통과

## simplify 적용 결과
- `integrity.rs`: `max_attempts` 변수 제거 → `.take(3)` 직접 사용
- `setup.rs`: WAL `map_err(|_|)` → `map_err(|e|)` + `eprintln!` (진단 로그 추가)
- 스킵: heartbeat 재추가 여부 — 설계 결정(1인 운영 전제), 사용자 확인 완료
