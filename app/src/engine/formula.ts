export type CellValue = number | string | boolean | null;

export type CellReader = (ref: string) => CellValue;

const ERROR = {
  div0: "#DIV/0!",
  value: "#VALUE!",
  name: "#NAME?",
  ref: "#REF!",
  na: "#N/A",
} as const;

type Token =
  | { kind: "num"; value: number }
  | { kind: "str"; value: string }
  | { kind: "id"; value: string }
  | { kind: "op"; value: string }
  | { kind: "ref"; value: string }
  | { kind: "eof" };

export function evaluate(formula: string, read: CellReader = () => null): CellValue | string {
  const source = formula.trim();
  if (!source.startsWith("=")) return coerceLiteral(source);
  try {
    const parser = new Parser(tokenize(source.slice(1)), read);
    const value = parser.parseComparison();
    parser.expect("eof");
    return value;
  } catch (cause) {
    return cause instanceof Error ? cause.message : ERROR.value;
  }
}

/** Quantity × rate, the same multiplication the sheet formula uses. */
export function quantityTimesRate(quantity: number, rate: number): number {
  const result = evaluate("=A1*B1", (ref) => (ref === "A1" ? quantity : rate));
  if (typeof result !== "number") throw new Error(String(result));
  return result;
}

/** Percent of a base, as Excel calculates A1*B1/100. */
export function percentOf(base: number, percent: number): number {
  const result = evaluate("=A1*B1/100", (ref) => (ref === "A1" ? base : percent));
  if (typeof result !== "number") throw new Error(String(result));
  return result;
}

export type RateBuildup = {
  materialQty: number;
  materialRate: number;
  labourQty: number;
  labourRate: number;
  plantQty: number;
  plantRate: number;
  wastagePercent: number;
  overheadPercent: number;
  profitPercent: number;
};

/** Material, labour, and plant, then wastage on material, overhead, and profit. */
export function compositeRate(input: RateBuildup): number {
  const material = quantityTimesRate(input.materialQty, input.materialRate);
  const labour = quantityTimesRate(input.labourQty, input.labourRate);
  const plant = quantityTimesRate(input.plantQty, input.plantRate);
  const wastage = percentOf(material, input.wastagePercent);
  const direct = evaluate("=A1+A2+A3+A4", (ref) => {
    const values: Record<string, number> = { A1: material, A2: labour, A3: plant, A4: wastage };
    return values[ref] ?? null;
  });
  if (typeof direct !== "number") throw new Error(String(direct));
  const overhead = percentOf(direct, input.overheadPercent);
  const afterOverhead = quantityTimesRate(1, direct + overhead);
  const profit = percentOf(afterOverhead, input.profitPercent);
  const total = evaluate("=A1+A2+A3", (ref) => {
    const values: Record<string, number> = { A1: direct, A2: overhead, A3: profit };
    return values[ref] ?? null;
  });
  if (typeof total !== "number") throw new Error(String(total));
  return total;
}

function coerceLiteral(source: string): CellValue {
  if (source === "") return null;
  if (source.startsWith("'")) return source.slice(1);
  const number = Number(source.replace(/,/g, ""));
  return source.trim() !== "" && Number.isFinite(number) && /^-?[\d,]+(\.\d+)?$/.test(source.trim())
    ? number
    : source;
}

function tokenize(input: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  while (i < input.length) {
    const char = input[i];
    if (char === " " || char === "\t") {
      i += 1;
      continue;
    }
    if (char === '"') {
      let value = "";
      i += 1;
      while (i < input.length && input[i] !== '"') {
        value += input[i];
        i += 1;
      }
      i += 1;
      tokens.push({ kind: "str", value });
      continue;
    }
    if (/[0-9.]/.test(char)) {
      let raw = "";
      while (i < input.length && /[0-9.]/.test(input[i])) {
        raw += input[i];
        i += 1;
      }
      if (input[i] === "%") {
        tokens.push({ kind: "num", value: Number(raw) / 100 });
        i += 1;
      } else {
        tokens.push({ kind: "num", value: Number(raw) });
      }
      continue;
    }
    if (/[A-Za-z]/.test(char)) {
      let raw = "";
      while (i < input.length && /[A-Za-z0-9_.]/.test(input[i])) {
        raw += input[i];
        i += 1;
      }
      const upper = raw.toUpperCase();
      if (/^(\$?[A-Z]{1,3})(\$?\d+)$/.test(upper)) tokens.push({ kind: "ref", value: normalizeRef(upper) });
      else tokens.push({ kind: "id", value: upper });
      continue;
    }
    if (char === "$") {
      const rest = input.slice(i).toUpperCase();
      const match = /^\$?[A-Z]{1,3}\$?\d+/.exec(rest);
      if (!match) throw new Error(ERROR.ref);
      tokens.push({ kind: "ref", value: normalizeRef(match[0]) });
      i += match[0].length;
      continue;
    }
    const two = input.slice(i, i + 2);
    if (two === "<>" || two === "<=" || two === ">=") {
      tokens.push({ kind: "op", value: two });
      i += 2;
      continue;
    }
    tokens.push({ kind: "op", value: char });
    i += 1;
  }
  tokens.push({ kind: "eof" });
  return tokens;
}

function normalizeRef(ref: string): string {
  return ref.replace(/\$/g, "").toUpperCase();
}

class Parser {
  private index = 0;

  constructor(
    private readonly tokens: Token[],
    private readonly read: CellReader,
  ) {}

  parseComparison(): CellValue | string {
    let left = this.parseAdd();
    while (this.isOp("=", "<>", "<", ">", "<=", ">=")) {
      const op = this.takeOp();
      const right = this.parseAdd();
      left = compare(left, op, right);
    }
    return left;
  }

  private parseAdd(): CellValue | string {
    let left = this.parseMul();
    while (this.isOp("+", "-")) {
      const op = this.takeOp();
      const right = this.parseMul();
      left = op === "+" ? add(left, right) : subtract(left, right);
    }
    return left;
  }

  private parseMul(): CellValue | string {
    let left = this.parseUnary();
    while (this.isOp("*", "/")) {
      const op = this.takeOp();
      const right = this.parseUnary();
      left = op === "*" ? multiply(left, right) : divide(left, right);
    }
    return left;
  }

  private parseUnary(): CellValue | string {
    if (this.isOp("-")) {
      this.take();
      return negate(this.parsePower());
    }
    if (this.isOp("+")) {
      this.take();
      return this.parsePower();
    }
    return this.parsePower();
  }

  private parsePower(): CellValue | string {
    const left = this.parsePrimary();
    if (!this.isOp("^")) return left;
    this.take();
    const right = this.parsePower();
    return power(left, right);
  }

  private parsePrimary(): CellValue | string {
    const token = this.peek();
    if (token.kind === "num") {
      this.take();
      return token.value;
    }
    if (token.kind === "str") {
      this.take();
      return token.value;
    }
    if (token.kind === "ref") {
      this.take();
      if (this.isOp(":")) {
        this.take();
        const end = this.take();
        if (end.kind !== "ref") throw new Error(ERROR.ref);
        return sumValues(expandRange(token.value, end.value).map((ref) => this.read(ref)));
      }
      return this.read(token.value);
    }
    if (token.kind === "id") {
      this.take();
      if (token.value === "TRUE") return true;
      if (token.value === "FALSE") return false;
      if (!this.isOp("(")) throw new Error(ERROR.name);
      this.take();
      const args = this.parseArgs();
      return call(token.value, args, this.read);
    }
    if (this.isOp("(")) {
      this.take();
      const value = this.parseComparison();
      if (!this.isOp(")")) throw new Error(ERROR.value);
      this.take();
      return value;
    }
    throw new Error(ERROR.value);
  }

  private parseArgs(): Array<CellValue | string | string[]> {
    const args: Array<CellValue | string | string[]> = [];
    if (this.isOp(")")) {
      this.take();
      return args;
    }
    args.push(this.parseArg());
    while (this.isOp(",")) {
      this.take();
      args.push(this.parseArg());
    }
    if (!this.isOp(")")) throw new Error(ERROR.value);
    this.take();
    return args;
  }

  private parseArg(): CellValue | string | string[] {
    const token = this.peek();
    const next = this.tokens[this.index + 1];
    if (token.kind === "ref" && next?.kind === "op" && next.value === ":") {
      this.take();
      this.take();
      const end = this.take();
      if (end.kind !== "ref") throw new Error(ERROR.ref);
      return expandRange(token.value, end.value);
    }
    return this.parseComparison();
  }

  private peek(): Token {
    return this.tokens[this.index] ?? { kind: "eof" };
  }

  private take(): Token {
    const token = this.peek();
    this.index += 1;
    return token;
  }

  private takeOp(): string {
    const token = this.take();
    if (token.kind !== "op") throw new Error(ERROR.value);
    return token.value;
  }

  private isOp(...ops: string[]): boolean {
    const token = this.peek();
    return token.kind === "op" && ops.includes(token.value);
  }

  expect(kind: Token["kind"]): void {
    if (this.peek().kind !== kind) throw new Error(ERROR.value);
  }
}

function call(name: string, args: Array<CellValue | string | string[]>, read: CellReader): CellValue | string {
  if (name === "SUM") return sum(args, read);
  if (name === "IF") return ifFunction(args);
  if (name === "ROUND") return roundFunction(args);
  throw new Error(ERROR.name);
}

function sum(args: Array<CellValue | string | string[]>, read: CellReader): number | string {
  const values: CellValue[] = [];
  for (const arg of args) {
    if (Array.isArray(arg)) values.push(...arg.map((ref) => read(ref)));
    else if (typeof arg === "string" && arg.startsWith("#")) return arg;
    else values.push(arg);
  }
  return sumValues(values);
}

function sumValues(values: CellValue[]): number | string {
  let total = 0;
  for (const value of values) {
    if (value === null || value === "" || typeof value === "boolean") continue;
    if (typeof value === "number") {
      total += value;
      continue;
    }
    if (typeof value === "string" && value.startsWith("#")) return value;
  }
  return total;
}

function ifFunction(args: Array<CellValue | string | string[]>): CellValue | string {
  if (args.length < 2 || Array.isArray(args[0]) || Array.isArray(args[1])) throw new Error(ERROR.value);
  const whenTrue = args[1];
  const whenFalse = args.length > 2 && !Array.isArray(args[2]) ? args[2] : false;
  return truthy(args[0]) ? whenTrue : whenFalse;
}

function roundFunction(args: Array<CellValue | string | string[]>): number | string {
  if (args.length < 1 || Array.isArray(args[0])) throw new Error(ERROR.value);
  const number = asNumber(args[0]);
  if (typeof number === "string") return number;
  const digitsArg = args.length > 1 && !Array.isArray(args[1]) ? args[1] : 0;
  const digits = asNumber(digitsArg ?? 0);
  if (typeof digits === "string") return digits;
  return excelRound(number, digits);
}

/** Excel ROUND: half away from zero. */
export function excelRound(value: number, digits: number): number {
  const places = Math.trunc(digits);
  const factor = 10 ** Math.abs(places);
  const shifted = places >= 0 ? value * factor : value / factor;
  const sign = shifted < 0 ? -1 : 1;
  const abs = Math.abs(shifted);
  const whole = Math.floor(abs);
  const fraction = abs - whole;
  const rounded = fraction >= 0.5 - 1e-10 ? whole + 1 : whole;
  const result = sign * rounded;
  return places >= 0 ? result / factor : result * factor;
}

function add(left: CellValue | string, right: CellValue | string): number | string {
  const a = asNumber(left);
  const b = asNumber(right);
  if (typeof a === "string") return a;
  if (typeof b === "string") return b;
  return a + b;
}

function subtract(left: CellValue | string, right: CellValue | string): number | string {
  const a = asNumber(left);
  const b = asNumber(right);
  if (typeof a === "string") return a;
  if (typeof b === "string") return b;
  return a - b;
}

function multiply(left: CellValue | string, right: CellValue | string): number | string {
  const a = asNumber(left);
  const b = asNumber(right);
  if (typeof a === "string") return a;
  if (typeof b === "string") return b;
  return a * b;
}

function divide(left: CellValue | string, right: CellValue | string): number | string {
  const a = asNumber(left);
  const b = asNumber(right);
  if (typeof a === "string") return a;
  if (typeof b === "string") return b;
  if (b === 0) return ERROR.div0;
  return a / b;
}

function power(left: CellValue | string, right: CellValue | string): number | string {
  const a = asNumber(left);
  const b = asNumber(right);
  if (typeof a === "string") return a;
  if (typeof b === "string") return b;
  return a ** b;
}

function negate(value: CellValue | string): number | string {
  const number = asNumber(value);
  return typeof number === "string" ? number : -number;
}

function compare(left: CellValue | string, op: string, right: CellValue | string): boolean | string {
  if (typeof left === "string" && left.startsWith("#")) return left;
  if (typeof right === "string" && right.startsWith("#")) return right;
  if (typeof left === "number" && typeof right === "number") {
    if (op === "=") return left === right;
    if (op === "<>") return left !== right;
    if (op === "<") return left < right;
    if (op === ">") return left > right;
    if (op === "<=") return left <= right;
    return left >= right;
  }
  const a = String(left ?? "").toLowerCase();
  const b = String(right ?? "").toLowerCase();
  if (op === "=") return a === b;
  if (op === "<>") return a !== b;
  if (op === "<") return a < b;
  if (op === ">") return a > b;
  if (op === "<=") return a <= b;
  return a >= b;
}

function truthy(value: CellValue | string): boolean {
  if (value === true) return true;
  if (value === false || value === null || value === "") return false;
  if (typeof value === "number") return value !== 0;
  return false;
}

function asNumber(value: CellValue | string): number | string {
  if (typeof value === "string" && value.startsWith("#")) return value;
  if (typeof value === "number") return value;
  if (value === null || value === "") return 0;
  if (typeof value === "boolean") return value ? 1 : 0;
  const number = Number(value);
  return Number.isFinite(number) ? number : ERROR.value;
}

function expandRange(start: string, end: string): string[] {
  const a = decode(start);
  const b = decode(end);
  const c1 = Math.min(a.col, b.col);
  const c2 = Math.max(a.col, b.col);
  const r1 = Math.min(a.row, b.row);
  const r2 = Math.max(a.row, b.row);
  const refs: string[] = [];
  for (let row = r1; row <= r2; row += 1) {
    for (let col = c1; col <= c2; col += 1) refs.push(encode(col, row));
  }
  return refs;
}

function decode(ref: string): { col: number; row: number } {
  const match = /^([A-Z]+)(\d+)$/.exec(ref);
  if (!match) throw new Error(ERROR.ref);
  let col = 0;
  for (const char of match[1]) col = col * 26 + (char.charCodeAt(0) - 64);
  return { col, row: Number(match[2]) };
}

function encode(col: number, row: number): string {
  let n = col;
  let letters = "";
  while (n > 0) {
    const rem = (n - 1) % 26;
    letters = String.fromCharCode(65 + rem) + letters;
    n = Math.floor((n - 1) / 26);
  }
  return `${letters}${row}`;
}
