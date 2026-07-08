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
  table { width: 100%; border-collapse: collapse; table-layout: auto; }
  th, td {
    border: 1pt solid #333;
    padding: 5pt 6pt;
    text-align: center;
    font-size: 11pt;
    white-space: nowrap;
  }
  /* 사용자 요청 — 학원명·제목을 표 thead 안에 넣어 여러 페이지에 걸쳐 인쇄돼도
     매 페이지 상단에 동일하게 반복되도록 한다(thead는 페이지가 나뉠 때마다 반복
     출력되는 표준 인쇄 동작 — 본문 밖에 두면 첫 페이지에만 표시됨). */
  .print-header-row th { border: none; padding: 0 0 2mm; text-align: right; font-size: 12pt; font-weight: 400; color: #333; }
  .print-title-row th { border: none; padding: 0 0 6mm; text-align: center; font-size: 22pt; font-weight: bold; letter-spacing: 4px; }
  thead .print-columns th { background: #f0f0f0; font-weight: 700; font-size: 10.5pt; border: 1pt solid #333; }
  td.roster-name { text-align: center; font-weight: 600; }
  td.roster-remark { text-align: left; white-space: normal; }
  tr { page-break-inside: avoid; }
  @media print {
    body { background: #fff; }
    /* margin-bottom 여백에 페이지 번호 표시 — 지원 브라우저(Chromium 최신)에서만
       렌더링되고, 미지원 환경에서는 조용히 무시된다(레이아웃에 영향 없음). */
    @page {
      size: A4 portrait;
      margin: 0 0 14mm 0;
      @bottom-center { content: "페이지 " counter(page) " / " counter(pages); font-size: 10pt; color: #555; }
    }
    .print-root { padding: 12mm 12mm 0; }
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
    <table>
      <thead>
        <tr class="print-header-row"><th colspan="6">${escapeHtml(academyName)}</th></tr>
        <tr class="print-title-row"><th colspan="6">${escapeHtml(title)}</th></tr>
        <tr class="print-columns">
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
