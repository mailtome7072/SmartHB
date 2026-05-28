---
name: migration-numbering
description: DB 마이그레이션 번호 정책 -- 3자리 zero-pad, 도메인 블록 100단위. 현재 사용 범위 추적.
metadata:
  type: project
---

번호 정책 (3자리 zero-pad, 도메인 블록 100단위):
- V001~V099: 인프라 (코드 테이블, 감사 로그, 앱 설정) -- V001, V008 사용
- V101~V199: 핵심 도메인 -- V101~V108 사용 (V108 = Sprint 10 소멸 상태 CHECK 정리)
- V200~V299: 시드 데이터 -- V200~V201 사용
- V301~V399: 학사 일정 패치 -- V301~V302 사용

**Why:** ROADMAP 초기 계획의 V007은 구 번호 체계. 실제 Sprint 11 청구 마이그레이션은 V109.
**How to apply:** 다음 도메인 마이그레이션은 V109 (bills + payments 테이블 -- Sprint 11).
Sprint 9~10은 V107~V108 사용 (보강 FK + 소멸 CHECK 정리).
