#![no_std]

pub mod chain_id;
pub mod deploys;

pub use chain_id::{ChainId, KnownChainId};

/// The VM used by a chain
pub enum Vm {
    Evm,
    Solana,
    CosmWasm,
}

/// The environment of the chain (dev, test, main)
pub enum NetEnv {
    DevNet,
    TestNet,
    MainNet,
}

/// Struct representing the core deployment info for a chain.
pub struct CoreDeployment {
    /// The chain id.
    pub chain_id: ChainId,
    /// The name of the chain.
    pub name: &'static str,
    /// The core contract address on the chain.
    pub core_address: &'static [u8],
    /// The token bridge contract (if any).
    pub token_bridge_address: Option<&'static [u8]>,
    /// The NFT bridge contract (if any).
    pub nft_bridge_address: Option<&'static [u8]>,
    /// The token router proxy contract (if any).
    pub token_router_proxy_address: Option<&'static [u8]>,
    /// The VM used by the chain.
    pub vm: Vm,
    /// The environment of the chain (dev, test, main).
    pub net_env: NetEnv,
}
