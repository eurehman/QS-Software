export type FirmFile = {
  path: string;
  schemaVersion: number;
  appVersion: string;
};

function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(command, args);
}

export async function appVersion(): Promise<string> {
  if (!inTauri()) return "0.1.0";
  return invoke<string>("app_version");
}

export async function defaultFirmPath(): Promise<string> {
  if (!inTauri()) return "Documents/QS/company.qsdb";
  return invoke<string>("default_firm_path");
}

export async function currentFirm(): Promise<FirmFile | null> {
  if (!inTauri()) return null;
  return invoke<FirmFile | null>("current_firm");
}

export async function createFirm(path: string): Promise<FirmFile> {
  return invoke<FirmFile>("create_firm", { path });
}

export async function openFirm(path: string): Promise<FirmFile> {
  return invoke<FirmFile>("open_firm", { path });
}

export type Session = {
  username: string;
  displayName: string;
  isAdministrator: boolean;
  roleName: string | null;
};

export type Role = {
  id: number;
  name: string;
  permissions: string[];
  modules: string[];
  projects: string[];
};

export type AccountStatus = {
  needsAdministrator: boolean;
  session: Session | null;
  users: Session[];
  roles: Role[];
};

export const PERMISSIONS = [
  "view", "create", "edit", "delete", "import", "export", "approve", "reject", "recommend",
  "print", "backup", "restore", "configure",
];

export const MODULES = [
  "project", "boq", "measure", "rates", "contract", "ipc", "variation", "cost", "reports", "security",
];

export async function saveRole(
  name: string,
  permissions: string[],
  modules: string[],
  projects: string[],
): Promise<void> {
  await invoke("save_role", { name, permissions, modules, projects });
}

export type StoredCell = { row: number; col: number; raw: string };

export async function loadSheet(): Promise<StoredCell[]> {
  return invoke<StoredCell[]>("load_sheet");
}

export async function saveSheet(cells: StoredCell[]): Promise<void> {
  await invoke("save_sheet", { cells });
}

export async function readXlsx(path: string): Promise<string[][]> {
  return invoke<string[][]>("read_xlsx", { path });
}

export async function writeXlsx(path: string, rows: string[][]): Promise<void> {
  await invoke("write_xlsx", { path, rows });
}

export type BackupRecord = {
  id: number;
  createdAt: string;
  folder: string;
  createdBy: string;
  verified: boolean;
};

export type BackupStatus = {
  folder: string;
  history: BackupRecord[];
};

export async function backupStatus(): Promise<BackupStatus> {
  return invoke<BackupStatus>("backup_status");
}

export async function setLocalServer(folder: string): Promise<void> {
  await invoke("set_local_server", { folder });
}

export async function createBackup(): Promise<BackupRecord> {
  return invoke<BackupRecord>("create_backup");
}

export async function restoreBackup(folder: string): Promise<void> {
  await invoke("restore_backup", { folder });
}

export type AuditEvent = {
  id: number;
  createdAt: string;
  username: string;
  action: string;
  target: string;
  oldValue: string;
  newValue: string;
};

export async function listAudit(): Promise<AuditEvent[]> {
  return invoke<AuditEvent[]>("list_audit");
}

export type Project = { id: number; code: string; name: string };
export type LocationNode = { id: number; parentId: number | null; label: string; name: string };

export async function listProjects(): Promise<Project[]> {
  return invoke<Project[]>("list_projects");
}

export async function saveProject(code: string, name: string): Promise<Project> {
  return invoke<Project>("save_project", { code, name });
}

export async function listLocations(projectCode: string): Promise<LocationNode[]> {
  return invoke<LocationNode[]>("list_locations", { projectCode });
}

export type CodeEntry = { id: number; kind: string; code: string; name: string };
export type WorkItem = {
  id: number;
  parentId: number | null;
  versionNo: number;
  name: string;
  quantity: string;
  rate: string;
  amount: string;
  wbsCode: string;
  cbsCode: string;
  costCode: string;
  unitCode: string;
};

export async function listCodes(kind: string): Promise<CodeEntry[]> {
  return invoke<CodeEntry[]>("list_codes", { kind });
}

export async function saveCode(kind: string, code: string, name: string): Promise<CodeEntry> {
  return invoke<CodeEntry>("save_code", { kind, code, name });
}

export async function listItems(projectCode: string): Promise<WorkItem[]> {
  return invoke<WorkItem[]>("list_items", { projectCode });
}

export async function listMeasures(itemId: number): Promise<MeasureLine[]> {
  return invoke<MeasureLine[]>("list_measures", { itemId });
}

export async function addMeasure(
  itemId: number,
  description: string,
  times: string,
  length: string,
  width: string,
  height: string,
): Promise<MeasureLine> {
  return invoke<MeasureLine>("add_measure", { itemId, description, times, length, width, height });
}

export type MeasureLine = {
  id: number;
  itemId: number;
  description: string;
  times: string;
  length: string;
  width: string;
  height: string;
  quantity: string;
};

export async function saveRate(itemId: number, figures: {
  materialQty: string;
  materialRate: string;
  labourQty: string;
  labourRate: string;
  plantQty: string;
  plantRate: string;
  wastagePercent: string;
  overheadPercent: string;
  profitPercent: string;
}): Promise<RateBuildup> {
  return invoke<RateBuildup>("save_rate", { itemId, ...figures });
}

export type RateBuildup = {
  itemId: number;
  compositeRate: string;
};

export async function saveContractor(name: string): Promise<number> {
  return invoke<number>("save_contractor", { name });
}

export async function saveContract(
  projectCode: string,
  contractorId: number,
  code: string,
  name: string,
): Promise<{ id: number; value: string }> {
  return invoke("save_contract", { projectCode, contractorId, code, name });
}

export async function saveWorkOrder(contractId: number, code: string, name: string): Promise<void> {
  await invoke("save_work_order", { contractId, code, name });
}

export async function addContractItem(
  contractId: number,
  description: string,
  quantity: string,
  rate: string,
): Promise<{ id: number; value: string }> {
  return invoke("add_contract_item", { contractId, description, quantity, rate });
}

export type Certificate = {
  id: number;
  certificateNo: number;
  previous: string;
  thisBill: string;
  retention: string;
  advanceRecovery: string;
  deductions: string;
  netPayable: string;
};

export async function saveCertificate(
  contractId: number,
  certificateNo: number,
  previous: string,
  workToDate: string,
  retentionPercent: string,
  advanceRecovery: string,
  deductions: string,
): Promise<Certificate> {
  return invoke("save_certificate", {
    contractId,
    certificateNo,
    previous,
    workToDate,
    retentionPercent,
    advanceRecovery,
    deductions,
  });
}

export async function saveVariation(
  contractId: number,
  kind: "add" | "omit" | "substitute",
  description: string,
  amount: string,
): Promise<{ id: number; status: string; revisedSum: string }> {
  return invoke("save_variation", { contractId, kind, description, amount });
}

export async function approveVariation(variationId: number): Promise<{ id: number; status: string; revisedSum: string }> {
  return invoke("approve_variation", { variationId });
}

export type ReportLine = {
  section: string;
  label: string;
  figure: string;
};

export async function projectReport(projectCode: string): Promise<ReportLine[]> {
  return invoke("project_report", { projectCode });
}

export type Reconciliation = {
  totalA: string;
  totalB: string;
  difference: string;
  approved: boolean;
  lines: { name: string; amountA: string; amountB: string; difference: string }[];
};

export async function reconcileBoq(projectCode: string, versionA: number, versionB: number): Promise<Reconciliation> {
  return invoke("reconcile_boq", { projectCode, versionA, versionB });
}

export async function approveBoq(projectCode: string, versionNo: number): Promise<void> {
  await invoke("approve_boq", { projectCode, versionNo });
}

export async function exportReport(projectCode: string, path: string): Promise<ReportLine[]> {
  return invoke("export_report", { projectCode, path });
}

export async function saveBudget(
  projectCode: string,
  amount: string,
): Promise<{ budget: string; actual: string; variance: string }> {
  return invoke("save_budget", { projectCode, amount });
}

export async function saveEstimate(
  projectCode: string,
  kind: "boq" | "area",
  versionNo: number,
  area: string,
  rate: string,
): Promise<{ id: number; kind: string; versionNo: number; total: string }> {
  return invoke("save_estimate", { projectCode, kind, versionNo, area, rate });
}

export async function saveItem(
  projectCode: string,
  parentId: number | null,
  versionNo: number,
  name: string,
  quantity: string,
  rate: string,
  wbsId: number,
  cbsId: number,
  costId: number,
  unitId: number,
): Promise<WorkItem> {
  return invoke<WorkItem>("save_item", {
    projectCode,
    parentId,
    versionNo,
    name,
    quantity,
    rate,
    wbsId,
    cbsId,
    costId,
    unitId,
  });
}

export async function addLocation(
  projectCode: string,
  parentId: number | null,
  label: string,
  name: string,
): Promise<LocationNode> {
  return invoke<LocationNode>("add_location", { projectCode, parentId, label, name });
}

export async function pickFolder(): Promise<string | null> {
  if (!inTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}

export async function pickXlsxPath(mode: "open" | "save"): Promise<string | null> {
  if (!inTauri()) return null;
  const { open, save } = await import("@tauri-apps/plugin-dialog");
  const filters = [{ name: "Excel workbook", extensions: ["xlsx", "xls"] }];
  if (mode === "open") {
    const selected = await open({ filters, multiple: false });
    return typeof selected === "string" ? selected : null;
  }
  return save({ filters: [{ name: "Excel workbook", extensions: ["xlsx"] }], defaultPath: "Sheet.xlsx" });
}

export async function assignRole(username: string, roleName: string): Promise<void> {
  await invoke("assign_role", { username, roleName });
}

export async function accountStatus(): Promise<AccountStatus> {
  return invoke<AccountStatus>("account_status");
}

export async function createAdministrator(username: string, password: string): Promise<Session> {
  return invoke<Session>("create_administrator", { username, password });
}

export async function login(username: string, password: string): Promise<Session> {
  return invoke<Session>("login", { username, password });
}

export async function logout(): Promise<void> {
  await invoke("logout");
}

export async function createUser(username: string, displayName: string, password: string): Promise<void> {
  await invoke("create_user", { username, displayName, password });
}

export async function saveCommitment(
  projectCode: string,
  description: string,
  orderAmount: string,
  alreadyCertified: string,
): Promise<{ actual: string; openCommitment: string; total: string }> {
  return invoke("save_commitment", { projectCode, description, orderAmount, alreadyCertified });
}

export async function addQuoteLine(projectCode: string, vendor: string, description: string, amount: string): Promise<void> {
  await invoke("add_quote_line", { projectCode, vendor, description, amount });
}

export async function comparativeStatement(projectCode: string): Promise<{ selectedVendor: string; selectedTotal: string }> {
  return invoke("comparative_statement", { projectCode });
}

export async function placeOrder(projectCode: string): Promise<{ selectedVendor: string; selectedTotal: string }> {
  return invoke("place_order", { projectCode });
}

export async function saveMaterial(projectCode: string, name: string, theoretical: string): Promise<number> {
  return invoke("save_material", { projectCode, name, theoretical });
}

export async function addMaterialMove(materialId: number, kind: string, quantity: string): Promise<{ theoretical: string; actual: string; wastage: string }> {
  return invoke("add_material_move", { materialId, kind, quantity });
}

export async function saveForecast(projectCode: string, remaining: string, cashFlow: string): Promise<{ eac: string; cashFlow: string }> {
  return invoke("save_forecast", { projectCode, remaining, cashFlow });
}

export async function saveLocationCost(locationId: number, amount: string): Promise<void> {
  await invoke("save_location_cost", { locationId, amount });
}

export async function locationRollup(projectCode: string): Promise<{ allocated: string; projectTotal: string }> {
  return invoke("location_rollup", { projectCode });
}

export async function recommendBill(certificateId: number): Promise<void> {
  await invoke("recommend_bill", { certificateId });
}

export async function approveBill(certificateId: number): Promise<void> {
  await invoke("approve_bill", { certificateId });
}

export async function rejectBill(certificateId: number): Promise<void> {
  await invoke("reject_bill", { certificateId });
}

export async function certifyBill(certificateId: number): Promise<void> {
  await invoke("certify_bill", { certificateId });
}

export async function saveLibraryRate(kind: string, code: string, rate: string): Promise<number> {
  return invoke("save_library_rate", { kind, code, rate });
}

export async function applyLibraryRate(itemId: number, libraryId: number): Promise<string> {
  return invoke("apply_library_rate", { itemId, libraryId });
}

export async function projectKpi(projectCode: string): Promise<{ budget: string; actual: string; commitment: string; forecast: string; paid: string }> {
  return invoke("project_kpi", { projectCode });
}

export async function saveArea(projectCode: string, name: string, kind: "saleable" | "common", area: string): Promise<{ saleable: string; common: string; gfa: string; schedule: string; costPerSqft: string; margin: string }> {
  return invoke("save_area", { projectCode, name, kind, area });
}

export async function saveSaleRate(projectCode: string, rate: string): Promise<{ saleable: string; common: string; gfa: string; schedule: string; costPerSqft: string; margin: string }> {
  return invoke("save_sale_rate", { projectCode, rate });
}

export async function saveScenario(projectCode: string, name: string, budget: string): Promise<{ liveBudget: string; scenarioBudget: string }> {
  return invoke("save_scenario", { projectCode, name, budget });
}

export async function closeAccount(projectCode: string): Promise<{ amount: string; certificateNo: number }> {
  return invoke("close_account", { projectCode });
}

export async function projectTotals(): Promise<{ code: string; total: string }[]> {
  return invoke("project_totals");
}

export async function setIntegration(key: string, enabled: boolean): Promise<boolean> {
  return invoke("set_integration", { key, enabled });
}

export async function exportLocalWorkbook(projectCode: string, path: string): Promise<void> {
  await invoke("export_local_workbook", { projectCode, path });
}

export async function pickFirmPath(mode: "open" | "save"): Promise<string | null> {
  if (!inTauri()) return null;
  const { open, save } = await import("@tauri-apps/plugin-dialog");
  const filters = [{ name: "QS firm database", extensions: ["qsdb"] }];
  if (mode === "open") {
    const selected = await open({ filters, multiple: false });
    return typeof selected === "string" ? selected : null;
  }
  return save({ filters, defaultPath: await defaultFirmPath() });
}
