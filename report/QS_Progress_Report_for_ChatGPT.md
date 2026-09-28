# QS Software — progress report for external analysis

Date: 28 September 2026, 23:53 UTC+5
Audience: paste this whole file into ChatGPT and ask it to analyse progress against the controlling specification.
Controlling specification: `QS_Cursor_Master_Development_Prompt.md` in the project root. Do not treat this report as a replacement for that file.
Repository: https://github.com/eurehman/QS-Software
Branch: `main`
Product version: 0.1.0
Firm-file schema version: 19

This report is factual. Where the internal work register says Completed, that means one automated test passed for a narrow worked example. It does not mean the master prompt’s full requirement for that topic is implemented.

---

## 1. What this product is supposed to be

A standalone, installable, updateable Windows application for quantity surveying, commercial management, and cost control in a real-estate / construction company.

Hard constraints from the specification:

- Zero paid dependencies, plugins, APIs, cloud, or licences.
- Offline. Local SQLite firm file. No paid cloud.
- At most 1 administrator and 4 other users.
- Excel-flexible calculation and import/export. Not an Excel replacement, and not a rigid form system that throws away spreadsheet flexibility.
- One-click Excel backup to a folder the administrator chooses (“QS Local Server”). That folder is a local or UNC path, not a paid host.
- Development order was Basic, then Intermediate, then Advanced.
- A feature counts as done only after implementation, testing, and verification against a known QS / Excel result.

The specification’s own product standard (section 49) says the application must feel like Excel plus a QS, commercial, and cost-control department, and must not be a demonstration or a collection of disconnected forms.

## 2. What exists today

Stack, all free:

- Windows desktop shell: Tauri 2, React 19, TypeScript, Vite.
- Local database: SQLite through rusqlite with the bundled SQLite library.
- Excel read: calamine. Excel write: rust_xlsxwriter.
- Passwords: Argon2. No chrono. No paid libraries.
- Formula engine: a custom TypeScript subset, not HyperFormula.
- Identifier `com.qs.desktop`. Window about 960×640. NSIS is named in the Tauri config. An installer package has not been built. The app the user opens is the development program (`npm run tauri dev`), not an installed product. It is unsigned. Windows SmartScreen would warn on a future installer.

The running window has three destinations after sign-in:

1. Sheet — one generic grid, 8 columns (A–H) by 40 rows. Keyboard movement, TSV copy/paste, column resize, frozen header. Cells persist. Formulas that are implemented display in the cell. XLSX open/save with a column-mapping preview. Administrator can write a one-click Excel backup of this sheet and restore it.
2. Projects — one long page of separate forms: location tree, codes, bill items, measurement lines, rate build-up, estimate, contract, work order, contract bill, certificate, variation, budget, report list, then an Intermediate block and an Advanced block.
3. Accounts — users, roles, QS Local Server folder, backup history, audit list, sign out.

There is no sidebar, no module navigation, no dashboard home, no search, no print preview, and no professional bill layout. Print on the report is the browser print of the short on-screen list.

The firm file is `company.qsdb` (default under Documents/QS). Opening an older file copies a backup and migrates it. The firm database is gitignored and is not in the GitHub repository.

Latest automated check: 36 Rust library tests passed, plus the TypeScript formula/sheet tests (Vitest: formula cases, paste, column mapping). Tests are worked numeric examples, not a full QS acceptance suite and not a test of the window a user clicks through.

## 3. Work register

36 items. All are marked Completed on the internal register. Basic 20/20, Intermediate 10/10, Advanced 6/6.

Each line below is the test that actually passed, not the full specification paragraph.

### Basic

| ID | Module | What the test proved |
| --- | --- | --- |
| B-000 | Control | Environment and architecture baseline only. No product code in that step. |
| B-001 | Platform | Creating a firm file writes the schema. Reopen works. |
| B-002 | Security | A sixth account is rejected. A second administrator is rejected. |
| B-003 | Security | A user without the right cannot act. Administrator bypasses roles. |
| B-004 | Engine | Arithmetic, SUM, IF, ROUND, quantity × rate. Includes Excel case −2^2 = −4. |
| B-005 | Sheet | Paste from Excel. `=A1*B1` shows 25. A running total shows 35. |
| B-006 | Sheet | XLSX round-trip kept the number 2.5 and the formula `=B2*10`. |
| B-007 | Backup | Backup workbook can be read back without the app. Restore puts sheet cells back. |
| B-008 | Audit | Changing a sheet cell from Concrete to Steel writes one history row. Saving the same value again does not. |
| B-009 | Project | Two different location trees save (tower/floor and phase/zone). |
| B-010 | Codes | Two items can share one unit code and one WBS code. |
| B-011 | BOQ | 2.5 × 10 = 25. A parent amount is the sum of children (45). Version 1 stays when version 2 is added. |
| B-012 | Measure | 2 × 2.5 × 2 plus 1 × 5 = 15. That quantity replaces the item quantity. Amount becomes 150 at rate 10. Blank dimensions are skipped, not treated as zero. |
| B-013 | Rates | Material 8×100, labour 2×50, plant 1×40, wastage 5% of material only, overhead 10% of direct cost, profit 10% after overhead. Composite rate 1185.8. That rate is written onto the leaf item. |
| B-014 | Estimate | Bill estimate sums root amounts only (worked total 55). Area estimate 100 × 25 = 2500. |
| B-015 | Contract | Contract bill 2.5×10 plus 1×20. Contract value 45. A work order can be stored under the contract. |
| B-016 | IPC | Work to date 1000 minus previous 400 = this bill 600. Retention 10% = 60. Advance recovery 50. Deductions 10. Net payable 480. |
| B-017 | Variation | Draft add of 200 leaves a 1000 contract at 1000. Approval makes 1200. Approved omit 50 and substitute +80 finish at 1230. |
| B-018 | Cost | Two certified bills of 400 and 600. Actual 1000. Budget 1500. Variance 500. |
| B-019 | Reports | Screen figures 10, 150, 15, 600, 480 match the printed/exported workbook. |

### Intermediate

| ID | Module | What the test proved |
| --- | --- | --- |
| I-001 | BOQ | Version 1 total 45, version 2 total 60, line differences sum to 15. Approving version 2 does not change that difference. Roots only, matched by name. |
| I-002 | Cost | Order 1000 with 400 already certified. Open commitment 600. Actual plus open commitment stays 1000, not 1400. |
| I-003 | Procurement | Alpha lines 100+50=150. Beta lines 80+40=120. Purchase order uses the lower total, 120. A requisition and RFQ are created automatically behind that. |
| I-004 | Material | Theoretical 100. Receipt 120 does not count as consumption. Issue 110. Wastage 10. |
| I-005 | Forecast | Certified actual 1000 plus remaining 500. Estimate at completion 1500. Cash-flow figure 200 is stored beside it. |
| I-006 | Location | Allocations 600+250+150 equal the certified project total 1000. |
| I-007 | Workflow | A draft certificate cannot be certified. Path is recommend, then approve, then certify. Cost actual counts certified bills only. |
| I-008 | Rates | A historical library rate of 100 applied to quantity 2 makes the item amount 200. Market is the other library kind. |
| I-009 | Platform | Two connections write two projects into one firm file. After reopen, both rows exist and the schema is intact. SQLite WAL and a 5-second busy timeout are on. This is not a proven multi-user network server. |
| I-010 | Dashboard | One function returns budget 2000, actual 400, commitment 600, forecast 900, paid 400, matching those source records. |

### Advanced

| ID | Module | What the test proved |
| --- | --- | --- |
| A-001 | Development | Saleable 500+300=800. Common 200. GFA 1000. The schedule total equals the saleable total. Sale rate can also produce cost per area and margin. |
| A-002 | Profit | A scenario budget of 2500 does not change the live budget of 2000. |
| A-003 | Closeout | Last certified bill cumulative (previous + this bill) is 1000. Final account is 1000. A historical cost row stores the same 1000. |
| A-004 | Analytics | Project AAA certified total 400 and project BBB certified total 100 stay separate. |
| A-005 | Integrate | With no integration switched on, saving a project still works. |
| A-006 | Migrate | A Google Sheets URL is refused. A workbook is written on the local disk. No Google account is used. |

## 4. Calculations that are locked to worked examples

These are the numbers a later change must not break:

- Formula: `=-2^2` is −4. SUM ignores blanks. IF treats 0 as false. ROUND is half away from zero. Division by zero is `#DIV/0!`. Circular reference is `#REF!`.
- Bill leaf: quantity × rate. Empty quantity and rate store amount 0. A heading amount is the sum of its children. A child must share the parent’s version.
- Measurement: multiply only the dimensions that are filled. At least one number is required. The sheet total replaces the item quantity.
- Rate build-up: wastage is a percent of material only. Overhead is a percent of direct cost after wastage. Profit is a percent after overhead. Worked composite 1185.8.
- Estimate from the bill sums root lines only, so children are not counted twice.
- Certificate: this bill = work to date − previous. Retention is a percent of this bill. Net = this bill − retention − advance recovery − deductions. Worked net 480.
- Commitment open amount = order − amount already certified against that order. Exposure = certified actual + open commitment.
- Forecast: estimate at completion = certified actual + cost to complete.
- Final account = previous + this bill of the latest certified certificate.
- Development: GFA = saleable + common. Schedule total = sum of saleable areas.
- Money uses ordinary JavaScript/Rust floating numbers so the locked Excel cases stay exact. Do not switch to a decimal library without rechecking those cases.

## 5. What the master prompt still requires, and the current gap

This is the important part for analysis. The register is closed. The specification is not fulfilled.

### The working style is forms, not an Excel QS environment

Sections 6, 7, 39, and 49 require an Excel-like working surface for the bill, measurement, rate build-up, and certificate: formulas, copy/paste, freeze panes, and the same kind of sheet a quantity surveyor already uses. Section 49 forbids a collection of disconnected forms.

What shipped: one blank 8×40 sheet that is not the bill. The bill, measurement, rates, contract, certificate, cost, procurement, and the rest are separate save-forms. Amounts are correct when Save is pressed. They are not calculated in the grid. A surveyor cannot lay out a bill the way they would in Excel.

The formula engine is a subset. It has arithmetic, SUM, IF, ROUND, cell references, and running totals. It does not have SUMIF, SUMIFS, COUNTIF, lookups, or cross-sheet references. Those are listed in the specification.

### Backup does not contain the company’s QS data

Section 44 says that if the application is unavailable, an authorised person must still read the company’s QS information from the Excel backup.

The one-click backup writes the working sheet and a small backup-info workbook. It does not write the bill, measurements, rates, contracts, certificates, variations, or cost records. Restore replaces sheet cells only.

### There is no installed, updateable product

Section 36 requires a Windows installer, safe update, configuration preservation, uninstall, and rollback where practical. None of that has been built. Version 0.1.0 is visible. Migrations exist for the firm file. A code-signing certificate was excluded because it is a paid item. An unsigned installer would still be possible later and would show a SmartScreen warning.

### Reporting, dashboard, and navigation are thin

Section 30 asks for screen, detail, summary, print preview, print, Excel, filtering, and sorting across the QS modules. What exists is one list of figures for the bill, measurement, and certificate, plus extra lines when intermediate data exists. Excel export of that list matches the screen. That was the test. There is no print preview, no PDF, and no management pack.

Section 31 dashboards and section 39 (sidebar, search, filters, grouping, right-click, keyboard shortcuts beyond the sheet) are not built. The “dashboard” is a button that prints five KPI numbers.

### Several lifecycle topics exist only as the worked example

The specification’s lifecycle includes tender, procurement, contract, measurement, IPC, variations, commitments, forecast, cost control, profitability, final account, and a historical cost database.

Present as a single happy-path calculation each:

- Procurement is quotation lines, a lowest-total comparison, and one purchase order. No tender board, no quotation comparison sheet a buyer would recognise, no contractor performance.
- Material is one theoretical quantity, receipts, issues, and wastage. No stock valuation, price variance, or reconciliation report.
- Forecast is one remaining-cost number and one cash-flow number. No monthly cash flow, no cost-to-complete by package.
- Workflow is four statuses on a certificate. It is not a recommend/approve trail across bills, variations, and orders, and it does not write those decisions to the audit log.
- Variations are a signed amount. They do not revise measured quantities or the contract bill line by line.
- Historical cost is one FINAL row written at closeout. It is not a rate library of past projects that can be searched and reused, beyond the small historical/market rate record in I-008.
- Development area is saleable, common, GFA, and an optional sale rate. It is not a real-estate area schedule (GFA, covered, net saleable, efficiency, cost per saleable square foot by tower and unit) tied to the location tree.
- Integration is an on/off flag that defaults to off. There is no accounts interface, no API, and no document system. The test only checks that the core still saves when the flag is off.
- Google Sheets is “refuse a URL and write a local xlsx”. There is no migration tool for historical workbooks: column mapping exists for the generic sheet only, not for bills, codes, units, and rates. Section 43’s migration checks (duplicates, invalid values, preview, correct-before-import) are not built for QS history.
- Multi-user “QS Local Server” is two SQLite connections in one process with WAL. It does not prove five people on a shared folder. SQLite on a network share can still corrupt a file. The specification’s five-user local server is not solved.

### Audit, security, and backup scope

Audit records sheet cell create/edit/delete for the signed-in user, with old and new values. It does not record bill approval, rate changes, contract revisions, or certificate certification.

Passwords are Argon2, minimum 8 characters. Session is in memory and clears when the firm file changes. Roles cover a fixed permission list and a fixed module list. The administrator bypasses roles.

### Project control canvases

The specification requires two different canvases: Targeted Works (the register) and Progress Done (files changed, database changes, calculations, tests, limitations, date). The project currently keeps three identical summary dashboards. They show status, a short pass line, and charts. They do not record files changed or remaining limitations per item. Completed rows have not been deleted.

## 6. Architecture that should stay

- One firm file per company. SQLite. Schema migrates forward and copies the previous file first.
- One administrator, maximum four other users.
- TypeScript owns the spreadsheet formula rules. Rust stores records and repeats the same numeric rules for stored amounts.
- Excel files are the interchange and backup format. No Google account. No network for the core.
- Blank measurement dimensions are omitted from the product, not treated as zero.
- Wastage applies to material only.
- Bill estimates and version comparison sum root lines only.
- A certificate in draft does not count as actual cost until it is recommended, approved, and certified.
- A what-if scenario must not overwrite the live budget.
- Project totals must not be added together when one project is displayed.
- Firm database files must stay out of git.

## 7. Suggested questions for the analysis

1. Is the register “36 of 36 complete” a fair description of progress, or does it overstate completion relative to sections 6, 39, 44, and 49?
2. What should be rebuilt first so a quantity surveyor can prepare a bill, a measurement, and a certificate on a sheet rather than through forms?
3. Which stored calculations are solid enough to keep, and which modules are only a single example and should be treated as prototypes?
4. What is the smallest backup that would let the company read the bill, measurements, and certificates in Excel if the program is gone?
5. Is SQLite-on-a-folder an acceptable five-user design, or does the local-server requirement need a different approach that is still free?
6. What must exist before this is safe to put in front of a real project: installer, audit of money fields, and a bill that cannot be edited only through a form?

## 8. Bottom line

The program is a local Windows QS prototype with a real firm file, login, a small Excel grid, and a chain of tested money calculations from the bill through the certificate, commitment, forecast, and final account.

The internal register is finished. The specification’s product is not. The largest miss is that QS work happens in forms, while the specification requires an Excel-like working environment, a backup of the QS data itself, and an installable application.
