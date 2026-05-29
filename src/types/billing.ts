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
