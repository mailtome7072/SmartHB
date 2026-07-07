-- Sprint 19 T9: 학교급 기반 학교 선택 필터링을 위한 기존 데이터 자동 보정.
--
-- schools.school_type 컬럼은 이미 CHECK(elementary/middle/high/etc)로 존재하지만,
-- 지금까지 등록 UI에 school_type 입력이 없어 전부 기본값 'etc'로 저장돼 있었다.
-- 학교명 텍스트 패턴으로 최선 추정(초등학교/중학교 포함 여부) 보정 — 그 외 이름은
-- 'etc' 유지(고등학교 등은 이번 스프린트 범위 밖, 원장님이 설정 화면에서 직접 재지정 가능).

UPDATE schools
   SET school_type = 'elementary'
 WHERE school_type = 'etc'
   AND name LIKE '%초등학교%';

UPDATE schools
   SET school_type = 'middle'
 WHERE school_type = 'etc'
   AND name LIKE '%중학교%';
