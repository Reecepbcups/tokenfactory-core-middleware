# TokenFactory Core (middleware)

This is a contract to which you give the admin of your token denomination from the TokenFactory module.

Then other contracts you/your DAO control can call this contract to perform functions like Minting tokens to a user via a WasmMsg.

This is a requirement since TokenFactoryMsg's do not allow for a standard Reponse type since its Reponse<TokenFactoryMsg>.

This actually makes it more flexible since multiple contracts can "mint" tokens on behalf of the contract admin :D

# How To Use

Add the following to your `Cargo.toml` dependencies for a contract:

```toml
[dependencies]
tokenfactory-types = { git = "https://github.com/Reecepbcups/tokenfactory-core-contract" }
```

You can view an example of how to use this in the [example contract](./contracts/tf_example/)

Instantiate the `tokenfactory_core` contract (view [e2e test](./e2e/test_e2e.sh) for examples)
