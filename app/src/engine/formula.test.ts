import { describe, expect, it } from "vitest";
import { evaluate, excelRound, quantityTimesRate, compositeRate, type CellValue } from "./formula";

function sheet(cells: Record<string, CellValue>) {
  return (ref: string) => cells[ref] ?? null;
}

describe("Excel-matching formulas", () => {
  it("follows arithmetic precedence", () => {
    expect(evaluate("=2+3*4")).toBe(14);
    expect(evaluate("=(2+3)*4")).toBe(20);
    expect(evaluate("=10/4")).toBe(2.5);
    expect(evaluate("=2^3")).toBe(8);
    expect(evaluate("=-2^2")).toBe(-4);
  });

  it("sums numbers and ignores blanks in a range", () => {
    expect(evaluate("=SUM(1,2,3)")).toBe(6);
    expect(evaluate("=SUM(A1:A3)", sheet({ A1: 1, A2: null, A3: 3 }))).toBe(4);
  });

  it("evaluates IF the way Excel does", () => {
    expect(evaluate("=IF(1>2,5,9)")).toBe(9);
    expect(evaluate('=IF(A1="QS",1,0)', sheet({ A1: "QS" }))).toBe(1);
    expect(evaluate("=IF(0,5,9)")).toBe(9);
  });

  it("rounds half away from zero", () => {
    expect(excelRound(2.5, 0)).toBe(3);
    expect(excelRound(-1.5, 0)).toBe(-2);
    expect(evaluate("=ROUND(2.5,0)")).toBe(3);
    expect(evaluate("=ROUND(-1.5,0)")).toBe(-2);
    expect(evaluate("=ROUND(1234,-2)")).toBe(1200);
  });

  it("multiplies quantity by rate", () => {
    expect(quantityTimesRate(2.5, 10)).toBe(25);
    expect(evaluate("=A1*B1", sheet({ A1: 2.5, B1: 10 }))).toBe(25);
  });

  it("builds a composite rate the way the worked sheet does", () => {
    const rate = compositeRate({
      materialQty: 8,
      materialRate: 100,
      labourQty: 2,
      labourRate: 50,
      plantQty: 1,
      plantRate: 40,
      wastagePercent: 5,
      overheadPercent: 10,
      profitPercent: 10,
    });
    expect(excelRound(rate, 1)).toBe(1185.8);
  });

  it("builds a running total with an expanding SUM", () => {
    const cells = { A1: 10, A2: 20, A3: 5 };
    expect(evaluate("=SUM($A$1:A1)", sheet(cells))).toBe(10);
    expect(evaluate("=SUM($A$1:A2)", sheet(cells))).toBe(30);
    expect(evaluate("=SUM($A$1:A3)", sheet(cells))).toBe(35);
  });

  it("reports division by zero", () => {
    expect(evaluate("=1/0")).toBe("#DIV/0!");
  });
});
