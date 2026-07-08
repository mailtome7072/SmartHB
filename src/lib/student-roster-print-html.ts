/**
 * 수강생대장 인쇄 — 독립 팝업창용 HTML 문서 생성.
 *
 * 교습일정 인쇄(academic-print-html.ts)와 동일한 아키텍처: 완전히 독립된 HTML 문서를
 * 문자열로 생성해 팝업창(WebviewWindow)에 주입한다 — 메인 앱 CSS/DOM과 무관.
 */

import type { Student } from '@/types/student'

interface BuildParams {
  students: Student[]
  academyName: string
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

/** 원생의 대표 연락처 — 본인 우선, 없으면 모/부 순. */
function primaryPhone(s: Student): string {
  return s.phone_student ?? s.phone_mother ?? s.phone_father ?? ''
}

const STYLE = `
  * { box-sizing: border-box; -webkit-print-color-adjust: exact; print-color-adjust: exact; }
  html, body { height: 100%; }
  body { margin: 0; font-family: Pretendard, -apple-system, sans-serif; background: #e5e7eb; }
  .print-root { padding: 15mm 12mm; background: #fff; }
  .print-header { display: flex; align-items: baseline; justify-content: flex-end; margin-bottom: 4mm; }
  .print-academy { font-size: 12pt; color: #333; }
  .print-title { text-align: center; font-size: 22pt; font-weight: bold; letter-spacing: 4px; margin-bottom: 10mm; }
  table { width: 100%; border-collapse: collapse; table-layout: auto; }
  th, td {
    border: 1pt solid #333;
    padding: 5pt 6pt;
    text-align: center;
    font-size: 11pt;
    white-space: nowrap;
  }
  thead th { background: #f0f0f0; font-weight: 700; font-size: 10.5pt; }
  td.roster-name { text-align: center; font-weight: 600; }
  td.roster-remark { text-align: left; white-space: normal; }
  tr { page-break-inside: avoid; }
  @media print {
    body { background: #fff; }
    /* margin: 0 — 브라우저 기본 머리글/바닥글(출력일시·페이지 제목 등)은 여백 공간에
       그려지므로 여백을 없애면 함께 사라진다. 대신 .print-root 자체 padding으로
       여백을 재현한다(완전 제어는 인쇄 대화상자 "머리글/바닥글" 옵션 몫). */
    @page { size: A4 portrait; margin: 0; }
    .print-root { padding: 12mm; }
  }
`

/** 수강생대장 인쇄 팝업창에 쓸 완결된 HTML 문서를 생성한다. */
export function buildStudentRosterHtml({ students, academyName }: BuildParams): string {
  const now = new Date()
  const title = `${now.getFullYear()}년 ${now.getMonth() + 1}월 수강생대장`

  const rowsHtml = students
    .map(
      (s, i) => `
        <tr>
          <td>${i + 1}</td>
          <td>${escapeHtml(s.enroll_date)}</td>
          <td>${escapeHtml(s.withdraw_date ?? '')}</td>
          <td class="roster-name">${escapeHtml(s.name)}</td>
          <td>${escapeHtml(primaryPhone(s))}</td>
          <td class="roster-remark"></td>
        </tr>
      `,
    )
    .join('')

  return `<!doctype html>
<html lang="ko">
<head>
<meta charset="utf-8" />
<title>${escapeHtml(title)}</title>
<style>${STYLE}</style>
</head>
<body>
  <div class="print-root">
    <div class="print-header"><span class="print-academy">${escapeHtml(academyName)}</span></div>
    <h1 class="print-title">${escapeHtml(title)}</h1>
    <table>
      <thead>
        <tr>
          <th style="width:7%">번호</th>
          <th style="width:14%">등록일자</th>
          <th style="width:14%">퇴교일자</th>
          <th style="width:14%">성명</th>
          <th style="width:18%">전화번호</th>
          <th>비고</th>
        </tr>
      </thead>
      <tbody>${rowsHtml}</tbody>
    </table>
  </div>
  <script>
    function closeThisWindow() {
      try {
        if (window.__TAURI__ && window.__TAURI__.window) {
          window.__TAURI__.window.getCurrentWindow().close()
          return
        }
      } catch (e) {}
      window.close()
    }
    window.addEventListener('afterprint', closeThisWindow)
    window.addEventListener('load', function () {
      if (document.fonts && document.fonts.ready) {
        document.fonts.ready.then(function () { window.print() })
      } else {
        window.print()
      }
    })
  </script>
</body>
</html>`
}
