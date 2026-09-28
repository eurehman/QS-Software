# Phase gate report

## Phase

0 — Architecture and reconciliation

## Objectives

Inspect the existing program, document the gap against the quantity-control specification, reclassify the 36 prototype tests, and define the quantity ledger before any new module is built.

## Completed

- Repository inspected. Stack remains Tauri 2, React 19, TypeScript, SQLite schema 19.
- Current tables, calculations, tests, and screens documented in `ARCHITECTURE.md`.
- Required quantity chain and migration rules defined. No table is scheduled for deletion.
- Targeted Works and Progress Done are separate canvases. The 36 items remain, classified as prototype, functional but incomplete, test only, or requires redesign.
- None of the 36 items is classified Production Ready.

## Not Completed

This phase does not build the quantity ledger, the Excel QS sheets, or the installer. Those are later phases.

## Tests

- `cargo test --lib` on 29 September 2026: 36 passed, 0 failed.
- `npm test` (Vitest) on 29 September 2026: 12 passed, 0 failed. Formula, paste, and column mapping.

## Test Results

Pass.

## Data Validation

Schema 19 tests passed, including firm-file create, migration from version 1, and two connections writing one file. No schema change was made in this phase.

## UI Validation

No screen was changed. The window was not relaunched. The recorded UI remains three screens: a generic sheet, a page of forms, and accounts.

## Excel Validation

The existing workbook round-trip test passed again. It covers the generic sheet, not a bill workbook.

## Backup Validation

The existing sheet backup test passed again. Company QS tables are still absent from the backup. That limit is unchanged and is Phase 8 work, not a Phase 0 defect.

## Integration Validation

There is no quantity ledger yet, so nothing new was connected to it. The design says `work_item` stays the authoritative bill item and later modules post balances to it.

## Known Limitations

- Contract lines do not reference bill items.
- Certificates store money, not certified quantity.
- Backup exports the generic sheet only.
- Formula support is a subset.
- The program is not an installed Windows product.
- Two database connections in one process are not a five-user server.

## Exit Criteria

| Criterion | Result | Evidence |
| --- | --- | --- |
| Repository inspected | PASS | `ARCHITECTURE.md` |
| Existing schema documented | PASS | Current data model table, schema 19 |
| Existing calculations documented | PASS | Calculation inventory |
| Existing tests documented | PASS | 36 Rust tests and 12 Vitest cases named by module |
| Existing modules classified | PASS | Targeted Works classification column |
| Required ERP entities defined | PASS | Quantity Control Core section |
| Quantity Control Core data model defined | PASS | Ledger balance list and opening example |
| Module dependency order documented | PASS | Phases 0 to 10 |
| Migration strategy defined | PASS | Additive schema 20 and later, no dropped tables |
| No working feature scheduled for deletion | PASS | Decision recorded in `ARCHITECTURE.md` |
| TARGETED WORKS corrected | PASS | `targeted-works.canvas.tsx` |
| PROGRESS DONE corrected | PASS | `progress-done.canvas.tsx` |
| ARCHITECTURE.md created | PASS | `ARCHITECTURE.md` |
| Build succeeds | PASS | Rust tests compiled and passed. Vitest passed. |
| Existing tests still pass | PASS | 36 Rust, 12 Vitest, 0 failed |

## Gate Decision

**GATE PASSED**
