//! Build

#[expect(clippy::unwrap_used, reason = "illustration purposes")]
#[cfg(feature = "grpc")]
mod grpc {
  use pb_rs::{ConfigBuilder, types::FileDescriptor};
  use std::{
    fs::{DirBuilder, remove_dir_all},
    path::Path,
  };

  pub(crate) fn run() {
    let cmd = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let in_dir = Path::new(&cmd).join("src");
    let out_dir = Path::new(&std::env::var("OUT_DIR").unwrap()).join("protos");
    if out_dir.exists() {
      remove_dir_all(&out_dir).unwrap();
    }
    DirBuilder::new().create(&out_dir).unwrap();
    FileDescriptor::run(
      &ConfigBuilder::new(
        &[Path::new(&cmd).join("src/grpc.proto").as_path()],
        None,
        Some(&out_dir.as_path()),
        &[in_dir.as_path()],
      )
      .unwrap()
      .build(),
    )
    .unwrap();
  }
}

fn main() {
  #[cfg(feature = "grpc")]
  grpc::run();
}
