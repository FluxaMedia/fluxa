use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    type Stream = tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >;

    pub struct Socket(Stream);

    impl Socket {
        pub async fn connect(url: &str) -> Result<Self, String> {
            let _ = rustls::crypto::ring::default_provider().install_default();
            let (stream, _) = tokio_tungstenite::connect_async(url)
                .await
                .map_err(|error| error.to_string())?;
            Ok(Self(stream))
        }

        pub async fn recv(&mut self) -> Result<String, String> {
            loop {
                match self.0.next().await {
                    Some(Ok(Message::Text(text))) => return Ok(text.as_str().to_owned()),
                    Some(Ok(Message::Close(_))) | None => return Err("socket closed".to_owned()),
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error.to_string()),
                }
            }
        }

        pub async fn send(&mut self, text: String) -> Result<(), String> {
            self.0
                .send(Message::text(text))
                .await
                .map_err(|error| error.to_string())
        }

        pub async fn close(&mut self) {
            let _ = self.0.close(None).await;
        }
    }

    pub struct Ticker(tokio::time::Interval);

    impl Ticker {
        pub async fn new(period: Duration) -> Self {
            let mut interval = tokio::time::interval(period);
            interval.tick().await;
            Self(interval)
        }

        pub async fn tick(&mut self) {
            self.0.tick().await;
        }
    }

    pub async fn sleep(duration: Duration) {
        tokio::time::sleep(duration).await;
    }

    pub struct Task(tokio::task::JoinHandle<()>);

    impl Task {
        pub fn abort(&self) {
            self.0.abort();
        }
    }

    pub fn spawn(future: impl std::future::Future<Output = ()> + Send + 'static) -> Task {
        Task(crate::executor::runtime().spawn(future))
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use tokio::sync::{mpsc, oneshot};
    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use web_sys::{MessageEvent, WebSocket};

    type Handler = Closure<dyn FnMut(JsValue)>;

    pub struct Socket {
        ws: WebSocket,
        inbox: mpsc::UnboundedReceiver<Result<String, String>>,
        _handlers: Vec<Handler>,
    }

    impl Socket {
        pub async fn connect(url: &str) -> Result<Self, String> {
            let ws = WebSocket::new(url).map_err(|error| format!("{error:?}"))?;
            let (feed, inbox) = mpsc::unbounded_channel();
            let (opened, ready) = oneshot::channel::<Result<(), String>>();
            let opened = Rc::new(RefCell::new(Some(opened)));
            let mut handlers: Vec<Handler> = Vec::new();

            let signal = opened.clone();
            let on_open = Closure::<dyn FnMut(JsValue)>::new(move |_| {
                if let Some(opened) = signal.borrow_mut().take() {
                    let _ = opened.send(Ok(()));
                }
            });
            ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
            handlers.push(on_open);

            let signal = opened.clone();
            let errors = feed.clone();
            let on_error = Closure::<dyn FnMut(JsValue)>::new(move |_| {
                let error = "socket error".to_owned();
                match signal.borrow_mut().take() {
                    Some(opened) => {
                        let _ = opened.send(Err(error));
                    }
                    None => {
                        let _ = errors.send(Err(error));
                    }
                }
            });
            ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
            handlers.push(on_error);

            let signal = opened;
            let closes = feed.clone();
            let on_close = Closure::<dyn FnMut(JsValue)>::new(move |_| {
                let error = "socket closed".to_owned();
                match signal.borrow_mut().take() {
                    Some(opened) => {
                        let _ = opened.send(Err(error));
                    }
                    None => {
                        let _ = closes.send(Err(error));
                    }
                }
            });
            ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
            handlers.push(on_close);

            let on_message = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                if let Some(text) = event
                    .dyn_into::<MessageEvent>()
                    .ok()
                    .and_then(|event| event.data().as_string())
                {
                    let _ = feed.send(Ok(text));
                }
            });
            ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
            handlers.push(on_message);

            ready
                .await
                .map_err(|_| "socket closed".to_owned())
                .and_then(|result| result)?;
            Ok(Self {
                ws,
                inbox,
                _handlers: handlers,
            })
        }

        pub async fn recv(&mut self) -> Result<String, String> {
            self.inbox
                .recv()
                .await
                .unwrap_or_else(|| Err("socket closed".to_owned()))
        }

        pub async fn send(&mut self, text: String) -> Result<(), String> {
            self.ws
                .send_with_str(&text)
                .map_err(|error| format!("{error:?}"))
        }

        pub async fn close(&mut self) {
            let _ = self.ws.close();
        }
    }

    impl Drop for Socket {
        fn drop(&mut self) {
            self.ws.set_onopen(None);
            self.ws.set_onerror(None);
            self.ws.set_onclose(None);
            self.ws.set_onmessage(None);
            let _ = self.ws.close();
        }
    }

    fn window() -> web_sys::Window {
        web_sys::window().expect("window is not available")
    }

    pub struct Ticker {
        id: i32,
        ticks: mpsc::UnboundedReceiver<()>,
        _handler: Handler,
    }

    impl Ticker {
        pub async fn new(period: Duration) -> Self {
            let (feed, ticks) = mpsc::unbounded_channel();
            let handler = Closure::<dyn FnMut(JsValue)>::new(move |_| {
                let _ = feed.send(());
            });
            let id = window()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    handler.as_ref().unchecked_ref(),
                    period.as_millis() as i32,
                )
                .unwrap_or(0);
            Self {
                id,
                ticks,
                _handler: handler,
            }
        }

        pub async fn tick(&mut self) {
            if self.ticks.recv().await.is_none() {
                std::future::pending::<()>().await;
            }
        }
    }

    impl Drop for Ticker {
        fn drop(&mut self) {
            window().clear_interval_with_handle(self.id);
        }
    }

    pub async fn sleep(duration: Duration) {
        let (done, wait) = oneshot::channel::<()>();
        let done = Rc::new(RefCell::new(Some(done)));
        let handler = Closure::<dyn FnMut(JsValue)>::new(move |_| {
            if let Some(done) = done.borrow_mut().take() {
                let _ = done.send(());
            }
        });
        let scheduled = window().set_timeout_with_callback_and_timeout_and_arguments_0(
            handler.as_ref().unchecked_ref(),
            duration.as_millis() as i32,
        );
        if scheduled.is_ok() {
            let _ = wait.await;
        }
    }

    pub struct Task(futures::future::AbortHandle);

    impl Task {
        pub fn abort(&self) {
            self.0.abort();
        }
    }

    pub fn spawn(future: impl std::future::Future<Output = ()> + 'static) -> Task {
        let (future, handle) = futures::future::abortable(future);
        wasm_bindgen_futures::spawn_local(async move {
            let _ = future.await;
        });
        Task(handle)
    }
}

pub use imp::{Socket, Task, Ticker, sleep, spawn};
