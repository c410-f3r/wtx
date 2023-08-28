//! WebSocket echo server.

mod common;

#[cfg(feature = "async-trait")]
mod cfg_hack {
    pub(crate) fn hack() -> wtx::Result<()> {
        Ok(())
    }
}

#[cfg(not(feature = "async-trait"))]
mod cfg_hack {
    use glommio::{net::TcpListener, CpuSet, LocalExecutorPoolBuilder, PoolPlacement};
    use std::thread::available_parallelism;

    pub(crate) fn hack() -> wtx::Result<()> {
        let builder = LocalExecutorPoolBuilder::new(PoolPlacement::MaxSpread(
            available_parallelism()?.into(),
            CpuSet::online().ok(),
        ));
        for result in builder.on_all_shards(exec)?.join_all() {
            result??;
        }
        Ok(())
    }

    async fn exec() -> wtx::Result<()> {
        let listener = TcpListener::bind(crate::common::_host_from_args())?;
        loop {
            let stream = listener.accept().await?;
            let _jh = glommio::spawn_local(async move {
                let fb = &mut <_>::default();
                let rb = &mut <_>::default();
                if let Err(err) = crate::common::_accept_conn_and_echo_frames(fb, rb, stream).await
                {
                    println!("{err}");
                }
            })
            .detach();
        }
    }
}

fn main() -> wtx::Result<()> {
    cfg_hack::hack()
}
