import { useEffect, useRef, useState } from "react";
import {
  COLUMNS,
  ROWS,
  applyPaste,
  cellRef,
  columnName,
  displayCell,
  parseTsv,
  selectionTsv,
  type SheetGrid as Grid,
} from "../engine/sheet";

type Selection = { row: number; col: number; row2: number; col2: number };

export function SheetGrid({
  initial,
  onChange,
}: {
  initial: Grid;
  onChange: (sheet: Grid) => void;
}) {
  const [sheet, setSheet] = useState(initial);
  const [selection, setSelection] = useState<Selection>({ row: 0, col: 0, row2: 0, col2: 0 });
  const [editing, setEditing] = useState<{ row: number; col: number; draft: string } | null>(null);
  const [widths, setWidths] = useState<number[]>(() => Array.from({ length: COLUMNS }, () => 120));
  const gridRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setSheet(initial);
  }, [initial]);

  function commit(next: Grid) {
    setSheet(next);
    onChange(next);
  }

  function move(row: number, col: number, extend: boolean) {
    const nextRow = Math.max(0, Math.min(ROWS - 1, row));
    const nextCol = Math.max(0, Math.min(COLUMNS - 1, col));
    setEditing(null);
    setSelection((current) => ({
      row: nextRow,
      col: nextCol,
      row2: extend ? current.row2 : nextRow,
      col2: extend ? current.col2 : nextCol,
    }));
  }

  function writeCell(row: number, col: number, raw: string) {
    const next = sheet.map((line) => [...line]);
    next[row][col] = raw;
    commit(next);
  }

  return (
    <div
      className="grid-scroll"
      ref={gridRef}
      tabIndex={0}
      onKeyDown={(event) => {
        if (editing) {
          if (event.key === "Escape") {
            event.preventDefault();
            setEditing(null);
          }
          return;
        }
        const { row, col } = selection;
        if (event.key === "ArrowUp") {
          event.preventDefault();
          move(row - 1, col, event.shiftKey);
        } else if (event.key === "ArrowDown") {
          event.preventDefault();
          move(row + 1, col, event.shiftKey);
        } else if (event.key === "ArrowLeft") {
          event.preventDefault();
          move(row, col - 1, event.shiftKey);
        } else if (event.key === "ArrowRight") {
          event.preventDefault();
          move(row, col + 1, event.shiftKey);
        }
        else if (event.key === "Tab") {
          event.preventDefault();
          move(row, col + (event.shiftKey ? -1 : 1), false);
        } else if (event.key === "Enter") {
          event.preventDefault();
          move(row + 1, col, false);
        } else if (event.key === "F2") setEditing({ row, col, draft: sheet[row][col] });
        else if (event.key === "Delete" || event.key === "Backspace") {
          event.preventDefault();
          writeCell(row, col, "");
        } else if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
          setEditing({ row, col, draft: event.key });
        }
      }}
      onCopy={(event) => {
        if (editing) return;
        event.preventDefault();
        event.clipboardData.setData(
          "text/plain",
          selectionTsv(sheet, selection.row, selection.col, selection.row2, selection.col2),
        );
      }}
      onPaste={(event) => {
        if (editing) return;
        event.preventDefault();
        const text = event.clipboardData.getData("text/plain");
        commit(applyPaste(sheet, selection.row, selection.col, parseTsv(text)));
      }}
    >
      <table className="sheet">
        <thead>
          <tr>
            <th className="row-head" />
            {widths.map((width, col) => (
              <th key={columnName(col)} style={{ width }}>
                {columnName(col)}
                <span
                  className="col-resize"
                  onMouseDown={(event) => {
                    event.preventDefault();
                    const startX = event.clientX;
                    const start = width;
                    function drag(ev: MouseEvent) {
                      setWidths((current) => {
                        const next = [...current];
                        next[col] = Math.max(64, start + ev.clientX - startX);
                        return next;
                      });
                    }
                    function up() {
                      window.removeEventListener("mousemove", drag);
                      window.removeEventListener("mouseup", up);
                    }
                    window.addEventListener("mousemove", drag);
                    window.addEventListener("mouseup", up);
                  }}
                />
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sheet.map((line, row) => (
            <tr key={row}>
              <th className="row-head">{row + 1}</th>
              {line.map((raw, col) => {
                const active = selection.row === row && selection.col === col;
                const isEditing = editing?.row === row && editing.col === col;
                return (
                  <td
                    key={cellRef(row, col)}
                    className={active ? "active" : ""}
                    style={{ width: widths[col] }}
                    onMouseDown={(event) => {
                      if (event.shiftKey) setSelection((current) => ({ ...current, row, col }));
                      else setSelection({ row, col, row2: row, col2: col });
                      setEditing(null);
                      gridRef.current?.focus();
                    }}
                    onDoubleClick={() => setEditing({ row, col, draft: raw })}
                  >
                    {isEditing ? (
                      <input
                        autoFocus
                        className="cell-editor"
                        value={editing.draft}
                        onChange={(event) => setEditing({ row, col, draft: event.target.value })}
                        onBlur={() => {
                          writeCell(row, col, editing.draft);
                          setEditing(null);
                        }}
                        onKeyDown={(event) => {
                          if (event.key === "Enter") {
                            event.preventDefault();
                            writeCell(row, col, editing.draft);
                            setEditing(null);
                            move(row + 1, col, false);
                            gridRef.current?.focus();
                          } else if (event.key === "Escape") {
                            event.preventDefault();
                            setEditing(null);
                            gridRef.current?.focus();
                          }
                        }}
                      />
                    ) : (
                      <span className="cell-text">{displayCell(sheet, row, col)}</span>
                    )}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
