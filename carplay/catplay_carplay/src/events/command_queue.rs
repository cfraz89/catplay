use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use catplay_plist::PlistSerializable;
use catplay_util::{EventSleeper, EventToken};
use log::{debug, warn};
use tokio::sync::{
    Notify,
    oneshot::{self, error::RecvError},
};

use crate::{
    events::EventServerCallback,
    msg::Command,
    rtsp_frame::{HttpStatus, RtspResponse, RtspResult},
};

pub struct CommandQueue {
    inner: Arc<Inner>,
}

pub struct CommandQueueDrain {
    inner: Arc<Inner>,
}

struct Inner {
    pending: Mutex<VecDeque<CommandPending>>,
    closed: Mutex<bool>,
    notify: Notify,
}

#[derive(Debug)]
pub struct CommandPending {
    cmd: Command,
    resp: Option<oneshot::Sender<RtspResult<RtspResponse>>>,
}

impl CommandPending {
    pub fn command(&self) -> &Command {
        &self.cmd
    }

    pub fn command_mut(&mut self) -> &mut Command {
        &mut self.cmd
    }

    pub fn respond(&mut self, ret: RtspResult<RtspResponse>) {
        if let Some(resp) = self.resp.take() {
            let _ = resp.send(ret);
        }
    }

    pub fn respond_ok(&mut self) {
        self.respond_status(HttpStatus::Ok);
    }

    pub fn respond_err(&mut self) {
        self.respond_status(HttpStatus::InternalServerError)
    }

    pub fn respond_status(&mut self, status: HttpStatus) {
        self.respond(Ok(RtspResponse::new(None, status)));
    }

    pub fn respond_plist<S: PlistSerializable>(&mut self, status: HttpStatus, plist: S) -> RtspResult<()> {
        let mut resp = RtspResponse::new(None, status);
        resp.set_plist(plist)?;
        self.respond(Ok(resp));
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.resp.is_none()
    }
}

impl CommandQueue {
    pub fn new() -> (CommandQueue, CommandQueueDrain) {
        let inner = Arc::new(Inner {
            pending: Mutex::new(VecDeque::new()),
            closed: Mutex::new(false),
            notify: Notify::new(),
        });

        (CommandQueue { inner: inner.clone() }, CommandQueueDrain { inner })
    }

    pub async fn run(&self, command: Command) -> Result<RtspResult<RtspResponse>, RecvError> {
        let (tx, rx) = oneshot::channel();
        debug!("Kicking command to main thread and waiting for response: {command:?}");

        {
            let closed = self.inner.closed.lock().unwrap();
            if *closed {
                drop(tx); // Pretend there is no receiver at the other end
            } else {
                let pending = CommandPending {
                    cmd: command,
                    resp: Some(tx),
                };
                self.inner.pending.lock().unwrap().push_back(pending);
            }
        }

        self.inner.notify.notify_one();
        let ret = rx.await;
        debug!("Received command response from main thread: {ret:?}");
        ret
    }
}

impl CommandQueueDrain {
    pub fn pop(&self) -> Option<CommandPending> {
        let ret = self.inner.pending.lock().unwrap().pop_front();
        if ret.is_some() {
            self.inner.notify.notify_one();
        }

        ret
    }

    pub async fn wait(&mut self) {
        self.inner.notify.notified().await;
    }

    pub fn close(&self) {
        debug!("Closing CommandQueueDrain!");
        let mut closed = self.inner.closed.lock().unwrap();
        self.inner.pending.lock().unwrap().clear();
        *closed = true;
    }
}

impl Drop for CommandQueueDrain {
    fn drop(&mut self) {
        debug!("CommandQueueDrain was dropped!");
        self.close()
    }
}

#[async_trait]
impl EventServerCallback for CommandQueue {
    async fn on_command(&self, cmd: Command) -> RtspResponse {
        let ret = self.run(cmd.clone()).await;
        match ret {
            Err(_) => {
                debug!("Ignoring command, because receiver is closed");
                RtspResponse::new(None, HttpStatus::Gone)
            }
            Ok(Err(err)) => {
                warn!("Returning error code in response to command {cmd:?} because of: {err}");
                RtspResponse::new(None, HttpStatus::InternalServerError)
            }
            Ok(Ok(v)) => v,
        }
    }
}

impl EventSleeper for CommandQueueDrain {
    async fn sleep(&mut self) -> Option<EventToken> {
        self.wait().await;
        Some(EventToken(1))
    }
}
