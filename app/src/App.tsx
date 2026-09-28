import { useEffect, useRef, useState } from "react";
import { SheetGrid } from "./components/SheetGrid";
import { applyColumnMap, identityMap } from "./engine/mapping";
import { COLUMNS, blankSheet, columnName } from "./engine/sheet";
import {
  accountStatus,
  appVersion,
  createAdministrator,
  backupStatus,
  createBackup,
  createFirm,
  createUser,
  currentFirm,
  assignRole,
  loadSheet,
  listAudit,
  listCodes,
  listItems,
  listMeasures,
  addMeasure,
  listLocations,
  listProjects,
  saveCertificate,
  saveCode,
  saveContract,
  saveContractor,
  saveBudget,
  saveEstimate,
  saveItem,
  saveRate,
  approveVariation,
  approveBoq,
  addContractItem,
  exportReport,
  addLocation,
  saveProject,
  pickFolder,
  pickXlsxPath,
  projectReport,
  reconcileBoq,
  readXlsx,
  restoreBackup,
  setLocalServer,
  writeXlsx,
  login,
  logout,
  MODULES,
  openFirm,
  PERMISSIONS,
  pickFirmPath,
  saveRole,
  saveVariation,
  saveWorkOrder,
  saveSheet,
  type AccountStatus,
  type AuditEvent,
  type BackupRecord,
  type CodeEntry,
  type FirmFile,
  type LocationNode,
  type MeasureLine,
  type Project,
  type Reconciliation,
  type ReportLine,
  type WorkItem,
} from "./api";

export function App() {
  const [version, setVersion] = useState("0.1.0");
  const [firm, setFirm] = useState<FirmFile | null>(null);
  const [accounts, setAccounts] = useState<AccountStatus | null>(null);
  const [screen, setScreen] = useState<"sheet" | "accounts" | "projects">("sheet");
  const [grid, setGrid] = useState(blankSheet);
  const [preview, setPreview] = useState<string[][] | null>(null);
  const [mapping, setMapping] = useState<Array<number | null>>([]);
  const [headerRow, setHeaderRow] = useState(true);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [backups, setBackups] = useState<BackupRecord[]>([]);
  const [audit, setAudit] = useState<AuditEvent[]>([]);
  const [serverFolder, setServerFolder] = useState("");
  const saveTimer = useRef<number | null>(null);

  useEffect(() => {
    void appVersion().then(setVersion);
    void currentFirm()
      .then(async (opened) => {
        setFirm(opened);
        if (opened) setAccounts(await accountStatus());
      })
      .catch((cause: unknown) => setError(text(cause)));
  }, []);

  useEffect(() => {
    if (!accounts?.session) return;
    void loadSheet()
      .then((cells) => {
        const next = blankSheet();
        for (const cell of cells) {
          if (next[cell.row]?.[cell.col] !== undefined) next[cell.row][cell.col] = cell.raw;
        }
        setGrid(next);
      })
      .catch((cause: unknown) => setError(text(cause)));
  }, [accounts?.session?.username]);

  function sheetCells(source: string[][]) {
    return source.flatMap((row, rowIndex) =>
      row.flatMap((raw, colIndex) => (raw === "" ? [] : [{ row: rowIndex, col: colIndex, raw }])),
    );
  }

  function queueSheetSave(next: string[][]) {
    setGrid(next);
    if (saveTimer.current) window.clearTimeout(saveTimer.current);
    const cells = sheetCells(next);
    saveTimer.current = window.setTimeout(() => {
      void saveSheet(cells).catch((cause: unknown) => setError(text(cause)));
    }, 300);
  }

  async function flushSheet(source: string[][]) {
    if (saveTimer.current) window.clearTimeout(saveTimer.current);
    await saveSheet(sheetCells(source));
  }

  async function refreshBackups() {
    const status = await backupStatus();
    setServerFolder(status.folder);
    setBackups(status.history);
  }

  async function chooseServer() {
    setError("");
    setNotice("");
    try {
      const folder = await pickFolder();
      if (!folder) return;
      await setLocalServer(folder);
      setServerFolder(folder);
      setNotice("QS Local Server folder saved.");
    } catch (cause: unknown) {
      setError(text(cause));
    }
  }

  async function onCreateBackup() {
    setError("");
    setNotice("");
    try {
      await flushSheet(grid);
      const record = await createBackup();
      setNotice(`Backup saved to ${record.folder}`);
      await refreshBackups();
    } catch (cause: unknown) {
      setError(text(cause));
    }
  }

  async function onRestore(folder: string) {
    setError("");
    setNotice("");
    try {
      await restoreBackup(folder);
      const cells = await loadSheet();
      const next = blankSheet();
      for (const cell of cells) {
        if (next[cell.row]?.[cell.col] !== undefined) next[cell.row][cell.col] = cell.raw;
      }
      setGrid(next);
      setNotice("Working sheet restored from the selected backup.");
    } catch (cause: unknown) {
      setError(text(cause));
    }
  }

  async function openWorkbook() {
    setError("");
    try {
      const path = await pickXlsxPath("open");
      if (!path) return;
      const rows = await readXlsx(path);
      setPreview(rows);
      setMapping(identityMap(rows[0]?.length ?? 0));
      setHeaderRow(true);
    } catch (cause: unknown) {
      setError(text(cause));
    }
  }

  async function saveWorkbook() {
    setError("");
    try {
      const path = await pickXlsxPath("save");
      if (!path) return;
      const lastRow = grid.reduce((max, row, index) => (row.some((cell) => cell !== "") ? index : max), -1);
      const rows = grid.slice(0, lastRow + 1).map((row) => {
        let last = -1;
        row.forEach((cell, index) => {
          if (cell !== "") last = index;
        });
        return row.slice(0, last + 1);
      });
      await writeXlsx(path, rows);
    } catch (cause: unknown) {
      setError(text(cause));
    }
  }

  async function refreshAccounts() {
    setAccounts(await accountStatus());
  }

  async function onCreate() {
    setError("");
    try {
      const path = await pickFirmPath("save");
      if (!path) return;
      setFirm(await createFirm(path));
      await refreshAccounts();
    } catch (cause: unknown) {
      setError(text(cause));
    }
  }

  async function onOpen() {
    setError("");
    try {
      const path = await pickFirmPath("open");
      if (!path) return;
      setFirm(await openFirm(path));
      await refreshAccounts();
    } catch (cause: unknown) {
      setError(text(cause));
    }
  }

  return (
    <div className="shell">
      <header>
        <strong>QS</strong>
        <span>Version {version}</span>
      </header>
      <main className={firm && accounts?.session && screen === "sheet" ? "wide" : ""}>
        {!firm && (
          <>
            <h1>Quantity surveying</h1>
            <p className="muted">Create a firm file on this PC, or open one that already exists.</p>
            {error && <p className="error">{error}</p>}
            <div className="actions">
              <button type="button" onClick={() => void onCreate()}>Create firm file</button>
              <button type="button" onClick={() => void onOpen()}>Open firm file</button>
            </div>
          </>
        )}
        {firm && accounts?.needsAdministrator && (
          <AccountForm
            title="Create the administrator"
            detail="This firm file has one administrator. Four other users can be added later."
            submitLabel="Create administrator"
            error={error}
            onSubmit={async (username, password) => {
              setError("");
              try {
                await createAdministrator(username, password);
                await refreshAccounts();
              } catch (cause: unknown) {
                setError(text(cause));
              }
            }}
          />
        )}
        {firm && accounts && !accounts.needsAdministrator && !accounts.session && (
          <AccountForm
            title="Sign in"
            detail="Enter the username and password for this firm file."
            submitLabel="Sign in"
            error={error}
            onSubmit={async (username, password) => {
              setError("");
              try {
                await login(username, password);
                await refreshAccounts();
              } catch (cause: unknown) {
                setError(text(cause));
              }
            }}
          />
        )}
        {firm && accounts?.session && (
          <>
            <div className="actions">
              <button type="button" onClick={() => setScreen("sheet")}>Sheet</button>
              <button type="button" onClick={() => setScreen("projects")}>Projects</button>
              <button type="button" onClick={() => setScreen("accounts")}>Accounts</button>
            </div>
            {screen === "sheet" && (
              <>
                <div className="actions">
                  <button type="button" onClick={() => void openWorkbook()}>Open workbook</button>
                  <button type="button" onClick={() => void saveWorkbook()}>Save workbook</button>
                  {accounts.session.isAdministrator && (
                    <button type="button" onClick={() => void onCreateBackup()}>Create backup</button>
                  )}
                </div>
                {notice && <p className="muted">{notice}</p>}
                {preview && (
                  <div className="preview">
                    <p className="muted">Map each workbook column onto the sheet. Skip a column by choosing Skip.</p>
                    <label>
                      <input type="checkbox" checked={headerRow} onChange={(event) => setHeaderRow(event.target.checked)} />
                      First row is a header
                    </label>
                    <table className="sheet">
                      <thead>
                        <tr>
                          {(preview[0] ?? []).map((header, index) => (
                            <th key={index}>
                              {header || `Column ${index + 1}`}
                              <select
                                aria-label={`Map ${header || index + 1}`}
                                value={mapping[index] === null || mapping[index] === undefined ? "" : String(mapping[index])}
                                onChange={(event) => {
                                  const value = event.target.value;
                                  setMapping((current) => {
                                    const next = [...current];
                                    next[index] = value === "" ? null : Number(value);
                                    return next;
                                  });
                                }}
                              >
                                <option value="">Skip</option>
                                {Array.from({ length: COLUMNS }, (_, col) => (
                                  <option key={col} value={col}>{columnName(col)}</option>
                                ))}
                              </select>
                            </th>
                          ))}
                        </tr>
                      </thead>
                      <tbody>
                        {preview.slice(headerRow ? 1 : 0, headerRow ? 4 : 3).map((row, rowIndex) => (
                          <tr key={rowIndex}>
                            {row.map((cell, col) => <td key={col}>{cell}</td>)}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                    <div className="actions">
                      <button
                        type="button"
                        onClick={() => {
                          queueSheetSave(applyColumnMap(preview, mapping, headerRow));
                          setPreview(null);
                        }}
                      >
                        Import
                      </button>
                    </div>
                  </div>
                )}
                <SheetGrid initial={grid} onChange={queueSheetSave} />
              </>
            )}
            {screen === "projects" && (
              <ProjectScreen onError={(message) => setError(message)} />
            )}
            {screen === "accounts" && (
          <>
            <h1>{accounts.session.displayName}</h1>
            <p className="muted">
              {accounts.session.isAdministrator ? "Administrator" : "User"} · {firm.path}
            </p>
            {accounts.session.isAdministrator && (
              <>
                <h2>QS Local Server</h2>
                <p className="muted">{serverFolder || "No backup folder set."}</p>
                <div className="actions">
                  <button type="button" onClick={() => void chooseServer()}>Choose backup folder</button>
                  <button type="button" onClick={() => void refreshBackups().catch((cause: unknown) => setError(text(cause)))}>Refresh history</button>
                </div>
                <ul className="users">
                  {backups.map((record) => (
                    <li key={record.id}>
                      {record.createdAt} · {record.createdBy} · {record.verified ? "verified" : "not verified"}
                      <button type="button" onClick={() => void onRestore(record.folder)}>Restore</button>
                      <span className="muted">{record.folder}</span>
                    </li>
                  ))}
                </ul>
                <h2>Audit</h2>
                <div className="actions">
                  <button type="button" onClick={() => void listAudit().then(setAudit).catch((cause: unknown) => setError(text(cause)))}>Refresh audit</button>
                </div>
                <ul className="users">
                  {audit.map((event) => (
                    <li key={event.id}>
                      {event.createdAt} · {event.username} · {event.action} {event.target}: {event.oldValue || "blank"} → {event.newValue || "blank"}
                    </li>
                  ))}
                </ul>
                <ul className="users">
                  {accounts.users.map((user) => (
                    <li key={user.username}>
                      {user.displayName} ({user.username})
                      {user.isAdministrator ? " · Administrator" : ` · ${user.roleName ?? "No role"}`}
                      {!user.isAdministrator && accounts.roles.length > 0 && (
                        <select
                          aria-label={`Role for ${user.username}`}
                          value={user.roleName ?? ""}
                          onChange={(event) => {
                            const roleName = event.target.value;
                            if (!roleName) return;
                            void assignRole(user.username, roleName)
                              .then(() => refreshAccounts())
                              .catch((cause: unknown) => setError(text(cause)));
                          }}
                        >
                          <option value="">Assign role</option>
                          {accounts.roles.map((role) => (
                            <option key={role.id} value={role.name}>{role.name}</option>
                          ))}
                        </select>
                      )}
                    </li>
                  ))}
                </ul>
                <RoleForm
                  error={error}
                  onSubmit={async (name, permissions, modules, projects) => {
                    setError("");
                    try {
                      await saveRole(name, permissions, modules, projects);
                      await refreshAccounts();
                    } catch (cause: unknown) {
                      setError(text(cause));
                    }
                  }}
                />
                <AccountForm
                  title="Add a user"
                  detail="Four users besides the administrator. A further account is refused."
                  submitLabel="Add user"
                  named
                  error={error}
                  onSubmit={async (username, password, displayName) => {
                    setError("");
                    try {
                      await createUser(username, displayName ?? username, password);
                      await refreshAccounts();
                    } catch (cause: unknown) {
                      setError(text(cause));
                    }
                  }}
                />
              </>
            )}
            <div className="actions">
              <button
                type="button"
                onClick={() => {
                  void logout().then(() => refreshAccounts()).catch((cause: unknown) => setError(text(cause)));
                }}
              >
                Sign out
              </button>
            </div>
          </>
            )}
          </>
        )}
      </main>
    </div>
  );
}

function ProjectScreen({ onError }: { onError: (message: string) => void }) {
  const [projects, setProjects] = useState<Project[]>([]);
  const [selected, setSelected] = useState("");
  const [nodes, setNodes] = useState<LocationNode[]>([]);
  const [code, setCode] = useState("");
  const [projectName, setProjectName] = useState("");
  const [label, setLabel] = useState("");
  const [locationName, setLocationName] = useState("");
  const [parentId, setParentId] = useState("");

  async function refreshProjects(codeToShow = selected) {
    const next = await listProjects();
    setProjects(next);
    const code = codeToShow && next.some((project) => project.code === codeToShow) ? codeToShow : next[0]?.code ?? "";
    setSelected(code);
    setNodes(code ? await listLocations(code) : []);
  }

  useEffect(() => {
    void refreshProjects().catch((cause: unknown) => onError(text(cause)));
  }, []);

  function depth(node: LocationNode): number {
    let level = 0;
    let parent = node.parentId;
    while (parent !== null) {
      level += 1;
      parent = nodes.find((item) => item.id === parent)?.parentId ?? null;
    }
    return level;
  }

  return (
    <>
      <h1>Projects</h1>
      <p className="muted">Each project can use its own location labels and depth.</p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void saveProject(code, projectName)
            .then(() => refreshProjects(code))
            .then(() => {
              setCode("");
              setProjectName("");
            })
            .catch((cause: unknown) => onError(text(cause)));
        }}
      >
        <label>
          Code
          <input value={code} onChange={(event) => setCode(event.target.value)} />
        </label>
        <label>
          Name
          <input value={projectName} onChange={(event) => setProjectName(event.target.value)} />
        </label>
        <div className="actions">
          <button type="submit">Save project</button>
        </div>
      </form>
      {projects.length > 0 && (
        <>
          <label>
            Project
            <select
              value={selected}
              onChange={(event) => {
                const code = event.target.value;
                setSelected(code);
                void listLocations(code).then(setNodes).catch((cause: unknown) => onError(text(cause)));
              }}
            >
              {projects.map((project) => (
                <option key={project.id} value={project.code}>{project.code} · {project.name}</option>
              ))}
            </select>
          </label>
          <ul className="users">
            {nodes.map((node) => (
              <li key={node.id} style={{ paddingLeft: depth(node) * 16 }}>
                {node.label}: {node.name}
              </li>
            ))}
          </ul>
          <form
            onSubmit={(event) => {
              event.preventDefault();
              void addLocation(selected, parentId === "" ? null : Number(parentId), label, locationName)
                .then(() => listLocations(selected))
                .then((next) => {
                  setNodes(next);
                  setLabel("");
                  setLocationName("");
                })
                .catch((cause: unknown) => onError(text(cause)));
            }}
          >
            <label>
              Parent
              <select value={parentId} onChange={(event) => setParentId(event.target.value)}>
                <option value="">Top of this project</option>
                {nodes.map((node) => (
                  <option key={node.id} value={node.id}>{node.label}: {node.name}</option>
                ))}
              </select>
            </label>
            <label>
              Label
              <input value={label} onChange={(event) => setLabel(event.target.value)} placeholder="Tower, Floor, Zone" />
            </label>
            <label>
              Location name
              <input value={locationName} onChange={(event) => setLocationName(event.target.value)} />
            </label>
            <div className="actions">
              <button type="submit">Add location</button>
            </div>
          </form>
          <CodePanel projectCode={selected} onError={onError} />
          <ContractPanel projectCode={selected} onError={onError} />
          <CostPanel projectCode={selected} onError={onError} />
          <ReportPanel projectCode={selected} onError={onError} />
        </>
      )}
    </>
  );
}

function ReportPanel({ projectCode, onError }: { projectCode: string; onError: (message: string) => void }) {
  const [lines, setLines] = useState<ReportLine[]>([]);

  return (
    <>
      <h2>Reports</h2>
      <div className="actions">
        <button
          type="button"
          onClick={() => {
            void projectReport(projectCode)
              .then(setLines)
              .catch((cause: unknown) => onError(text(cause)));
          }}
        >
          Show report
        </button>
        <button
          type="button"
          disabled={lines.length === 0}
          onClick={() => {
            void pickXlsxPath("save").then((path) => {
              if (!path) return;
              void exportReport(projectCode, path).catch((cause: unknown) => onError(text(cause)));
            });
          }}
        >
          Excel
        </button>
        <button type="button" disabled={lines.length === 0} onClick={() => window.print()}>
          Print
        </button>
      </div>
      {lines.map((line, index) => (
        <p key={`${line.section}-${line.label}-${index}`}>
          {line.section} {line.label} {line.figure}
        </p>
      ))}
    </>
  );
}

function CostPanel({ projectCode, onError }: { projectCode: string; onError: (message: string) => void }) {
  const [actual, setActual] = useState("");
  const [variance, setVariance] = useState("");

  return (
    <>
      <h2>Cost</h2>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void saveBudget(projectCode, String(form.get("budget") ?? ""))
            .then((position) => {
              setActual(position.actual);
              setVariance(position.variance);
            })
            .catch((cause: unknown) => onError(text(cause)));
        }}
      >
        <label>Budget<input name="budget" /></label>
        <div className="actions">
          <button type="submit">Save budget</button>
        </div>
      </form>
      {variance !== "" && (
        <p className="muted">
          Actual from certificates {actual}. Variance {variance}
        </p>
      )}
    </>
  );
}

function ContractPanel({ projectCode, onError }: { projectCode: string; onError: (message: string) => void }) {
  const [contractId, setContractId] = useState<number | null>(null);
  const [value, setValue] = useState("");
  const [netPayable, setNetPayable] = useState("");
  const [thisBill, setThisBill] = useState("");
  const [variationId, setVariationId] = useState<number | null>(null);
  const [revisedSum, setRevisedSum] = useState("");

  return (
    <>
      <h2>Contract</h2>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void saveContractor(String(form.get("contractor") ?? ""))
            .then((id) => saveContract(projectCode, id, String(form.get("code") ?? ""), String(form.get("name") ?? "")))
            .then((contract) => {
              setContractId(contract.id);
              setValue(contract.value);
            })
            .catch((cause: unknown) => onError(text(cause)));
        }}
      >
        <label>Contractor<input name="contractor" /></label>
        <label>Contract code<input name="code" /></label>
        <label>Contract name<input name="name" /></label>
        <div className="actions">
          <button type="submit">Save contract</button>
        </div>
      </form>
      {contractId !== null && (
        <>
          <form
            onSubmit={(event) => {
              event.preventDefault();
              const form = new FormData(event.currentTarget);
              void saveWorkOrder(contractId, String(form.get("woCode") ?? ""), String(form.get("woName") ?? ""))
                .catch((cause: unknown) => onError(text(cause)));
            }}
          >
            <label>Work order code<input name="woCode" /></label>
            <label>Work order name<input name="woName" /></label>
            <div className="actions">
              <button type="submit">Save work order</button>
            </div>
          </form>
          <form
            onSubmit={(event) => {
              event.preventDefault();
              const form = new FormData(event.currentTarget);
              void addContractItem(
                contractId,
                String(form.get("description") ?? ""),
                String(form.get("quantity") ?? ""),
                String(form.get("rate") ?? ""),
              )
                .then((contract) => setValue(contract.value))
                .catch((cause: unknown) => onError(text(cause)));
            }}
          >
            <label>Bill description<input name="description" /></label>
            <label>Quantity<input name="quantity" /></label>
            <label>Rate<input name="rate" /></label>
            <div className="actions">
              <button type="submit">Add contract item</button>
            </div>
          </form>
          <p className="muted">Contract value {value}</p>
          <form
            onSubmit={(event) => {
              event.preventDefault();
              const form = new FormData(event.currentTarget);
              const kind = String(form.get("kind") ?? "add");
              if (kind !== "add" && kind !== "omit" && kind !== "substitute") return;
              void saveVariation(contractId, kind, String(form.get("variation") ?? ""), String(form.get("variationAmount") ?? ""))
                .then((variation) => {
                  setVariationId(variation.id);
                  setRevisedSum(variation.revisedSum);
                })
                .catch((cause: unknown) => onError(text(cause)));
            }}
          >
            <label>
              Variation
              <select name="kind" defaultValue="add">
                <option value="add">Add</option>
                <option value="omit">Omit</option>
                <option value="substitute">Substitute</option>
              </select>
            </label>
            <label>Description<input name="variation" /></label>
            <label>Amount<input name="variationAmount" /></label>
            <div className="actions">
              <button type="submit">Save variation</button>
              <button
                type="button"
                disabled={variationId === null}
                onClick={() => {
                  if (variationId === null) return;
                  void approveVariation(variationId)
                    .then((variation) => setRevisedSum(variation.revisedSum))
                    .catch((cause: unknown) => onError(text(cause)));
                }}
              >
                Approve variation
              </button>
            </div>
          </form>
          {revisedSum !== "" && <p className="muted">Revised contract sum {revisedSum}</p>}
          <form
            onSubmit={(event) => {
              event.preventDefault();
              const form = new FormData(event.currentTarget);
              const certificateNo = Number(form.get("certificateNo") ?? "");
              void saveCertificate(
                contractId,
                certificateNo,
                String(form.get("previous") ?? ""),
                String(form.get("workToDate") ?? ""),
                String(form.get("retentionPercent") ?? ""),
                String(form.get("advanceRecovery") ?? ""),
                String(form.get("deductions") ?? ""),
              )
                .then((certificate) => {
                  setThisBill(certificate.thisBill);
                  setNetPayable(certificate.netPayable);
                })
                .catch((cause: unknown) => onError(text(cause)));
            }}
          >
            <label>Certificate no<input name="certificateNo" /></label>
            <label>Previous<input name="previous" /></label>
            <label>Work to date<input name="workToDate" /></label>
            <label>Retention %<input name="retentionPercent" /></label>
            <label>Advance recovery<input name="advanceRecovery" /></label>
            <label>Deductions<input name="deductions" /></label>
            <div className="actions">
              <button type="submit">Save certificate</button>
            </div>
          </form>
          {netPayable !== "" && (
            <p className="muted">
              This bill {thisBill}. Net payable {netPayable}
            </p>
          )}
        </>
      )}
    </>
  );
}

function CodePanel({ projectCode, onError }: { projectCode: string; onError: (message: string) => void }) {
  const [kind, setKind] = useState("unit");
  const [code, setCode] = useState("");
  const [name, setName] = useState("");
  const [catalog, setCatalog] = useState<CodeEntry[]>([]);
  const [wbs, setWbs] = useState<CodeEntry[]>([]);
  const [cbs, setCbs] = useState<CodeEntry[]>([]);
  const [costs, setCosts] = useState<CodeEntry[]>([]);
  const [units, setUnits] = useState<CodeEntry[]>([]);
  const [items, setItems] = useState<WorkItem[]>([]);
  const [estimateTotal, setEstimateTotal] = useState("");
  const [measures, setMeasures] = useState<MeasureLine[]>([]);
  const [measureItem, setMeasureItem] = useState("");
  const [measureName, setMeasureName] = useState("");
  const [times, setTimes] = useState("");
  const [length, setLength] = useState("");
  const [width, setWidth] = useState("");
  const [height, setHeight] = useState("");
  const [itemName, setItemName] = useState("");
  const [quantity, setQuantity] = useState("");
  const [rate, setRate] = useState("");
  const [versionNo, setVersionNo] = useState("1");
  const [parentId, setParentId] = useState("");
  const [wbsId, setWbsId] = useState("");
  const [cbsId, setCbsId] = useState("");
  const [costId, setCostId] = useState("");
  const [unitId, setUnitId] = useState("");
  const [comparison, setComparison] = useState<Reconciliation | null>(null);

  async function refresh(nextKind = kind) {
    const [shown, wbsRows, cbsRows, costRows, unitRows, itemRows] = await Promise.all([
      listCodes(nextKind),
      listCodes("wbs"),
      listCodes("cbs"),
      listCodes("cost"),
      listCodes("unit"),
      listItems(projectCode),
    ]);
    setCatalog(shown);
    setWbs(wbsRows);
    setCbs(cbsRows);
    setCosts(costRows);
    setUnits(unitRows);
    setItems(itemRows);
  }

  useEffect(() => {
    void refresh().catch((cause: unknown) => onError(text(cause)));
  }, [projectCode]);

  return (
    <>
      <h2>Codes</h2>
      <p className="muted">WBS, CBS, cost codes, and units are shared. Two items can use the same code.</p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void saveCode(kind, code, name)
            .then(() => refresh())
            .then(() => {
              setCode("");
              setName("");
            })
            .catch((cause: unknown) => onError(text(cause)));
        }}
      >
        <label>
          Kind
          <select value={kind} onChange={(event) => {
            const next = event.target.value;
            setKind(next);
            void listCodes(next).then(setCatalog).catch((cause: unknown) => onError(text(cause)));
          }}>
            <option value="wbs">WBS</option>
            <option value="cbs">CBS</option>
            <option value="cost">Cost code</option>
            <option value="unit">Unit</option>
          </select>
        </label>
        <label>
          Code
          <input value={code} onChange={(event) => setCode(event.target.value)} />
        </label>
        <label>
          Name
          <input value={name} onChange={(event) => setName(event.target.value)} />
        </label>
        <div className="actions">
          <button type="submit">Save code</button>
        </div>
      </form>
      <ul className="users">
        {catalog.map((entry) => (
          <li key={entry.id}>{entry.code} · {entry.name}</li>
        ))}
      </ul>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void saveItem(
            projectCode,
            parentId === "" ? null : Number(parentId),
            Number(versionNo),
            itemName,
            quantity,
            rate,
            Number(wbsId),
            Number(cbsId),
            Number(costId),
            Number(unitId),
          )
            .then(() => listItems(projectCode))
            .then((next) => {
              setItems(next);
              setItemName("");
            })
            .catch((cause: unknown) => onError(text(cause)));
        }}
      >
        <label>
          Item
          <input value={itemName} onChange={(event) => setItemName(event.target.value)} />
        </label>
        <label>
          Parent
          <select value={parentId} onChange={(event) => setParentId(event.target.value)}>
            <option value="">No parent</option>
            {items.filter((item) => String(item.versionNo) === versionNo).map((item) => (
              <option key={item.id} value={item.id}>v{item.versionNo} {item.name}</option>
            ))}
          </select>
        </label>
        <label>
          Version
          <input value={versionNo} onChange={(event) => setVersionNo(event.target.value)} />
        </label>
        <label>
          Quantity
          <input value={quantity} onChange={(event) => setQuantity(event.target.value)} />
        </label>
        <label>
          Rate
          <input value={rate} onChange={(event) => setRate(event.target.value)} />
        </label>
        <label>
          WBS
          <select value={wbsId} onChange={(event) => setWbsId(event.target.value)}>
            <option value="">Choose</option>
            {wbs.map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
          </select>
        </label>
        <label>
          CBS
          <select value={cbsId} onChange={(event) => setCbsId(event.target.value)}>
            <option value="">Choose</option>
            {cbs.map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
          </select>
        </label>
        <label>
          Cost code
          <select value={costId} onChange={(event) => setCostId(event.target.value)}>
            <option value="">Choose</option>
            {costs.map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
          </select>
        </label>
        <label>
          Unit
          <select value={unitId} onChange={(event) => setUnitId(event.target.value)}>
            <option value="">Choose</option>
            {units.map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
          </select>
        </label>
        <div className="actions">
          <button type="submit">Add item</button>
        </div>
      </form>
      <ul className="users">
        {items.map((item) => (
          <li key={item.id}>v{item.versionNo} {item.name} · qty {item.quantity || "—"} × {item.rate || "—"} = {item.amount} · {item.wbsCode} · {item.unitCode}</li>
        ))}
      </ul>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void reconcileBoq(projectCode, Number(form.get("versionA") ?? "1"), Number(form.get("versionB") ?? "2"))
            .then(setComparison)
            .catch((cause: unknown) => onError(text(cause)));
        }}
      >
        <label>Version A<input name="versionA" defaultValue="1" /></label>
        <label>Version B<input name="versionB" defaultValue="2" /></label>
        <div className="actions">
          <button type="submit">Compare versions</button>
          {comparison !== null && !comparison.approved && (
            <button
              type="button"
              onClick={(click) => {
                const form = click.currentTarget.form;
                if (form === null) return;
                const versionB = Number(new FormData(form).get("versionB") ?? "2");
                const versionA = Number(new FormData(form).get("versionA") ?? "1");
                void approveBoq(projectCode, versionB)
                  .then(() => reconcileBoq(projectCode, versionA, versionB))
                  .then(setComparison)
                  .catch((cause: unknown) => onError(text(cause)));
              }}
            >
              Approve version B
            </button>
          )}
        </div>
      </form>
      {comparison !== null && (
        <>
          <p className="muted">
            Version A {comparison.totalA}. Version B {comparison.totalB}. Difference {comparison.difference}.
            {comparison.approved ? " Version B is approved." : ""}
          </p>
          <ul className="users">
            {comparison.lines.map((line) => (
              <li key={line.name}>{line.name} {line.amountA} to {line.amountB} ({line.difference})</li>
            ))}
          </ul>
        </>
      )}
      {items.length > 0 && (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            const itemId = Number(measureItem || items[0].id);
            void addMeasure(itemId, measureName, times, length, width, height)
              .then(() => Promise.all([listMeasures(itemId), listItems(projectCode)]))
              .then(([lines, nextItems]) => {
                setMeasures(lines);
                setItems(nextItems);
                setMeasureName("");
                setTimes("");
                setLength("");
                setWidth("");
                setHeight("");
              })
              .catch((cause: unknown) => onError(text(cause)));
          }}
        >
          <h2>Measurement</h2>
          <label>
            Item
            <select value={measureItem} onChange={(event) => {
              const next = event.target.value;
              setMeasureItem(next);
              void listMeasures(Number(next)).then(setMeasures).catch((cause: unknown) => onError(text(cause)));
            }}>
              {items.map((item) => (
                <option key={item.id} value={item.id}>{item.name}</option>
              ))}
            </select>
          </label>
          <label>
            Description
            <input value={measureName} onChange={(event) => setMeasureName(event.target.value)} />
          </label>
          <label>
            Times
            <input value={times} onChange={(event) => setTimes(event.target.value)} />
          </label>
          <label>
            Length
            <input value={length} onChange={(event) => setLength(event.target.value)} />
          </label>
          <label>
            Width
            <input value={width} onChange={(event) => setWidth(event.target.value)} />
          </label>
          <label>
            Height
            <input value={height} onChange={(event) => setHeight(event.target.value)} />
          </label>
          <div className="actions">
            <button type="submit">Add dimension</button>
          </div>
        </form>
      )}
      <ul className="users">
        {measures.map((line) => (
          <li key={line.id}>{line.description} · {line.quantity}</li>
        ))}
      </ul>
      {items.length > 0 && (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            const form = new FormData(event.currentTarget);
            const field = (key: string) => String(form.get(key) ?? "");
            void saveRate(Number(measureItem || items[0].id), {
              materialQty: field("materialQty"),
              materialRate: field("materialRate"),
              labourQty: field("labourQty"),
              labourRate: field("labourRate"),
              plantQty: field("plantQty"),
              plantRate: field("plantRate"),
              wastagePercent: field("wastagePercent"),
              overheadPercent: field("overheadPercent"),
              profitPercent: field("profitPercent"),
            })
              .then(() => listItems(projectCode))
              .then(setItems)
              .catch((cause: unknown) => onError(text(cause)));
          }}
        >
          <h2>Rate analysis</h2>
          <p className="muted">Wastage applies to material. Overhead applies to the direct cost. Profit applies after overhead.</p>
          <label>Material qty<input name="materialQty" defaultValue="0" /></label>
          <label>Material rate<input name="materialRate" defaultValue="0" /></label>
          <label>Labour qty<input name="labourQty" defaultValue="0" /></label>
          <label>Labour rate<input name="labourRate" defaultValue="0" /></label>
          <label>Plant qty<input name="plantQty" defaultValue="0" /></label>
          <label>Plant rate<input name="plantRate" defaultValue="0" /></label>
          <label>Wastage %<input name="wastagePercent" defaultValue="0" /></label>
          <label>Overhead %<input name="overheadPercent" defaultValue="0" /></label>
          <label>Profit %<input name="profitPercent" defaultValue="0" /></label>
          <div className="actions">
            <button type="submit">Save rate</button>
          </div>
        </form>
      )}
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          const kind = String(form.get("kind") ?? "boq");
          void saveEstimate(
            projectCode,
            kind === "area" ? "area" : "boq",
            Number(versionNo || "1"),
            String(form.get("area") ?? ""),
            String(form.get("ratePerArea") ?? ""),
          )
            .then((estimate) => setEstimateTotal(`${estimate.kind} total ${estimate.total}`))
            .catch((cause: unknown) => onError(text(cause)));
        }}
      >
        <h2>Estimate</h2>
        <p className="muted">A BOQ estimate uses the top-level bill amounts for the version. An area estimate is area times rate.</p>
        <label>
          Source
          <select name="kind" defaultValue="boq">
            <option value="boq">From the bill</option>
            <option value="area">Area rate</option>
          </select>
        </label>
        <label>Area<input name="area" /></label>
        <label>Rate per area<input name="ratePerArea" /></label>
        <div className="actions">
          <button type="submit">Save estimate</button>
        </div>
        {estimateTotal && <p className="muted">{estimateTotal}</p>}
      </form>
    </>
  );
}

function AccountForm({
  title,
  detail,
  submitLabel,
  named,
  error,
  onSubmit,
}: {
  title: string;
  detail: string;
  submitLabel: string;
  named?: boolean;
  error: string;
  onSubmit: (username: string, password: string, displayName?: string) => Promise<void>;
}) {
  const [username, setUsername] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
        void onSubmit(username, password, displayName);
      }}
    >
      <h1>{title}</h1>
      <p className="muted">{detail}</p>
      {named && (
        <label>
          Name
          <input value={displayName} onChange={(event) => setDisplayName(event.target.value)} />
        </label>
      )}
      <label>
        Username
        <input value={username} onChange={(event) => setUsername(event.target.value)} autoComplete="username" />
      </label>
      <label>
        Password
        <input
          type="password"
          value={password}
          onChange={(event) => setPassword(event.target.value)}
          autoComplete={named ? "new-password" : "current-password"}
        />
      </label>
      {error && <p className="error">{error}</p>}
      <div className="actions">
        <button type="submit">{submitLabel}</button>
      </div>
    </form>
  );
}

function RoleForm({
  error,
  onSubmit,
}: {
  error: string;
  onSubmit: (name: string, permissions: string[], modules: string[], projects: string[]) => Promise<void>;
}) {
  const [name, setName] = useState("");
  const [permissions, setPermissions] = useState<string[]>(["view"]);
  const [modules, setModules] = useState<string[]>([]);
  const [projects, setProjects] = useState("");

  function toggle(list: string[], value: string, setList: (next: string[]) => void) {
    setList(list.includes(value) ? list.filter((item) => item !== value) : [...list, value]);
  }

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
        void onSubmit(
          name,
          permissions,
          modules,
          projects.split(",").map((item) => item.trim()).filter(Boolean),
        );
      }}
    >
      <h1>Add a role</h1>
      <p className="muted">A user can act only where the role allows the permission, the module, and the project.</p>
      <label>
        Role name
        <input value={name} onChange={(event) => setName(event.target.value)} />
      </label>
      <fieldset>
        <legend>Permissions</legend>
        {PERMISSIONS.map((permission) => (
          <label key={permission}>
            <input
              type="checkbox"
              checked={permissions.includes(permission)}
              onChange={() => toggle(permissions, permission, setPermissions)}
            />
            {permission}
          </label>
        ))}
      </fieldset>
      <fieldset>
        <legend>Modules</legend>
        {MODULES.map((moduleName) => (
          <label key={moduleName}>
            <input
              type="checkbox"
              checked={modules.includes(moduleName)}
              onChange={() => toggle(modules, moduleName, setModules)}
            />
            {moduleName}
          </label>
        ))}
      </fieldset>
      <label>
        Project codes
        <input value={projects} onChange={(event) => setProjects(event.target.value)} placeholder="Tower A, Tower B" />
      </label>
      {error && <p className="error">{error}</p>}
      <div className="actions">
        <button type="submit">Save role</button>
      </div>
    </form>
  );
}

function text(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}
