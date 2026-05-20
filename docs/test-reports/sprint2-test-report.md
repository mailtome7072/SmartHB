# Test Report — Sprint 2 (2026-05-20)

> 검토 대상: `sprint2` 브랜치
> 실행 환경: Windows 11 Enterprise (개발 머신, cipher feature off)

---

## 자동 검증 결과

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo test --lib` | ✅ 통과 | 97 passed, 0 failed (30.79s) |
| `cargo clippy --all-targets -- -D warnings` | ✅ 통과 | 경고 없음 |
| `pnpm tsc --noEmit` | ✅ 통과 | 타입 오류 없음 |
| `pnpm lint` | ✅ 통과 | ESLint 경고/오류 없음 |
| `pnpm build` | ✅ 통과 | 5/5 static pages prerendered (/, /_not-found, /lock) |

### cargo test 상세

```
test result: ok. 97 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.79s
```

Sprint 1 대비 +33건 (64 → 97). 신규 테스트 분포:
- `students::tests` — 자동 채번(PI-05), override 연속성, UNIQUE 위반 한국어 메시지, 영문 prefix 제외, enum 직렬화
- `schedules::tests` — 주 수업시간 합산, 변경 이력 생성, 부분 인덱스 UNIQUE 제약
- `fees::tests` — 매칭 정확 일치·이하 최댓값·미등록 구간, UNIQUE 위반 한국어 메시지
- `codes::tests` — 시드 데이터 존재, V105 컬럼 추가, UNIQUE 위반 한국어 메시지
- `audit::tests` — 레코드·목록 round-trip, try_record silent fail, cleanup 기간 경계값
- `startup::tests` — 상수 PRD 정합, 직렬화 필드, timing breakdown 합계 ≤ 총 elapsed

### pnpm build 상세

```
Route (app)                                 Size  First Load JS
┌ ○ /                                    1.36 kB         103 kB
├ ○ /_not-found                            977 B         103 kB
└ ○ /lock                                2.11 kB         104 kB

○  (Static) prerendered as static content
```

3개 라우트 모두 정적 export 성공.

---

## 수동 검증 항목

- ⬜ `pnpm tauri:dev` 실행 후 앱 동작 확인 (개발자 수행 필요)
  - 최초 실행 → `/lock?mode=setup` redirect 확인
  - 비밀번호 설정 후 재시작 → `/lock` redirect 확인
  - 잠금 해제 성공 → 메인 화면 진입 + `startup.elapsed_ms` 표시 확인
- ⬜ cipher on 빌드에서 `app_startup_sequence` timing breakdown 콘솔 출력 확인 (v0.2.0 배포 후)

---

## 배포 준비도 사전 확인

| 항목 | 결과 |
|------|------|
| CHANGELOG.md `[Unreleased]` 섹션 업데이트 | ✅ Sprint 2 변경사항 기재됨 (Added 14건, Changed 2건) |
| 하드코딩 시크릿 패턴 스캔 | ✅ 없음 (startup.rs의 `password=` 패턴은 timing log 변수명, 시크릿 값 아님) |

---

## 결론

5개 자동 검증 항목 전체 통과. 수동 검증 2항목은 개발자 수행 필요.
