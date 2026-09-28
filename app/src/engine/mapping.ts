import { COLUMNS, ROWS, blankSheet, type SheetGrid } from "./sheet";

/** Source column index to destination column, or null when that source column is skipped. */
export function applyColumnMap(
  rows: string[][],
  mapping: Array<number | null>,
  firstRowIsHeader: boolean,
): SheetGrid {
  const sheet = blankSheet();
  const data = firstRowIsHeader ? rows.slice(1) : rows;
  data.forEach((line, rowIndex) => {
    if (rowIndex >= ROWS) return;
    line.forEach((value, sourceCol) => {
      const dest = mapping[sourceCol];
      if (dest === null || dest === undefined || dest < 0 || dest >= COLUMNS) return;
      sheet[rowIndex][dest] = value;
    });
  });
  return sheet;
}

export function identityMap(columnCount: number): Array<number | null> {
  return Array.from({ length: columnCount }, (_, index) => (index < COLUMNS ? index : null));
}
