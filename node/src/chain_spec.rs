use sp_core::{Pair, Public, sr25519, crypto::UncheckedInto};
use subsocial_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, UtilsConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature, constants::currency::DOLLARS,
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use hex_literal::hex;

// Note this is the URL for the telemetry server
const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
const DEFAULT_PROTOCOL_ID: &str = "sub";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key for Aura
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn development_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Development",
		"dev",
		ChainType::Development,
		|| {
			let endowed_accounts = vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			];

			testnet_genesis(
				vec![
					authority_keys_from_seed("Alice"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				endowed_accounts.iter().cloned().map(|k| (k, 10_000)).collect(),
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				true,
			)
		},
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		Some(subsocial_properties()),
		None,
	)
}

pub fn local_testnet_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Local Testnet",
		"local_testnet",
		ChainType::Local,
		|| {
			let endowed_accounts = vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			];

			testnet_genesis(
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
				],
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				endowed_accounts.iter().cloned().map(|k| (k, 10_000)).collect(),
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				true,
			)
		},
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		Some(subsocial_properties()),
		None,
	)
}

pub fn subsocial_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/subsocialSpecJson.json")[..])
}

pub fn subsocial_staging_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Subsocial",
		"subsocial",
		ChainType::Live,
		|| testnet_genesis(
			vec![
				(
					/* AuraId */
					hex!["ac940b8ee399d42faeb7169f322e6623f8219d12ad4c42dfe0995fa9f9713a0d"].unchecked_into(),
					/* GrandpaId */
					hex!["e97b51af33429b5c4ab8ddd9b3fc542d24154bbeef807d559eff3906afca8413"].unchecked_into()
				),
				(
					/* AuraId */
					hex!["0c053087dd7782de467228b5f826c5031be2faf315baa766a89b48bb6e2dfb71"].unchecked_into(),
					/* GrandpaId */
					hex!["b48a83ed87ef39bc90c205fb551af3c076e1a952881d7fefec08cbb76e17ab8b"].unchecked_into()
				),
			],
			hex!["24d6d7cd9a0500be768efc7b5508e7861cbde7cfc06819e4dfd9120b97d46d3e"].into(),
			vec![
				(
					/* Account */
					hex!["a8d5b1558ee63ed2c55c8fb71afd2cbe7a2f61c0fc2dbab741ca652ecf6a3f45"].into(),
					/* Balance */
					1_000
				),
				(
					/* Account */
					hex!["24d6d996a8bb42a63904afc36d610986e8d502f65898da62cb281cfe7f23b02f"].into(),
					/* Balance */
					2_499_750
				),
				(
					/* Account */
					hex!["24d6d8fc5d051fd471e275f14c83e95287d2b863e4cc802de1f78dea06c6ca78"].into(),
					/* Balance */
					2_499_750
				),
				(
					/* Account */
					hex!["24d6d901fb0531124040630e52cfd746ef7d037922c4baf290f513dbc3d47d66"].into(),
					/* Balance */
					2_499_750
				),
				(
					/* Account */
					hex!["24d6d22d63313e82f9461281cb69aacad1828dc74273274751fd24333b182c68"].into(),
					/* Balance */
					2_499_750
				),
			],
			hex!["24d6d683750c4c10e90dd81430efec95133e1ec1f5be781d3267390d03174706"].into(),
			true,
		),
		vec![],
		Some(TelemetryEndpoints::new(
			vec![(STAGING_TELEMETRY_URL.to_string(), 0)]
		).expect("Staging telemetry url is valid; qed")),
		Some(DEFAULT_PROTOCOL_ID),
		Some(subsocial_properties()),
		None,
	)
}

fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	treasury_account_id: AccountId,
	_enable_println: bool
) -> GenesisConfig {
	GenesisConfig {
		system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|(k, b)|(k, b * DOLLARS)).collect(),
		}),
		aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		}),
		grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		pallet_utils: Some(UtilsConfig {
			treasury_account: treasury_account_id,
		}),
	}
}

pub fn subsocial_properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 28.into());
	properties.insert("tokenDecimals".into(), 10.into());
	properties.insert("tokenSymbol".into(), "SMN".into());

	properties
}
