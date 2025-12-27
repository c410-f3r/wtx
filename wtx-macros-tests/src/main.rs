use std::path::{Path, PathBuf};
use ui_test::{
  CommandBuilder, Config, dependencies::DependencyBuilder, ignore_output_conflict, run_tests,
  spanned::Spanned,
};

enum Mode {
  Compile,
  Expand,
  Panic,
  Test,
}

fn cfg(path: &Path, mode: Mode) -> Config {
  let mut config = Config {
    output_conflict_handling: if std::env::var_os("BLESS").is_some() {
      ui_test::bless_output_files
    } else {
      ui_test::error_on_output_conflict
    },
    program: CommandBuilder::rustc(),
    ..Config::rustc(path)
  };

  let exit_status = match mode {
    Mode::Compile => {
      config.output_conflict_handling = ignore_output_conflict;
      0
    }
    Mode::Expand => {
      config.program.args.push("-Zunpretty=expanded".into());
      0
    }
    Mode::Panic => 1,
    Mode::Test => {
      config.output_conflict_handling = ignore_output_conflict;
      config.program.args.push("--test".into());
      //config.comment_defaults.base().run_command = Spanned::dummy(Some(CommandBuilder::cmd("{output}"))).into();
      0
    }
  };
  config.comment_defaults.base().exit_status = Spanned::dummy(exit_status).into();
  config.comment_defaults.base().require_annotations = Spanned::dummy(false).into();
  config.comment_defaults.base().set_custom(
    "dependencies",
    DependencyBuilder {
      crate_manifest_path: PathBuf::from("Cargo.toml"),
      ..DependencyBuilder::default()
    },
  );
  config
}

fn main() -> ui_test::color_eyre::eyre::Result<()> {
  run_tests(cfg(&Path::new("tests/expand"), Mode::Expand))?;
  run_tests(cfg(&Path::new("tests/fail"), Mode::Panic))?;
  run_tests(cfg(&Path::new("tests/pass"), Mode::Compile))?;
  run_tests(cfg(&Path::new("tests/test"), Mode::Test))?;
  Ok(())
}
