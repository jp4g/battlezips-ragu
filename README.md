# BattleZips-Ragu

Zero-knowledge Battleship on [Ragu](https://github.com/tachyon-zcash/ragu), the
Rust proof-carrying data (PCD) framework built for Zcash's Project Tachyon.

This is the next entry in the BattleZips series
([Circom](https://github.com/BattleZips/battlezip-frontend),
[Halo2](https://github.com/BattleZips/BattleZips-Halo2),
[Noir](https://github.com/BattleZips/BattleZips-Noir),
[Plonky2](https://github.com/BattleZips/BattleZips-Plonky2)). Each one proves
the same thing: a player committed to a valid fleet, and every shot they answer
is honestly reported as a hit or a miss.

> **Status: exploration / design.** No circuit code yet. Read the docs below,
> then [PLAN.md](PLAN.md) for the roadmap.

## Goal of the first proof of concept

A browser app where two players:

1. place a fleet and commit to it with a proof that the fleet is valid,
2. take turns shooting at `(x, y)`,
3. answer each shot with a Ragu proof that it hits or misses the committed board,
4. exchange those proofs over a WebSocket relay,
5. win by collecting 17 verified hits (5 + 4 + 3 + 3 + 2).

Smart contracts and a full game-state PCD chain come later.

## Docs

| Doc | Contents |
| --- | --- |
| [docs/01-prior-art.md](docs/01-prior-art.md) | How the earlier BattleZips circuits (Halo2, Noir) work and what to carry over |
| [docs/02-ragu-primer.md](docs/02-ragu-primer.md) | Ragu concepts that matter here, its limits, and open risks (**including zero-knowledge**) |
| [docs/03-circuit-design.md](docs/03-circuit-design.md) | Proposed board and shot steps, with gate budgets |
| [docs/04-architecture.md](docs/04-architecture.md) | PCD graph, message protocol, and web app layout |
| [PLAN.md](PLAN.md) | Phased roadmap and open questions |

## Important caveat

Ragu is under heavy development and unaudited. It **does not yet provide zero
knowledge**: uncompressed proofs carry the witness polynomials, and compressed
proofs carry evaluations of them. A hidden-information game depends on that
property, so this demo is a correctness and ergonomics exercise, not a private
game, until upstream lands hiding. See
[docs/02-ragu-primer.md](docs/02-ragu-primer.md#zero-knowledge-status).

## Reference versions

| Repo | Commit |
| --- | --- |
| tachyon-zcash/ragu | `3219e086` (2026-11-04, `main`) |
| BattleZips/BattleZips-Halo2 | `419d8c9` |
| BattleZips/BattleZips-Noir | `99beb27` |

Ragu needs Rust **1.97.1** (pinned in its `rust-toolchain.toml`).

## License

MIT OR Apache-2.0, matching Ragu.
