# wasixv2-abi

The wasixv2 guest ABI, the Linux syscall ABI for wasm32 guests, and the generator for its
bindings.

- `wit/`: the ABI, written as annotated WIT. [`wit/README.md`](wit/README.md) describes the
  conventions and every annotation.
- `src/`: `wasixv2-codegen`, which reads the WIT and generates the host-side Rust (the ABI types,
  the `Syscalls` trait and the wasm import shims) and the musl C glue.
- `tests/`: golden tests for the generator, including a snapshot of the C glue for the real ABI,
  so that an ABI change shows up in review as a C diff.

This repository is a git submodule of its consumers. Generated code is committed in the
consumers, never here:

- [wasmer](https://github.com/wasmerio/wasmer) mounts it at `lib/wasixv2/codegen`, where it is a
  member of the cargo workspace. wasmer's `lib/wasixv2/abi/tests/generated.rs` checks that its
  generated crates match the WIT at the pinned commit.
- [libc2](https://github.com/wasix-org/libc2) mounts it at `wasix2/abi`.

## Changing the ABI

An ABI change lands here first. Each consumer then bumps its submodule to the same commit and
regenerates in the same commit as the bump. Keep the two pins in step: wasmer and libc2 only
interoperate when they were generated from the same ABI.

From a wasmer checkout:

```sh
WASIXV2_CODEGEN_BLESS=1 cargo test -p wasixv2-abi --test generated
```

From a libc2 checkout:

```sh
cargo run --manifest-path wasix2/abi/Cargo.toml -- c --wit wasix2/abi/wit --libc .
```

Here, `cargo test` runs the generator's own tests, and `WASIXV2_CODEGEN_BLESS=1 cargo test`
rewrites its golden files.
