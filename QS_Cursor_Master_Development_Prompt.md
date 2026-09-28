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
