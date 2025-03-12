use core::{fmt, hash::Hash};

macro_rules! known_chains {
    ($($($chain:ident = $val:literal),+ $(,)?)?) => {
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[non_exhaustive]
pub enum KnownChainId {$($(
    $chain = $val,)+)?
}

impl KnownChainId {
    pub const fn try_from(value: u16) -> Result<Self, UnknownChainId> {
        use KnownChainId::*;
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
    use super::KnownChainId;
    use cphf::{ConstKey, Hasher, PhfKey, PhfKeyProxy};

    pub struct KnownChainIdsMarker;

    impl PhfKey for KnownChainId {
        type ConstKey = KnownChainIdsMarker;
    }
    impl ConstKey for KnownChainIdsMarker {
        type PhfKey = KnownChainId;
    }
    impl KnownChainIdsMarker {
        pub const fn pfh_hash(value: &KnownChainId, state: &mut Hasher) {
            let value = *value as u16;
            <u16 as PhfKey>::ConstKey::pfh_hash(&value, state)
        }
        pub const fn pfh_eq(lhs: &KnownChainId, rhs: &KnownChainId) -> bool {
            *lhs as u16 == *rhs as u16
        }
    }
    // Finally we add a trait implementation to allow usage of indexing methods like `get`.
    // You can make this broad or narrow as you would like, doing so just modifies what types callers can pass to `OrderedMap::get`,
    // but `?Sized + Borrow<Self>` is a pretty good baseline.
    impl<PK: Clone + TryInto<KnownChainId>> PhfKeyProxy<PK> for KnownChainId {
        fn pfh_hash(pk: &PK, state: &mut Hasher) {
            if let Ok(pk) = pk.clone().try_into() {
                KnownChainIdsMarker::pfh_hash(&pk, state)
            }
        }
        fn pfh_eq(&self, other: &PK) -> bool {
            if let Ok(other) = other.clone().try_into() {
                *self == other as KnownChainId
            } else {
                false
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnknownChainId(u16);

impl UnknownChainId {
    pub const fn to_u16(self) -> u16 {
        self.0
    }
}

impl KnownChainId {
    pub const fn to_u16(self) -> u16 {
        self as u16
    }
}

impl fmt::Display for UnknownChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}
impl fmt::Display for KnownChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

impl From<KnownChainId> for u16 {
    fn from(value: KnownChainId) -> Self {
        value.to_u16()
    }
}
impl From<UnknownChainId> for u16 {
    fn from(value: UnknownChainId) -> Self {
        value.to_u16()
    }
}

impl TryFrom<u16> for KnownChainId {
    type Error = UnknownChainId;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_from(value)
    }
}

impl TryFrom<u16> for UnknownChainId {
    type Error = KnownChainId;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match KnownChainId::try_from(value) {
            Ok(v) => Err(v),
            Err(e) => Ok(e),
        }
    }
}
impl TryFrom<ChainId> for KnownChainId {
    type Error = UnknownChainId;

    fn try_from(id: ChainId) -> Result<Self, Self::Error> {
        match id {
            ChainId::Known(id) => Ok(id),
            ChainId::Unknown(id) => Err(id),
        }
    }
}
impl TryFrom<ChainId> for UnknownChainId {
    type Error = KnownChainId;

    fn try_from(id: ChainId) -> Result<Self, Self::Error> {
        match <KnownChainId as TryFrom<ChainId>>::try_from(id) {
            Ok(v) => Err(v),
            Err(e) => Ok(e),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainId {
    Known(KnownChainId),
    Unknown(UnknownChainId),
}

impl ChainId {
    pub const fn from_u16(id: u16) -> Self {
        if let Ok(id) = KnownChainId::try_from(id) {
            ChainId::Known(id)
        } else {
            ChainId::Unknown(UnknownChainId(id))
        }
    }
    pub const fn to_u16(&self) -> u16 {
        match self {
            ChainId::Known(id) => id.to_u16(),
            ChainId::Unknown(id) => id.to_u16(),
        }
    }
    pub const fn to_known(self) -> Option<KnownChainId> {
        match self {
            ChainId::Known(id) => Some(id),
            ChainId::Unknown(_) => None,
        }
    }
    pub const fn to_unknown(self) -> Option<UnknownChainId> {
        match self {
            ChainId::Known(_) => None,
            ChainId::Unknown(id) => Some(id),
        }
    }
}

impl Hash for ChainId {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.to_u16().hash(state);
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

impl PartialEq<ChainId> for UnknownChainId {
    fn eq(&self, other: &ChainId) -> bool {
        Some(*self) == other.to_unknown()
    }
}
impl PartialEq<u16> for UnknownChainId {
    fn eq(&self, other: &u16) -> bool {
        self.to_u16() == *other
    }
}

impl PartialEq<ChainId> for KnownChainId {
    fn eq(&self, other: &ChainId) -> bool {
        Some(*self) == other.to_known()
    }
}
impl PartialEq<u16> for KnownChainId {
    fn eq(&self, other: &u16) -> bool {
        self.to_u16() == *other
    }
}

impl PartialEq<KnownChainId> for ChainId {
    fn eq(&self, other: &KnownChainId) -> bool {
        self.to_known() == Some(*other)
    }
}
impl PartialEq<u16> for ChainId {
    fn eq(&self, other: &u16) -> bool {
        self.to_u16() == *other
    }
}

#[cfg(feature = "serde")]
pub mod serde {
    use core::fmt;

    use serde::{de::Error as _, Deserialize as _, Deserializer, Serializer};

    /// Serializes as a `u16`
    pub fn serialize_value<T: Clone + Into<u16>, S>(t: &T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_u16(t.clone().into())
    }

    /// Deserializes as a `u16`
    pub fn deserialize_value<'de, T: TryFrom<u16, Error: fmt::Display>, D>(
        d: D,
    ) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        let n = u16::deserialize(d)?;
        T::try_from(n).map_err(<D::Error>::custom)
    }
}
