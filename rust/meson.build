cargo = find_program('cargo')
cargo_build = find_program('../build-aux/cargo-build.sh')
rust_source_root = meson.current_source_dir() / '..'

rust_profile = 'debug'
if get_option('buildtype') in ['release', 'minsize']
  rust_profile = 'release'
endif

rust_target_dir = meson.current_build_dir() / 'cargo-target'
rust_sources = files(
  '../Cargo.toml',
  '../rust-toolchain.toml',
  '../src/lib.rs',
  '../src/main.rs',
  '../src/reference.rs',
  '../tests/reference_contract.rs',
)

custom_target(
  'xfce4-terminal-rs',
  input: rust_sources,
  output: 'xfce4-terminal-rs',
  command: [
    cargo_build,
    cargo,
    rust_source_root,
    rust_target_dir,
    '@OUTPUT@',
    rust_profile,
  ],
  build_by_default: true,
  build_always_stale: true,
)

test(
  'rust',
  cargo,
  args: [
    'test',
    '--manifest-path', rust_source_root / 'Cargo.toml',
    '--target-dir', rust_target_dir,
    '--all-targets',
    '--all-features',
    '--locked',
  ],
)
