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
  certifyBill,
  closeAccount,
  comparativeStatement,
  currentFirm,
  assignRole,
  loadSheet,
  listAudit,
  listCodes,
  listItems,
  itemLedger,
  listMeasures,
  addMeasure,
  addMaterialMove,
  addQuoteLine,
  listLocations,
  listProjects,
  saveCertificate,
  saveCommitment,
  saveCode,
  saveContract,
  saveContractor,
  saveArea,
  saveBudget,
  saveEstimate,
  saveForecast,
  reviseItemQuantity,
  saveItem,
  saveLibraryRate,
  saveLocationCost,
  saveMaterial,
  saveRate,
  approveVariation,
  applyLibraryRate,
  approveBill,
  approveBoq,
  addContractItem,
  linkContractItem,
  exportLocalWorkbook,
  exportReport,
  exportCore,
  addLocation,
  saveProject,
  pickFolder,
  pickXlsxPath,
  placeOrder,
  projectKpi,
  projectReport,
  projectTotals,
  recommendBill,
  reconcileBoq,
  readXlsx,
  rejectBill,
  restoreBackup,
  setIntegration,
  setLocalServer,
  writeXlsx,
  locationRollup,
  login,
  logout,
  MODULES,
  openFirm,
  PERMISSIONS,
  pickFirmPath,
  saveRole,
  saveVariation,
  saveWorkOrder,
  saveSaleRate,
  saveScenario,
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
  type QuantityLedger,
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
  const [locationKind, setLocationKind] = useState("tower");
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
      <p className="muted">Each project can use its own location labels and depth. A location can be a development, building, tower, floor, unit, zone, or a custom label.</p>
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
                {node.kind === "custom" ? node.label : node.kind}: {node.name}
                {node.kind !== "custom" && node.label.toLowerCase() !== node.kind ? ` · ${node.label}` : ""}
              </li>
            ))}
          </ul>
          <form
            onSubmit={(event) => {
              event.preventDefault();
              void addLocation(selected, parentId === "" ? null : Number(parentId), locationKind, label, locationName)
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
              Kind
              <select value={locationKind} onChange={(event) => setLocationKind(event.target.value)}>
                <option value="development">Development</option>
                <option value="building">Building</option>
                <option value="tower">Tower</option>
                <option value="floor">Floor</option>
                <option value="unit">Unit</option>
                <option value="zone">Zone</option>
                <option value="custom">Custom</option>
              </select>
            </label>
            <label>
              Label
              <input value={label} onChange={(event) => setLabel(event.target.value)} placeholder="Shown name of this level" />
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
          <IntermediatePanel projectCode={selected} onError={onError} />
          <AdvancedPanel projectCode={selected} onError={onError} />
        </>
      )}
    </>
  );
}

function AdvancedPanel({ projectCode, onError }: { projectCode: string; onError: (message: string) => void }) {
  const [note, setNote] = useState("");
  const fail = (cause: unknown) => onError(text(cause));

  return (
    <>
      <h2>Advanced</h2>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          const kind = String(form.get("kind") ?? "saleable");
          if (kind !== "saleable" && kind !== "common") return;
          void saveArea(projectCode, String(form.get("name") ?? ""), kind, String(form.get("area") ?? ""))
            .then((buildup) => setNote(`Saleable ${buildup.saleable}. GFA ${buildup.gfa}. Schedule ${buildup.schedule}`))
            .catch(fail);
        }}
      >
        <label>Area name<input name="name" /></label>
        <label>
          Kind
          <select name="kind" defaultValue="saleable">
            <option value="saleable">Saleable</option>
            <option value="common">Common</option>
          </select>
        </label>
        <label>Area<input name="area" /></label>
        <div className="actions"><button type="submit">Add area</button></div>
      </form>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void saveSaleRate(projectCode, String(form.get("rate") ?? ""))
            .then((buildup) => setNote(`Cost per area ${buildup.costPerSqft}. Margin ${buildup.margin}`))
            .catch(fail);
        }}
      >
        <label>Sale rate per area<input name="rate" /></label>
        <div className="actions"><button type="submit">Save sale rate</button></div>
      </form>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void saveScenario(projectCode, String(form.get("name") ?? ""), String(form.get("budget") ?? ""))
            .then((scenario) => setNote(`Scenario ${scenario.scenarioBudget}. Live budget ${scenario.liveBudget}`))
            .catch(fail);
        }}
      >
        <label>Scenario<input name="name" /></label>
        <label>Scenario budget<input name="budget" /></label>
        <div className="actions"><button type="submit">Save scenario</button></div>
      </form>
      <div className="actions">
        <button type="button" onClick={() => void closeAccount(projectCode).then((account) => setNote(`Final account ${account.amount} from certificate ${account.certificateNo}`)).catch(fail)}>Close final account</button>
        <button type="button" onClick={() => void projectTotals().then((rows) => setNote(rows.map((row) => `${row.code} ${row.total}`).join(". "))).catch(fail)}>Compare projects</button>
        <button type="button" onClick={() => void setIntegration("accounts", false).then((off) => setNote(off ? "Integrations are off. Core records stay available." : "An integration is enabled.")).catch(fail)}>Keep integrations off</button>
        <button
          type="button"
          onClick={() => {
            void pickXlsxPath("save").then((path) => {
              if (!path) return;
              void exportLocalWorkbook(projectCode, path).then(() => setNote("Workbook saved on this PC.")).catch(fail);
            });
          }}
        >
          Export workbook
        </button>
      </div>
      {note !== "" && <p className="muted">{note}</p>}
    </>
  );
}

function IntermediatePanel({ projectCode, onError }: { projectCode: string; onError: (message: string) => void }) {
  const [note, setNote] = useState("");
  const show = (text: string) => setNote(text);
  const fail = (cause: unknown) => onError(text(cause));

  return (
    <>
      <h2>Intermediate</h2>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void saveCommitment(projectCode, String(form.get("description") ?? ""), String(form.get("order") ?? ""), String(form.get("already") ?? ""))
            .then((exposure) => show(`Open commitment ${exposure.openCommitment}. Actual plus commitment ${exposure.total}`))
            .catch(fail);
        }}
      >
        <label>Commitment<input name="description" /></label>
        <label>Order amount<input name="order" /></label>
        <label>Already certified<input name="already" /></label>
        <div className="actions"><button type="submit">Save commitment</button></div>
      </form>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void addQuoteLine(projectCode, String(form.get("vendor") ?? ""), String(form.get("description") ?? ""), String(form.get("amount") ?? ""))
            .then(() => comparativeStatement(projectCode))
            .then((statement) => show(`Selected ${statement.selectedVendor} at ${statement.selectedTotal}`))
            .catch(fail);
        }}
      >
        <label>Vendor<input name="vendor" /></label>
        <label>Quote line<input name="description" /></label>
        <label>Amount<input name="amount" /></label>
        <div className="actions">
          <button type="submit">Add quotation</button>
          <button type="button" onClick={() => void placeOrder(projectCode).then((order) => show(`Order placed with ${order.selectedVendor} for ${order.selectedTotal}`)).catch(fail)}>Place order</button>
        </div>
      </form>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void saveMaterial(projectCode, String(form.get("name") ?? ""), String(form.get("theoretical") ?? ""))
            .then((materialId) => addMaterialMove(materialId, String(form.get("kind") ?? "issue"), String(form.get("quantity") ?? "")))
            .then((check) => show(`Theoretical ${check.theoretical}. Actual ${check.actual}. Wastage ${check.wastage}`))
            .catch(fail);
        }}
      >
        <label>Material<input name="name" /></label>
        <label>Theoretical<input name="theoretical" /></label>
        <label>
          Movement
          <select name="kind" defaultValue="issue">
            <option value="receipt">Receipt</option>
            <option value="issue">Issue</option>
          </select>
        </label>
        <label>Quantity<input name="quantity" /></label>
        <div className="actions"><button type="submit">Record material</button></div>
      </form>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void saveForecast(projectCode, String(form.get("remaining") ?? ""), String(form.get("cashFlow") ?? ""))
            .then((forecast) => show(`Estimate at completion ${forecast.eac}. Cash flow ${forecast.cashFlow}`))
            .catch(fail);
        }}
      >
        <label>Cost to complete<input name="remaining" /></label>
        <label>Cash flow<input name="cashFlow" /></label>
        <div className="actions"><button type="submit">Save forecast</button></div>
      </form>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          void saveLocationCost(Number(form.get("locationId") ?? ""), String(form.get("amount") ?? ""))
            .then(() => locationRollup(projectCode))
            .then((rollup) => show(`Locations ${rollup.allocated}. Project ${rollup.projectTotal}`))
            .catch(fail);
        }}
      >
        <label>Location id<input name="locationId" /></label>
        <label>Location cost<input name="amount" /></label>
        <div className="actions"><button type="submit">Save location cost</button></div>
      </form>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          const certificateId = Number(form.get("certificateId") ?? "");
          const action = String(form.get("action") ?? "recommend");
          const call = action === "approve" ? approveBill(certificateId) : action === "reject" ? rejectBill(certificateId) : action === "certify" ? certifyBill(certificateId) : recommendBill(certificateId);
          void call.then(() => show(`Bill ${certificateId} ${action}`)).catch(fail);
        }}
      >
        <label>Certificate id<input name="certificateId" /></label>
        <label>
          Action
          <select name="action" defaultValue="recommend">
            <option value="recommend">Recommend</option>
            <option value="approve">Approve</option>
            <option value="reject">Reject</option>
            <option value="certify">Certify</option>
          </select>
        </label>
        <div className="actions"><button type="submit">Update bill</button></div>
      </form>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const form = new FormData(event.currentTarget);
          const kind = String(form.get("kind") ?? "historical");
          void saveLibraryRate(kind, String(form.get("code") ?? ""), String(form.get("rate") ?? ""))
            .then((libraryId) => applyLibraryRate(Number(form.get("itemId") ?? ""), libraryId))
            .then((amount) => show(`Applied rate. Item amount ${amount}`))
            .catch(fail);
        }}
      >
        <label>Item id<input name="itemId" /></label>
        <label>
          Rate library
          <select name="kind" defaultValue="historical">
            <option value="historical">Historical</option>
            <option value="market">Market</option>
          </select>
        </label>
        <label>Code<input name="code" /></label>
        <label>Rate<input name="rate" /></label>
        <div className="actions"><button type="submit">Apply rate</button></div>
      </form>
      <div className="actions">
        <button
          type="button"
          onClick={() => {
            void projectKpi(projectCode)
              .then((kpi) => show(`Budget ${kpi.budget}. Actual ${kpi.actual}. Commitment ${kpi.commitment}. Forecast ${kpi.forecast}. Paid ${kpi.paid}`))
              .catch(fail);
          }}
        >
          Show dashboard
        </button>
      </div>
      {note !== "" && <p className="muted">{note}</p>}
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
        <button
          type="button"
          onClick={() => {
            void pickXlsxPath("save").then((path) => {
              if (!path) return;
              void exportCore(projectCode, path).catch((cause: unknown) => onError(text(cause)));
            });
          }}
        >
          Export core
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
  const [billItems, setBillItems] = useState<WorkItem[]>([]);
  const [billItemId, setBillItemId] = useState("");

  useEffect(() => {
    void listItems(projectCode).then(setBillItems).catch((cause: unknown) => onError(text(cause)));
  }, [projectCode]);

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
              if (billItemId === "") return;
              void linkContractItem(contractId, Number(billItemId))
                .then((contract) => setValue(contract.value))
                .catch((cause: unknown) => onError(text(cause)));
            }}
          >
            <label>
              Bill item
              <select value={billItemId} onChange={(event) => setBillItemId(event.target.value)}>
                <option value="">Choose</option>
                {billItems.map((item) => (
                  <option key={item.id} value={item.id}>
                    {item.name} · {item.quantity || "—"} {item.unitCode}
                  </option>
                ))}
              </select>
            </label>
            <div className="actions">
              <button type="submit">Add the bill item</button>
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
  const [works, setWorks] = useState<CodeEntry[]>([]);
  const [disciplines, setDisciplines] = useState<CodeEntry[]>([]);
  const [packages, setPackages] = useState<CodeEntry[]>([]);
  const [items, setItems] = useState<WorkItem[]>([]);
  const [ledger, setLedger] = useState<QuantityLedger | null>(null);
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
  const [workId, setWorkId] = useState("");
  const [disciplineId, setDisciplineId] = useState("");
  const [packageId, setPackageId] = useState("");
  const [comparison, setComparison] = useState<Reconciliation | null>(null);

  async function refresh(nextKind = kind) {
    const [shown, wbsRows, cbsRows, costRows, unitRows, workRows, disciplineRows, packageRows, itemRows] = await Promise.all([
      listCodes(nextKind),
      listCodes("wbs"),
      listCodes("cbs"),
      listCodes("cost"),
      listCodes("unit"),
      listCodes("work"),
      listCodes("discipline"),
      listCodes("package"),
      listItems(projectCode),
    ]);
    setCatalog(shown);
    setWbs(wbsRows);
    setCbs(cbsRows);
    setCosts(costRows);
    setUnits(unitRows);
    setWorks(workRows);
    setDisciplines(disciplineRows);
    setPackages(packageRows);
    setItems(itemRows);
  }

  useEffect(() => {
    void refresh().catch((cause: unknown) => onError(text(cause)));
  }, [projectCode]);

  return (
    <>
      <h2>Codes</h2>
      <p className="muted">WBS, CBS, cost, unit, work, discipline, and package codes are shared. Two items can use the same code.</p>
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
            <option value="work">Work code</option>
            <option value="discipline">Discipline</option>
            <option value="package">Work package</option>
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
            workId === "" ? null : Number(workId),
            disciplineId === "" ? null : Number(disciplineId),
            packageId === "" ? null : Number(packageId),
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
        <label>
          Work code
          <select value={workId} onChange={(event) => setWorkId(event.target.value)}>
            <option value="">None</option>
            {works.map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
          </select>
        </label>
        <label>
          Discipline
          <select value={disciplineId} onChange={(event) => setDisciplineId(event.target.value)}>
            <option value="">None</option>
            {disciplines.map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
          </select>
        </label>
        <label>
          Work package
          <select value={packageId} onChange={(event) => setPackageId(event.target.value)}>
            <option value="">None</option>
            {packages.map((entry) => <option key={entry.id} value={entry.id}>{entry.code}</option>)}
          </select>
        </label>
        <div className="actions">
          <button type="submit">Add item</button>
        </div>
      </form>
      <ul className="users">
        {items.map((item) => (
          <li key={item.id}>
            v{item.versionNo} {item.name} · qty {item.quantity || "—"} × {item.rate || "—"} = {item.amount} · {item.wbsCode} · {item.unitCode}{item.packageCode ? ` · ${item.packageCode}` : ""}
            {" "}
            <button
              type="button"
              onClick={() => {
                void itemLedger(item.id)
                  .then(setLedger)
                  .catch((cause: unknown) => onError(text(cause)));
              }}
            >
              Ledger
            </button>
            <form
              key={`${item.id}:${item.quantity}`}
              className="actions"
              onSubmit={(event) => {
                event.preventDefault();
                const form = new FormData(event.currentTarget);
                void reviseItemQuantity(item.id, String(form.get("quantity") ?? ""))
                  .then(() => refresh())
                  .catch((cause: unknown) => onError(text(cause)));
              }}
            >
              <input
                name="quantity"
                aria-label={`Bill quantity for ${item.name}`}
                defaultValue={item.quantity}
              />
              <button type="submit">Save quantity</button>
            </form>
          </li>
        ))}
      </ul>
      {ledger !== null && (
        <>
          <h2>Quantity ledger</h2>
          <p className="muted">
            {items.find((item) => item.id === ledger.itemId)?.name ?? "Item"} · remaining is original minus the larger of certified and billed.
          </p>
          <ul className="users">
            <li>Original {ledger.originalQty}</li>
            <li>Revised {ledger.revisedQty}</li>
            <li>Planned {ledger.plannedQty}</li>
            <li>Contracted {ledger.contractQty}</li>
            <li>Executed {ledger.executedQty}</li>
            <li>Measured {ledger.measuredQty}</li>
            <li>Certified {ledger.certifiedQty}</li>
            <li>Billed {ledger.billedQty}</li>
            <li>Paid {ledger.paidQty}</li>
            <li>Forecast {ledger.forecastQty}</li>
            <li>Final {ledger.finalQty}</li>
            <li>Remaining {ledger.remainingQty}</li>
          </ul>
        </>
      )}
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
