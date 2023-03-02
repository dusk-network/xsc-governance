# Dusk XSC Governance

The backend generates transactions to be sent to the governance smart contract

Project structure 

```rust
json.rs // handles json conversion
lib.rs // holds logic to send data to blockchain, main backend struct
models.rs // All the helper types and the models folder
    events.rs // Types needed for serializing json
    transfer.rs // Transfer struct we send to the blockchain
config.rs // rusk config and SecureWallet
```
