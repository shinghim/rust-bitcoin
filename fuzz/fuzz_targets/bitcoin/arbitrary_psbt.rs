#![cfg_attr(fuzzing, no_main)]
#![cfg_attr(not(fuzzing), allow(unused))]

use libfuzzer_sys::fuzz_target;
use arbitrary::{Arbitrary, Unstructured};

use bitcoin::{FeeRate, Psbt, Transaction, bip32::Xpriv};

#[cfg(not(fuzzing))]
fn main() {}

fn do_test(data: &[u8]) {
    let mut u = Unstructured::new(data);
    let p = Psbt::arbitrary(&mut u);

    if let Ok(mut psbt) = p {

        if let Ok(tx) = Transaction::arbitrary(&mut u) {
            let _ = Psbt::from_unsigned_tx(tx);
        }

        // There is a known panic when calling Psbt::iter_funding_utxo() if this condition isn't met, and the
        // function calls here call Psbt::iter_funding_utxo() somewhere down the line
        if psbt.inputs.len() == psbt.unsigned_tx.inputs.len() {
            let _ = psbt.clone().extract_tx();

            if let Ok(fee_rate) = FeeRate::arbitrary(&mut u) {
                let _ = psbt.clone().extract_tx_with_fee_rate_limit(fee_rate);
            }
        }

        if let Ok(index) = usize::arbitrary(&mut u) {
            let _ = psbt.spend_utxo(index);
        }

        if let Ok(xpriv) = Xpriv::arbitrary(&mut u) {
            let _ = psbt.sign(&xpriv);
        }

        if let Ok(other) = Psbt::arbitrary(&mut u) {
            let _ = psbt.combine(other);
        }
    }
}

fuzz_target!(|data| {
    do_test(data);
});
