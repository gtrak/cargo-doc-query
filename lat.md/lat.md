This directory defines the high-level concepts, business logic, and architecture of this project using markdown. It is managed by [lat.md](https://www.npmjs.com/package/lat.md) — a tool that anchors source code to these definitions. Install the `lat` command with `npm i -g lat.md` and run `lat --help`.

- [[type-expansion]] — resolving a qualified path into an expandable tree of types, fields, variants, and module items with cycle detection, depth limiting, and token budgeting
- [[query-engine]] — executes path-based queries against the cached rustdoc index
- [[token-budgeting]] — controls output size via token budget estimation
- [[crate-loading]] — loading rustdoc JSON from global cache with binary presence semantics
- [[generic-rendering-fidelity]] — generic type rendering fidelity guarantees
- [[rendering]] — output rendering rules
- [[path-resolution]] — resolving items by qualified path or stable identifier against the crate index
- [[filter-engine]] — deterministic inclusion/exclusion decisions over query results
- [[two-tier-caching]] — path-based caching for crate documentation with self-healing on corruption
- [[error-handling]] — single dispatch boundary for error categorization and exit signaling
- [[type-suggestion]] — fuzzy-text suggestions from a known identifier set
- [[build-pipeline]] — producing queryable index of dependencies by building and caching rustdoc JSON
