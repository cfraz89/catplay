use bytes::BytesMut;
use catplay_iap2_link::PacketFrame;
use catplay_util::{AsyncShutdown, EventReconciler, EventSleeper, EventToken, event_select, sleeper};
use log::{debug, trace};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf, split},
    select,
};

use crate::{
    CsmSessionError, CsmSessionResult,
    tokio::{AsyncClient, AsyncClientDrain},
};

#[derive(EventReconciler, AsyncShutdown)]
#[reconcile_error(CsmSessionError)]
#[reconcile_func(reconcile)]
pub struct AsyncClientStream<S: AsyncRead + AsyncWrite + Send> {
    #[reconcile_pop]
    error: Option<CsmSessionError>,

    #[reconcile]
    #[shutdown]
    client: AsyncClient,
    client_tx: AsyncClientDrain,
    read_half: ReadHalf<S>,
    write_half: WriteHalf<S>,
    rx_frame: Option<PacketFrame<Vec<u8>>>,
    rx_buf: BytesMut,
    tx_buf: BytesMut,
}

impl<S: AsyncRead + AsyncWrite + Send> AsyncClientStream<S> {
    const PIPE_BUFFER: usize = 16384;

    pub fn new(client: AsyncClient, client_tx: AsyncClientDrain, stream: S) -> Self {
        let spl = split(stream);

        Self {
            read_half: spl.0,
            write_half: spl.1,
            rx_frame: None,
            rx_buf: BytesMut::with_capacity(Self::PIPE_BUFFER),
            tx_buf: BytesMut::new(),
            error: None,
            client,
            client_tx,
        }
    }

    async fn reconcile(&mut self) -> CsmSessionResult<()> {
        if self.tx_buf.is_empty()
            && let Some(frame) = self.client_tx.take()
        {
            debug!("Received TX frame sized {} / total buf {}", frame.frame.len(), self.tx_buf.len());
            trace!("... frame that was written is {frame:?}");
            self.tx_buf.extend_from_slice(frame.frame.as_ref());
        }

        Ok(())
    }

    async fn sleep_io(
        read_half: &mut ReadHalf<S>,
        write_half: &mut WriteHalf<S>,
        rx_frame: &mut Option<PacketFrame<Vec<u8>>>,
        rx_buf: &mut BytesMut,
        tx_buf: &mut BytesMut,
        error: &mut Option<CsmSessionError>,
    ) -> Option<EventToken> {
        select! {
            v = read_half.read_buf(rx_buf) => match v {
                Ok(n) => {
                    debug!("Read {n} bytes");
                    debug_assert!(rx_frame.is_none());
                    let frame = PacketFrame::new(None, rx_buf.split().into());
                    trace!("... frame that was read is {frame:?}");
                    rx_frame.replace(frame);
                }
                Err(err) => {
                    error.replace(err.into());
                }
            },
            v = write_half.write_buf(tx_buf), if !tx_buf.is_empty() => match v {
                Ok(n) => {
                    debug!("Wrote {n} bytes");
                }
                Err(err) => {
                    error.replace(err.into());
                }
            }
        }

        Some(EventToken(1))
    }

    pub fn client_mut(&mut self) -> &mut AsyncClient {
        &mut self.client
    }
}

impl<S: AsyncRead + AsyncWrite + Send> EventSleeper for AsyncClientStream<S> {
    async fn sleep(&mut self) -> Option<EventToken> {
        let token = {
            let mut client = &mut self.client;
            let mut client_tx = &mut self.client_tx;
            let read_half = &mut self.read_half;
            let write_half = &mut self.write_half;
            let rx_frame = &mut self.rx_frame;
            let rx_buf = &mut self.rx_buf;
            let tx_buf = &mut self.tx_buf;
            let error = &mut self.error;

            event_select!(
                client,
                client_tx,
                sleeper(Self::sleep_io(read_half, write_half, rx_frame, rx_buf, tx_buf, error))
            )
        };

        if let Some(frame) = self.rx_frame.take() {
            self.client.read_frame(&frame);
        }

        token
    }
}
