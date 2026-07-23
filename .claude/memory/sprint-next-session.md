---
name: sprint-next-session
description: "⚠️ v1.4.0 후 2026-07-22 데이터 소실 사고→복구 완료. 다음 세션 = 회사 PC 릴레이 시작: git pull + 메모리 동기화 + /sprint-dev 23 (Sprint 23 재발방지 A+B, ADR-012 A안, 미구현). 새 세션 진입 시 가장 먼저 확인"
metadata:
  node_type: memory
  type: project
  originSessionId: dataloss-incident-2026-07-22
---

## ⬜ 다음 세션 진입 시 — 회사 PC 릴레이 시작 (최우선)

> **요약: git pull + 메모리 동기화 + `/sprint-dev 23`**

1. `git checkout develop && git pull` — 최신 develop(≥`8ba1102`) 받기
2. **메모리 미러 → 하네스 사용자 메모리 동기화** (절차: `.claude/memory/README.md`) — ⚠️ 안 하면 회사 PC의 Claude가 사고·복구·Sprint 23 맥락을 모름
3. (그 PC 첫 클론이면) `./SETUP.sh` + `.env` 준비 + `sqlx migrate run`으로 로컬 `SmartHB-dev.db` 생성
4. **`/sprint-dev 23`** 입력 → sprint23 브랜치 자동 생성. **T0(ADR-012) 완료 상태라 T1부터** 진행. (`/sprint-dev`는 사용자가 직접 입력)

- 병행(독립 트랙): **학원 PC 데이터 복구 최종 확인**(2026-07-23~, MYBOX 동기화 후 앱 실행→원생 31명 확인)
- 이연: A114(sync_single_date 이력 패턴), A127(cancel_makeup N+1)
- 주의: cipher 검증(T9)은 `--features cipher` 빌드 필요(Windows=Strawberry Perl, [[cipher-test-gate-trap]]). 나머지는 평문 빌드 가능. 회사 PC는 dev DB로 작업 → 프로덕션 실데이터 무관.

---

## ⚠️ 2026-07-22 프로덕션 데이터 소실 사고 + 복구 + Sprint 23 계획 (배경)

### 직전 상태
- **v1.4.0(Sprint 22)** 배포 완료 — 보강 분단위 부분차감(V311/V312), 출결 그리드 z-index, UX 개선. 마이그레이션 최신 **V312**.

### 사고 (학원 Windows PC, v1.3 / schema V310)
장시간 방치 + 강제종료 반복 → 로그인 오류 → 재로그인하니 원생 등 전체 데이터 0건(전면 소실). + 이전부터 "장시간 미사용 후 저장 오류(재시작하면 정상)" 간헐 발생.
- **근본원인**: 라이브 SQLite/SQLCipher DB를 클라우드 동기화 폴더(MYBOX)에 열어둔 채 사용.
  - ① 전면소실: 클라우드 파일 일시부재(dehydration) 시 `db.rs create_if_missing(true)`가 가드 없이 빈 DB 날조 + 마이그레이션/시드 → 무결성 quick_check가 빈 DB를 "정상"으로 fail-soft 판정(auto_restore 미발동). **삭제 버그 아님**(전수감사 확인). 빈 DB=229KB(시드만)/정상=499KB급, salt.bin 원본유지(mtime 6/28)라 salt 재생성 없음.
  - ② 유휴 저장오류: startup PRAGMA(key 등)가 풀에 1회만 적용 → 유휴 중 커넥션 교체 시 "맨 커넥션"이 키 없어 NOTADB, 재시작만 복구.
- 상세 RCA: `docs/incidents/2026-07-22-data-loss-rca.md` (결함 C1~C3/H1~H5/M1~M5/B).

### 복구 (완료)
- 오프라인 복호화로 폴더 내 전체 .db 행수 검사 → 최신 온전본 `backup/exit/app_20260722_074410.db`(원생 31, 1238행, 7/22 16:44) 식별 → MYBOX `app.db` 원자적 교체(빈 DB 보존) → 재검증(원생 31). 복구법: [[data-loss-recovery-method]].
- v1.3(V310)→v1.4(V311/V312) 마이그레이션도 복구 복사본으로 테스트 통과(무손실, makeup_allocations 백필 정상).

### Sprint 23 계획 (커밋·push 완료 `521c3ef`, 미구현)
- 주제: 사고 재발방지 A(데이터안전) + B(2번째PC로그인). **DB 마이그레이션·새 의존성 없음.**
- **ADR-012 = A안**(`docs/arch/adr-012-db-live-location.md`): 라이브 DB를 **클라우드 폴더에 유지 + 접근 강화**(데이터 로컬 이전 안 함). 매트릭스 A 4.30 > B 3.55 > C 3.30. B안(로컬+핸드오프)은 ROADMAP에 향후 Phase 후보 등록.
- Task T0~T9(29h), 계획 `docs/sprint/sprint23.md`: T0 ADR완료 / T1 after_connect 키재적용(②유휴오류) / T2 create_if_missing 가드(①) / T3 복원강화 / T4 백업검증 / T5 config통일+salt가드 / T6 유휴 close+재연결 / T7 2번째PC 키채택(B) / T8 device.id+STALE / T9 통합검증.

관련: [[workflow-no-pr]], [[deploy-version-three-files]], [[data-loss-recovery-method]], [[ntfs-power-loss-pattern]]
