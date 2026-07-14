#![allow(unused)]
use handle::ImportHandle;
use rsil_engine_primitives::EngineTypes;
use rsil_eth_wire::NewBlock;
use rsil_network::import::{BlockImport, BlockImportOutcome, NewBlockEvent};
use rsil_network_peers::PeerId;
use rsil_payload_primitives::{BuiltPayload, PayloadTypes};
use rsil_primitives_traits::NodePrimitives;
use service::{BlockMsg, BscBlock, ImportEvent, Outcome};
use std::{
    fmt,
    task::{ready, Context, Poll},
};

mod handle;
mod parlia;
mod service;

pub struct BscBlockImport<T: PayloadTypes> {
    handle: ImportHandle<T>,
}

impl<T: PayloadTypes> BscBlockImport<T> {
    pub fn new(handle: ImportHandle<T>) -> Self {
        Self { handle }
    }
}

impl<T: PayloadTypes> BlockImport<NewBlock<BscBlock<T>>> for BscBlockImport<T> {
    fn on_new_block(
        &mut self,
        peer_id: PeerId,
        incoming_block: NewBlockEvent<NewBlock<BscBlock<T>>>,
    ) {
        if let NewBlockEvent::Block(block) = incoming_block {
            let _ = self.handle.send_block(block, peer_id);
        }
    }

    fn poll(&mut self, cx: &mut Context<'_>) -> Poll<ImportEvent<T>> {
        match ready!(self.handle.poll_outcome(cx)) {
            Some(outcome) => Poll::Ready(outcome),
            None => Poll::Pending,
        }
    }
}

impl<T: PayloadTypes> fmt::Debug for BscBlockImport<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BscBlockImport")
            .field("engine_handle", &"ConsensusEngineHandle")
            .field("service_handle", &"BscBlockImportHandle")
            .finish()
    }
}
