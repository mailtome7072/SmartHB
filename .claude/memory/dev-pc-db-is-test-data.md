---
name: dev-pc-db-is-test-data
description: 개발용 PC(이 세션 환경)의 클라우드 동기화 폴더 DB는 실사용 데이터가 아니라 개발/테스트 전용 DB
metadata: 
  node_type: memory
  type: project
  originSessionId: 866d1018-c33b-4a1b-a215-c65ef4d71817
---

이 세션에서 작업하는 PC의 앱 데이터 DB(`C:\개인폴더\스마트해법\smarthb\app.db`, config.json의 `cloud_folder_path` 하위)는 **실사용(프로덕션) 데이터가 아니라 개발자가 기능 검증용으로 쓰는 테스트 DB**다.

**Why**: 경로 구조(클라우드 동기화 폴더 + `smarthb/app.db`)가 PRD상 프로덕션 DB와 동일한 규칙을 따르기 때문에, 사용자 확인 없이는 이 DB를 "실제 원생 PII가 담긴 프로덕션 DB"로 오인하기 쉽다. 실제로는 개발 PC에서 기능 확인(실사용 확인)을 위해 쓰는 데이터라 사용자가 직접 확인해줬다.

**How to apply**: 이 경로의 DB에 대해 SQL 직접 조작(대량 테스트 데이터 삽입/수정 등)을 요청받으면, "프로덕션 DB 직접 조작" 경고 없이 정상 진행한다. 다만 안전을 위해 편집 전 타임스탬프 백업본(`app.db.bak-YYYYMMDD` 등 동일 폴더)은 계속 만들어두는 습관은 유지한다. 다른 PC(실제 배포 환경)에서는 이 판단이 적용되지 않을 수 있으므로, 환경이 바뀌면 재확인할 것.
