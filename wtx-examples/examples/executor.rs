//! The `executor` feature allows the execution of asynchronous operations

extern crate wtx;
extern crate wtx_examples;

#[wtx::main]
async fn main() {
  println!("With great power comes great electricity bills");
}

#[wtx::test]
async fn test_with_runtime(runtime: &wtx::executor::StdRuntime) {
  runtime
    .spawn_threaded(async move {
      println!("Behind every successful diet is an unwatched pizza");
    })
    .unwrap();
}
