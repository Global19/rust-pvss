extern crate pvss;
extern crate bincode;

use bincode::{serialize, deserialize, Infinite};

use std::fmt;

/// Slice pretty print helper
pub struct PrettySlice<'a> (&'a [u8]);

impl<'a> fmt::Display for PrettySlice<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		for i in 0..self.0.len() {
			write!(f, "{:02x}", self.0[i])?;
		}
		Ok(())
	}
}

/// Trait to allow a type to be pretty-printed in `format!`, where unoverridable
/// defaults cannot otherwise be avoided.
pub trait ToPretty {
	/// Convert a type into a derivative form in order to make `format!` print it prettily.
	fn pretty(&self) -> PrettySlice;
	/// Express the object as a hex string.
	fn to_hex(&self) -> String {
		format!("{}", self.pretty())
	}
}

impl<T: AsRef<[u8]>> ToPretty for T {
	fn pretty(&self) -> PrettySlice {
		PrettySlice(self.as_ref())
	}
}

fn main() {
    let t = 10;

    let mut keys = Vec::with_capacity(100);
    let mut pubs = Vec::with_capacity(100);
    for _ in 0..100 {
        let (public, private) = pvss::crypto::create_keypair();

        keys.push(private);
        pubs.push(public);
    }

    // Round trip public key through bytes
    let pub_bytes = pubs[1].to_bytes();
    println!("pub_bytes = {}", pub_bytes.to_hex());
    let pub_0 = pvss::crypto::PublicKey::from_bytes(&pub_bytes);
    assert!(pub_0 == pubs[1]);

    // Round trip private key through bytes
    let priv_bytes = keys[1].to_bytes();
    println!("priv_bytes = {}", priv_bytes.to_hex());
    let priv_0 = pvss::crypto::PrivateKey::from_bytes(&priv_bytes);
    assert!(priv_0 == keys[1]);

    let escrow = pvss::simple::escrow(t);

    let commitments = pvss::simple::commitments(&escrow);

    // Round trip commitments through bytes
    let commitment_bytes = serialize(&commitments, Infinite).unwrap();
    let commitments_deserialized: Vec<pvss::simple::Commitment> = deserialize(&commitment_bytes).unwrap();
    assert!(commitments_deserialized == commitments);

    let shares = pvss::simple::create_shares(&escrow, &pubs);

    // Round trip shares through bytes
    let share_bytes = serialize(&shares, Infinite).unwrap();
    let shares_deserialized: Vec<pvss::simple::EncryptedShare> = deserialize(&share_bytes).unwrap();
    assert!(shares_deserialized == shares);

    let mut decrypted = Vec::with_capacity(100);

    println!("publickeys: {nb_keys}", nb_keys = pubs.len());
    println!("shares: {nb_shares}", nb_shares = shares.len());

    for share in shares {
        let idx = share.id as usize;
        let verified_encrypted =
            share.verify(share.id, &pubs[idx], &escrow.extra_generator, &commitments);
        println!("encrypted share {id}: {verified}",
                 id = share.id,
                 verified = verified_encrypted);

        let d = pvss::simple::decrypt_share(&keys[idx], &pubs[idx], &share);
        let verified_decrypted = d.verify(&pubs[idx], &share);
        println!("decrypted share {id}: {verified}",
                 id = share.id,
                 verified = verified_decrypted);
        decrypted.push(d);
    }

    let recovered = pvss::simple::recover(t, decrypted.as_slice()).unwrap();
    println!("equal: {b}", b = (recovered == escrow.secret));
}
