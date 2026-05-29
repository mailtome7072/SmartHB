/**
 * 청구·수납 도메인 타입 (Sprint 11 T6, PRD §4.9).
 *
 * 백엔드 `src-tauri/src/commands/billing.rs` 의 serde camelCase 직렬화와 1:1 정합.
 */

export type BillStatus = 'draft' | 'confirmed' | 'closed'

export type MidMonthType = 'enrolled' | 'withdrawn'

export interface Bill {
  id: number
  studentId: number
  studentName: string
  studentSerialNo: string
  studentGrade: number
  studentSchoolLevel: 'elementary' | 'middle'
  billYearMonth: string // "YYYY-MM"
  weeklyHours: number
  billAmount: number
  adjustedAmount: number
  status: BillStatus
  isMidMonth: boolean
  midMonthType: MidMonthType | null
  closeReason: string | null
  closedAt: string | null
  confirmedAt: string | null
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
  adjustedAmount: number
  isMidMonth: boolean
  midMonthType: MidMonthType | null
}

export interface BillingSummary {
  yearMonth: string
  billCount: number
  totalBilled: number
  totalPaid: number
  totalUnpaid: number
  paidCount: number
  unpaidCount: number
}
