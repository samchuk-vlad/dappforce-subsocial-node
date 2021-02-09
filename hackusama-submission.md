# What we built during [Hackusama](https://hackusama.devpost.com/)

During Hackusama hackathon we developed several new pallets that can be used by Kusama community:

1. [Session Keys](./pallets/session-keys) pallet (not related to validator's session keys) - it allows to set up a proxy for your main account and specify the max limit of tokens that this proxy can spend, plus it is possible to specify the expiration block tims after which this proxy will disfunction. This pallet is very useful for UX of Substrate dapps in order to create a utility proxy account that will act (sign txs) on behalf of the main account so that UI will not ask a "Sign tx" confirmation modal for a specific set of extrinsic initiated by this proxy session key.

2. [Moderation](./pallets/moderation) pallet - it allows space moderators to block any unwanted content by AccountId, IPFS CID, Post Id, Space Id. Users will be able to report to community moderators the bad content by its IPFS CID or Post Id to the moderators of the community where they saw this bad content. Moderators of the community will review reports and suggest a new status for the reported entities: Blocked or Allowed. Then the owner (or main manager) of the community is up to decide whether to Block or Allow the entities based on the review feedback from community moderators. This pallet also has a setting to auto-block the content after a specific number of statuses from moderators that suggest to Block the entity. If the entity is added to AllowList, then the entity cannot be blocked.

3. [Donations](./pallets/donations) pallet. Allows a user to donate any amount of tokens to space, post, or account they like. This feature allows content creators to monetize their content via community donations. Also, it's possible to use this pallet for nonprofit / charity projects to collect contributions. Pallet provides a few settings to specify a donation wallet per-account, per-space, per-post. It's possible to enable/disable donations per space. It's possible to specify an optional min and max amount per donation that could be accepted.

4. [Paid Subscriptions](./pallets/subscriptions) pallet. Similar to the feature of Patreon and Substack, where fans/patrons/supporters of the creator or nonprofit organization can contribute to the project/person they love on a monthly, quarterly, yearly bases. This feature allows content creators to monetize their content via their community. Pallet provides a way for creators to create a list of subscription plans (aka levels, tiers) and specify a different price and period per each plan. There are several pre-built subscription plans: Daily, Weekly, Monthly, Yearly. We are planning to use Substrate Schedule pallet to schedule recurring transfers from patrons' wallets to creators' wallets.

## Improved web UI

During Hackusama we also improved our web UI a lot:

- Created and used a new improved TxButton & Substrate Context for React.
- Implemented hide/show spaces, posts via UI and blockchain.
- Started using Redux for instant add of comments on UI.
- Remove Semantic UI dependency (legacy from Polkadot Apps)
- Migrate from Ant Design v3 to v4 (UI components)
- Migrate to Substrate v2.0.0-rc4
- Added support Kusama identities and showing role labels: Validator, Council member, etc.

To see changes that have been made during Hackusama to Subsocial's web UI, check out this PR:
https://github.com/dappforce/dappforce-subsocial-ui/pull/260

## Live chain and Apps

For your convenience, we launched a live chain that includes the pallets that we have created during Hackusama:
http://161.35.193.43/bc

## Run Subsocial node in a `dev` mode:

To run the node locally, execute the next command:

```
docker run -d --rm --name subsocial-node -p 9944:9944 dappforce/subsocial-apps:hackusama subsocial-node --dev
```

## Stop Subsocial node

```
docker stop subsocial-node
```

## Run Polkadot.js Apps with [Subsocial types](./types.json)

```
docker run -d --rm --name subsocial-apps -p 3000:80 dappforce/subsocial-apps:hackusama
```

Note: port `3000` could be changed to another value.

## Stop Polkadot.js Apps

```
docker stop subsocial-apps
```
