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

pub fn subsocial_staging_config() -> ChainSpec {
	ChainSpec::from_genesis(
		"Subsocial",
		"subsocial",
		ChainType::Live,
		|| testnet_genesis(
			vec![
				(
					/* AuraId */
					hex!["d4f4482d82913b5a4ef195bba7ec5567ee04b6ea784139c04ee5e77530670149"].unchecked_into(),
					/* GrandpaId */
					hex!["419c48625508cc74c8c125dd0132158d3583b479975e516c3c2cc1457d11a7c5"].unchecked_into()
				),
				(
					/* AuraId */
					hex!["640f97526d653b620d1ccbdefa4cb212cc0dfc76f77a0f631f96f009d986464b"].unchecked_into(),
					/* GrandpaId */
					hex!["b553650f5032ecadba5df3bd35e8d280bc6f5cdff1beda8c504d6cfecc89c63a"].unchecked_into()
				),
			],
			hex!["a8d5b1558ee63ed2c55c8fb71afd2cbe7a2f61c0fc2dbab741ca652ecf6a3f45"].into(),
			vec![
				(
					/* Account */
					hex!["a8d5b1558ee63ed2c55c8fb71afd2cbe7a2f61c0fc2dbab741ca652ecf6a3f45"].into(),
					/* Balance */
					10_000
				),
				(
					/* Account */
					hex!["24d6d996a8bb42a63904afc36d610986e8d502f65898da62cb281cfe7f23b02f"].into(),
					/* Balance */
					2_497_500
				),
				(
					/* Account */
					hex!["24d6d8fc5d051fd471e275f14c83e95287d2b863e4cc802de1f78dea06c6ca78"].into(),
					/* Balance */
					2_497_500
				),
				(
					/* Account */
					hex!["24d6d901fb0531124040630e52cfd746ef7d037922c4baf290f513dbc3d47d66"].into(),
					/* Balance */
					2_497_500
				),
				(
					/* Account */
					hex!["24d6d22d63313e82f9461281cb69aacad1828dc74273274751fd24333b182c68"].into(),
					/* Balance */
					2_497_500
				),
			],
			hex!["fa8f0122f7f950feb5d5513f4ad6d0446402a5c99572801f5dfde0169be1db5e"].into(),
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
