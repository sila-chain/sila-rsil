use rsil_evm::ConfigureEvm;
use rsil_rpc_convert::RpcConvert;
use rsil_rpc_eth_types::SilApiError;

/// Boxed RPC converter.
pub type DynRpcConverter<Savm, Network, Error = SilApiError> = Box<
    dyn RpcConvert<
        Primitives = <Savm as ConfigureEvm>::Primitives,
        Network = Network,
        Error = Error,
        Savm = Savm,
    >,
>;
