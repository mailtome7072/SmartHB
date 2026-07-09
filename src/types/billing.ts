/**
 * 청구·수납 도메인 타입 (Sprint 11 T6, PRD §4.9).
 *
 * 백엔드 `src-tauri/src/commands/billing.rs` 의 serde camelCase 직렬화와 1:1 정합.
 */

import type { SchoolLevel } from '@/types/student'

export type BillStatus = 'draft' | 'confirmed'

export type MidMonthType = 'enrolled' | 'withdrawn'

export interface Bill {
  id: number
  studentId: number
  studentName: string
  studentSerialNo: string
  studentGrade: number
  studentSchoolLevel: SchoolLevel
  billYearMonth: string // "YYYY-MM"
  weeklyHours: number
  billAmount: number
  adjustedAmount: number
  status: BillStatus
  isMidMonth: boolean
  midMonthType: MidMonthType | null
  confirmedAt: string | null
  /** payments.is_paid=1 행 존재 시 true — BillingGrid 수납완료 라벨용. */
  isPaid: boolean
}

export interface PaymentViewRow {
  billId: number
  paymentId: number | null
  studentId: number
  studentName: string
  studentSerialNo: string
  /** Sprint 19 사용자 요청 — 기본 정렬(학년별+이름) 및 화면 표시용. */
  studentGrade: number
  studentSchoolLevel: SchoolLevel
  adjustedAmount: number
  isMidMonth: boolean
  midMonthType: MidMonthType | null
  isPaid: boolean
  paidDate: string | null
  payerName: string | null
  paymentMethodId: number | null
  paymentMethodLabel: string | null
  cardCompanyId: number | null
  cardCompanyLabel: string | null
}

export interface GenerateBillsResult {
  yearMonth: string
  generatedCount: number
  skippedCount: number
}

export interface Payment {
  id: number
  billId: number
  isPaid: boolean
  paidDate: string | null
  payerName: string | null
  paymentMethodId: number | null
  paymentMethodLabel: string | null
  cardCompanyId: number | null
  cardCompanyLabel: string | null
}

export interface PaymentInput {
  billId: number
  isPaid: boolean
  paidDate: string | null
  payerName: string | null
  paymentMethodId: number | null
  cardCompanyId: number | null
}

export interface UnpaidBill {
  billId: number
  studentId: number
  studentName: string
  studentSerialNo: string
  /** Sprint 19 사용자 요청 — 기본 정렬(학년별+이름) 및 화면 표시용. */
  studentGrade: number
  studentSchoolLevel: SchoolLevel
  adjustedAmount: number
  isMidMonth: boolean
  midMonthType: MidMonthType | null
}

export interface BillingSearchResult {
  studentId: number
  studentName: string
  /** 가장 최근 is_paid=1 payments 의 입금자 — 자동 채움용. */
  latestPayerName: string | null
  latestPaymentMethodId: number | null
  latestCardCompanyId: number | null
}

/** 결제수단별 수납 집계 (월별 집계 탭). is_paid=1 한정. */
export interface PaymentMethodSummary {
  paymentMethodId: number | null
  paymentMethodLabel: string
  paidCount: number
  totalPaid: number
}

/** 기간(연도 'YYYY' 또는 월 'YYYY-MM') 청구·수납 집계 (월별 집계 탭). */
export interface BillingPeriodStats {
  period: string
  billCount: number
  totalBilled: number
  paidCount: number
  totalPaid: number
  totalUnpaid: number
  unpaidCount: number
  byMethod: PaymentMethodSummary[]
}

export interface BillingSummary {
  yearMonth: string
  /**
   * 해당 월에 수업을 진행한 원생 수 (hotfix post-Sprint 11).
   * 청구년월 'YYYY-MM' 은 그 해·달 수업 원생의 교습비 청구서를 의미.
   * `totalBillableStudents > billCount` 일 때 UI 는 "추가 청구 데이터 생성" 라벨 표시.
   */
  totalBillableStudents: number
  billCount: number
  totalBilled: number
  totalPaid: number
  totalUnpaid: number
  paidCount: number
  unpaidCount: number
}
