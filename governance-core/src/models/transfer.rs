use std::iter;

use canonical_derive::Canon;
use dusk_bls12_381::BlsScalar;
use dusk_pki::PublicKey;

// TODO: The same struct exists in the governance contract, do we just import that?
#[derive(Debug, Clone, PartialEq, Eq, Canon)]
pub struct Transfer {
    pub from: PublicKey,
    pub to: PublicKey,
    pub amount: u64,
    pub timestamp: u64,
}

impl Transfer {
    pub fn as_scalars(&self) -> impl Iterator<Item = BlsScalar> {
        let from = self.from.as_ref().to_hash_inputs();
        let to = self.from.as_ref().to_hash_inputs();
        let amount = BlsScalar::from(self.amount);
        let timestamp = BlsScalar::from(self.timestamp);

        iter::once(from)
            .chain(iter::once(to))
            .chain(iter::once([amount, timestamp]))
            .flatten()
    }
}
