---
name: v151-migration-hang
description: "v1.5.1 프로덕션 사고(2026-08-01) — V313 pending 첫 실행 시 migrator.run 무한 블록(Windows 실환경). 근본원인 미해결, V314 때 재발. 우회=DB에 마이그레이션 선적용. v1.5.2 계획 포함"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2de8b3ba-b8a7-4d3d-bec8-87adf8a9081b
  modified: 2026-08-01T11:46:08.814Z
---

**사고 (2026-08-01, 집 Windows PC)**: v1.5.0→v1.5.1 첫 실행 시 PIN 인증 통과 직후 "처리 중..."에서 **무한 멈춤(CPU ~2% = 블록, 루프 아님)**. 로그 마지막 줄이 `[auth] verify_password: 인증 통과`.

**진단 (실 암호화 DB를 이 Mac에 동기화된 사본으로 재현)**:
- 멈춤 지점 = **pre-migration 백업 완료 직후 `db::initialize` 내 `migrator.run`(V313)**. 근거: backup/exit에 시도마다 백업 생성됨(백업은 완료) + app.db mtime 미변경 + -wal 0바이트(마이그레이션 쓰기 안 일어남) + version 312 고정.
- **v1.5.0이 정상이던 이유** = 신규 마이그레이션이 없어 이 백업+마이그레이트 구간을 **아예 건너뜀**. v1.5.1은 V313 pending이라 처음 진입 → 블록.
- **Mac 독립 harness로는 재현 불가**: 평문 sqlx 마이그레이터 / rusqlite 백업+동시연결+쓰기트랜잭션 / **sqlx+SQLCipher(libsqlite3-sys 공유) 마이그레이터** / 전체 interleaving 전부 <20ms 정상. → **실 Windows Tauri 런타임 고유의 무한 블록**으로 추정(파일락/AV/스레드풀 미확정).

**Why**: 근본원인 미해결. 증상만 우회했으므로 **다음 DB 마이그레이션(V314) 배포 시 동일 재발**한다.

**우회 처리(완료)**: 앱이 못 하는 마이그레이션을 오프라인 선적용 → pending 제거 → v1.5.1이 구간 건너뜀. 절차=[[offline-migration-apply]]. 2026-08-01 프로덕션 DB에 V313 선적용+재암호화+교체로 양 PC 정상화(원생33/출결978 무손실, 검사8 오태깅 0건, CS 7/30 표시 정상).
- 함정: config.json(`%APPDATA%\co.kr.ubcare.smarthb\config.json`)의 cloud_folder_path를 테스트로 로컬로 바꾸면 앱이 MyBox 아닌 로컬 DB를 열어 혼선 — 진단 후 반드시 MyBox 경로로 원복. 다월 함정=[[multi-month-period-pitfall]].

## v1.5.2 계획 (다음 착수 — 아직 미시작)
> 착수 시 `docs/sprint/sprint25.md` 정식 문서화. 검증은 **실 Windows PC 필요**(Mac/CI 불충분)이라 원장 PC 접근 가능할 때 진행.

- **트랙 A · 진단**: `build_pool`/`try_create_backup`/`migrator.run` 전후에 granular `eprintln!` 추가(open_pool_only·has_pending·백업 시작/완료·migrate 시작/완료). 릴리스는 `앱.exe 2> log.txt`로 stderr 캡처됨. **version 312 DB 사본**을 테스트 폴더에 두고 진단 빌드를 실 Windows에서 실행 → 로그 마지막 줄로 블록 지점 특정.
- **트랙 B · 하드닝(원인 무관, 즉시 배포 가능)**: 마이그레이션/백업 구간을 `tokio::time::timeout`으로 감싸 **무한 hang → 명시적 에러(fail-soft)** 전환. 보강: migrate 직전 WAL checkpoint + 백업 커넥션 완전 종료 보장 / 마이그레이션 전용 커넥션 / 백업을 pool 오픈 이전으로 이동.
- **범위**: db.rs(build_pool)·startup.rs·(필요 시)backup.rs 2~3개. **DB 스키마 변경/새 의존성 없음**. 트랙 B만이면 ≤50줄(hotfix급). 프로덕션은 이미 우회 안정화라 긴급 아님 → **Sprint 25 소규모** 권장.
- **DoD**: 블록 지점 로그 확보 + timeout·fail-soft로 무한 hang 제거 + 실 Windows 312 사본으로 정상 마이그레이션(또는 명시적 에러) 확인 + CHANGELOG/DEPLOY/ROADMAP 갱신 + v1.5.2 배포.

## 재현 자산 (다음 세션)
- **version 312 DB 필요**: 현 MyBox app.db는 이미 313. 312 원본 = **원장이 사고 전 떠둔 폴더 전체 백업** 또는 MyBox `backup/daily/`·`backup/exit/`의 V313 이전(2026-07-31 이하) 백업 파일. (스크래치패드 `PREREPLACE-app.db.bak`는 세션 종료 시 소멸).
- 이번 세션의 재현/오프라인마이그레이션 Rust harness(sqlx+SQLCipher, rusqlite, 키유도)는 스크래치패드에 있었음 — 소멸되므로 [[offline-migration-apply]] 절차로 재작성.
