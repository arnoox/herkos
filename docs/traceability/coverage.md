# Coverage Analysis

This page provides an overview of traceability coverage across the herkos project.

## Wasm 1.0 Spec Features

All Wasm 1.0 specification features tracked by this project.

```{needtable}
:style: table
:columns: id;title;wasm_section;tags
:filter: type == 'wasm_spec'
:sort: id
```

## Requirements

All herkos requirements with their outgoing links to Wasm spec features.

```{needtable}
:style: table
:columns: id;title;status;links
:filter: type == 'req'
:sort: id
```

## Specification Sections

Herkos specification sections with their satisfaction links.

```{needtable}
:style: table
:columns: id;title;satisfies
:filter: type == 'spec'
:sort: id
```

## Implementation Modules

Implementation modules with their requirement and spec links.

```{needtable}
:style: table
:columns: id;title;satisfies;implements
:filter: type == 'impl'
:sort: id
```

## Test Cases by File

Test cases with their verification links to Wasm spec features.

```{needtable}
:style: table
:columns: id;title;verifies
:filter: type == 'test'
:sort: id
```
