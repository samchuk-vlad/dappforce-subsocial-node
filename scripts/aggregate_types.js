// Reads in the type definitions from all pallets in the runtime and the runtime's own types
// Naively aggregates types and writes them to disk.

const fs = require('fs');

const pallets = [
  "donations",
  "moderation",
  "permissions",
  "post-history",
  "posts",
  "profile-history",
  "profiles",
  "reactions",
  "roles",
  "scores",
  "session-keys",
  "space-history",
  "spaces",
  "subscriptions",
  "utils",
]

// Types that are native to the runtime itself (ie come from lib.rs)
// These specifics are from https://polkadot.js.org/api/start/types.extend.html#impact-on-extrinsics
const runtimeOwnTypes = {
  "Address": "AccountId",
  "LookupSource": "AccountId"
}

const subsocialCustomTypes = {
  "IpfsCid": "Text"
}

// Loop through all pallets aggregating types
let finalTypes = {...runtimeOwnTypes, ...subsocialCustomTypes};
let palletTypes;
for (let dirname of pallets) {
  let path = `../pallets/${dirname}/types.json`;
  palletTypes = JSON.parse(fs.readFileSync(path, 'utf8'));
  finalTypes = {...finalTypes, ...palletTypes};
}

// Write output to disk
fs.writeFileSync("../types.json", JSON.stringify(finalTypes, null, 2), 'utf8');