import { describe, expect, it } from "vitest";
import { applyPaste, blankSheet, displayCell, parseTsv } from "./sheet";

describe("Excel paste", () => {
  it("pastes a tab-separated range from Excel", () => {
    const pasted = applyPaste(blankSheet(), 0, 0, parseTsv("Concrete\t2.5\t10\r\nSteel\t1\t20\r\n"));
    expect(pasted[0].slice(0, 3)).toEqual(["Concrete", "2.5", "10"]);
    expect(pasted[1].slice(0, 3)).toEqual(["Steel", "1", "20"]);
    expect(displayCell(pasted, 0, 1)).toBe("2.5");
  });

  it("evaluates a pasted quantity times rate formula", () => {
    const pasted = applyPaste(blankSheet(), 0, 0, parseTsv("2.5\t10\t=A1*B1"));
    expect(displayCell(pasted, 0, 2)).toBe("25");
  });

  it("keeps a running total after paste", () => {
    const pasted = applyPaste(blankSheet(), 0, 0, parseTsv("10\n20\n5"));
    const withTotal = applyPaste(pasted, 0, 1, parseTsv("=SUM($A$1:A1)\n=SUM($A$1:A2)\n=SUM($A$1:A3)"));
    expect(displayCell(withTotal, 2, 1)).toBe("35");
  });
});
