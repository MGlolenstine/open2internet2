use relm4::{send, MessageHandler, Sender};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::Sender as TokioSender;

use crate::utils::server_scanner::scan_ports;
use crate::{AppModel, AppMsg};

pub struct AsyncHandler {
    _rt: Runtime,
    sender: TokioSender<AsyncHandlerMsg>,
}

#[derive(Debug)]
pub enum AsyncHandlerMsg {
    RescanServers,
}

impl MessageHandler<AppModel> for AsyncHandler {
    type Msg = AsyncHandlerMsg;
    type Sender = TokioSender<AsyncHandlerMsg>;

    fn init(_parent_model: &AppModel, parent_sender: Sender<AppMsg>) -> Self {
        let (sender, mut rx) = tokio::sync::mpsc::channel::<AsyncHandlerMsg>(10);

        let rt = Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        rt.spawn(async move {
            while let Some(msg) = rx.recv().await {
                let parent_sender = parent_sender.clone();
                tokio::spawn(async move {
                    match msg {
                        AsyncHandlerMsg::RescanServers => {
                            let results = scan_ports().await;
                            send!(parent_sender, AppMsg::ServerScanResults(results));
                        }
                    }
                });
            }
        });
        AsyncHandler { _rt: rt, sender }
    }

    fn send(&self, msg: Self::Msg) {
        self.sender.blocking_send(msg).unwrap();
    }

    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}
