# DEWRENCH Protection Delegation Manifesto

**Status:** Foundational Architecture Contract  
**Scope:** Core, modules, IPC, system access, credentials, destructive actions, future plugins and AI agents  
**Principle:** Security must be architectural before it becomes feature-specific.

---

## 1. Purpose

DEWRENCH is designed to become an interface between a human and increasingly powerful system capabilities.

Git operations, filesystem access, process management, Docker, databases, Kubernetes, Terraform, CI/CD, Redis, Kafka, monitoring tools and future modules may each possess enough authority to alter, destroy, expose or remotely propagate system state.

For this reason, DEWRENCH must not depend on every module being individually perfect.

Protection must exist **before modules reach the system**.

The objective of this architecture is not to prevent the owner of a machine from performing dangerous actions deliberately.

Its objective is to prevent:

- unintended destructive actions;
- privilege escalation through DEWRENCH;
- modules bypassing safety controls;
- compromised frontend code acquiring unrestricted backend authority;
- unsafe projects executing automatically;
- secrets leaking through UI, logs or child processes;
- race conditions causing state divergence;
- approvals authorizing a different action from the one reviewed;
- new modules inventing their own security model;
- future AI agents receiving unrestricted system authority.

DEWRENCH must make powerful operations understandable, inspectable and deliberate.

---

# 2. Constitutional Rule

No DEWRENCH module owns system authority.

A module may describe an intention.

The DEWRENCH Core decides whether that intention may become an effect.

The intended architecture is:

```text
User / Agent
     │
     ▼
Frontend
     │
     │ Intent
     ▼
IPC Boundary
     │
     ▼
Security Core
     │
     ├── Identity / Actor
     ├── Context Resolution
     ├── Capability Evaluation
     ├── Resource Resolution
     ├── Path Validation
     ├── Risk Classification
     ├── Preflight
     ├── Approval
     ├── Resource Lock
     ├── Secret Handling
     ├── Privilege Handling
     ├── Safe Execution
     ├── Verification
     └── Audit / Recovery Metadata
     │
     ▼
Module Adapter
     │
     ▼
Operating System / External Tool / Remote Service
```

Direct shortcuts around this path are architectural violations.

---

# 3. Protection Is a Core Responsibility

Security must not become a future module called `security`.

It is a transversal property of the application.

Git must obey it.

Docker must obey it.

Database tools must obey it.

Kubernetes must obey it.

Terraform must obey it.

Plugins must obey it.

AI agents must obey it.

The Security Core must remain independent from the implementation details of these modules whenever possible.

Modules understand their domains.

The Security Core understands authority.

---

# 4. Deny by Default

Unknown authority is denied.

Unknown capability is denied.

Unknown resource scope is denied.

Expired approval is denied.

Changed execution context invalidates authorization.

Failure while evaluating security must not silently become permission.

```text
UNKNOWN → DENY
ERROR   → DENY
STALE   → DENY
INVALID → DENY
```

A module must never gain authority merely because no rule exists forbidding it.

Authority must be explicitly granted.

---

# 5. Intent Instead of Arbitrary Execution

Frontend and modules should communicate through structured intentions instead of unrestricted commands.

Avoid interfaces equivalent to:

```text
execute(any_string)
delete(any_path)
connect(any_address)
run(any_binary)
```

Prefer domain actions:

```text
GitPush
GitSwitchBranch
DockerRemoveContainer
DatabaseExecuteMutation
KubernetesDeleteDeployment
TerraformApplyPlan
```

The system should know the semantic meaning of an operation before executing it.

Security decisions cannot reliably protect operations the Core cannot understand.

---

# 6. Stable Resource Identity

Where practical, frontend code should operate on resolved resource identifiers rather than unrestricted raw paths, URLs or command strings.

For example:

```text
RepoId
WorkspaceId
RemoteId
ContainerId
DatabaseConnectionId
ClusterId
CredentialRef
```

The backend resolves those identifiers to their real targets.

This prevents the frontend from redefining the meaning of an already-authorized resource during execution.

---

# 7. Capability Model

Every powerful operation must correspond to an explicit capability.

Examples include:

```text
git.read
git.local.write
git.remote.read
git.remote.write
git.history.rewrite

filesystem.project.read
filesystem.project.write
filesystem.external.read
filesystem.external.write
filesystem.system.write

process.inspect
process.spawn
process.kill

network.github
network.external

credential.use
credential.export

privilege.elevate
```

Capabilities describe authority.

They do not describe risk.

An action may be both authorized and extremely dangerous.

These concepts must remain separate.

---

# 8. Risk Model

The Risk Engine determines how much ceremony an authorized action requires.

A possible baseline:

```text
OBSERVE
LOW
MEDIUM
HIGH
CRITICAL
PRIVILEGED
```

Risk must be contextual.

The same operation against a disposable local environment and against production may receive different classifications.

Risk affects:

```text
preview requirements
confirmation requirements
preflight requirements
audit requirements
recovery requirements
privilege requirements
```

Risk does not grant authority.

---

# 9. Preflight Is a Snapshot, Not a Question

Dangerous actions must not be reduced to:

```text
Are you sure?
YES / NO
```

A preflight describes the exact state the user is approving.

It should identify, when relevant:

```text
actor
operation
target
current state
expected mutation
environment
remote destination
risk
recoverability
resource version/hash
```

Approval must refer to that preflight.

If material state changes between review and execution, approval becomes invalid.

```text
PREPARE
   ↓
PREFLIGHT
   ↓
APPROVE EXACT STATE
   ↓
REVALIDATE
   ↓
EXECUTE
```

Approval of action A must never silently authorize action B.

---

# 10. Process Execution Boundary

Process creation is privileged infrastructure.

Modules must not independently invent process execution behavior.

The Core should provide a controlled process broker responsible for:

```text
executable resolution
argument construction
working directory
environment inheritance
timeouts
cancellation
stdout/stderr handling
secret redaction
exit status
resource limits where available
```

Shell execution must not be the default abstraction.

Structured arguments are preferred to interpolated command strings.

Any future direct process execution outside approved infrastructure must be treated as a security-sensitive architecture exception.

---

# 11. Filesystem Boundary

Filesystem access must be evaluated against canonical resource scope, not merely textual paths.

Path security must consider platform-specific behavior, including:

```text
relative traversal
symlinks
junctions
reparse points
canonical parents
case behavior
UNC/network paths
nonexistent targets
filesystem boundaries
```

The question is not:

> Does this string look safe?

The question is:

> What object will this operation actually reach?

---

# 12. Secrets Are References

Modules and frontend code should avoid treating credentials as ordinary strings.

Prefer:

```text
CredentialRef
SecretRef
CredentialHandle
```

over:

```text
password: String
token: String
private_key: String
```

The final credential storage mechanism may evolve.

The architectural contract must exist before that choice.

Secrets should be revealed only to the smallest component that requires them and only for the duration necessary.

Secrets must never intentionally enter normal logs.

---

# 13. Privilege Is Borrowed

DEWRENCH should not normally run permanently as administrator or root.

When elevation is genuinely required, privilege should be acquired for the smallest possible operation and released immediately afterward.

```text
normal DEWRENCH
      │
      ▼
Privilege Broker
      │
      ▼
one authorized operation
      │
      ▼
normal DEWRENCH
```

Privileged application state must not become the default operating condition.

---

# 14. Workspace Trust

Opening a project does not imply executing it.

DEWRENCH must distinguish between inspecting a workspace and trusting executable content inside that workspace.

A repository may initially be considered readable while scripts, binaries, hooks, installers or infrastructure definitions remain untrusted for execution.

Examples of content that must never become automatically trusted merely because a workspace was opened include:

```text
package installation scripts
Git hooks
Makefiles
shell scripts
Dockerfiles
Compose definitions
Terraform configuration
CI definitions
downloaded binaries
project-local executables
```

Data may be inspected before it is trusted as executable behavior.

---

# 15. Resource Locking and Concurrency

Security also includes protection against DEWRENCH itself racing against DEWRENCH.

Mutating operations against the same logical resource should use controlled locking where appropriate.

Examples:

```text
repository
workspace filesystem
database
container
infrastructure state
```

Operations should also support idempotency where meaningful.

Double clicks, duplicated IPC requests or retries must not accidentally become duplicated destructive actions.

---

# 16. Auditability

Every meaningful mutation should produce structured evidence.

The audit system should be capable of representing:

```text
ActionRequested
ActionDenied
ActionPrepared
ActionApproved
ActionStarted
ActionSucceeded
ActionFailed
ActionVerified
```

Audit records describe actions.

They must not become a second credential database.

Secrets are redacted.

Audit data may later power DEWRENCH's visual history and Temporal Matrix, but its first responsibility is factual reconstruction.

---

# 17. Recovery Must Never Be Invented

DEWRENCH must distinguish between:

```text
reversible
recoverable with prerequisites
partially recoverable
irreversible
unknown
```

The application must never claim rollback guarantees that the underlying system cannot provide.

A destructive operation without guaranteed recovery may still be allowed.

It must simply be represented truthfully.

Trust is created by accurate limits, not false reassurance.

---

# 18. AI Agent Authority

AI agents are not trusted execution authorities.

An AI may:

```text
inspect
analyze
recommend
generate
propose
review
attack within authorized labs
```

An AI should not automatically receive unrestricted:

```text
shell access
filesystem authority
credentials
administrator privileges
production access
remote write authority
```

Agents should produce structured action proposals that pass through the same Security Core as human actions.

AI must never become a privileged shortcut around normal DEWRENCH policy.

---

# 19. Delegation of Security Responsibilities

DEWRENCH security uses role separation deliberately.

## Project Owner / Architect

The human owner defines intended behavior, risk tolerance, accepted architecture and final merge decisions.

No AI agent receives constitutional authority over the project.

Agents produce implementation and evidence.

The owner decides what becomes DEWRENCH.

## Codex — Backend Developer

Codex is responsible for implementing backend architecture and migrating existing backend behavior into protected infrastructure.

Primary concern:

> Make the intended system work correctly.

## DeepSeek — Backend QA

DeepSeek validates expected behavior, edge cases, regression, compatibility and contract correctness.

Primary concern:

> Find where the implementation fails to behave as intended.

## Claude — Red Team

Claude treats the implemented system as adversarial territory.

Primary concern:

> Find ways to make the implementation behave in ways it was not intended to behave.

Claude must not declare a component universally secure.

Acceptable conclusions include:

```text
Exploit reproduced.
Exploit not reproduced.
No exploit found within tested scope.
Residual concerns remain.
```

## Codex — Blue Team

Blue Team work must be performed with a defensive context distinct from normal implementation whenever practical.

Primary concern:

> Remove the class of vulnerability, not merely the demonstrated payload.

A security patch is incomplete until it has been subjected to regression testing and adversarial retesting.

---

# 20. Security Issue Lifecycle

Security work follows:

```text
BUILD
  ↓
FUNCTIONAL QA
  ↓
RED TEAM
  ↓
FINDING
  ↓
BLUE TEAM
  ↓
REGRESSION QA
  ↓
RED TEAM RETEST
  ↓
CLOSE
```

A reported exploit is not closed because a patch compiles.

It is closed when:

```text
the original reproduction fails;
the intended behavior still works;
relevant regression tests pass;
reasonable variant attacks were considered;
the final evidence is recorded.
```

---

# 21. Security Evidence Over Agent Confidence

Statements such as:

```text
looks secure
should be safe
probably fixed
```

are not sufficient security evidence.

Prefer:

```text
test executed
attack attempted
input used
expected result
observed result
affected commit
patch commit
regression test
residual risk
```

Confidence is useful.

Reproduction is better.

---

# 22. No Big-Bang Security Refactor

Protection infrastructure must not destroy the working product in order to protect the future product.

Migration follows the strangler principle.

Existing behavior is progressively routed through protected infrastructure.

The old path is removed only after the protected path reproduces required behavior.

The Git module becomes the first reference implementation.

```text
CURRENT GIT
     │
     ▼
document behavior
     │
     ▼
introduce Core primitive
     │
     ▼
route one operation through primitive
     │
     ▼
test parity
     │
     ▼
remove old route
     │
     ▼
next operation
```

No giant rewrite of every Git action simultaneously.

---

# 23. Existing Behavior Is a Migration Constraint

Security refactoring must preserve known-good functionality unless a behavior is explicitly identified as unsafe and intentionally changed.

Before migrating a behavior, establish:

```text
current inputs
current outputs
expected side effects
known error behavior
current tests
known limitations
```

If behavior changes for security reasons, document that change.

Accidental behavior drift is considered a regression.

---

# 24. Architecture Enforcement

Rules that can be enforced mechanically should not depend exclusively on documentation.

CI and architecture tests should eventually detect forbidden patterns such as direct system access from modules where a protected Core primitive exists.

Examples may include:

```text
direct process spawning
direct destructive filesystem calls
direct privilege elevation
raw secret persistence
security-sensitive IPC outside approved registration
```

The objective is not stylistic purity.

The objective is ensuring the secure path is easier than bypassing it accidentally.

---

# 25. Reference Module Strategy

Git is the first module used to validate the Security Core.

The migration is successful only if real Git workflows remain usable through protected infrastructure.

Once Git proves the model, future modules inherit the pattern.

```text
Git          → reference
Docker       → adopt
DB Viewer    → adopt
Kubernetes   → adopt
Terraform    → adopt
CI/CD        → adopt
Redis        → adopt
Kafka        → adopt
future       → adopt
```

New modules should not introduce an alternative authority architecture without an explicit architectural decision.

---

# 26. Migration Order

The initial protection refactor should proceed in this order:

```text
1. Freeze and document current Git behavior.

2. Establish common security types:
   ActionId
   Actor
   ResourceId
   Capability
   Risk
   ExecutionContext

3. Implement path-security infrastructure.

4. Implement controlled process execution.

5. Implement capability evaluation.

6. Implement resource locking/idempotency primitives.

7. Implement preflight and approval contracts.

8. Implement structured audit/redaction.

9. Introduce workspace-trust primitives.

10. Create interfaces for future:
    SecretBroker
    PrivilegeBroker
    NetworkBroker
    RecoveryProvider
    SandboxProvider

11. Migrate one Git operation.

12. Verify behavior parity.

13. Migrate remaining Git operations incrementally.

14. Add architecture-enforcement tests.

15. Declare Git the protected reference module.

16. Only then begin integrating new high-authority modules.
```

---

# 27. Stop Conditions

A refactor stage must stop rather than continue blindly when:

```text
existing behavior becomes unexplained;
tests cannot distinguish regression from intended change;
a protected abstraction requires unrestricted authority to function;
a security rule is being bypassed merely to finish migration;
the migration requires unrelated large architectural changes;
rollback to the last working state becomes unclear.
```

When one of these conditions occurs, the architecture must be reassessed before additional migration.

Progress is not measured by lines moved.

Progress is measured by authority successfully placed behind controlled boundaries.

---

# 28. Definition of Protected

A DEWRENCH operation may be considered protected when:

```text
its actor is known;
its target is resolved;
its required capabilities are explicit;
its context is known;
its risk is classified;
unsafe paths cannot bypass validation;
external execution passes through controlled infrastructure;
required approvals are bound to the reviewed state;
concurrent mutation is handled where necessary;
secrets are not unnecessarily exposed;
the result is verified where practical;
the action is auditable;
known failure behavior is safe;
tests prove intended behavior remains functional.
```

This definition will evolve.

Its purpose is to prevent "security implemented" from becoming a vague status label.

---

# 29. Future Modules Inherit Protection

Future module development begins with three questions:

```text
What resources does this module control?

What capabilities does it require?

What effects can it cause?
```

Not:

```text
How do we add security after implementing it?
```

The protection architecture exists before the soldier.

The soldier receives the shield because the shield's attachment points were already designed.

---

# 30. Final Principle

DEWRENCH should never require blind trust from its user.

It should earn trust by making authority explicit.

The application must know:

```text
who is acting;
what is being requested;
which resource will be affected;
what authority is required;
what may change;
how dangerous the change is;
whether state has changed since approval;
what actually happened;
whether recovery is possible.
```

The goal of the protection architecture is therefore not:

> Prevent everything dangerous.

The goal is:

> Make accidental danger difficult, unauthorized authority unavailable, and deliberate power understandable.

**DEWRENCH may become powerful.  
Its authority must never become casual.**