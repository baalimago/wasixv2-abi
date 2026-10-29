# wasixv2-abi

The wasixv2 guest ABI, the Linux syscall ABI for wasm32 guests, and the generator for its
bindings.

- `wit/`: the ABI, written as annotated WIT. [`wit/README.md`](wit/README.md) describes the
  conventions and every annotation.
- `src/`: `wasixv2-codegen`, which reads the WIT and generates the host-side Rust (the ABI types,
  the `Syscalls` trait and the wasm import shims), the musl C glue and the syscall glue of Go's
  `wasix/wasm` port.
- `tests/`: golden tests for the generator, including snapshots of the C and Go glue for the real
  ABI, so that an ABI change shows up in review as a C and Go diff.

This repository is a git submodule of its consumers. Generated code is committed in the
consumers, never here:

- [wasmer](https://github.com/wasmerio/wasmer) mounts it at `lib/wasixv2/codegen`, where it is a
  member of the cargo workspace. wasmer's `lib/wasixv2/abi/tests/generated.rs` checks that its
  generated crates match the WIT at the pinned commit.
- [libc2](https://github.com/wasix-org/libc2) mounts it at `wasixv2/abi`.
- Go's `wasix/wasm` port ([wasix-org/go](https://github.com/wasix-org/go)) commits the generated
  `zsyscall_wasix_wasm.go` and `zsysnum_wasix_wasm.go`; it does not mount this repository.

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
cargo run --manifest-path wasixv2/abi/Cargo.toml -- c --wit wasixv2/abi/wit --libc .
```

From this repository, for a Go checkout:

```sh
cargo run -- go --wit wit --goroot <go>
```

Here, `cargo test` runs the generator's own tests, and `WASIXV2_CODEGEN_BLESS=1 cargo test`
rewrites its golden files.
