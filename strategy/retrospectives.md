# 회고 기본 지침

> **역할**: 스프린트 회고의 형식, 주기, 항목을 정의한다.
> **SSOT**: 회고 프로세스는 [`docs/dev-process.md`](../docs/dev-process.md) 섹션 3.4를 참조한다.

---

## 회고 주기

- **스프린트 회고**: 매 스프린트 마무리 시 (sprint-review agent가 자동 생성)
- **장기 회고**: 필요 시 `docs/retrospectives/`에 별도 기록

## 산출물 위치

| 종류 | 경로 | 설명 |
|------|------|------|
| 스프린트 회고 | `docs/sprint-retrospectives/sprint{n}.md` | 매 스프린트 완료 후 |
| 장기/팀 회고 | `docs/retrospectives/` | 분기별 또는 필요 시 |

## 회고 항목

```markdown
# Sprint Retrospective (스프린트 번호)
## 잘한 점
- 항목
## 개선할 점
- 항목
## 액션 아이템
- 항목
```

## 참조

- 스프린트 회고 프로세스: [`docs/dev-process.md`](../docs/dev-process.md) 섹션 3.4
- sprint-review agent: [`.claude/agents/sprint-review.md`](../.claude/agents/sprint-review.md)
