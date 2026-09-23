# Domain Language

Seed uses its vocabulary as architecture, not branding. Each term names a role in the application;
prefer the canonical term over synonyms.

## Terms

- **Seed** — the placeholder product this skeleton instantiates into. Replace this list with the
  product's own terms, one role per term, each stating what the core owns and what the domain
  supplies.

## Rules

- A new term is settled through an OpenSpec change and recorded here before code uses it.
- A term that names a mechanism a composed brick owns does not belong in the application's
  vocabulary; use the brick's term.
