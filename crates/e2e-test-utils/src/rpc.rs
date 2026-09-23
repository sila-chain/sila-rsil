use alloy_consensus::{EthereumTxEnvelope as SilaTxEnvelope, TxEip4844Variant};
use alloy_network::eip2718::Decodable2718;
use alloy_primitives::{Bytes, B256};
use alloy_sips::eip7594::BlobTransactionSidecarVariant;
use rsil_chainspec::SilaHardforks;
use rsil_node_api::{BlockTy, FullNodeComponents};
use rsil_node_builder::{rpc::RpcRegistry, NodeTypes};
use rsil_provider::BlockReader;
use rsil_rpc_api::DebugApiServer;
use rsil_rpc_eth_api::{
    helpers::{SilApiSpec, SilTransactions, TraceExt},
    SilApiTypes,
};

#[expect(missing_debug_implementations)]
pub struct RpcTestContext<Node: FullNodeComponents, SilApi: SilApiTypes> {
    pub inner: RpcRegistry<Node, SilApi>,
}

impl<Node, SilApi> RpcTestContext<Node, SilApi>
where
    Node: FullNodeComponents<Types: NodeTypes<ChainSpec: SilaHardforks>>,
    SilApi: SilApiSpec<Provider: BlockReader<Block = BlockTy<Node::Types>>>
        + SilTransactions
        + TraceExt,
{
    /// Injects a raw transaction into the node tx pool via RPC server
    pub async fn inject_tx(&self, raw_tx: Bytes) -> Result<B256, SilApi::Error> {
        let eth_api = self.inner.eth_api();
        eth_api.send_raw_transaction(raw_tx).await
    }

    /// Retrieves a transaction envelope by its hash
    pub async fn envelope_by_hash(
        &self,
        hash: B256,
    ) -> eyre::Result<SilaTxEnvelope<TxEip4844Variant<BlobTransactionSidecarVariant>>> {
        let tx = self.inner.debug_api().raw_transaction(hash).await?.unwrap();
        let tx = tx.to_vec();
        Ok(SilaTxEnvelope::decode_2718(&mut tx.as_ref()).unwrap())
    }
}
