import { evaluate, type CellValue } from "./formula";

export const COLUMNS = 8;
export const ROWS = 40;

export type SheetGrid = string[][];

export function blankSheet(): SheetGrid {
  return Array.from({ length: ROWS }, () => Array.from({ length: COLUMNS }, () => ""));
}

export function columnName(index: number): string {
  let n = index + 1;
  let letters = "";
  while (n > 0) {
    const rem = (n - 1) % 26;
    letters = String.fromCharCode(65 + rem) + letters;
    n = Math.floor((n - 1) / 26);
  }
  return letters;
}

export function cellRef(row: number, col: number): string {
  return `${columnName(col)}${row + 1}`;
}

export function parseTsv(text: string): string[][] {
  const source = text.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
  const body = source.endsWith("\n") ? source.slice(0, -1) : source;
  if (body === "") return [];
  return body.split("\n").map((line) => line.split("\t"));
}

export function applyPaste(sheet: SheetGrid, startRow: number, startCol: number, grid: string[][]): SheetGrid {
  const next = sheet.map((row) => [...row]);
  grid.forEach((line, rowOffset) => {
    const row = startRow + rowOffset;
    if (row < 0 || row >= next.length) return;
    line.forEach((value, colOffset) => {
      const col = startCol + colOffset;
      if (col < 0 || col >= next[row].length) return;
      next[row][col] = value;
    });
  });
  return next;
}

export function selectionTsv(sheet: SheetGrid, row: number, col: number, row2: number, col2: number): string {
  const r0 = Math.min(row, row2);
  const r1 = Math.max(row, row2);
  const c0 = Math.min(col, col2);
  const c1 = Math.max(col, col2);
  const lines: string[] = [];
  for (let r = r0; r <= r1; r += 1) {
    const cells: string[] = [];
    for (let c = c0; c <= c1; c += 1) cells.push(sheet[r]?.[c] ?? "");
    lines.push(cells.join("\t"));
  }
  return lines.join("\n");
}

export function displayCell(sheet: SheetGrid, row: number, col: number): string {
  const value = readCell(sheet, cellRef(row, col), new Set());
  if (value === null) return "";
  if (typeof value === "boolean") return value ? "TRUE" : "FALSE";
  return String(value);
}

function readCell(sheet: SheetGrid, ref: string, stack: Set<string>): CellValue | string {
  const decoded = decodeRef(ref);
  if (!decoded) return "#REF!";
  const raw = sheet[decoded.row]?.[decoded.col] ?? "";
  if (raw.trim() === "") return null;
  if (!raw.trim().startsWith("=")) {
    const literal = evaluate(raw);
    return literal;
  }
  if (stack.has(ref)) return "#REF!";
  stack.add(ref);
  const value = evaluate(raw, (next) => readCell(sheet, next, stack));
  stack.delete(ref);
  return value;
}

function decodeRef(ref: string): { row: number; col: number } | null {
  const match = /^([A-Z]+)(\d+)$/.exec(ref.toUpperCase());
  if (!match) return null;
  let col = 0;
  for (const char of match[1]) col = col * 26 + (char.charCodeAt(0) - 64);
  return { col: col - 1, row: Number(match[2]) - 1 };
}
