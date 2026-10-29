---
Register: episteme
Standing: architecture-contract
---

# Local governing authority storage

`actuation authority issue/resolve/revoke/list` keep the owner's governing
records in an authority store: `--store`, `ACTUATION_AUTHORITY_STORE` or
`~/.actuation/authority`. The
[store](../crates/actuation-stream/src/authority_store.rs) holds one
append-only history per source: one Issue event, then at most one Revoke.

## Identity

A source is the literal `authority_source_ref` its Issue record names. Where a
history lies only helps find it; it never makes a record another source's
authority. `issue` refuses a record that does not name the source it is issued
under, and refuses an empty ref.

## Layout

Each source's history is

```text
sources-v2/<hex>/events.jsonl
```

`<hex>` is every UTF-8 byte of the ref as lowercase hex, cut into directory
names of at most 180 characters. Two refs can never share a file: nothing is
normalised, Unicode and punctuation survive, case-insensitive filesystems
cannot fold two refs together, and a ref longer than a filename is a deeper
path rather than an error.

Before V2 the store kept `<key>.jsonl` at its root, where the key replaced
every character outside `A-Z a-z 0-9 - _ .` with `_`. So
`authority-source:x`, `authority-source/x` and `authority-source_x` shared one
file: the second to issue was refused as already existing, and a read or
revoke of any of them reached the first one's authority.

## Pre-V2 stores

A store the pre-V2 binary wrote reads as it is.

- Pre-V2 files are read in place and never moved or rewritten.
- A revocation of a source held in a pre-V2 file is appended to that file, as
  the pre-V2 binary would have appended it.
- Every new issue goes into `sources-v2/`.
- A pre-V2 file belongs to the source its own record names. If another ref
  shares its key, that ref is a different source: it reads as unknown, has
  nothing to revoke, and issues into `sources-v2/` beside it.
- A pre-V2 file under a key that its record's source does not map to is
  misfiled. It admits nothing (see below).

Nothing is migrated, so a pre-V2 binary still on the machine keeps seeing every
pre-V2 authority and any revocation made since. It cannot see sources issued
into `sources-v2/`. If it issues a ref that already has a V2 history, that
source has two histories; the store keeps both and refuses the source until
the owner resolves it. Retire pre-V2 binaries once every one that writes the
store is upgraded.

## Folding a history

Every read checks the whole history. A history refuses, and nothing is folded
around it, when:

- a line is not a complete event, or the final line has no newline;
- an event's schema is not `actuation.authority-event/v1`;
- the Issue record names no `authority_source_ref`;
- there is a second Issue, a Revoke before the Issue, or a second Revoke;
- the history names a source other than the one requested;
- the source has both a V2 history and a pre-V2 file naming it.

A file with no events, such as one left by a crash between creating a history
and writing its Issue, holds no authority: the source reads as unknown and a
retried issue completes it.

The pre-V2 binary folded leniently: the last Issue won and a Revoke counted
wherever it stood. No history that binary wrote under its lock breaks these
rules. A store written before it took the lock could hold duplicate events.
Such a source now refuses, and its bytes stay where they are.

## Resolution

`authority resolve` returns `unavailable` with code
`authority.source_identity_mismatch` when the history found for the requested
ref names another source. The store's error carries that cause as a typed
source (`SourceIdentityMismatch`). Resolution also compares the record's
`authority_source_ref` with the requested ref before revocation, expiry or
holder.
Other unreadable histories stay `authority.source_unavailable`.

## Listing

`authority list` walks the pre-V2 files and the `sources-v2/` tree. Each listed
authority folds exactly as `read` folds it, in ref order. A file that cannot be
folded, holds no Issue, lies where its record's source does not, sits on a
path that does not encode a ref, cannot be opened without following a link, or
is one of two histories for one source is listed under `invalid` with its
reason. Nothing is dropped.

## Durability and safety

Every read-check-append holds a lock on the store directory: exclusive to
issue or revoke, shared to read or list. Every path component is opened from a
held directory descriptor without following symlinks, so a planted link
cannot redirect a history out of the store, and a long ref never meets the
system's whole-path limit. Appends are `sync_data`'d. A new directory has its
parent synced before anything is created inside it, and a new history file has
its directory synced after its first line. New V2 directories are created
`0700` and new history files `0600`.
