use const_decoder::{decode, Decoder};
use cphf::{phf_ordered_map, OrderedMap, UncasedStr};

use crate::{ChainId, CoreDeployment, KnownChainIds, NetEnv, Vm};

const BECH32: Decoder = Decoder::custom("qpzry9x8gf2tvdw0s3jn54khce6mua7l");
macro_rules! hex {
    ($v:literal) => {{
        const BYTES: &[u8] = &decode!(Decoder::Hex, $v.as_bytes());
        BYTES
    }};
}
macro_rules! bech32 {
    ($v:literal) => {{
        const INPUT: &[u8] = {
            let mut input = $v.as_bytes();
            while {
                let (h, tail) = input.split_first().expect("string did not contain `1`");
                input = tail;
                *h != b'1'
            } {}
            let (input, _checksum) = input
                .split_last_chunk::<6>()
                .expect("string did not contain checksum");
            // TODO: check checksum
            input
        };
        const BYTES: &[u8] = &decode!(BECH32, INPUT);
        BYTES
    }};
}
macro_rules! b58 {
    ($v:literal) => {{
        const BYTES: &[u8] = &substreams_solana_macro::b58!($v);
        BYTES
    }};
}

const DUMMY: CoreDeployment = CoreDeployment {
    chain_id: ChainId::Known(KnownChainIds::Unset),
    name: "",
    core_address: &[],
    token_bridge_address: None,
    nft_bridge_address: None,
    vm: Vm::Evm,
    net_env: NetEnv::MainNet,
};

macro_rules! allnets {
    (
        $all:ident, $by_id:ident, $by_name:ident, $net_env:ident;
        $($nets:expr),*;
        $($aliases:expr),*;
    ) => {
#[doc = concat!("A list of all known [`", stringify!($net_env), "`](NetEnv::", stringify!($net_env), ") environments.")]
pub const $all: &[&CoreDeployment] = &{
    const COUNT: usize = {
        let mut count = 0;
        $({
            let mut i = 0;
            while i < $nets.len() {
                if let NetEnv::$net_env = $nets[i].net_env {
                    count += 1;
                }
                i += 1;
            }
        })*

        count
    };

    let mut array = [&DUMMY; COUNT];
    let mut count = 0;
    $({
        let mut i = 0;
        while i < $nets.len() {
            if let NetEnv::$net_env = $nets[i].net_env {
                array[count] = &$nets[i];
                count += 1;
            }
            i += 1;
        }
    })*
    array
};

#[doc = concat!("A map indexed by [`KnownChainIds`] of all known [`", stringify!($net_env), "`](NetEnv::", stringify!($net_env), ") environments.")]
pub const $by_id: OrderedMap<KnownChainIds, &'static CoreDeployment> = phf_ordered_map!{KnownChainIds, &'static CoreDeployment; ={
    const COUNT: usize = $all.len();
    let mut data = [(KnownChainIds::Unset, &DUMMY); COUNT];
    let mut i = 0;
    while i < $all.len() {
        data[i] = ($all[i].chain_id.to_known().expect("unknown chain"), &$all[i]);
        i += 1;
    }
    data
}};

/// A map indexed by case-insensitive name of all known [`MainNet`](NetEnv::MainNet) environments.
pub const $by_name: OrderedMap<&'static UncasedStr, &'static CoreDeployment> = phf_ordered_map!{&'static UncasedStr, &'static CoreDeployment; ={
    const COUNT: usize = {
        let mut count = 0;
        $({
            let mut i = 0;
            while i < $nets.len() {
                if let NetEnv::$net_env = $nets[i].net_env {
                    count += 1;
                }
                i += 1;
            }
        })*
        $({
            let mut i = 0;
            while i < $aliases.len() {
                if let NetEnv::$net_env = $aliases[i].1.net_env {
                    count += 1;
                }
                i += 1;
            }
        })*

        count
    };

    let mut data = [(UncasedStr::new(""), &DUMMY); COUNT];
    let mut count = 0;
    {
        let mut i = 0;
        while i < $all.len() {
            if let NetEnv::$net_env = $all[i].net_env {
                data[count] = (UncasedStr::new($all[i].name), &$all[i]);
                count += 1;
            }
            i += 1;
        }
    }
    $({
        let mut i = 0;
        while i < $aliases.len() {
            if let NetEnv::$net_env = $aliases[i].1.net_env {
                data[count] = (UncasedStr::new($aliases[i].0), $aliases[i].1);
                count += 1;
            }
            i += 1;
        }
    })*
    // If you get a `duplicate keys` error, just uncomment these lines
    // if let NetEnv::$net_env = NetEnv::TestNet {
    //     let err_index_a = 22;
    //     let err_index_b = 23;
    //     const_panic::concat_panic!(
    //         data[err_index_a].1.name, " (", data[err_index_a].0.as_str(), ")",
    //         " was duplicate of ",
    //         data[err_index_b].1.name, " (", data[err_index_b].0.as_str(), ")",
    //     );
    // }
    data
}};
    };
}

allnets! {
    MAINNETS, MAINNETS_BY_ID, MAINNET_BY_NAME, MainNet;
    EVM_NETS, SOLANA_NETS, COSM_WASM_NETS;
    evm::ALIASES, solana::ALIASES, cosm_wasm::ALIASES;
}

allnets! {
    TESTNETS, TESTNETS_BY_ID, TESTNETS_BY_NAME, TestNet;
    EVM_NETS, SOLANA_NETS, COSM_WASM_NETS;
    evm::ALIASES, solana::ALIASES, cosm_wasm::ALIASES;
}

allnets! {
    DEVNETS, DEVNETS_BY_ID, DEVNETS_BY_NAME, DevNet;
    EVM_NETS, SOLANA_NETS, COSM_WASM_NETS;
    evm::ALIASES, solana::ALIASES, cosm_wasm::ALIASES;
}

macro_rules! nets_group {
    ($modi:ident, $enumi:ident, $consti:ident, $decoder:ident; $($name:ident {
        chain_id: $chain_id:ident,
        name: $name_field:literal,
        aliases: [$($alias:literal),*],
        core: $core_address:literal,
        $(token_bridge: $token_bridge_address:literal,)?
        $(nft_bridge: $nft_bridge_address:literal,)?
        net_env: $net_env:ident,
     }),* $(,)?) => {
pub mod $modi {
    use super::*;

    /// A list of all aliases
    #[doc(hidden)]
    pub const ALIASES: &[(&str, &CoreDeployment)] = &[$($(($alias, &$name),)*)*];

    $(
    #[doc = concat!("An [`", stringify!($enumi), "`](Vm::", stringify!($enumi), ")-based ")]
    #[doc = concat!("[`", stringify!($net_env), "`](NetEnv::", stringify!($net_env), ") ")]
    #[doc = concat!("named `\"", $name_field, "\"`.\n")]
    #[doc = concat!(" * core address is `\"", $core_address, "\"` ")]
    $(#[doc = concat!(" * token bridge address is `\"", $token_bridge_address, "\"` ")])?
    $(#[doc = concat!(" * nft bridge address is `\"", $nft_bridge_address, "\"` ")])?
    pub const $name: CoreDeployment = CoreDeployment {
        chain_id: ChainId::Known(KnownChainIds::$chain_id),
        name: $name_field,
        core_address: $decoder!($core_address),
        token_bridge_address: loop {
            $(break Some($decoder!($token_bridge_address));)?
            #[allow(unreachable_code)]
            break None;
        },
        nft_bridge_address: loop {
            $(break Some($decoder!($nft_bridge_address));)?
            #[allow(unreachable_code)]
            break None
        },
        vm: Vm::$enumi,
        net_env: NetEnv::$net_env,
    };)*

    #[doc = concat!("All the known [`", stringify!($modi), "`] chains.\n")]
    $(#[doc = " * [`"]
    #[doc = stringify!($name)]
    #[doc = "`]\n"])*
    #[doc(hidden)]
    pub const _NETS: &[CoreDeployment] = &[$($name),*];
}

#[doc(inline)]
pub use $modi::_NETS as $consti;
};
}

// EVM
nets_group! {evm, Evm, EVM_NETS, hex;
    // MainNets

    ETHEREUM {
        chain_id: Ethereum,
        name: "Ethereum",
        aliases: [],
        core: "98f3c9e6E3fAce36bAAd05FE09d375Ef1464288B",
        token_bridge: "3ee18B2214AFF97000D974cf647E7C347E8fa585",
        nft_bridge: "6FFd7EdE62328b3Af38FCD61461Bbfc52F5651fE",
        net_env: MainNet,
    },
    BSC {
        chain_id: Bsc,
        name: "Bsc",
        aliases: [],
        core: "98f3c9e6E3fAce36bAAd05FE09d375Ef1464288B",
        token_bridge: "B6F6D86a8f9879A9c87f643768d9efc38c1Da6E7",
        nft_bridge: "5a58505a96D1dbf8dF91cB21B54419FC36e93fdE",
        net_env: MainNet,
    },
    POLYGON {
        chain_id: Polygon,
        name: "Polygon",
        aliases: [],
        core: "7A4B5a56256163F07b2C80A7cA55aBE66c4ec4d7",
        token_bridge: "5a58505a96D1dbf8dF91cB21B54419FC36e93fdE",
        nft_bridge: "90BBd86a6Fe93D3bc3ed6335935447E75fAb7fCf",
        net_env: MainNet,
    },
    AVALANCHE {
        chain_id: Avalanche,
        name: "Avalanche",
        aliases: [],
        core: "54a8e5f9c4CbA08F9943965859F6c34eAF03E26c",
        token_bridge: "0e082F06FF657D94310cB8cE8B0D9a04541d8052",
        nft_bridge: "f7B6737Ca9c4e08aE573F75A97B73D7a813f5De5",
        net_env: MainNet,
    },
    OASIS {
        chain_id: Oasis,
        name: "Oasis",
        aliases: [],
        core: "fE8cD454b4A1CA468B57D79c0cc77Ef5B6f64585",
        token_bridge: "5848C791e09901b40A9Ef749f2a6735b418d7564",
        nft_bridge: "04952D522Ff217f40B5Ef3cbF659EcA7b952a6c1",
        net_env: MainNet,
    },
    AURORA {
        chain_id: Aurora,
        name: "Aurora",
        aliases: [],
        core: "a321448d90d4e5b0A732867c18eA198e75CAC48E",
        token_bridge: "51b5123a7b0F9b2bA265f9c4C8de7D78D52f510F",
        nft_bridge: "6dcC0484472523ed9Cdc017F711Bcbf909789284",
        net_env: MainNet,
    },
    FANTOM {
        chain_id: Fantom,
        name: "Fantom",
        aliases: [],
        core: "126783A6Cb203a3E35344528B26ca3a0489a1485",
        token_bridge: "7C9Fc5741288cDFdD83CeB07f3ea7e22618D79D2",
        nft_bridge: "A9c7119aBDa80d4a4E0C06C8F4d8cF5893234535",
        net_env: MainNet,
    },
    KARURA {
        chain_id: Karura,
        name: "Karura",
        aliases: [],
        core: "a321448d90d4e5b0A732867c18eA198e75CAC48E",
        token_bridge: "ae9d7fe007b3327AA64A32824Aaac52C42a6E624",
        nft_bridge: "b91e3638F82A1fACb28690b37e3aAE45d2c33808",
        net_env: MainNet,
    },
    ACALA {
        chain_id: Acala,
        name: "Acala",
        aliases: [],
        core: "a321448d90d4e5b0A732867c18eA198e75CAC48E",
        token_bridge: "ae9d7fe007b3327AA64A32824Aaac52C42a6E624",
        nft_bridge: "b91e3638F82A1fACb28690b37e3aAE45d2c33808",
        net_env: MainNet,
    },
    KLAYTN {
        chain_id: Klaytn,
        name: "Klaytn",
        aliases: [],
        core: "0C21603c4f3a6387e241c0091A7EA39E43E90bb7",
        token_bridge: "5b08ac39EAED75c0439FC750d9FE7E1F9dD0193F",
        nft_bridge: "3c3c561757BAa0b78c5C025CdEAa4ee24C1dFfEf",
        net_env: MainNet,
    },
    CELO {
        chain_id: Celo,
        name: "Celo",
        aliases: [],
        core: "a321448d90d4e5b0A732867c18eA198e75CAC48E",
        token_bridge: "796Dff6D74F3E27060B71255Fe517BFb23C93eed",
        nft_bridge: "A6A377d75ca5c9052c9a77ED1e865Cc25Bd97bf3",
        net_env: MainNet,
    },
    MOONBEAM {
        chain_id: Moonbeam,
        name: "Moonbeam",
        aliases: [],
        core: "C8e2b0cD52Cf01b0Ce87d389Daa3d414d4cE29f3",
        token_bridge: "b1731c586ca89a23809861c6103f0b96b3f57d92",
        nft_bridge: "453cfbe096c0f8d763e8c5f24b441097d577bde2",
        net_env: MainNet,
    },
    ARBITRUM {
        chain_id: Arbitrum,
        name: "Arbitrum",
        aliases: [],
        core: "a5f208e072434bC67592E4C49C1B991BA79BCA46",
        token_bridge: "0b2402144Bb366A632D14B83F244D2e0e21bD39c",
        nft_bridge: "3dD14D553cFD986EAC8e3bddF629d82073e188c8",
        net_env: MainNet,
    },
    OPTIMISM {
        chain_id: Optimism,
        name: "Optimism",
        aliases: [],
        core: "Ee91C335eab126dF5fDB3797EA9d6aD93aeC9722",
        token_bridge: "1D68124e65faFC907325e3EDbF8c4d84499DAa8b",
        nft_bridge: "fE8cD454b4A1CA468B57D79c0cc77Ef5B6f64585",
        net_env: MainNet,
    },
    GNOSIS {
        chain_id: Gnosis,
        name: "Gnosis",
        aliases: [],
        core: "a321448d90d4e5b0A732867c18eA198e75CAC48E",
        net_env: MainNet,
    },
    BASE {
        chain_id: Base,
        name: "Base",
        aliases: [],
        core: "bebdb6C8ddC678FfA9f8748f85C815C556Dd8ac6",
        token_bridge: "8d2de8d2f73F1F4cAB472AC9A881C9b123C79627",
        nft_bridge: "DA3adC6621B2677BEf9aD26598e6939CF0D92f88",
        net_env: MainNet,
    },
    ROOTSTOCK {
        chain_id: Rootstock,
        name: "Rootstock",
        aliases: [],
        core: "bebdb6C8ddC678FfA9f8748f85C815C556Dd8ac6",
        net_env: MainNet,
    },
    SCROLL {
        chain_id: Scroll,
        name: "Scroll",
        aliases: [],
        core: "bebdb6C8ddC678FfA9f8748f85C815C556Dd8ac6",
        token_bridge: "24850c6f61C438823F01B7A3BF2B89B72174Fa9d",
        net_env: MainNet,
    },
    MANTLE {
        chain_id: Mantle,
        name: "Mantle",
        aliases: [],
        core: "bebdb6C8ddC678FfA9f8748f85C815C556Dd8ac6",
        token_bridge: "24850c6f61C438823F01B7A3BF2B89B72174Fa9d",
        net_env: MainNet,
    },
    BLAST {
        chain_id: Blast,
        name: "Blast",
        aliases: [],
        core: "bebdb6C8ddC678FfA9f8748f85C815C556Dd8ac6",
        token_bridge: "24850c6f61C438823F01B7A3BF2B89B72174Fa9d",
        net_env: MainNet,
    },
    XLAYER {
        chain_id: XLayer,
        name: "XLayer",
        aliases: [],
        core: "194B123c5E96B9b2E49763619985790Dc241CAC0",
        token_bridge: "5537857664B0f9eFe38C9f320F75fEf23234D904",
        net_env: MainNet,
    },
    SNAXCHAIN {
        chain_id: Snaxchain,
        name: "Snaxchain",
        aliases: [],
        core: "c1BA3CC4bFE724A08FbbFbF64F8db196738665f4",
        token_bridge: "8B94bfE456B48a6025b92E11Be393BAa86e68410",
        net_env: MainNet,
    },

    // TestNets

    GOERLI {
        chain_id: Ethereum,
        name: "Goerli",
        aliases: ["Ethereum"],
        core: "706abc4E45D419950511e474C7B9Ed348A4a716c",
        token_bridge: "F890982f9310df57d00f659cf4fd87e65adEd8d7",
        nft_bridge: "D8E4C2DbDd2e2bd8F1336EA691dBFF6952B1a6eB",
        net_env: TestNet,
    },
    BSC_TESTNET {
        chain_id: Bsc,
        name: "BscTestnet",
        aliases: ["Bsc"],
        core: "68605AD7b15c732a30b1BbC62BE8F2A509D74b4D",
        token_bridge: "9dcF9D205C9De35334D646BeE44b2D2859712A09",
        nft_bridge: "cD16E5613EF35599dc82B24Cb45B5A93D779f1EE",
        net_env: TestNet,
    },
    POLYGON_TESTNET {
        chain_id: Polygon,
        name: "PolygonTestnet",
        aliases: ["Polygon"],
        core: "0CBE91CF822c73C2315FB05100C2F714765d5c20",
        token_bridge: "377D55a7928c046E18eEbb61977e714d2a76472a",
        nft_bridge: "51a02d0dcb5e52F5b92bdAA38FA013C91c7309A9",
        net_env: TestNet,
    },
    FUJI {
        chain_id: Avalanche,
        name: "Fuji",
        aliases: ["Avalanche"],
        core: "7bbcE28e64B3F8b84d876Ab298393c38ad7aac4C",
        token_bridge: "61E44E506Ca5659E6c0bba9b678586fA2d729756",
        nft_bridge: "D601BAf2EEE3C028344471684F6b27E789D9075D",
        net_env: TestNet,
    },
    OASIS_TESTNET {
        chain_id: Oasis,
        name: "OasisTestnet",
        aliases: ["Oasis"],
        core: "c1C338397ffA53a2Eb12A7038b4eeb34791F8aCb",
        token_bridge: "88d8004A9BdbfD9D28090A02010C19897a29605c",
        nft_bridge: "C5c25B41AB0b797571620F5204Afa116A44c0ebA",
        net_env: TestNet,
    },
    AURORA_TESTNET {
        chain_id: Aurora,
        name: "AuroraTestnet",
        aliases: ["Aurora"],
        core: "Bd07292de7b505a4E803CEe286184f7Acf908F5e",
        token_bridge: "D05eD3ad637b890D68a854d607eEAF11aF456fba",
        nft_bridge: "8F399607E9BA2405D87F5f3e1B78D950b44b2e24",
        net_env: TestNet,
    },
    FANTOM_TESTNET {
        chain_id: Fantom,
        name: "FantomTestnet",
        aliases: ["Fantom"],
        core: "1BB3B4119b7BA9dfad76B0545fb3F531383c3bB7",
        token_bridge: "599CEa2204B4FaECd584Ab1F2b6aCA137a0afbE8",
        nft_bridge: "63eD9318628D26BdCB15df58B53BB27231D1B227",
        net_env: TestNet,
    },
    KARURA_TESTNET {
        chain_id: Karura,
        name: "KaruraTestnet",
        aliases: ["Karura"],
        core: "64fb09E405D2043ed7785a29E296C766D56F2056",
        token_bridge: "e157115ef34c93145Fec2FE53706846853B07F42",
        net_env: TestNet,
    },
    MANDALA {
        chain_id: Acala,
        name: "Mandala",
        aliases: ["Acala"],
        core: "64fb09E405D2043ed7785a29E296C766D56F2056",
        token_bridge: "e157115ef34c93145Fec2FE53706846853B07F42",
        net_env: TestNet,
    },
    BAOBAB {
        chain_id: Klaytn,
        name: "Baobab",
        aliases: ["Klaytn"],
        core: "1830CC6eE66c84D2F177B94D544967c774E624cA",
        token_bridge: "C7A13BE098720840dEa132D860fDfa030884b09A",
        nft_bridge: "94c994fC51c13101062958b567e743f1a04432dE",
        net_env: TestNet,
    },
    ALFAJORES {
        chain_id: Celo,
        name: "Alfajores",
        aliases: ["Celo"],
        core: "88505117CA88e7dd2eC6EA1E13f0948db2D50D56",
        token_bridge: "05ca6037eC51F8b712eD2E6Fa72219FEaE74E153",
        nft_bridge: "aCD8190F647a31E56A656748bC30F69259f245Db",
        net_env: TestNet,
    },
    MOONBASE_ALPHA {
        chain_id: Moonbeam,
        name: "MoonbaseAlpha",
        aliases: ["Moonbeam"],
        core: "a5B7D85a8f27dd7907dc8FdC21FA5657D5E2F901",
        token_bridge: "bc976D4b9D57E57c3cA52e1Fd136C45FF7955A96",
        nft_bridge: "98A0F4B96972b32Fcb3BD03cAeB66A44a6aB9Edb",
        net_env: TestNet,
    },
    ARBITRUM_GOERLI {
        chain_id: Arbitrum,
        name: "ArbitrumGoerli",
        aliases: ["Arbitrum"],
        core: "C7A204bDBFe983FCD8d8E61D02b475D4073fF97e",
        token_bridge: "23908A62110e21C04F3A4e011d24F901F911744A",
        nft_bridge: "Ee3dB83916Ccdc3593b734F7F2d16D630F39F1D0",
        net_env: TestNet,
    },
    OP_GOERLI {
        chain_id: Optimism,
        name: "OPGoerli",
        aliases: ["Optimism"],
        core: "6b9C8671cdDC8dEab9c719bB87cBd3e782bA6a35",
        token_bridge: "C7A204bDBFe983FCD8d8E61D02b475D4073fF97e",
        nft_bridge: "23908A62110e21C04F3A4e011d24F901F911744A",
        net_env: TestNet,
    },
    CHIADO {
        chain_id: Gnosis,
        name: "Chiado",
        aliases: ["Gnosis"],
        core: "BB73cB66C26740F31d1FabDC6b7A46a038A300dd",
        net_env: TestNet,
    },
    BASE_GOERLI {
        chain_id: Base,
        name: "BaseGoerli",
        aliases: ["Base"],
        core: "23908A62110e21C04F3A4e011d24F901F911744A",
        token_bridge: "A31aa3FDb7aF7Db93d18DDA4e19F811342EDF780",
        nft_bridge: "F681d1cc5F25a3694E348e7975d7564Aa581db59",
        net_env: TestNet,
    },
    ROOTSTOCK_TESTNET {
        chain_id: Rootstock,
        name: "RootstockTestnet",
        aliases: ["Rootstock"],
        core: "bebdb6C8ddC678FfA9f8748f85C815C556Dd8ac6",
        net_env: TestNet,
    },
    SCROLL_SEPOLIA {
        chain_id: Scroll,
        name: "ScrollSepolia",
        aliases: ["Scroll"],
        core: "055F47F1250012C6B20c436570a76e52c17Af2D5",
        token_bridge: "22427d90B7dA3fA4642F7025A854c7254E4e45BF",
        net_env: TestNet,
    },
    MANTLE_TESTNET {
        chain_id: Mantle,
        name: "MantleTestnet",
        aliases: ["Mantle"],
        core: "376428e7f26D5867e69201b275553C45B09EE090",
        token_bridge: "75Bfa155a9D7A3714b0861c8a8aF0C4633c45b5D",
        net_env: TestNet,
    },
    BLAST_SEPOLIA {
        chain_id: Blast,
        name: "BlastSepolia",
        aliases: ["Blast"],
        core: "473e002D7add6fB67a4964F13bFd61280Ca46886",
        token_bridge: "430855B4D43b8AEB9D2B9869B74d58dda79C0dB2",
        net_env: TestNet,
    },
    XLAYER_TESTNET {
        chain_id: XLayer,
        name: "XLayerTestnet",
        aliases: ["XLayer"],
        core: "A31aa3FDb7aF7Db93d18DDA4e19F811342EDF780",
        token_bridge: "dA91a06299BBF302091B053c6B9EF86Eff0f930D",
        net_env: TestNet,
    },
    LINEA_SEPOLIA {
        chain_id: Linea,
        name: "LineaSepolia",
        aliases: ["Linea"],
        core: "79A1027a6A159502049F10906D333EC57E95F083",
        token_bridge: "C7A204bDBFe983FCD8d8E61D02b475D4073fF97e",
        net_env: TestNet,
    },
    ARTIO {
        chain_id: Berachain,
        name: "Artio",
        aliases: ["Berachain", "bArtio"],
        core: "BB73cB66C26740F31d1FabDC6b7A46a038A300dd",
        token_bridge: "a10f2eF61dE1f19f586ab8B6F2EbA89bACE63F7a",
        net_env: TestNet,
    },

    ATLANTIC_2_EVM {
        chain_id: SeiEvm,
        name: "atlantic-2-evm",
        aliases: ["seievm"],
        core: "07782FCe991dAb4DE7a3124032E534A0D059B4d8",
        net_env: TestNet,
    },
    SNAXCHAIN_TESTNET {
        chain_id: Snaxchain,
        name: "SnaxchainTestnet",
        aliases: ["Snaxchain"],
        core: "BB73cB66C26740F31d1FabDC6b7A46a038A300dd",
        token_bridge: "a10f2eF61dE1f19f586ab8B6F2EbA89bACE63F7a",
        net_env: TestNet,
    },
    SEPOLIA {
        chain_id: Sepolia,
        name: "Sepolia",
        aliases: [],
        core: "4a8bc80Ed5a4067f1CCf107057b8270E0cC11A78",
        token_bridge: "DB5492265f6038831E89f495670FF909aDe94bd9",
        nft_bridge: "6a0B52ac198e4870e5F3797d5B403838a5bbFD99",
        net_env: TestNet,
    },
    ARBITRUM_SEPOLIA {
        chain_id: ArbitrumSepolia,
        name: "ArbitrumSepolia",
        aliases: [],
        core: "6b9C8671cdDC8dEab9c719bB87cBd3e782bA6a35",
        token_bridge: "C7A204bDBFe983FCD8d8E61D02b475D4073fF97e",
        nft_bridge: "23908A62110e21C04F3A4e011d24F901F911744A",
        net_env: TestNet,
    },
    BASE_SEPOLIA {
        chain_id: BaseSepolia,
        name: "BaseSepolia",
        aliases: [],
        core: "79A1027a6A159502049F10906D333EC57E95F083",
        token_bridge: "86F55A04690fd7815A3D802bD587e83eA888B239",
        nft_bridge: "268557122Ffd64c85750d630b716471118F323c8",
        net_env: TestNet,
    },
    OPTIMISM_SEPOLIA {
        chain_id: OptimismSepolia,
        name: "OptimismSepolia",
        aliases: [],
        core: "31377888146f3253211EFEf5c676D41ECe7D58Fe",
        token_bridge: "99737Ec4B815d816c49A385943baf0380e75c0Ac",
        nft_bridge: "27812285fbe85BA1DF242929B906B31EE3dd1b9f",
        net_env: TestNet,
    },
    HOLESKY {
        chain_id: Holesky,
        name: "Holesky",
        aliases: [],
        core: "a10f2eF61dE1f19f586ab8B6F2EbA89bACE63F7a",
        token_bridge: "76d093BbaE4529a342080546cAFEec4AcbA59EC6",
        nft_bridge: "c8941d483c45eF8FB72E4d1F9dDE089C95fF8171",
        net_env: TestNet,
    },
    POLYGON_SEPOLIA {
        chain_id: PolygonSepolia,
        name: "PolygonSepolia",
        aliases: [],
        core: "6b9C8671cdDC8dEab9c719bB87cBd3e782bA6a35",
        token_bridge: "C7A204bDBFe983FCD8d8E61D02b475D4073fF97e",
        nft_bridge: "23908A62110e21C04F3A4e011d24F901F911744A",
        net_env: TestNet,
    },

    // DevNets

    ETHEREUM_DEVNET {
        chain_id: Ethereum,
        name: "EthereumDevnet",
        aliases: ["Ethereum"],
        core: "C89Ce4735882C9F0f0FE26686c53074E09B0D550",
        token_bridge: "0290FB167208Af455bB137780163b7B7a9a10C16",
        nft_bridge: "26b4afb60d6c903165150c6f0aa14f8016be4aec",
        net_env: DevNet,
    },
    BSC_DEVNET {
        chain_id: Bsc,
        name: "BscDevnet",
        aliases: ["Bsc"],
        core: "C89Ce4735882C9F0f0FE26686c53074E09B0D550",
        token_bridge: "0290FB167208Af455bB137780163b7B7a9a10C16",
        nft_bridge: "26b4afb60d6c903165150c6f0aa14f8016be4aec",
        net_env: DevNet,
    },

}

// Solana
nets_group! {solana, Solana, SOLANA_NETS, b58;
    // MainNets

    SOLANA {
        chain_id: Solana,
        name: "Solana",
        aliases: [],
        core: "worm2ZoG2kUd4vFXhvjh93UUH596ayRfgQ2MgjNMTth",
        token_bridge: "wormDTUJ6AWPNvk59vGQbDvGJmqbDTdgWgAqcLBCgUb",
        nft_bridge: "WnFt12ZrnzZrFZkt2xsNsaNWoQribnuQ5B5FrDbwDhD",
        net_env: MainNet,
    },
    PYTHNET {
        chain_id: Pythnet,
        name: "Pythnet",
        aliases: [],
        core: "H3fxXJ86ADW2PNuDDmZJg6mzTtPxkYCpNuQUTgmJ7AjU",
        net_env: MainNet,
    },

    // TestNets

    SOLANA_TESTNET {
        chain_id: Solana,
        name: "SolanaTestnet",
        aliases: ["Solana"],
        core: "3u8hJUVTA4jH1wYAyUur7FFZVQ8H635K3tSHHF4ssjQ5",
        token_bridge: "DZnkkTmCiFWfYTfT41X3Rd1kDgozqzxWaHqsw6W4x2oe",
        nft_bridge: "2rHhojZ7hpu1zA91nvZmT8TqWWvMcKmmNBCr2mKTtMq4",
        net_env: TestNet,
    },
    PYTHNET_TESTNET {
        chain_id: Pythnet,
        name: "PythnetTestnet",
        aliases: ["Pythnet"],
        core: "EUrRARh92Cdc54xrDn6qzaqjA77NRrCcfbr8kPwoTL4z",
        net_env: TestNet,
    },

    // DevNets

    SOLANA_DEVNET {
        chain_id: Solana,
        name: "SolanaDevnet",
        aliases: ["Solana"],
        core: "Bridge1p5gheXUvJ6jGWGeCsgPKgnE3YgdGKRVCMY9o",
        token_bridge: "B6RHG3mfcckmrYN1UhmJzyS1XX3fZKbkeUcpJe9Sy3FE",
        nft_bridge: "NFTWqJR8YnRVqPDvTJrYuLrQDitTG5AScqbeghi4zSA",
        net_env: DevNet,
    },
}

// CosmWasm
nets_group! {cosm_wasm, CosmWasm, COSM_WASM_NETS, bech32;
    // MainNets
    TERRA {
        chain_id: Terra,
        name: "Terra",
        aliases: [],
        core: "terra1dq03ugtd40zu9hcgdzrsq6z2z4hwhc9tqk2uy5",
        token_bridge: "terra10nmmwe8r3g99a9newtqa7a75xfgs2e8z87r2sf",
        net_env: MainNet,
    },
    TERRA2 {
        chain_id: Terra2,
        name: "Terra2",
        aliases: [],
        core: "terra12mrnzvhx3rpej6843uge2yyfppfyd3u9c3uq223q8sl48huz9juqffcnhp",
        token_bridge: "terra153366q50k7t8nn7gec00hg66crnhkdggpgdtaxltaq6xrutkkz3s992fw9",
        net_env: MainNet,
    },
    INJECTIVE {
        chain_id: Injective,
        name: "Injective",
        aliases: [],
        core: "inj17p9rzwnnfxcjp32un9ug7yhhzgtkhvl9l2q74d",
        token_bridge: "inj1ghd753shjuwexxywmgs4xz7x2q732vcnxxynfn",
        net_env: MainNet,
    },
    XPLA {
        chain_id: Xpla,
        name: "Xpla",
        aliases: [],
        core: "xpla1jn8qmdda5m6f6fqu9qv46rt7ajhklg40ukpqchkejcvy8x7w26cqxamv3w",
        token_bridge: "xpla137w0wfch2dfmz7jl2ap8pcmswasj8kg06ay4dtjzw7tzkn77ufxqfw7acv",
        net_env: MainNet,
    },
    SEI {
        chain_id: Sei,
        name: "Sei",
        aliases: [],
        core: "sei1gjrrme22cyha4ht2xapn3f08zzw6z3d4uxx6fyy9zd5dyr3yxgzqqncdqn",
        token_bridge: "sei1smzlm9t79kur392nu9egl8p8je9j92q4gzguewj56a05kyxxra0qy0nuf3",
        net_env: MainNet,
    },
    WORMCHAIN {
        chain_id: Wormchain,
        name: "Wormchain",
        aliases: [],
        core: "wormhole1ufs3tlq4umljk0qfe8k5ya0x6hpavn897u2cnf9k0en9jr7qarqqaqfk2j",
        token_bridge: "wormhole1466nf3zuxpya8q9emxukd7vftaf6h4psr0a07srl5zw74zh84yjq4lyjmh",
        net_env: MainNet,
    },
    NEUTRON {
        chain_id: Neutron,
        name: "Neutron",
        aliases: [""],
        core: "neutron16rerygcpahqcxx5t8vjla46ym8ccn7xz7rtc6ju5ujcd36cmc7zs9zrunh",
        net_env: MainNet,
    },

    // TestNets

    BOMBAY {
        chain_id: Terra,
        name: "Bombay",
        aliases: ["Terra"],
        core: "terra1pd65m0q9tl3v8znnz5f5ltsfegyzah7g42cx5v",
        token_bridge: "terra1pseddrv0yfsn76u4zxrjmtf45kdlmalswdv39a",
        net_env: TestNet,
    },
    PISCO {
        chain_id: Terra2,
        name: "Pisco",
        aliases: ["Terra2"],
        core: "terra1pd65m0q9tl3v8znnz5f5ltsfegyzah7g42cx5v",
        token_bridge: "terra1pseddrv0yfsn76u4zxrjmtf45kdlmalswdv39a",
        net_env: TestNet,
    },
    INJECTIVE_TESTNET {
        chain_id: Injective,
        name: "InjectiveTestnet",
        aliases: ["injective"],
        core: "inj1xx3aupmgv3ce537c0yce8zzd3sz567syuyedpg",
        token_bridge: "inj1q0e70vhrv063eah90mu97sazhywmeegp7myvnh",
        net_env: TestNet,
    },
    XPLA_TESTNET {
        chain_id: Xpla,
        name: "XplaTestnet",
        aliases: ["Xpla"],
        core: "xpla1upkjn4mthr0047kahvn0llqx4qpqfn75lnph4jpxfn8walmm8mqsanyy35",
        token_bridge:  "xpla1kek6zgdaxcsu35nqfsyvs2t9vs87dqkkq6hjdgczacysjn67vt8sern93x",
        net_env: TestNet,
    },
    ATLANTIC_2 {
        chain_id: Sei,
        name: "atlantic-2",
        aliases: ["Sei"],
        core: "sei1nna9mzp274djrgzhzkac2gvm3j27l402s4xzr08chq57pjsupqnqaj0d5s",
        token_bridge: "sei1jv5xw094mclanxt5emammy875qelf3v62u4tl4lp5nhte3w3s9ts9w9az2",
        net_env: TestNet,
    },
    WORMCHAIN_TESTNET {
        chain_id: Wormchain,
        name: "WormchainTestnet",
        aliases: ["Wormchain"],
        core: "wormhole16jzpxp0e8550c9aht6q9svcux30vtyyyyxv5w2l2djjra46580wsazcjwp",
        token_bridge: "wormhole1aaf9r6s7nxhysuegqrxv0wpm27ypyv4886medd3mrkrw6t4yfcnst3qpex",
        net_env: TestNet,
    },
    OSMOSIS_TESTNET {
        chain_id: Osmosis,
        name: "OsmosisTestnet",
        aliases: ["Osmosis"],
        core: "osmo1hggkxr0hpw83f8vuft7ruvmmamsxmwk2hzz6nytdkzyup9krt0dq27sgyx",
        net_env: TestNet,
    },
    PION {
        chain_id: Neutron,
        name: "Pion",
        aliases: ["Neutron"],
        core: "neutron1enf63k37nnv9cugggpm06mg70emcnxgj9p64v2s8yx7a2yhhzk2q6xesk4",
        net_env: TestNet,
    },

    // DevNets

    TERRA_DEVNET {
        chain_id: Terra,
        name: "TerraDevnet",
        aliases: ["Terra"],
        core: "terra14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9ssrc8au",
        token_bridge:
        "terra1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrquka9l6",
        net_env: DevNet,
    },
    TERRA2_DEVNET {
        chain_id: Terra2,
        name: "Terra2Devnet",
        aliases: ["Terra2"],
        core: "terra14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9ssrc8au",
        token_bridge: "terra1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrquka9l6",
        net_env: DevNet,
    },
    WORMCHAIN_DEVNET {
        chain_id: Wormchain,
        name: "WormchainDevnet",
        aliases: ["Wormchain"],
        core: "wormhole1ghd753shjuwexxywmgs4xz7x2q732vcnkm6h2pyv9s6ah3hylvrqtm7t3h",
        token_bridge: "wormhole1eyfccmjm6732k7wp4p6gdjwhxjwsvje44j0hfx8nkgrm8fs7vqfssvpdkx",
        net_env: DevNet,
    },
}
