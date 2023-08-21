module.exports = {
  validator: {
    killRunningValidators: true,
    programs: [],
    accounts: [
      {
        label: "Token Program",
        accountId: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        executable: true,
        cluster: "https://api.mainnet-beta.solana.com",
      },
      {
        label: "Associated Token Program",
        accountId: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
        executable: true,
        cluster: "https://api.mainnet-beta.solana.com",
      },
      {
        label: "Token Metadata Program",
        accountId: "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
        executable: true,
        cluster: "https://api.mainnet-beta.solana.com",
      },
    ],
    jsonRpcUrl: "127.0.0.1",
    websocketUrl: "",
    commitment: "confirmed",
    ledgerDir: "./test-ledger",
    resetLedger: true,
    verifyFees: false,
    detached: false,
  },
  relay: {
    enabled: true,
    killlRunningRelay: true,
  },
  storage: {
    enabled: true,
    storageId: "mock-storage",
    clearOnStart: true,
  },
};
