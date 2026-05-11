# 문서화 규칙

> **역할**: 프로젝트 문서 작성 기준, 저장 위치, 갱신 주기를 정의한다.
> **SSOT**: 문서 관리 규칙 상세는 [`docs/dev-process.md`](../docs/dev-process.md) 섹션 8을 참조한다.

---

## 문서 체계

```
CLAUDE.md              ← AI 협업 최상위 지침
docs/dev-process.md    ← 프로세스 상세 (스프린트, 검증, 배포, 코드리뷰)
docs/ci-policy.md      ← CI/CD 인프라 정책
docs/prompt-guide.md   ← 작업 경로 선택 가이드
DEPLOY.md              ← 현재 미완료 수동 작업 목록
docs/deploy-history/   ← 완료된 배포 기록 아카이브
docs/phase/            ← Phase 설계 문서 (phase-planner 산출물)
docs/sprint/           ← 스프린트 계획/완료 문서
docs/sprint-retrospectives/ ← 스프린트 회고 (sprint-review 산출물)
docs/test-reports/     ← 테스트 실행 결과
docs/risk-register/    ← 리스크 이력
```

## 문서 갱신 트리거

갱신 트리거 상세는 [`docs/dev-process.md`](../docs/dev-process.md) 섹션 8.6을 참조한다.

## 공통 작성 규칙

- Markdown 형식, 섹션 제목 `##` 이상
- 체크리스트: 이모지(`✅`/`⬜`) 사용 (GFM `[x]`/`[ ]` 대신)
- 언어: 한국어 (변수명/함수명은 영어)

## 참조

- 문서 관리 규칙 상세: [`docs/dev-process.md`](../docs/dev-process.md) 섹션 8
- CLAUDE.md 워크플로우 지침: [`CLAUDE.md`](../CLAUDE.md#워크플로우-지침)
