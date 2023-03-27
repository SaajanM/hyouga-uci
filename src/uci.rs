use std::sync::Arc;

use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot, RwLock,
    },
};
use vampirc_uci::{parse_one, CommunicationDirection, MessageList, UciMessage};

use crate::{EngineSettings, ENGINE_AUTHOR, ENGINE_NAME};

pub struct UciArguments {
    pub engine_settings: Arc<RwLock<EngineSettings>>,
}

pub struct IncomingMessage {
    pub message: UciMessage,
    pub handler: Option<oneshot::Sender<UciMessage>>,
}

pub struct UciService {
    pub work_queue: UnboundedReceiver<IncomingMessage>,
    pub writer: Arc<UciWriter>,
}

pub struct UciWriter {
    print_queue_send: RwLock<UnboundedSender<UciMessage>>,
}

impl UciWriter {
    #[allow(dead_code)]
    pub fn queue_message_one(&self, msg: UciMessage) {
        let sender = self.print_queue_send.blocking_write();
        let _ = sender.send(msg);
    }
    pub fn queue_message_many(&self, msg_list: MessageList) {
        let sender = self.print_queue_send.blocking_write();
        for msg in msg_list {
            let _ = sender.send(msg);
        }
    }
}

impl Default for UciWriter {
    fn default() -> Self {
        let (print_queue_send, mut print_queue_recv) = unbounded_channel();
        let print_queue_send = RwLock::new(print_queue_send);
        let _ = tokio::spawn(async move {
            while let Some(msg) = print_queue_recv.recv().await {
                println!("{}\n", msg);
            }
        });
        Self { print_queue_send }
    }
}

impl UciService {
    pub fn new(args: UciArguments) -> Self {
        let writer = Arc::new(UciWriter::default());
        let writer_ref = writer.clone();

        let (work_sender, work_queue) = unbounded_channel();

        let _ = tokio::spawn(async move {
            let mut in_handle = BufReader::new(stdin()).lines();
            while let Ok(Some(raw_cmd)) = in_handle.next_line().await {
                let uci_command = parse_one(&raw_cmd);
                if uci_command.direction() == CommunicationDirection::EngineToGui {
                    continue;
                }
                let responses: MessageList = match uci_command {
                    UciMessage::Uci => vec![
                        UciMessage::id_name(ENGINE_NAME),
                        UciMessage::id_author(ENGINE_AUTHOR),
                        UciMessage::UciOk,
                    ],
                    UciMessage::Debug(debug) => {
                        let mut guard = args.engine_settings.blocking_write();
                        guard.debug = debug;
                        continue;
                    }
                    UciMessage::IsReady => {
                        let (tx, rx) = oneshot::channel();
                        let _ = work_sender.send(IncomingMessage {
                            message: UciMessage::IsReady,
                            handler: Some(tx),
                        });
                        let _ = rx.await;
                        vec![UciMessage::ReadyOk]
                    }
                    UciMessage::Position {
                        startpos: _,
                        fen: _,
                        moves: _,
                    } => continue,
                    UciMessage::SetOption { name, value } => {
                        if let Some(value) = value {
                            let mut guard = args.engine_settings.blocking_write();
                            guard.options.insert(name, value);
                        }
                        continue;
                    }
                    UciMessage::UciNewGame => continue,
                    UciMessage::Stop => continue,
                    UciMessage::PonderHit => continue,
                    UciMessage::Quit => break,
                    UciMessage::Go {
                        time_control: _,
                        search_control: _,
                    } => continue,
                    UciMessage::Unknown(_, _) => continue,
                    _ => continue,
                };
                writer_ref.queue_message_many(responses);
            }
        });

        Self { work_queue, writer }
    }
}
