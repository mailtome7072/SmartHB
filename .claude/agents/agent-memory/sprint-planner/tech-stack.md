---
name: tech-stack
description: "프로젝트 기술 스택 및 아키텍처 요약 — Tauri 2 + Next.js 15 + SQLite/sqlx 0.8"
metadata:
  type: project
---

- **데스크톱 셸**: Tauri 2 (Rust)
- **프론트엔드**: Next.js 15 (React 19), output: 'export' 정적 빌드
- **DB**: SQLite + sqlx 0.8, SQLCipher AES-256 (cipher feature)
- **상태 관리**: Zustand + TanStack Query
- **UI**: shadcn/ui + Tailwind CSS, Pretendard 18pt
- **차트**: Recharts 3.8.1
- **캘린더**: FullCalendar (MIT)
- **최신 마이그레이션**: V305 (students.birth_date)
- **마이그레이션 블록**: V001~V099(인프라), V101~V199(도메인), V200~V299(시드), V301~V305(보정/확장)
- **버전**: 0.2.1 → Sprint 16에서 1.0.0으로 업데이트 예정
- **포트**: dev server 1420 고정
