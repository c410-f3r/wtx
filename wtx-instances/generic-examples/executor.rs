//! The `executor` feature allows the execution of asynchronous operations

extern crate wtx;
extern crate wtx_instances;

#[wtx::main]
async fn main() {
  println!("Hello from program!");
}

#[wtx::test]
async fn test_with_runtime(runtime: &wtx::executor::Runtime) {
  runtime
    .spawn_threaded(async move {
      println!("Hello from test!");
    })
    .unwrap();
}
