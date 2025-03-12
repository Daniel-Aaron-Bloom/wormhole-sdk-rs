macro_rules! known_chains {
    ($($($chain:ident = $val:literal),+ $(,)?)?) => {
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum KnownChainIds {$($(
    $chain = $val,)+)?
}

impl KnownChainIds {
    pub const fn try_from(value: u16) -> Result<Self, UnknownChainId> {
        use KnownChainIds::*;
        match value {$($(
            $val => Ok($chain),)+)?
            id => Err(UnknownChainId(id)),
        }
    }
}
    };
}

known_chains! {
    Unset = 0,
    Solana = 1,
    Ethereum = 2,
    Terra = 3,
    Bsc = 4,
    Polygon = 5,
    Avalanche = 6,
    Oasis = 7,
    Algorand = 8,
    Aurora = 9,
    Fantom = 10,
    Karura = 11,
    Acala = 12,
    Klaytn = 13,
    Celo = 14,
    Near = 15,
    Moonbeam = 16,
    Neon = 17,
    Terra2 = 18,
    Injective = 19,
    Osmosis = 20,
    Sui = 21,
    Aptos = 22,
    Arbitrum = 23,
    Optimism = 24,
    Gnosis = 25,
    Pythnet = 26,
    Xpla = 28,
    Btc = 29,
    Base = 30,
    Sei = 32,
    Rootstock = 33,
    Scroll = 34,
    Mantle = 35,
    Blast = 36,
    XLayer = 37,
    Linea = 38,
    Berachain = 39,
    SeiEvm = 40,
    Snaxchain = 43,
    Wormchain = 3104,
    Cosmoshub = 4000,
    Evmos = 4001,
    Kujira = 4002,
    Neutron = 4003,
    Celestia = 4004,
    Stargaze = 4005,
    Seda = 4006,
    Dymension = 4007,
    Provenance = 4008,
    Sepolia = 10002,
    ArbitrumSepolia = 10003,
    BaseSepolia = 10004,
    OptimismSepolia = 10005,
    Holesky = 10006,
    PolygonSepolia = 10007,
}

mod __private {
    use super::KnownChainIds;
    use cphf::{ConstKey, Hasher, PhfKey, PhfKeyProxy};

    pub struct KnownChainIdsMarker;

    impl PhfKey for KnownChainIds {
        type ConstKey = KnownChainIdsMarker;
    }
    impl ConstKey for KnownChainIdsMarker {
        type PhfKey = KnownChainIds;
    }
    impl KnownChainIdsMarker {
        pub const fn pfh_hash(value: &KnownChainIds, state: &mut Hasher) {
            let value = *value as u16;
            <u16 as PhfKey>::ConstKey::pfh_hash(&value, state)
        }
        pub const fn pfh_eq(lhs: &KnownChainIds, rhs: &KnownChainIds) -> bool {
            *lhs as u16 == *rhs as u16
        }
    }
    // Finally we add a trait implementation to allow usage of indexing methods like `get`.
    // You can make this broad or narrow as you would like, doing so just modifies what types callers can pass to `OrderedMap::get`,
    // but `?Sized + Borrow<Self>` is a pretty good baseline.
    impl<PK: Clone + TryInto<KnownChainIds>> PhfKeyProxy<PK> for KnownChainIds {
        fn pfh_hash(pk: &PK, state: &mut Hasher) {
            if let Ok(pk) = pk.clone().try_into() {
                KnownChainIdsMarker::pfh_hash(&pk, state)
            }
        }
        fn pfh_eq(&self, other: &PK) -> bool {
            if let Ok(other) = other.clone().try_into() {
                *self == other as KnownChainIds
            } else {
                false
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnknownChainId(pub u16);

impl TryFrom<u16> for KnownChainIds {
    type Error = UnknownChainId;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_from(value)
    }
}
impl TryFrom<ChainId> for KnownChainIds {
    type Error = UnknownChainId;

    fn try_from(id: ChainId) -> Result<Self, Self::Error> {
        match id {
            ChainId::Known(id) => Ok(id),
            ChainId::Unknown(id) => Err(UnknownChainId(id)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainId {
    Known(KnownChainIds),
    Unknown(u16),
}

impl ChainId {
    pub const fn from_u16(id: u16) -> Self {
        if let Ok(id) = KnownChainIds::try_from(id) {
            ChainId::Known(id)
        } else {
            ChainId::Unknown(id)
        }
    }
    pub const fn to_u16(&self) -> u16 {
        match self {
            ChainId::Known(id) => *id as u16,
            ChainId::Unknown(id) => *id,
        }
    }
    pub const fn to_known(self) -> Option<KnownChainIds> {
        match self {
            ChainId::Known(id) => Some(id),
            ChainId::Unknown(_) => None,
        }
    }
}

impl From<ChainId> for u16 {
    fn from(id: ChainId) -> Self {
        id.to_u16()
    }
}

impl From<u16> for ChainId {
    fn from(id: u16) -> Self {
        Self::from_u16(id)
    }
}
