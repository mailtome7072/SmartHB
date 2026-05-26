---
name: sprint-next-session
description: "Sprint 9 완전 종료 (Session #12 K1~K7 흡수 포함). 다음: sprint9 → develop 직접 머지"
metadata:
  node_type: memory
  type: project
  originSessionId: sprint9-k7-complete
---

Sprint 9 완전 종료 (2026-05-26). sprint-close + sprint-review + Session #12 사후 흡수 모두 완료. 다음은 **sprint9 → develop 직접 머지** ([[workflow-no-pr]]).

## Sprint 9 최종 현황

| Phase | 상태 | 커밋 |
|-------|------|------|
| T1~T8 (백엔드 IPC + UI 기본) | ✅ | 8 커밋 |
| T9 (자동 검증 + A39/A40 적용) | ✅ | `70c856a` |
| T10 (I3 보강 가능일 재정의) | ✅ | `4b21450` |
| T11 (I1/I2/I4-I8 시간 단위 + UX) | ✅ | `a2e3169` |
| T12 (J1~J10 도메인 정제) | ✅ | `e6e3a39` |
| **sprint-close** | ✅ | `135397f` |
| **sprint-review** | ✅ | `ccfa533` |
| chore esbuild allowBuilds | ✅ | `2667a3c` |
| **Session #12 K1~K7** (4차 시각 검증) | ✅ | `5be93ed` |

## Session #12 K1~K7 핵심 결정 (스테이징 검증 사후 흡수)

| 코드 | 내용 |
|------|------|
| K1' | 비수업일 '+' 표시 조건 정밀화 — 백엔드 응답 `earliest_pending_absence_date` 신규(이전 월 결석 포함). 셀 일자 이전 만기 미도래 결석 존재 시에만 표시 |
| K2/K2' | '재원중만' 체크박스, 디폴트 ON |
| K3 | 정규 수업 셀(present/makeup_done/expired) 우클릭 → 보강 등록 진입. 결석 셀 우클릭 = 메모 유지 |
| K4 | 단원평가 응시일 헤더 sky 배경 제거 / 보강데이 헤더 작은 폰트 '보강데이' 라벨 |
| K6 | '보강대상' 체크박스 — `earliestPendingAbsenceDate !== null` 필터, 디폴트 OFF |
| K7 | "재원중(N명)" / "보강대상(M명)" 라벨 병기. 보강대상 카운트는 재원중 필터와 연계 |

## 최종 자동 검증

- cargo test cipher off **256 passed** (sprint-close 254 → K1' 신규 단위 테스트 3건 추가)
- cargo clippy cipher off clean
- pnpm lint / tsc --noEmit clean
- pnpm build 13 라우트 static export

> cipher on 빌드는 환경(Strawberry Perl) 의존 — sprint-close때 133 passed 통과한 기록 그대로 유지

## 사용자 시각 검증 — 7라운드 누적

- 1~3차 (T10~T12 흡수): I1~I8 + J1~J10 — Sprint 9 본체 흡수
- 4차 (K1~K4): 4건 발견 → Sprint 9 흡수 결정 (사용자, 2026-05-26)
- 5차 (K1'/K2'/K6): K1' 백엔드 정밀화 + K6 신규 → Sprint 9 흡수
- 6차 (K7): 카운트 표기 + 재원중 연계 → Sprint 9 흡수
- 7차 종합: "검수완료. 모두 pass" (사용자, 2026-05-26)

## 다음 액션

새 대화 또는 같은 세션에서:

```bash
git checkout develop
git merge --no-ff sprint9 -m "feat: Sprint 9 완료 — 보강 등록 + 매칭 + 결석 이력 + UX 정밀화 (Phase 3 첫 마일스톤)"
git push origin develop
```

`DEPLOY.md` 체크리스트의 남은 ⬜ 항목:
- ⬜ sprint9 → develop 직접 머지
- ⬜ pnpm tauri:dev 스테이징 검증 (이미 6라운드 완료 — 머지 직전 한 번 더 봐도 무방)

## Sprint 10 carry-over (정리용, sprint-close 시점 + Session #12)

- `mark_makeup_absent` 백엔드 IPC + audit variant 정리 (dead code, code-review F1)
- `batch_create_makeups` 백엔드 IPC + 관련 코드 정리 (dead code)
- `makeup_attendances.status='makeup_absent'` CHECK 제약 마이그레이션 정리 (선택)
- 원래 Sprint 10 마일스톤: 소멸 자동 전이 + 퇴교 보강 처리 + 캘린더 뷰

## 정책 (재확인)

- **PR 단계 생략** — 단일 개발자, sprint9 → develop 직접 머지 ([[workflow-no-pr]])
- **사용자 메모리 미러 동기화** — 사용자 메모리 + `.claude/memory/` 두 곳 모두 갱신 후 commit
- Capacity 실측: 계획 38h → 약 58h (시각 검증 4라운드 흡수 결과, sprint-review 후 +6h 추가)
