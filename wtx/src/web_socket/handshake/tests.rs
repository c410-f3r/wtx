macro_rules! call_tests {
  (($ty:ident, $fb:expr, $ws:expr), $($struct:ident),+ $(,)?) => {
    $(
      println!("***** {} - {}", stringify!($ty), stringify!($struct));
      $struct::$ty($fb, $ws).await;
      tokio::time::sleep(Duration::from_millis(200)).await;
    )+
  };
}

use crate::web_socket::{
    frame::FrameMutVec,
    handshake::{WebSocketAccept, WebSocketAcceptRaw, WebSocketHandshake, WebSocketHandshakeRaw},
    FrameBufferVec, OpCode, WebSocketClientOwned, WebSocketServerOwned,
};
use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio::net::{TcpListener, TcpStream};

static HAS_SERVER_FINISHED: AtomicBool = AtomicBool::new(false);

#[tokio::test]
async fn client_and_server_frames() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    let _server_jh = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut fb = <_>::default();
        let (_, mut ws) = WebSocketAcceptRaw {
            fb: &mut fb,
            headers_buffer: &mut <_>::default(),
            rb: <_>::default(),
            key_buffer: &mut <_>::default(),
            stream,
        }
        .accept()
        .await
        .unwrap();
        call_tests!(
            (server, &mut fb, &mut ws),
            FragmentedMessage,
            LargeFragmentedMessage,
            PingAndText,
            SeveralBytes,
            TwoPings,
            // Last
            HelloAndGoodbye,
        );
        HAS_SERVER_FINISHED.store(true, Ordering::Relaxed);
    });

    let mut fb = <_>::default();
    let (_res, mut ws) = WebSocketHandshakeRaw {
        fb: &mut fb,
        headers_buffer: &mut <_>::default(),
        rb: <_>::default(),
        stream: TcpStream::connect("127.0.0.1:8080").await.unwrap(),
        uri: "http://127.0.0.1:8080",
    }
    .handshake()
    .await
    .unwrap();
    call_tests!(
        (client, &mut fb, &mut ws),
        FragmentedMessage,
        LargeFragmentedMessage,
        PingAndText,
        SeveralBytes,
        TwoPings,
        // Last
        HelloAndGoodbye,
    );

    let mut has_server_finished = false;
    for _ in 0..15 {
        let local_has_server_finished = HAS_SERVER_FINISHED.load(Ordering::Relaxed);
        if local_has_server_finished {
            has_server_finished = local_has_server_finished;
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    if !has_server_finished {
        panic!("Server didn't finish");
    }
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
trait Test {
    async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<TcpStream>);

    async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<TcpStream>);
}

struct FragmentedMessage;
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Test for FragmentedMessage {
    async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<TcpStream>) {
        ws.write_frame(&mut FrameMutVec::new_unfin(fb, OpCode::Text, b"1").unwrap())
            .await
            .unwrap();
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Continuation, b"23").unwrap())
            .await
            .unwrap();
    }

    async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<TcpStream>) {
        let text = ws.read_msg(fb).await.unwrap();
        assert_eq!(OpCode::Text, text.op_code());
        assert_eq!(b"123", text.fb().payload());
    }
}

struct HelloAndGoodbye;
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Test for HelloAndGoodbye {
    async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<TcpStream>) {
        let hello = ws.read_frame(fb).await.unwrap();
        assert_eq!(OpCode::Text, hello.op_code());
        assert_eq!(b"Hello!", hello.fb().payload());
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, b"Goodbye!").unwrap())
            .await
            .unwrap();
        assert_eq!(OpCode::Close, ws.read_frame(fb).await.unwrap().op_code());
    }

    async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<TcpStream>) {
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, b"Hello!").unwrap())
            .await
            .unwrap();
        assert_eq!(
            ws.read_frame(&mut *fb).await.unwrap().fb().payload(),
            b"Goodbye!"
        );
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Close, &[]).unwrap())
            .await
            .unwrap();
    }
}

struct LargeFragmentedMessage;
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Test for LargeFragmentedMessage {
    async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<TcpStream>) {
        async fn write(
            frame: &mut FrameMutVec<'_, true>,
            ws: &mut WebSocketClientOwned<TcpStream>,
        ) {
            ws.write_frame(frame).await.unwrap();
        }
        let bytes = vec![51; 256 * 1024];
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Text, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_fin(fb, OpCode::Continuation, &bytes).unwrap(),
            ws,
        )
        .await;
    }

    async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<TcpStream>) {
        let text = ws.read_msg(fb).await.unwrap();
        assert_eq!(OpCode::Text, text.op_code());
        assert_eq!(&vec![51; 10 * 256 * 1024], text.fb().payload());
    }
}

struct PingAndText;
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Test for PingAndText {
    async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<TcpStream>) {
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Ping, b"").unwrap())
            .await
            .unwrap();
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, b"ipat").unwrap())
            .await
            .unwrap();
        assert_eq!(OpCode::Pong, ws.read_frame(fb).await.unwrap().op_code());
    }

    async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<TcpStream>) {
        assert_eq!(b"ipat", ws.read_frame(fb).await.unwrap().fb().payload());
    }
}

struct SeveralBytes;
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Test for SeveralBytes {
    async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<TcpStream>) {
        async fn write(
            frame: &mut FrameMutVec<'_, true>,
            ws: &mut WebSocketClientOwned<TcpStream>,
        ) {
            ws.write_frame(frame).await.unwrap();
        }
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Text, &[206]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[186]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[225]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[189]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[185]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[207]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[131]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[206]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[188]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[206]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_unfin(fb, OpCode::Continuation, &[181]).unwrap(),
            ws,
        )
        .await;
        write(
            &mut FrameMutVec::new_fin(fb, OpCode::Continuation, &[]).unwrap(),
            ws,
        )
        .await;
    }

    async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<TcpStream>) {
        let text = ws.read_msg(fb).await.unwrap();
        assert_eq!(OpCode::Text, text.op_code());
        assert_eq!("κόσμε".as_bytes(), text.fb().payload());
    }
}

struct TwoPings;
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Test for TwoPings {
    async fn client(fb: &mut FrameBufferVec, ws: &mut WebSocketClientOwned<TcpStream>) {
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Ping, b"0").unwrap())
            .await
            .unwrap();
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Ping, b"1").unwrap())
            .await
            .unwrap();
        let _0 = ws.read_frame(fb).await.unwrap();
        assert_eq!(OpCode::Pong, _0.op_code());
        assert_eq!(b"0", _0.fb().payload());
        let _1 = ws.read_frame(fb).await.unwrap();
        assert_eq!(OpCode::Pong, _1.op_code());
        assert_eq!(b"1", _1.fb().payload());
        ws.write_frame(&mut FrameMutVec::new_fin(fb, OpCode::Text, b"").unwrap())
            .await
            .unwrap();
    }

    async fn server(fb: &mut FrameBufferVec, ws: &mut WebSocketServerOwned<TcpStream>) {
        let _0 = ws.read_frame(fb).await.unwrap();
        assert_eq!(OpCode::Text, _0.op_code());
        assert_eq!(b"", _0.fb().payload());
    }
}
