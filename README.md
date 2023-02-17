# TokenFactory Core (middleware)

This is a contract which you give admin of your token denomination to from the tokenfactory module.

Then other contracts you/your DAO control can call this contract to perform functions like Minting tokens to a user via a WasmMsg.

This is a requirement since TokenFactoryMsg's do not allow for a standard Reponse type since its Reponse<TokenFactoryMsg>.

This actually makes it more flexible since multiple contracts can "mint" tokens on behalf of the contract admin :D