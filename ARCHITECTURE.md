# QS architecture

Date: 29 September 2026
Controlling specification: `QS_Cursor_Master_Quantity_Control_ERP_Prompt.md`
Product version: 0.1.0
Firm-file schema: 24
Repository: https://github.com/eurehman/QS-Software

This document is the Phase 0 architecture record. It does not declare the ERP complete.

## Decision recorded before further build

The new specification requires two different canvases: Targeted Works and Progress Done. An earlier project rule kept three canvases as identical copies. That conflict is resolved by the new specification. The canvases are no longer copies of each other. No working feature is scheduled for deletion.

## What the program is today

A Windows desktop prototype: Tauri 2, React 19, TypeScript, Vite, SQLite (rusqlite, bundled). Excel read uses calamine. Excel write uses rust_xlsxwriter. Passwords use Argon2. The formula engine is custom TypeScript. There is no paid dependency.

The firm file is one SQLite database (`.qsdb`). Opening an older file copies `.bak-v{n}.qsdb` and migrates forward. The firm file is gitignored.

The window has three screens:

- Sheet: one generic grid, 8 columns by 40 rows, with paste, resize, a frozen header, formula display, and XLSX open/save.
- Projects: one page of separate save forms for the project tree, codes, bill items, measurements, rates, estimates, contracts, certificates, variations, budget, a short report, and the intermediate and advanced forms.
- Accounts: users, roles, the QS Local Server folder, backup history, the sheet audit list, and sign out.

An installer has not been built. The window the user opens is the development program.

## Current data model (schema 24)

One firm file stands in for the company. There is no company row.

| Table | Role today |
| --- | --- |
| app_meta | Schema version and the local-server folder path |
| user_account, role, role_permission, role_module, role_project | One administrator and at most four other users |
| sheet_cell | The generic 8×40 sheet, not the bill |
| backup_record | History of sheet backups |
| audit_event | Sheet cells, and create or edit of a project, code, or bill quantity. User, time, old value, and new value. |
| company | One company per firm file. Projects point at that row. |
| project | Code and name |
| location_node | Parent/child tree. Kind is development, building, tower, floor, unit, zone, or custom. The label stays free. |
| code_entry | Kinds: wbs, cbs, cost, unit, work, discipline, package |
| work_item | Bill line: name, four code references, parent, version, quantity, rate, amount |
| quantity_ledger | One balance row per bill item: original, revised, planned, contract, executed, measured, certified, billed, paid, forecast, final. Remaining is calculated. |
| measure_line | Times, length, width, height against a work item |
| rate_buildup | One composite-rate row per work item |
| estimate | Kind boq or area, one total |
| contractor, contract, work_order | Contractor, contract, and work order |
| contract_item | Contract bill line. It may point at a work_item. Older lines keep a free description. |
| ipc | Certificate money columns and status draft, recommended, approved, rejected, or certified |
| variation | Add, omit, or substitute as one signed amount. Draft or approved |
| cost_budget | One budget amount per project |
| boq_approval | An approved bill version number |
| commitment | Order amount and an amount already certified, typed in |
| requisition, rfq, quotation, quotation_line, purchase_order | A short procurement chain. Amounts are not quantities |
| material_item, material_move | Theoretical quantity, receipt, and issue |
| forecast | One remaining amount and one cash-flow amount |
| location_cost | One amount typed onto a location |
| rate_library | Historical or market rate by code |
| development_area, development_sale | Saleable or common area, and one sale rate |
| scenario | A named budget that is not the live budget |
| final_account | One amount taken from the latest certified certificate |
| historical_cost | A code, rate, and amount. Closeout writes code FINAL |
| integration_setting | An on/off flag. The core does not call out |

## Calculation inventory

Locked and tested:

- Formula subset: arithmetic, power, SUM, IF, ROUND, cell references, running totals. `=-2^2` is −4. SUM ignores blanks. IF treats 0 as false. ROUND is half away from zero. Division by zero is `#DIV/0!`. A circular reference is `#REF!`.
- Leaf amount = quantity × rate. A heading amount is the sum of its children. Estimates and version comparison sum root lines only.
- Measurement multiplies only the dimensions that are filled. The sheet total replaces the item quantity.
- Composite rate: material + labour + plant, wastage as a percent of material only, overhead on that direct cost, profit after overhead. Worked result 1185.8.
- Certificate: this bill = work to date − previous. Retention is a percent of this bill. Net = this bill − retention − advance recovery − deductions. Worked net 480. Work to date is an input. It is not a stored quantity.
- Revised contract sum = contract value + approved variation amounts. A draft variation does not change the sum.
- Cost actual = certified certificate this-bill amounts. Variance = budget − actual.
- Open commitment = order − already certified. Exposure = actual + open commitment.
- Estimate at completion = certified actual + remaining.
- Final account = previous + this bill of the latest certified certificate.
- GFA = saleable + common. A scenario does not write the live budget.

Not implemented: SUMIF, SUMIFS, COUNT, COUNTIF, AVERAGE, MIN, MAX, AND, OR, ROUNDUP, ROUNDDOWN, IFERROR, INDEX, MATCH, VLOOKUP, XLOOKUP, and cross-sheet references. Taxes are not in the rate build-up. There is no quantity ledger.

## Test inventory

36 Rust library tests, plus Vitest cases for formulas, paste, and column mapping. Each Rust test proves one worked example. None of them walks one quantity from the bill through contract, measurement, certification, billing, variation, and final account.

The tests are kept. Their meaning is now “verified component / prototype test”, not “module complete”.

## UI inventory

| Surface | What a user can do | Classification |
| --- | --- | --- |
| Sign-in and firm file | Create or open a firm file, sign in | Functional but incomplete |
| Accounts | Users, roles, backup folder, sheet audit | Functional but incomplete |
| Sheet | Edit a generic grid and open or save XLSX | Prototype |
| Projects page | Separate forms for each stored record | Prototype |
| Report list | Show a short figure list, export it, print the window | Prototype |

Missing from the interface: sidebar, module navigation, dashboard home, search, filters, formula bar, undo, QS sheets for the bill, measurement, rate, and certificate.

## Classification of the existing 36 items

No item is Production Ready.

Functional but incomplete: B-001 firm file, B-002 user cap, B-003 roles, B-004 formula subset, B-009 location tree, B-010 four code kinds, I-007 certificate status path, A-002 scenario isolation, A-004 project-total isolation.

Prototype: B-005, B-006, B-007, B-008, B-011, B-012, B-013, B-014, B-016, B-017, B-018, B-019, I-001, I-002, I-003, I-004, I-005, I-006, I-008, I-010, A-001, A-003.

Test only: B-000 inspection, I-009 two connections in one process, A-005 integration flag left off, A-006 refusal of a Google URL.

Requires redesign: B-015 and the bill model behind B-011. `contract_item` is a second bill with no link to `work_item`. There is no single quantity that later modules share.

## Required Quantity Control Core

Phase 1 adds this chain without deleting the tables above. One firm file remains one company. A company row may be added so the hierarchy is explicit.

```text
Company
  Project
    Location (typed: development, building, tower, floor, unit, zone, or a custom label)
      WBS
        CBS
          Work package
            BOQ item  (the existing work_item, extended, not replaced)
              Unit, rate, amount
              Quantity ledger
```

The quantity ledger, one row of balances per BOQ item, holds:

Original, revised, planned, contract, executed, measured, certified, billed, paid, forecast, final.

Remaining = the governing quantity minus the furthest certified or billed quantity, as defined by the Phase 1 gate. Opening state for the mandatory test:

```text
Concrete — Structural
Original 100 m³
Contracted 100
Executed 0
Measured 0
Certified 0
Billed 0
Remaining 100
```

`contract_item` stays. A later phase points it at the BOQ item instead of copying a free description. Until that link exists, contract quantity is not the same quantity as the bill.

## Module dependency order

0. Architecture and this reclassification. No new module.
1. Quantity ledger and one authoritative BOQ item.
2. Spreadsheet working surface bound to the bill, measurement, and rate.
3. Measurement, rate, and contract all read and write that same item.
4. Execution, procurement, material, and the certificate post quantities back to the ledger.
5. Budget, commitment, actual, and forecast use those postings.
6. Variations change the ledger only when approved. Final account and historical cost read the same item.
7. Reports and dashboards read the ledger. They are not built first.
8. Backup writes the QS tables, not only the generic sheet. Multi-user behaviour is proven or its limit is stated.
9. A real Windows installer.
10. One project walked through the full chain.

## Transaction flow that does not exist yet

```text
BOQ quantity
  -> contract quantity on the same item
  -> executed quantity
  -> measured quantity
  -> certified quantity
  -> billed quantity and actual cost
  -> approved variation revises the quantity
  -> forecast and final account
  -> historical cost
```

Today each box is a separate form and a separate number. A certificate stores money, not a certified quantity. A variation stores an amount, not a quantity change on a bill line.

## Migration strategy

- Keep schema 19 files. New structure is added as schema 20 and later.
- The existing open path already copies a backup before migration.
- Do not drop `work_item`, `contract_item`, `ipc`, or the generic sheet.
- Existing quantity and rate on `work_item` become the original quantity and rate when the ledger is introduced.
- Existing certificates stay money records. They are not rewritten into certified quantities, because they do not store a quantity.
- Firm files stay out of git.

## Risks

- Treating a green test as a finished module already happened once. Phase gates now forbid that.
- Two bill tables will drift further if Phase 1 does not make `work_item` the only authoritative item.
- SQLite on a shared folder is not a five-user design. I-009 does not prove it.
- The formula subset cannot yet run a real QS workbook.
- Backup of the sheet alone cannot recover the company QS data.

## Phase 0 status

Repository inspected. Schema, calculations, tests, modules, and the required core are documented here. Targeted Works and Progress Done are updated as separate canvases. Existing features are not scheduled for deletion.

Phase 0 is GATE PASSED. Evidence is in `report/Phase_00_Gate_Report.md`. The existing Rust library tests (36) and Vitest cases (12) passed again on 29 September 2026. No feature was deleted.
