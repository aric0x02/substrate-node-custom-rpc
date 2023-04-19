use codec::{Decode, Encode};

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct MVMApiEstimation {
	pub gas_used: u64,
	pub status_code: u64,
}
