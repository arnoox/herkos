Metamodel
=========

This document describes the **quality metamodel** used in the herkos project —
the need types, custom fields, and link types that together form the
traceability structure.

All configuration lives in ``ubproject.toml`` and is consumed by both
Sphinx-Needs (for documentation builds) and ubCode (for live analysis in the
editor).

.. contents::
   :local:
   :depth: 2

V-model overview
----------------

The herkos traceability follows a **V-model** structure. The left branch
decomposes the external Wasm specification into increasingly concrete
artifacts. The right branch provides verification at each level.

**Fundamental rule: links always point upward.** Every traceability link is
authored on the *lower-level* artifact and points *up* to the higher-level
artifact it derives from. Higher-level artifacts never depend on or reference
lower-level ones. This keeps each level self-contained and independently
reviewable.

Development branch (decomposition)
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

The left side of the V breaks the external specification down into
project artifacts, from abstract to concrete:

.. mermaid::

   flowchart TB
       WASM["🔵 Wasm Spec (WASM_)<br/>External reference baseline"]
       REQ["🟢 Requirement (REQ_)<br/>What the system must do"]
       SPEC["🟡 Specification (SPEC_)<br/>How the design realizes it"]
       IMPL["🟠 Implementation (IMPL_)<br/>Actual source code artifacts"]

       IMPL -- "satisfies" --> REQ
       IMPL -- "implements" --> WASM
       SPEC -- "satisfies" --> REQ
       REQ -. "links" .-> WASM

       style WASM fill:#9DC3E6,stroke:#333,color:#000
       style REQ fill:#A9D18E,stroke:#333,color:#000
       style SPEC fill:#FFD966,stroke:#333,color:#000
       style IMPL fill:#F4B183,stroke:#333,color:#000

Each arrow represents a traceability link authored on the source node,
pointing up toward the target it derives from.

Verification branch
^^^^^^^^^^^^^^^^^^^

The right side of the V provides evidence that each level behaves correctly:

.. mermaid::

   flowchart TB
       WASM["🔵 Wasm Spec (WASM_)"]
       REQ["🟢 Requirement (REQ_)"]
       SPEC["🟡 Specification (SPEC_)"]
       IMPL["🟠 Implementation (IMPL_)"]
       TEST["🟣 Test (TEST_)<br/>Verification evidence"]

       TEST -- "verifies" --> WASM
       TEST -- "verifies" --> REQ
       TEST -- "verifies" --> SPEC
       TEST -- "verifies" --> IMPL

       style WASM fill:#9DC3E6,stroke:#333,color:#000
       style REQ fill:#A9D18E,stroke:#333,color:#000
       style SPEC fill:#FFD966,stroke:#333,color:#000
       style IMPL fill:#F4B183,stroke:#333,color:#000
       style TEST fill:#C5B0D5,stroke:#333,color:#000

Tests can verify artifacts at any level. A test always carries the
``:verifies:`` link, pointing up to the artifact it validates.

Need types
----------

The project defines five need types, each representing a distinct level
in the V-model:

.. list-table::
   :header-rows: 1
   :widths: 20 15 15 15 35

   * - Type
     - Directive
     - Prefix
     - Color
     - Purpose
   * - **Wasm Spec**
     - ``.. wasm_spec::``
     - ``WASM_``
     - .. raw:: html

          <span style="background:#9DC3E6;padding:2px 8px;border-radius:3px;">#9DC3E6</span>
     - Captures individual clauses and rules from the
       `WebAssembly 1.0 specification <https://www.w3.org/TR/wasm-core-1/>`_.
       These items are the external reference baseline — the top of the V.
       They are never authored with outgoing links; they only *receive*
       incoming links from lower levels.
   * - **Requirement**
     - ``.. req::``
     - ``REQ_``
     - .. raw:: html

          <span style="background:#A9D18E;padding:2px 8px;border-radius:3px;">#A9D18E</span>
     - Project-level requirements derived from or motivated by the Wasm spec.
       Each requirement states *what* the system must do. Requirements
       reference Wasm spec items via ``:links:``.
   * - **Specification**
     - ``.. spec::``
     - ``SPEC_``
     - .. raw:: html

          <span style="background:#FFD966;padding:2px 8px;border-radius:3px;">#FFD966</span>
     - Design-level specifications that describe *how* a requirement is
       realized in the herkos architecture. Specifications link upward to
       requirements via ``:satisfies:``.
   * - **Implementation**
     - ``.. impl::``
     - ``IMPL_``
     - .. raw:: html

          <span style="background:#F4B183;padding:2px 8px;border-radius:3px;">#F4B183</span>
     - Traces to actual source code artifacts (functions, modules, structs)
       that realize the design. Implementations link upward to requirements
       via ``:satisfies:`` and to Wasm spec items via ``:implements:``.
   * - **Test**
     - ``.. test::``
     - ``TEST_``
     - .. raw:: html

          <span style="background:#C5B0D5;padding:2px 8px;border-radius:3px;">#C5B0D5</span>
     - Test cases that provide verification evidence. Tests link upward to
       the artifact they validate via ``:verifies:``.

Custom fields
-------------

In addition to the built-in fields (``id``, ``title``, ``status``, ``tags``),
three custom fields are defined:

.. list-table::
   :header-rows: 1
   :widths: 20 15 65

   * - Field
     - Default
     - Description
   * - ``wasm_section``
     - ``""``
     - Reference to a section in the WebAssembly specification
       (e.g. ``§5.4.1``). Used primarily on ``wasm_spec`` needs.
   * - ``source_file``
     - ``""``
     - Path to the source file that contains the relevant code.
       Used primarily on ``impl`` needs.
   * - ``wasm_opcode``
     - ``""``
     - Name of the WebAssembly opcode (e.g. ``i32.add``).
       Used on ``wasm_spec`` and ``req`` needs that relate to a specific
       instruction.

Link types
----------

Three directed link types express the traceability relationships between
needs. **All links are authored on the lower-level artifact, pointing upward**
to the higher-level artifact it derives from or validates.

.. list-table::
   :header-rows: 1
   :widths: 18 22 22 38

   * - Link type
     - Outgoing label
     - Incoming label
     - Semantics
   * - ``satisfies``
     - *satisfies*
     - *is_satisfied_by*
     - A lower-level artifact *satisfies* a higher-level one.
       Used by ``SPEC_`` and ``IMPL_`` to link upward to ``REQ_`` items.
       Example: ``SPEC_MEMORY_MODEL`` *satisfies* ``REQ_MEM_PAGE_MODEL``.
   * - ``implements``
     - *implements*
     - *is_implemented_by*
     - An ``IMPL_`` artifact *implements* a ``WASM_`` spec item, showing
       that a specific external specification clause is realized in code.
       Example: ``IMPL_RUNTIME_MEMORY`` *implements* ``WASM_MEMORY_TYPE``.
   * - ``verifies``
     - *verifies*
     - *is_verified_by*
     - A ``TEST_`` artifact *verifies* a higher-level artifact, providing
       evidence that it behaves correctly. Can target ``WASM_``, ``REQ_``,
       ``SPEC_``, or ``IMPL_`` items.
       Example: ``TEST_ARITHMETIC_ADD_CORRECTNESS`` *verifies* ``WASM_I32_ADD``.

In addition, the built-in ``:links:`` option is used by ``REQ_`` needs to
reference related ``WASM_`` spec items (a weaker association than
``satisfies``).

Link direction principle
^^^^^^^^^^^^^^^^^^^^^^^^

The direction rule is essential for maintaining the independence of each
V-model level:

- **Higher levels never reference lower levels.** A ``WASM_`` item has no
  knowledge of which ``REQ_`` items exist. A ``REQ_`` item has no knowledge
  of which ``SPEC_`` or ``IMPL_`` items realize it.
- **Lower levels always link upward.** The ``SPEC_``, ``IMPL_``, and
  ``TEST_`` directives carry the ``:satisfies:``, ``:implements:``, and
  ``:verifies:`` options respectively.
- **Sphinx-Needs computes the reverse.** When ``SPEC_X`` declares
  ``:satisfies: REQ_Y``, Sphinx-Needs automatically shows ``REQ_Y`` as
  *is_satisfied_by* ``SPEC_X``. No manual reverse links are needed.

This ensures that requirements can be reviewed and baselined independently
of how — or whether — they have been implemented or tested.

Traceability chain
------------------

The complete V-model traceability, showing link directions as authored
(lower → higher):

.. list-table::
   :header-rows: 1
   :widths: 20 25 20 35

   * - Source (lower)
     - Link option
     - Target (higher)
     - Meaning
   * - ``REQ_``
     - ``:links:``
     - ``WASM_``
     - Requirement references the Wasm spec clauses it addresses
   * - ``SPEC_``
     - ``:satisfies:``
     - ``REQ_``
     - Specification satisfies a requirement
   * - ``IMPL_``
     - ``:satisfies:``
     - ``REQ_``
     - Implementation satisfies a requirement
   * - ``IMPL_``
     - ``:implements:``
     - ``WASM_``
     - Implementation realizes a Wasm spec clause in code
   * - ``TEST_``
     - ``:verifies:``
     - ``WASM_`` / ``REQ_`` / ``SPEC_`` / ``IMPL_``
     - Test provides verification evidence for any level

ID convention
-------------

All need IDs must match the regex ``^[A-Z][A-Z0-9_]+`` — they start with an
uppercase letter followed by one or more uppercase letters, digits, or
underscores. Each type uses its prefix (``WASM_``, ``REQ_``, ``SPEC_``,
``IMPL_``, ``TEST_``) to make the artifact type immediately recognizable
from the ID alone.

Coverage analysis
-----------------

The :doc:`coverage` page uses Sphinx-Needs filters to identify gaps in
the traceability chain — requirements without tests, specifications without
implementations, and Wasm spec items without corresponding requirements.
