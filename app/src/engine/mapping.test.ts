import { describe, expect, it } from "vitest";
import { applyColumnMap } from "./mapping";

describe("workbook column mapping", () => {
  it("places mapped columns and skips the header row", () => {
    const sheet = applyColumnMap(
      [
        ["Item", "Qty", "Rate"],
        ["Concrete", "2.5", "10"],
      ],
      [0, 1, null],
      true,
    );
    expect(sheet[0].slice(0, 3)).toEqual(["Concrete", "2.5", ""]);
  });
});
