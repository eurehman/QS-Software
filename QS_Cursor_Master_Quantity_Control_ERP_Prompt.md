# MASTER CURSOR DEVELOPMENT PROMPT

## Standalone Windows Quantity Surveying & Cost Control System

You are the lead software architect, senior developer, QS-domain
analyst, database architect, UI/UX designer, QA engineer, and technical
project manager for this project.

Your task is to design and progressively develop a **professional
standalone Windows software application covering the complete Quantity
Surveying, Commercial Management and Cost Control system of a real
estate/construction company.**

Do not treat this as a simple billing application or an Excel
replacement.

The target is:

> **A complete QS + Commercial + Cost Control operating system that
> combines the flexibility and calculation capability of Excel with the
> reliability, structure, security, workflow, reporting, backup, and
> scalability of professional enterprise software.**

------------------------------------------------------------------------

# 1. ABSOLUTE DEVELOPMENT RULES

Before writing or modifying code:

1.  Inspect the existing project/repository completely.
2.  Understand the existing architecture before making changes.
3.  Do not delete working functionality.
4.  Do not unnecessarily rewrite existing modules.
5.  Do not invent requirements that conflict with this specification.
6.  If a requirement is ambiguous, choose the safest professional
    architecture and document the assumption.
7.  Maintain backward compatibility wherever practical.
8.  Keep the system modular.
9.  Keep business logic separate from UI.
10. Keep calculation logic separate from database logic.
11. Keep integration interfaces separate from core functionality.
12. Every major development task must be tracked in the two mandatory
    project canvases described below.
13. Develop progressively: **BASIC → INTERMEDIATE → ADVANCED**
14. Do not jump to advanced functionality before the required foundation
    is stable.
15. Test every major feature before marking it complete.
16. Never mark a feature as completed merely because code has been
    written.
17. A feature is complete only after implementation + testing +
    verification.
18. Do not introduce paid services, paid plugins, paid APIs,
    subscriptions, or proprietary dependencies without explicit
    approval.

------------------------------------------------------------------------

# 2. TWO MANDATORY DEVELOPMENT CANVASES

Cursor must continuously maintain two permanent development canvases.

## CANVAS 1 --- TARGETED WORKS

This is the master list of work that needs to be done.

Maintain:

-   Requirement ID
-   Work ID
-   Module
-   Feature
-   Sub-feature
-   Description
-   Development level
-   Priority
-   Dependencies
-   Status
-   Testing requirement
-   Notes

Statuses should include:

-   Not Started
-   Planned
-   In Progress
-   Blocked
-   Ready for Testing
-   Testing
-   Failed
-   Completed

Do not remove completed items from project history.

## CANVAS 2 --- PROGRESS DONE

This is the permanent development history.

For every completed work record:

-   Work ID
-   Requirement ID
-   Module
-   Feature
-   What was implemented
-   Files/components changed
-   Database changes
-   Calculations implemented
-   Tests performed
-   Test result
-   Issues resolved
-   Remaining limitations
-   Date/time of completion

Every completed item must be traceable back to TARGETED WORKS.

The two canvases are mandatory project-control mechanisms and must
remain updated throughout development.

------------------------------------------------------------------------

# 3. PRODUCT VISION

Develop a:

**Standalone + Installable + Updateable + Integratable Windows QS
Software**

for a real estate/construction company.

The system must cover the complete lifecycle:

> Project → WBS/CBS → Estimate → BOQ → Quantity Takeoff → Rate Analysis
> → Tender → Procurement → Contract → Measurement → IPC/Billing →
> Variations → Actual Cost → Commitments → Forecast → Cost Control →
> Profitability → Final Account → Historical Cost Database

The system should ultimately function as the company's central QS and
cost-control system.

------------------------------------------------------------------------

# 4. WINDOWS SOFTWARE REQUIREMENT

The application must be:

-   Standalone
-   Installable on Windows
-   Updateable
-   Maintainable
-   Recoverable
-   Integratable
-   Offline-capable
-   Suitable for local network/server operation
-   Suitable for future expansion

Provide a professional Windows installation process.

The software must not require a paid cloud service to operate.

------------------------------------------------------------------------

# 5. ZERO-COST PRINCIPLE --- HARD REQUIREMENT

The user does not want to spend money on:

-   Paid software
-   Paid plugins
-   Paid APIs
-   Paid libraries
-   Paid subscriptions
-   Paid cloud services
-   Paid SaaS
-   Paid hosting
-   Paid database services
-   Paid monitoring
-   Paid backup services
-   Paid integration services
-   Paid licenses

Use free and open-source technologies wherever possible.

Prefer:

-   Open source
-   Local processing
-   Self-hosted services
-   Local database
-   Local server
-   Standard protocols
-   Free libraries
-   Free development tools

Do not introduce a paid dependency without explicit approval.

If an implementation could create future mandatory costs, clearly
identify it before implementing it.

------------------------------------------------------------------------

# 6. EXCEL-CENTRIC CORE

The core philosophy of the software must be:

> **Excel flexibility + professional QS system control.**

Historical company data exists primarily in:

-   Microsoft Excel
-   Google Sheets

Therefore Excel compatibility is a core architectural requirement.

The system must support:

-   XLSX import
-   XLS import where technically practical
-   XLSX export
-   Excel-style tables
-   Excel-like calculations
-   Formula-driven calculations
-   Cross-sheet references
-   Cell/range references
-   Automatic recalculation
-   Formula validation
-   Calculation error detection
-   Bulk data import
-   Bulk data export
-   Historical data migration
-   Excel template-based workflows

The system must not simply import Excel into a rigid database and lose
the flexibility of the original data.

------------------------------------------------------------------------

# 7. EXCEL CALCULATION ENGINE

Create a robust calculation layer capable of reproducing common
Excel-style QS calculations.

It should support, where practical:

-   Arithmetic formulas
-   SUM
-   SUMIF
-   SUMIFS
-   COUNT
-   COUNTIF
-   COUNTIFS
-   AVERAGE
-   MIN
-   MAX
-   IF
-   AND
-   OR
-   ROUND
-   ROUNDUP
-   ROUNDDOWN
-   IFERROR
-   INDEX
-   MATCH
-   XLOOKUP where practical
-   VLOOKUP where practical
-   HLOOKUP where practical
-   Percentage calculations
-   Quantity × Rate
-   Quantity × Rate × Factor
-   Cumulative calculations
-   Running totals
-   Variance calculations
-   Cross-sheet calculations

Design the calculation engine so additional functions can be added
later.

QS calculations must be tested against known Excel results.

------------------------------------------------------------------------

# 8. GOOGLE SHEETS

Support migration/import from Google Sheets.

Where possible:

-   Import Google Sheets data
-   Export data to Google-compatible formats
-   Support future synchronization architecture

However:

**Google integration must not be mandatory for the core application.**

The application must remain fully functional without Google services.

Do not introduce a paid Google service dependency.

------------------------------------------------------------------------

# 9. DATA OWNERSHIP & PORTABILITY

The company's data must remain portable.

The system must provide:

-   Excel export
-   Structured backup
-   Data export
-   Project export
-   Module export
-   Database recovery
-   Migration capability

Do not create vendor lock-in.

The company's QS data must remain usable independently of the software.

------------------------------------------------------------------------

# 10. USER STRUCTURE

The system shall have:

## 1 Administrator

Exactly one Administrator account shall be supported as the primary
system administrator.

Administrator controls:

-   Users
-   Roles
-   Permissions
-   Projects
-   Modules
-   Settings
-   Backup
-   Restore
-   Configuration
-   Audit
-   System maintenance

## Maximum 4 additional users

There shall be a maximum of:

**4 role-based users**

Therefore:

**Maximum total active accounts = 5**

1 Administrator 2--5 Role-based users

The system must enforce the maximum user limit.

Do not assume that more than 4 additional users are required.

------------------------------------------------------------------------

# 11. ROLE-BASED ACCESS CONTROL

The four users shall receive configurable roles.

Possible examples:

-   Quantity Surveyor
-   Billing QS
-   Cost Control / Planning
-   Commercial / Procurement

These are examples, not fixed mandatory roles.

Administrator must be able to configure:

> User → Role → Permissions → Project Access → Module Access → Approval
> Authority

Permissions should include, where applicable:

-   View
-   Create
-   Edit
-   Delete
-   Import
-   Export
-   Approve
-   Reject
-   Recommend
-   Print
-   Backup
-   Restore
-   Configure

------------------------------------------------------------------------

# 12. PROJECT DATA HIERARCHY

Design a structured hierarchy such as:

Company → Project → Development → Building → Tower → Floor → Unit → Zone
→ WBS → CBS → Discipline → Work Package → BOQ Item

Support different project structures because real estate developments
may not always follow exactly the same hierarchy.

------------------------------------------------------------------------

# 13. CORE QS MODULES

The software shall progressively cover the complete QS lifecycle.

## PROJECT & COST STRUCTURE

-   Project setup
-   Project hierarchy
-   WBS
-   CBS
-   Cost codes
-   Work codes
-   Item codes
-   Disciplines
-   Work packages
-   Units of measurement
-   Location structure
-   Historical cost database

------------------------------------------------------------------------

# 14. ESTIMATION

Support:

-   Preliminary estimates
-   Concept estimates
-   Feasibility estimates
-   Detailed estimates
-   Elemental cost plans
-   Area-based estimates
-   Unit-rate estimates
-   BOQ estimates
-   Quantity takeoff
-   Rate analysis
-   Material rate analysis
-   Labour rate analysis
-   Plant rate analysis
-   Composite rate analysis
-   Historical rate analysis
-   Market rates
-   Vendor quotation analysis
-   Budget preparation

------------------------------------------------------------------------

# 15. BOQ MANAGEMENT

Provide:

-   BOQ creation
-   BOQ import from Excel
-   BOQ export to Excel
-   BOQ item library
-   Standard item library
-   Specification library
-   Item coding
-   Parent/child items
-   Quantity revision
-   Rate revision
-   Amount calculation
-   BOQ versions
-   BOQ approval
-   BOQ comparison
-   BOQ reconciliation

------------------------------------------------------------------------

# 16. QUANTITY TAKEOFF & MEASUREMENT

Provide:

-   Quantity takeoff
-   Drawing-based takeoff architecture
-   Manual measurement
-   Measurement sheets
-   Dimension sheets
-   Abstract quantities
-   Measurement Book
-   Site measurements
-   Joint measurements
-   Contractor measurements
-   QS verification
-   Measurement approval
-   Re-measurement
-   Quantity reconciliation
-   Planned vs actual quantity
-   Drawing revision impact
-   As-built quantities

------------------------------------------------------------------------

# 17. RATE ANALYSIS

Support:

-   Material components
-   Labour components
-   Plant/equipment
-   Wastage
-   Overheads
-   Profit
-   Taxes
-   Composite rates
-   Historical rates
-   Market rates
-   Vendor rates
-   Subcontractor rates
-   Rate comparison
-   Rate revision history

Rate analysis must remain formula-driven and Excel-compatible.

------------------------------------------------------------------------

# 18. PROCUREMENT / COMMERCIAL

Support:

-   Material requisitions
-   Purchase requisitions
-   RFQs
-   Vendor quotations
-   Comparative statements
-   Technical comparison
-   Commercial comparison
-   Negotiation records
-   Purchase orders
-   Supply contracts
-   Delivery tracking
-   Material receipts
-   Procurement commitments
-   Procurement cost comparison

------------------------------------------------------------------------

# 19. CONTRACTOR / SUBCONTRACT MANAGEMENT

Support:

-   Contractor database
-   Subcontractor database
-   Qualification
-   Scope
-   Tender package
-   Tender issuance
-   Tender receipt
-   Tender analysis
-   Tender comparison
-   Negotiation
-   Contractor selection
-   Work orders
-   Subcontracts
-   Contract BOQ
-   Contract rates
-   Contract value
-   Amendments
-   Extensions
-   Closeout

------------------------------------------------------------------------

# 20. BILLING / IPC

Provide:

-   Contractor bills
-   Interim Payment Certificates
-   Running bills
-   Measurement verification
-   BOQ quantity certification
-   Variation certification
-   Material-on-site claims
-   Advance payment
-   Advance recovery
-   Retention
-   Retention release
-   Mobilization advance
-   Deductions
-   Penalties
-   Taxes
-   Other recoveries
-   Net payable
-   Payment recommendation
-   Payment tracking
-   Bill history
-   Bill reconciliation

All calculations must be transparent and auditable.

------------------------------------------------------------------------

# 21. VARIATIONS

Provide:

-   Variation identification
-   Variation request
-   Variation order
-   Additional quantities
-   Omitted quantities
-   Substituted items
-   New items
-   New rates
-   New rate analysis
-   Variation valuation
-   Approval
-   Variation log
-   Contract impact
-   Budget impact
-   Forecast impact
-   Profitability impact

------------------------------------------------------------------------

# 22. COST CONTROL

Core cost-control functions:

-   Original budget
-   Approved budget
-   Revised budget
-   Actual cost
-   Committed cost
-   Accrued cost
-   Forecast cost
-   Cost-to-complete
-   Cost variance
-   Budget variance
-   Quantity variance
-   Rate variance
-   Productivity variance
-   Material variance
-   Labour variance
-   Subcontract variance

Allow analysis by:

-   Project
-   Tower
-   Building
-   Floor
-   Unit
-   Discipline
-   Work package
-   Cost code
-   Contractor
-   Material

------------------------------------------------------------------------

# 23. FORECASTING

Support:

-   Monthly forecast
-   Project forecast
-   Final cost forecast
-   Estimate at Completion
-   Estimate to Complete
-   Cash-flow forecast
-   Commitment forecast
-   Procurement forecast
-   Contractor payment forecast
-   Expenditure forecast
-   Budget utilization forecast
-   Forecast vs actual

------------------------------------------------------------------------

# 24. MATERIAL CONTROL

Support:

-   Material budget
-   Procurement
-   Receipt
-   Issue
-   Consumption
-   Wastage
-   Return
-   Reconciliation
-   Theoretical consumption
-   Actual consumption
-   Consumption variance
-   Wastage analysis
-   Stock valuation
-   Price variance

------------------------------------------------------------------------

# 25. LABOUR CONTROL

Support:

-   Labour quantities
-   Labour rates
-   Labour cost
-   Productivity norms
-   Actual productivity
-   Productivity variance
-   Labour cost variance
-   Man-hours
-   Labour allocation
-   Labour productivity reports

------------------------------------------------------------------------

# 26. PLANT & EQUIPMENT

Support:

-   Equipment database
-   Equipment utilization
-   Equipment rates
-   Equipment hours
-   Equipment cost
-   Fuel consumption
-   Productivity
-   Equipment cost variance

------------------------------------------------------------------------

# 27. REAL ESTATE-SPECIFIC QS

This is a major component.

Support:

-   GFA
-   Covered area
-   Net saleable area
-   Saleable area
-   Common area
-   Efficiency
-   Unit areas
-   Apartment areas
-   Unit construction cost
-   Cost/saleable sqft
-   Tower cost
-   Floor cost
-   Unit cost
-   Cost allocation
-   Development cost
-   Infrastructure cost
-   Amenity cost
-   Parking cost
-   External development
-   Land cost allocation
-   Soft costs
-   Finance costs
-   Marketing costs
-   Development overhead
-   Total development cost
-   Cost per saleable unit
-   Expected revenue
-   Development margin

------------------------------------------------------------------------

# 28. PROJECT PROFITABILITY

Provide:

-   Revenue
-   Cost
-   Gross profit
-   Gross margin
-   Budget profit
-   Forecast profit
-   Actual profit
-   Profit variance
-   Package profitability
-   Cost/sqft
-   Cost/unit
-   Cost/apartment
-   Cost/saleable area

------------------------------------------------------------------------

# 29. FINAL ACCOUNT / CLOSEOUT

Support:

-   Final measurement
-   Final quantities
-   Final BOQ
-   Final variations
-   Final contractor account
-   Final payment
-   Retention release
-   Final reconciliation
-   Final account
-   Contract closeout
-   Cost closeout
-   Project cost finalization
-   Lessons learned
-   Historical cost database update

------------------------------------------------------------------------

# 30. REPORTING

Every major module should support:

-   Screen report
-   Detail report
-   Summary report
-   Print preview
-   Print
-   Excel
-   PDF where possible
-   Filtering
-   Sorting
-   Date ranges
-   Project selection
-   Discipline selection
-   Contractor selection

Management reporting should include:

-   BOQ
-   Measurement
-   IPC
-   Variations
-   Contracts
-   Procurement
-   Commitments
-   Budget vs actual
-   Cost variance
-   Forecast
-   Material reconciliation
-   Contractor performance
-   Project cost
-   Profitability
-   Cost/sqft
-   Cost/unit

------------------------------------------------------------------------

# 31. DASHBOARDS

Provide dashboards at:

-   Company
-   Project
-   Building
-   Tower
-   Floor
-   Unit
-   Work Package

KPIs should include:

-   Budget
-   Actual
-   Commitment
-   Forecast
-   Variance
-   Contract value
-   Variations
-   Paid
-   Outstanding
-   Cost/sqft
-   Cost/unit
-   Profitability

------------------------------------------------------------------------

# 32. DOCUMENT MANAGEMENT

Allow supporting documents to be attached to relevant records:

-   Drawings
-   BOQs
-   Quotations
-   Contracts
-   Work orders
-   Measurement sheets
-   Invoices
-   Delivery notes
-   Certificates
-   Approvals
-   Correspondence

Documents should be linked to relevant:

Project → Contract → BOQ → Item → Transaction

------------------------------------------------------------------------

# 33. AUDIT TRAIL & VERSION CONTROL

Do not silently overwrite important historical information.

Track:

-   Creator
-   Modifier
-   Date/time
-   Old value
-   New value
-   Approval
-   Rejection
-   BOQ revision
-   Rate revision
-   Contract revision
-   Document revision

Maintain a complete audit history.

------------------------------------------------------------------------

# 34. ONE-CLICK BACKUP

The software shall provide a prominent:

**CREATE BACKUP**

function.

When activated:

1.  Export relevant application data to Excel files.
2.  Organize files into a structured backup folder.
3.  Add date/time/version.
4.  Save backup automatically to the **QS Local Server**.
5.  Verify backup completion.
6.  Record backup in backup history.
7.  Notify user of success/failure.

Example:

``` text
QS Local Server
└── QS Software Backups
    └── Project / Company
        └── Year
            └── Timestamped Backup
                ├── Projects.xlsx
                ├── WBS.xlsx
                ├── CBS.xlsx
                ├── BOQ.xlsx
                ├── Measurements.xlsx
                ├── Rate_Analysis.xlsx
                ├── Contracts.xlsx
                ├── IPC_Bills.xlsx
                ├── Variations.xlsx
                ├── Procurement.xlsx
                ├── Cost_Control.xlsx
                ├── Budgets.xlsx
                ├── Forecasts.xlsx
                ├── Contractors.xlsx
                ├── Materials.xlsx
                └── Backup_Info.xlsx
```

------------------------------------------------------------------------

# 35. BACKUP & RESTORE

Also provide:

-   Backup history
-   Manual backup
-   Automatic backup
-   Backup verification
-   Backup integrity check
-   Restore
-   Restore from selected version
-   Backup logs
-   Multiple backup versions
-   Disaster recovery procedure

Backup must not rely on paid cloud storage.

------------------------------------------------------------------------

# 36. INSTALLATION & UPDATES

Provide:

-   Windows installer
-   Application version
-   Database migration
-   Configuration preservation
-   Safe update
-   Recovery/rollback where practical
-   Uninstall
-   Reinstallation
-   Version history

Updates must not unnecessarily destroy or overwrite user data.

------------------------------------------------------------------------

# 37. INTEGRATION ARCHITECTURE

The application must be integration-ready.

Design interfaces for future integration with:

-   Excel
-   Google Sheets
-   Accounting systems
-   ERP
-   Document systems
-   Other company systems
-   APIs
-   Future Buildora or other platforms

Core functionality must not depend on external integrations.

------------------------------------------------------------------------

# 38. CONFIGURATION

Avoid hard-coding business rules unnecessarily.

Administrator should be able to configure:

-   Units
-   Disciplines
-   WBS
-   CBS
-   Cost codes
-   Work codes
-   Approval levels
-   Tax rules
-   Retention
-   Payment terms
-   Contract types
-   Project structures
-   Roles
-   Permissions
-   Report formats

------------------------------------------------------------------------

# 39. UI / UX

The application should combine:

> **Professional ERP interface + Excel-like working environment**

Provide:

-   Clear navigation
-   Dashboard
-   Sidebar
-   Module navigation
-   Search
-   Global search
-   Filters
-   Sorting
-   Grouping
-   Freeze panes
-   Column resizing
-   Bulk editing
-   Copy/paste
-   Excel-like grid
-   Keyboard shortcuts
-   Right-click actions
-   Validation
-   Print preview
-   Professional forms
-   Professional tables

Prioritize usability for QS professionals who are already accustomed to
Excel.

------------------------------------------------------------------------

# 40. PERFORMANCE & SCALABILITY

The architecture should support progression from:

## BASIC

-   One user
-   One project
-   Small datasets

## INTERMEDIATE

-   Multiple projects
-   Up to 4 additional users
-   Large BOQs
-   Large measurement datasets

## ADVANCED

-   Large real-estate projects
-   Multiple towers
-   Multiple buildings
-   Millions of records
-   Multiple concurrent users
-   QS Local Server deployment

Avoid unnecessary recalculation or database queries that make large
projects slow.

------------------------------------------------------------------------

# 41. BASIC → INTERMEDIATE → ADVANCED

Development shall occur in three levels.

## BASIC

Build the operational QS foundation:

-   Project
-   WBS/CBS
-   BOQ
-   BOQ items
-   Units
-   Quantity takeoff
-   Measurements
-   Rate analysis
-   Estimates
-   Contractors
-   Contracts
-   Work orders
-   IPC
-   Basic variations
-   Basic cost control
-   Excel import/export
-   Basic reports
-   Backup

## INTERMEDIATE

Then develop:

-   Advanced BOQ
-   Advanced WBS/CBS
-   Budget vs actual
-   Commitments
-   Procurement
-   Material reconciliation
-   Contractor performance
-   Advanced variations
-   Forecasting
-   Cost-to-complete
-   Cash flow
-   Variance analysis
-   Tower/floor/unit analysis
-   Approval workflows
-   Advanced Excel integration
-   Historical rate database
-   Advanced dashboards

## ADVANCED

Then develop:

-   Advanced forecasting
-   Cost prediction
-   Profitability analytics
-   Real-estate development cost modelling
-   Unit-level cost allocation
-   Benchmarking
-   Historical cost intelligence
-   Scenario modelling
-   What-if analysis
-   Automated management reports
-   Advanced analytics
-   Cross-project analytics
-   Advanced API/integration
-   Advanced anomaly detection
-   AI-assisted QS functions only where achievable with zero-cost/local
    technologies

------------------------------------------------------------------------

# 42. TESTING

No major module shall be considered complete without testing.

Test:

-   Functional behavior
-   Calculations
-   Excel import
-   Excel export
-   Formula accuracy
-   Database integrity
-   Backup
-   Restore
-   Permissions
-   Approval workflow
-   Audit trail
-   Performance
-   Regression

Most importantly:

> **QS calculations must be verified against known Excel calculations.**

Create test datasets for:

-   BOQ
-   Rate analysis
-   Measurement
-   IPC
-   Variations
-   Budget
-   Actual
-   Forecast
-   Material reconciliation
-   Profitability

------------------------------------------------------------------------

# 43. DATA MIGRATION

Historical Excel and Google Sheets data must be treated as an important
source of truth.

Build migration tools capable of:

-   Reading historical workbooks
-   Mapping columns
-   Mapping codes
-   Mapping units
-   Mapping projects
-   Mapping BOQs
-   Mapping historical rates
-   Detecting duplicates
-   Detecting invalid values
-   Validating imported data
-   Showing import preview
-   Reporting errors
-   Allowing corrections before final import

Never blindly import corrupted or ambiguous historical data.

------------------------------------------------------------------------

# 44. EXCEL BACKUP PHILOSOPHY

The Excel backup should be:

-   Human-readable
-   Structured
-   Organized
-   Recoverable
-   Versioned
-   Independent of the application

Even if the application becomes unavailable, authorized personnel should
still be able to access the company's QS information from the backup
Excel files.

------------------------------------------------------------------------

# 45. SECURITY

Implement:

-   Secure login
-   Password protection
-   Role-based access
-   Project-level access
-   Module-level access
-   Audit logs
-   Protected configuration
-   Safe database access
-   Backup security
-   Restore authorization

Administrator functions must be restricted.

------------------------------------------------------------------------

# 46. DEVELOPMENT METHODOLOGY

Before implementing any module:

1.  Add it to TARGETED WORKS.
2.  Define dependencies.
3.  Design data model.
4.  Design UI.
5.  Design calculations.
6.  Implement.
7.  Test.
8.  Verify against expected QS/Excel results.
9.  Record in PROGRESS DONE.
10. Only then proceed to dependent functionality.

Never skip project tracking.

------------------------------------------------------------------------

# 47. CHANGE CONTROL

If a new requirement is provided later:

1.  Do not overwrite existing requirements.
2.  Add the new requirement.
3.  Identify affected modules.
4.  Identify dependencies.
5.  Update TARGETED WORKS.
6.  Implement carefully.
7.  Test affected functionality.
8.  Update PROGRESS DONE.

If a new requirement conflicts with an earlier requirement:

**Stop and identify the conflict before implementing the change.**

Do not silently choose one.

------------------------------------------------------------------------

# 48. NO PREMATURE COMPLEXITY

Do not build advanced functionality merely because it is technically
possible.

Always ask:

> Is this required for the current development level?

Prioritize:

**Correctness → Reliability → Data Integrity → Usability → Performance →
Advanced Features**

------------------------------------------------------------------------

# 49. FINAL PRODUCT STANDARD

The final application should feel like:

**Excel + QS Department + Commercial Department + Cost Control + Project
Controls + Real Estate Development Cost System**

in one controlled application.

It must be suitable for real-world professional QS use.

It must not merely be a demonstration, prototype, or collection of
disconnected forms.

------------------------------------------------------------------------

# 50. FIRST ACTION --- DO NOT START CODING BLINDLY

Before writing substantial code:

### STEP 1

Inspect the complete repository/environment.

### STEP 2

Identify existing technologies, files, modules and dependencies.

### STEP 3

Determine the most suitable zero-cost architecture for a Windows
application.

### STEP 4

Create the initial:

**TARGETED WORKS**

and

**PROGRESS DONE**

canvases.

### STEP 5

Create the complete system architecture.

### STEP 6

Create the proposed database/data model.

### STEP 7

Create the Basic → Intermediate → Advanced implementation roadmap.

### STEP 8

Identify risks, dependencies and unresolved technical decisions.

### STEP 9

Present the architecture and roadmap before undertaking major
implementation.

### STEP 10

After the foundation is approved/defined, begin BASIC development
incrementally.

------------------------------------------------------------------------

# 51. CRITICAL FINAL INSTRUCTION

Do not interpret this prompt as permission to build everything in one
uncontrolled operation.

This is a **large long-term software project**.

Build it systematically.

Maintain continuity.

Never lose track of:

-   Requirements
-   Decisions
-   Completed work
-   Pending work
-   Changes
-   Bugs
-   Tests
-   Database changes
-   Architecture decisions

The two permanent canvases --- **TARGETED WORKS** and **PROGRESS DONE**
--- are mandatory.

The ultimate objective is:

> **A reliable, zero-cost, standalone Windows Quantity Surveying and
> Cost Control system for a real estate company, with Excel-compatible
> calculations, historical data migration, structured QS workflows,
> professional reporting, local-server backup, controlled users, future
> integration capability, and a development path from Basic to
> Intermediate to Advanced functionality.**

Begin by inspecting the existing environment and producing the initial
architecture, project structure, and two development canvases. Do not
make destructive changes.


---

# 61. MANDATORY PHASE SEQUENCING & EXIT GATES

The old Basic → Intermediate → Advanced labels are product maturity labels only. They are NOT permission to skip architectural dependencies.

No phase may be declared complete merely because individual tests pass.

A phase is complete only when its exit gate passes.

## Phase Sequence

0. Architecture & Reconciliation
1. Quantity Control Foundation
2. Excel-like QS Working Environment
3. BOQ + Measurement + Rate + Contract
4. Execution + Procurement + Material + Billing
5. Cost Control + Budget + Commitment + Forecast
6. Variations + Final Account + Historical Cost
7. ERP Reporting + Dashboards + Management Control
8. Security + Backup + Recovery + Multi-user
9. Windows Productization
10. Production Acceptance

---

## PHASE 0 — ARCHITECTURE & RECONCILIATION

### Objective

Understand the existing application and establish the correct ERP foundation before major implementation.

### Required

Create:

- Architecture Gap Analysis
- Current Data Model Map
- Required Data Model
- Existing Module Inventory
- Existing Calculation Inventory
- Existing Test Inventory
- Existing UI Inventory
- Migration Plan
- Quantity Control Core design
- Module dependency map
- Transaction flow map

Classify every existing feature:

- Production Ready
- Functional but incomplete
- Prototype
- Test only
- Missing
- Requires redesign

### Exit Criteria

- Repository inspected.
- Existing schema documented.
- Existing calculations documented.
- Existing tests documented.
- Existing modules classified.
- Required ERP entities defined.
- Quantity Control Core data model defined.
- Module dependency order documented.
- Migration strategy defined.
- No working feature scheduled for deletion without justification.
- TARGETED WORKS corrected.
- PROGRESS DONE corrected.
- ARCHITECTURE.md created/updated.
- Build succeeds.
- Existing tests still pass.

### Gate

**No major new module development until Phase 0 passes.**

---

## PHASE 1 — QUANTITY CONTROL FOUNDATION

### Objective

Create the common ERP data structure.

```text
Company
 ↓
Project
 ↓
Location
 ↓
WBS
 ↓
CBS
 ↓
Work Package
 ↓
BOQ Item
 ↓
Quantity
 ↓
Unit
 ↓
Rate
 ↓
Amount
```

### Required

Authoritative entities:

- Company
- Project
- Development
- Building
- Tower
- Floor
- Unit
- Zone
- Location
- WBS
- CBS
- Cost Code
- Work Code
- Discipline
- Work Package
- BOQ
- BOQ Item
- Unit
- Quantity
- Rate

### Quantity Ledger

Implement:

- Original Quantity
- Revised Quantity
- Planned Quantity
- Contract Quantity
- Executed Quantity
- Measured Quantity
- Certified Quantity
- Billed Quantity
- Paid Quantity
- Forecast Quantity
- Final Quantity

### Exit Criteria

- Real project can be created.
- Project hierarchy is configurable.
- WBS/CBS can be created.
- BOQ items reference WBS/CBS.
- BOQ items have authoritative quantity/unit/rate.
- Parent/child BOQ relationships work.
- Quantity Ledger exists.
- Quantity balances calculate automatically.
- No duplicate authoritative BOQ entities exist across modules.
- Project separation works.
- Data persists after restart.
- Migration works.
- Permissions work.
- Important master-data changes are audited.
- Excel export of core data works.
- Realistic construction project can be entered.
- Quantity reconciliation passes.

### Mandatory Gate Test

Create:

**Concrete — Structural**

BOQ = 100 m³

Verify:

```text
Original       100
Contracted     100
Executed         0
Measured        0
Certified       0
Billed           0
Remaining      100
```

Process transactions and verify the ledger after every transaction.

### Gate

**Quantity Control Core stable.**

---

## PHASE 2 — EXCEL-LIKE QS WORKING ENVIRONMENT

### Objective

Make the application genuinely usable by a QS who currently works in Excel.

### Required

Create reusable spreadsheet functionality for:

- BOQ
- Measurement
- Rate Analysis
- Certificate
- Cost
- Procurement
- Quantity Ledger

Required:

- Cell editing
- Formula bar
- Copy/paste
- Fill
- Multi-cell selection
- Keyboard navigation
- Undo/redo
- Column resize
- Freeze panes
- Filtering
- Sorting
- Grouping
- Search
- Find/replace
- Bulk paste
- Excel import
- Excel export
- Validation
- Formula recalculation

### Formula Engine

Minimum:

```text
SUM
SUMIF
SUMIFS
COUNT
COUNTA
COUNTIF
COUNTIFS
AVERAGE
MIN
MAX
ROUND
ROUNDUP
ROUNDDOWN
IF
AND
OR
NOT
IFERROR
VLOOKUP
XLOOKUP
INDEX
MATCH
```

Also:

- Cross-sheet references
- Absolute references
- Relative references
- Mixed references
- Range references
- Error handling
- Circular reference detection
- Dependency recalculation

Do not claim full Excel compatibility unless implemented.

### Exit Criteria

- QS can prepare BOQ in spreadsheet interface.
- QS can prepare measurement in spreadsheet.
- QS can prepare rate analysis in spreadsheet.
- Formulas calculate without separate Save action.
- Historical workbook can be mapped to ERP fields.
- Import works.
- Export works.
- Copy/paste works.
- Formula references work.
- Cross-sheet calculations work.
- Existing locked formula tests pass.
- 100+ realistic rows perform acceptably.
- Realistic BOQ workflow can be completed without disconnected forms.

### Mandatory Gate Test

Import a realistic BOQ workbook:

- Item Code
- Description
- Unit
- Quantity
- Rate
- Amount
- WBS
- CBS

Map, modify, recalculate, export, reopen, verify.

### Gate

**ERP is genuinely spreadsheet-capable.**

---

## PHASE 3 — BOQ + MEASUREMENT + RATE + CONTRACT

### Objective

Create the first integrated QS commercial workflow.

```text
BOQ
 ↓
Measurement
 ↓
Rate
 ↓
Contract
```

### Required

BOQ:

- Versions
- Revisions
- Parent/child
- Quantities
- Rates
- Specifications
- WBS/CBS
- Locations

Measurement:

- Dimension sheets
- Measurement sheets
- Site measurement
- Joint measurement
- Re-measurement
- As-built
- Abstract quantities

Rate Analysis:

- Material
- Labour
- Plant
- Wastage
- Overhead
- Profit
- Composite
- Historical
- Market
- Vendor

Contract:

- Contractor
- Contract
- Contract BOQ
- Contract rates
- Work order
- Contract value

### Exit Criteria

- BOQ quantity flows into measurement.
- Measurement updates measured quantity.
- Rate analysis updates applicable rate.
- Contract references authoritative BOQ items.
- Contract quantity traces to BOQ.
- Contract amount traces to quantity × rate.
- Revisions preserve history.
- Rate history preserved.
- Measurement history preserved.
- Contract history preserved.
- Calculations reconcile with Excel references.
- UI workflow works without database manipulation.

### Mandatory Gate Test

```text
BOQ = 100 m³
Rate = 25,000
Contract = 100 m³
Measured = 35 m³
```

Verify every value through the UI.

### Gate

**BOQ, Measurement, Rate and Contract share the same data.**

---

## PHASE 4 — EXECUTION + PROCUREMENT + MATERIAL + BILLING

### Objective

Connect physical project execution to commercial transactions.

```text
Contract
 ↓
Procurement
 ↓
Material
 ↓
Execution
 ↓
Measurement
 ↓
Certification
 ↓
IPC
```

### Required

Procurement:

- Requisition
- RFQ
- Quotations
- Comparison
- PO
- Delivery
- Receipt

Material:

- Stock
- Issue
- Consumption
- Wastage
- Return
- Reconciliation

Execution:

- Planned quantity
- Executed quantity
- Progress quantity

Billing:

- Measurement
- Certification
- IPC
- Retention
- Advance recovery
- Deductions
- Net payable

### Exit Criteria

- Procurement quantity links to project/work/package.
- Material receipt does not equal consumption automatically.
- Material issue records actual issue.
- Consumption can link to work quantity.
- Execution updates quantity progress.
- Measurement references execution where applicable.
- Certification references measurement.
- IPC references certified quantity.
- Billing cannot exceed allowable certified quantity without approved variation.
- Certified bills become actual cost only after required approval.
- Draft certificates do not count as actual cost.
- Transactions are auditable.
- Quantity reconciliation works.

### Mandatory Gate Test

```text
Contract = 100 m³
Executed = 40 m³
Measured = 38 m³
Certified = 35 m³
Billed = 35 m³
```

Verify all balances.

### Gate

**Physical quantity and commercial quantity reconcile.**

---

## PHASE 5 — COST CONTROL + BUDGET + COMMITMENT + FORECAST

### Objective

Turn quantities and transactions into real cost control.

```text
Budget
 ↓
Commitment
 ↓
Actual
 ↓
Forecast
 ↓
Variance
```

### Required

Budget:

- Original
- Approved
- Revised
- Scenario

Commitment:

- Contract
- PO
- Open commitment
- Exposure

Actual:

- Certified
- Paid
- Accrued where applicable

Forecast:

- ETC
- EAC
- Cash flow
- Monthly forecast

Variance:

- Quantity variance
- Rate variance
- Productivity variance
- Material variance
- Labour variance
- Contractor variance

### Exit Criteria

- Budget links to CBS.
- Contract creates commitment.
- PO creates commitment.
- Certified bill creates actual.
- Open commitment calculated correctly.
- Actual + open commitment does not double-count.
- Forecast uses actual + ETC.
- Scenario does not modify live budget.
- Variance calculated.
- Cost analyzable by project/location/WBS/CBS/work package.
- Monthly forecast exists.
- Cash flow exists.
- Cost-to-complete exists.

### Mandatory Gate Test

```text
Budget             2,000,000
Contract           1,500,000
Certified Actual     400,000
Open Commitment   1,100,000
ETC                 700,000
```

Verify Actual, Commitment, Exposure, EAC and Variance.

### Gate

**Management can determine project financial position from ERP data without reconstructing it in Excel.**

---

## PHASE 6 — VARIATIONS + FINAL ACCOUNT + HISTORICAL COST

### Objective

Close the commercial lifecycle.

```text
Original Contract
 ↓
Variation
 ↓
Revised Contract
 ↓
Final Measurement
 ↓
Final Account
 ↓
Historical Cost
```

### Required

Variation:

- Addition
- Omission
- Substitution
- New item
- New rate
- Quantity change
- Approval

Final Account:

- Final quantities
- Final rates
- Final variations
- Payments
- Retention
- Outstanding
- Reconciliation

Historical Cost:

- Project
- Location
- BOQ item
- Quantity
- Rate
- Cost
- Date
- Source

### Exit Criteria

- Approved variation changes quantity/value.
- Draft variation does not alter live contract.
- Variation line-item traceability works.
- Revised BOQ generated where required.
- Revised contract traceable.
- Final measurement reconciles.
- Final account reconciles with certified transactions.
- Historical cost searchable.
- Historical rates reusable.
- Closeout preserves project history.

### Gate

A project must move through:

**Original BOQ → Contract → Execution → Billing → Variation → Final Account → Historical Cost**

without manually recreating data.

---

## PHASE 7 — ERP REPORTING + DASHBOARDS + MANAGEMENT CONTROL

### Objective

Expose integrated data through professional ERP reporting.

Do not build dashboards before underlying transaction data is reliable.

### Required

Dashboards:

- Company
- Project
- Development
- Building
- Tower
- Floor
- Unit
- Work Package

Reports:

- BOQ
- Measurement
- Contract
- Procurement
- Material
- Billing
- Cost
- Budget
- Commitment
- Forecast
- Variation
- Final Account
- Profitability
- Historical Cost

Where applicable:

- Filter
- Sort
- Group
- Search
- Detail
- Summary
- Print Preview
- Print
- Excel Export
- PDF Export

### Exit Criteria

- Dashboard values come from live ERP data.
- No hard-coded demonstration figures.
- Drill-down works.
- Reports reconcile with transactions.
- Project totals remain separated.
- Cost/sqft works.
- Cost/unit works.
- Profitability works.
- Management reports can be produced without manual reconstruction.

### Gate

**Management reporting is based on real ERP transactions.**

---

## PHASE 8 — SECURITY + BACKUP + RECOVERY + MULTI-USER

### Objective

Make company data safe and recoverable.

### Backup

Include:

- Projects
- WBS
- CBS
- BOQ
- Measurements
- Rates
- Contracts
- Procurement
- Materials
- Certificates
- Bills
- Variations
- Cost
- Forecast
- Final accounts
- Audit
- Configuration
- Attachments where applicable

### Backup Exit Criteria

- One-click backup.
- Backup history.
- Integrity verification.
- Restore.
- Restore test.
- Excel-readable data export.
- Full company QS dataset represented.
- Backup survives application failure.
- Restore produces equivalent business data.

### Multi-user Exit Criteria

- Supported deployment topology documented.
- Concurrent access tested.
- Locking tested.
- Simultaneous edits tested.
- Backup while users are active tested.
- Failure recovery tested.
- Limitations documented.

Do not claim five-user network support without actual concurrency testing.

If SQLite network sharing is unsafe, implement the safest zero-cost local-server architecture possible.

### Gate

**Company data is recoverable and supported multi-user behavior is proven.**

---

## PHASE 9 — WINDOWS PRODUCTIZATION

### Objective

Turn the development application into a real Windows product.

### Required

- Windows installer
- Versioning
- Database migration
- Configuration preservation
- Uninstall
- Reinstall
- Update
- Recovery
- Release build
- Clean-machine installation test

### Exit Criteria

- Installer builds.
- Installer works on clean Windows environment.
- Application launches without development tools.
- Database created correctly.
- Existing database migrates.
- Configuration survives update.
- User data survives update.
- Uninstall works.
- Reinstall works.
- Version number correct.
- Release build reproducible.
- No paid signing dependency required.

Unsigned status must be documented honestly.

### Gate

**Actual installable Windows product exists.**

---

## PHASE 10 — PRODUCTION ACCEPTANCE

Final full project lifecycle:

```text
Company
 ↓
Project
 ↓
Development
 ↓
Building
 ↓
Tower
 ↓
Floor
 ↓
WBS
 ↓
CBS
 ↓
BOQ
 ↓
Estimate
 ↓
Rate Analysis
 ↓
Contract
 ↓
Procurement
 ↓
Material
 ↓
Execution
 ↓
Measurement
 ↓
Certification
 ↓
IPC
 ↓
Actual Cost
 ↓
Variation
 ↓
Revised Budget
 ↓
Commitment
 ↓
Forecast
 ↓
Final Measurement
 ↓
Final Account
 ↓
Historical Cost
 ↓
Management Reports
```

### Final Acceptance

All must pass:

Functional, calculation, data, security, Excel, UI, productization, backup/recovery and regression criteria.

The final acceptance question is:

> Can a real Quantity Surveying / Commercial / Cost Control department run a complete project lifecycle through this system without recreating the same information manually in disconnected Excel files?

---

# 62. HARD PHASE-GATE RULES

1. A passing unit test does not equal feature completion.
2. A feature cannot be Completed if its UI is missing.
3. A module cannot be Completed if disconnected from Quantity Control Core.
4. A module cannot be Completed if its data is excluded from backup.
5. A financial feature cannot be Completed without reconciliation testing.
6. A workflow cannot be Completed without approval/audit testing where required.
7. An Excel feature cannot be Completed based on one simple formula.
8. An integration cannot be Completed based only on a feature flag.
9. Multi-user cannot be Completed without concurrency testing.
10. Installer cannot be Completed until actually built and tested.
11. Do not advance because a phase is "mostly done."
12. Any failed exit criterion keeps the phase OPEN.

---

# 63. PHASE STATUS MODEL

Each phase must have exactly one status:

- NOT STARTED
- IN PROGRESS
- BLOCKED
- READY FOR GATE
- GATE FAILED
- GATE PASSED

Only **GATE PASSED** allows progression.

---

# 64. PHASE GATE REPORT

At the end of every phase create a gate report containing:

## Phase
Name and number.

## Objectives
What the phase intended to achieve.

## Completed
Actual implementation.

## Not Completed
Remaining work.

## Tests
Tests executed.

## Test Results
Pass/fail.

## Data Validation
Database integrity and reconciliation.

## UI Validation
Actual UI workflow.

## Excel Validation
Import/export/formula verification.

## Backup Validation
Whether phase data is included in backup.

## Integration Validation
Whether phase connects to Quantity Control Core.

## Known Limitations
Explicit limitations.

## Exit Criteria

| Criterion | Result | Evidence |
|---|---|---|
| Criterion | PASS/FAIL | Test/evidence |

## Gate Decision

Only:

**GATE PASSED**

or

**GATE FAILED**

Never use "approximately complete".

---

# 65. REGRESSION RULE

Every phase must rerun all previous phase tests.

```text
Phase 1:
P1 tests

Phase 2:
P1 + P2

Phase 3:
P1 + P2 + P3

...

Phase 10:
P1 + P2 + ... + P10
```

A later feature must never silently break an earlier phase.

---

# 66. CURRENT PROJECT RECLASSIFICATION

Immediately reclassify the existing 36/36 register.

Do NOT delete the 36 existing tests.

Convert their meaning from:

`Completed Feature`

to:

`Verified Component / Prototype Test`

where appropriate.

Examples:

B-016 proves an IPC calculation works.

It does NOT prove the complete IPC module is production-ready.

B-007 proves sheet backup works.

It does NOT prove complete company QS backup.

I-009 proves two connections can write to one firm file.

It does NOT prove five-user network operation.

A-003 proves a final-account calculation.

It does NOT prove a complete final-account module.

---

# 67. MOST IMPORTANT GATE

Before calling the system an ERP, prove this:

## ONE QUANTITY CAN BE FOLLOWED THROUGH THE ENTIRE SYSTEM.

Example:

```text
Concrete
100 m³
        ↓
BOQ
100 m³
        ↓
Contract
100 m³
        ↓
Execution
40 m³
        ↓
Measurement
38 m³
        ↓
Certification
35 m³
        ↓
Billing
35 m³
        ↓
Actual Cost
35 m³
        ↓
Variation
+20 m³
        ↓
Revised Contract
120 m³
        ↓
Forecast
        ↓
Final Account
        ↓
Historical Cost
```

At every stage the system must know:

- Project
- Location
- WBS
- CBS
- BOQ item
- Quantity
- Unit
- Rate
- Amount
- Transaction
- Creator
- Approver
- Revision
- Remaining quantity

If this chain cannot be demonstrated, the Quantity Control ERP is not complete.

---

# 68. FINAL EXECUTION COMMAND

Now inspect the existing repository.

Do NOT ask me to rewrite the requirements.

Do NOT start by deleting the existing project.

Do NOT declare the project complete.

First produce the architecture gap analysis and update:

1. TARGETED WORKS
2. PROGRESS DONE

Then begin implementation in the mandatory phase sequence.

At the end of every phase:

1. Run the phase tests.
2. Run all prior regression tests.
3. Test the actual UI.
4. Verify database integrity.
5. Verify Excel import/export where applicable.
6. Verify backup inclusion.
7. Generate the phase gate report.
8. Set phase status.
9. Only proceed if the gate is PASSED.

Every development cycle must preserve verified functionality while moving toward the end-to-end customizable Quantity Control ERP.

The objective is NOT to make the test suite green.

The objective is to make the actual ERP usable for a real Quantity Surveying / Commercial / Cost Control department.

# ONE COMPANY DATASET
# ONE QUANTITY CONTROL CORE
# ONE CONNECTED ERP
