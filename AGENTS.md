DoWhiz — Long-Term Vision for a Digital Employee Platform (Vision)

1. North Star Vision

Enable every user to have a “digital employee team” that collaborates like real coworkers via email / collaborative documents / team tools—able to execute tasks independently, follow through continuously, proactively sync progress, and escalate to humans when needed (clarify / approval / escalation).

Core experience:
	•	The user only needs to send a task to a given digital employee’s email address; it will execute, produce outputs, and return results through the appropriate channel.
	•	The digital employee autonomously selects the most suitable toolchain in the background (LLMs, code, documents, editing, data retrieval); the user doesn’t need to care about the details.
	•	Each user has a fully isolated memory and data management system that is auditable, revocable, and portable.

2. Product Form and Role Model

A digital employee is not a single bot, but a set of job-based roles:
	•	Alice (Researcher): deep research, daily paper monitoring, survey/brief production.
	•	Bob (TPM): task scheduling, progress tracking, cross-team reminders, meeting coordination.
	•	Eve (Docs/Notion/Overleaf collaborator): document organization, comment handling, version updates (after being granted access to a document, modify the doc based on Overleaf comments and leave responses/comments).

“Role = strategy package”:
	•	Each role corresponds to a set of goals, tool strategies, default models, risk level, and approval thresholds.
	•	Roles can use permissions granted by the user (e.g., access/edit permissions to different Notion workspaces), but act only within the authorized scope (least privilege).

3. Design Principles
	1.	Isolation first: user data, memory, credentials, and execution environments are isolated from each other.
	2.	Low interruption: complete the task if possible; if unclear, ask precise follow-up questions.
	3.	Tool-neutral: tools are replaceable and upgradable; users don’t need to know the details.
	4.	Controllable long-term memory: memory is file-based / structured storage (in the future, users can view, edit, and delete it).

4. Core Abstractions Referencing OpenClaw (Design Ideas That Can Be Reused Directly)

OpenClaw has mature abstractions for “multi-channel messaging + agent execution + task queue.” Key takeaways:
	•	Gateway (resident daemon process): uniformly maintains connections for all messaging channels and exposes a unified control interface.
	•	Agent Loop (task execution loop): standardizes the execution path of “context assembly → reasoning → tool calls → output → logging.”
	•	Command queue (Queue): serializes by session and rate-limits globally to ensure concurrency safety.
	•	Multi-agent routing: multiple agentIds work independently within the same process, each with its own workspace/session/credentials.
	•	Workspace + file-based memory: memory sources are readable/writable files, enabling auditability and rollback.
	•	Hooks / Plugins: insert security/audit/automation logic before and after execution.

These abstractions align highly with the core needs of a “digital employee platform,” and can serve as foundational design references.

5. Target System Architecture (Multi-tenant SaaS)

5.1 Logical Layers
	1.	Ingress layer (Channel Ingress)
	•	Email (priority)
	•	Docs/Notion comment events
	•	Later: Slack / Zoom / Calendar
	•	Unified event format (Inbound Event)
	2.	Control Plane
	•	user/org/subscription management
	•	role catalog and permission policies
	•	connector configuration and credential hosting (Vault)
	3.	Task Orchestrator
	•	task classification: registration / delinquent payment / irrelevant / direct reply / requires execution
	•	task state machine: queued → in-progress → done / archived
	•	scheduled and recurring tasks (daily / weekly / monthly / yearly: cron-like)
	4.	Execution Layer (Agent Runtime)
	•	assemble task context
	•	role strategy and tool selection
	•	tool execution, output generation, and return delivery
	5.	Storage Layer (Data & Memory)
	•	Azure Blob: user-private memory + artifacts
	•	DB (Postgres): metadata, tasks, indexes, permissions, audit logs
	6.	Observability Layer
	•	task logs, costs, failure reasons
	•	auditing and replay of external write operations

5.2 Execution Flow (Email Ingress Example)

Email → Ingress → Triage → Task Queue → Agent Runtime → Output
Scheduler ↗                                           ↘ Scheduler

	•	Triage categories:
a) needs account creation and a reply
b) insufficient account balance → reply prompting recharge
c) irrelevant email → ignore
d) execute locally and reply (e.g., need more info; user’s direct task request)
e) execute directly in third-party services (e.g., modify documents based on Google Docs / Slides / Overleaf comments) + reply in comments + email summary report (not a direct email reply, because the received email is a machine notification from a third-party app)
	•	After entering the execution layer:
	1.	load user history + current thread + attachments
	2.	evaluate role strategy: need clarification vs execute directly
	3.	run the toolchain, generate results/attachments/external document edits
	4.	return via email or the specified channel
	5.	if it’s a recurring task, write into the Scheduler

6. Multi-tenant Isolation Strategy (Key Problem)

Question: How do we ensure Alice can only access the current user’s data?

Recommended hard boundaries:
	•	Data isolation: each user has an independent Azure Blob Storage folder.
	•	Execution isolation:
	•	single container serving multiple users, but each run mounts only that user’s workspace (temporary directory).
	•	Runtime constraints:
	•	when accessing storage/search/indexing, only allow the workspace (temporary directory) corresponding to user_id;
	•	any cross-user resource access is rejected immediately and audited.

Borrowing from OpenClaw:
	•	when stronger isolation is needed, enable sandboxing (containerized tool execution).

7. “Next Step” Design After Alice Is Granted Shared Access

Question: After the user shares Notion / Docs with Alice, how does Alice move on?

For each kind of access permission, add an abstraction-layer workflow with safety guardrails, ensuring that when preparing the workspace it only loads information the user has already granted access to.

8. Recurring Tasks and Task Management
	•	each user has an independent Scheduler (supports CRON-like)
	•	task entities have a lifecycle:
	•	create, pause, cancel, retry on failure
	•	new tasks generated by recurring tasks still go through the unified queue and permission checks
	•	support “automatic denoising”: merge or summarize identical tasks within a short window (if similar content arrives repeatedly while the agent is still executing and hasn’t replied yet, treat the new info as additional instructions; after completion, reply once with a unified response)

9. Should We Build Directly on OpenClaw as a Secondary Development?

Recommended conclusion:
	•	Short-term prototype: borrow OpenClaw’s ideas—Gateway + Agent Loop + Queue + Multi-agent routing—to quickly build a PoC.
	•	Mid-to-long-term SaaS: you need to build your own control plane and multi-tenant security boundaries. OpenClaw is more local/single-host oriented; direct reuse will run into:
	•	insufficient multi-tenant permissions and isolation
	•	SaaS-grade ingress like Email/Docs/Notion needs dedicated connectors
	•	memory/file system must migrate to cloud object storage

Feasible approach:
	1.	Rebuild a “Gateway + Agent Loop + Queue” framework following OpenClaw’s design;
	2.	keep the concept of “multi-agent / multi-session isolation,” but cloudify storage and execution for multi-tenancy;
	3.	later, if private/on-prem deployment is needed, introduce OpenClaw’s native gateway as the runtime for a private edition.

10. Roadmap (Recommended)

Phase 0: Email + Research PoC (single role + simple tasks)
	•	Gmail/SMTP ingress + task classification + replies
	•	basic research pipeline + attachment return

Phase 1: Multi-role + scheduling + Docs/Notion integration
	•	role strategy packages + task scheduler
	•	Docs/Notion comment-driven tasks

Phase 2: Team collaboration (TPM mode)
	•	multi-user task assignment
	•	task prioritization and reminder system
	•	Slack/Zoom ingress

Phase 3: Tool marketplace
	•	pluggable tools and automatic selection of the optimal tool
	•	org-level workflow templates and KPIs