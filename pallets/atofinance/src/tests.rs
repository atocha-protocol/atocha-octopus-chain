use crate::traits::*;
use crate::types::*;
use crate::{mock::*, Error, AtoPointLedger, AtoPointTotal, AtoFinanceLedger};
// use frame_support::sp_runtime::sp_std::convert::TryInto;
use frame_support::sp_runtime::Permill;
use frame_support::traits::{Len, OnInitialize};
use frame_support::{
	assert_err, assert_noop, assert_ok, assert_storage_noop,
	traits::{
		Currency, ExistenceRequirement::AllowDeath, ExistenceRequirement::KeepAlive,
		LockIdentifier, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
};
use sp_runtime::{traits::Hash, Perbill};

mod test_IAtoChallenge;
mod test_IPointExchange;
mod test_IPuzzleLedger;
mod test_IPuzzlePoints;
mod test_IPuzzleReward_Of_Points;
mod test_IPuzzleReward_Of_Tokens;

#[test]
fn test_Perbill() {
	new_test_ext().execute_with(|| {
		// This is 0.42
		let x = Permill::from_parts(42_0_000);
		let x2 = Permill::from_float(0.42);
		assert_eq!(x, x2);
		// This is 100.000
		let y = x * 100_0000u64;
		assert_eq!(y, 42_0000)
	});
}


#[test]
fn test_PreStorage() {
	new_test_ext().execute_with(|| {
		let current_bn: u64 = 800;
		System::set_block_number(current_bn);
		<AtochaPot as OnInitialize<u64>>::on_initialize(current_bn);
		const ACCOUNT_ID_1: u64 = 2;
		assert_eq!(Balances::free_balance(ACCOUNT_ID_1), 200000000000000);
		assert_noop!(
			AtochaPot::pre_storage(Origin::signed(ACCOUNT_ID_1), "STORAGE_HASH".as_bytes().to_vec(), 9000, 9000 ),
			Error::<Test>::ExceededMaximumFeeLimit,
		);
		assert_ok!(AtochaPot::pre_storage(Origin::signed(ACCOUNT_ID_1), "STORAGE_HASH".as_bytes().to_vec(), 9000, 100000000000000 ));

		assert_eq!(
			AtochaPot::storage_ledger("STORAGE_HASH".as_bytes().to_vec(), 9000),
			Some((ACCOUNT_ID_1, current_bn, 1000 * 9000)),
		);

		assert_eq!(
			AtochaPot::storage_ledger("STORAGE_HASH".as_bytes().to_vec(), 8000),
			None,
		);
	});
}

#[test]
fn test_Transfer() {
	new_test_ext().execute_with(|| {
		let current_bn: u64 = 1;
		System::set_block_number(current_bn);
		<AtochaPot as OnInitialize<u64>>::on_initialize(current_bn);

		const ACCOUNT_ID_1: u64 = 2;
		const ACCOUNT_ID_2: u64 = 2;

		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200000000000000);
		Balances::set_lock(*b"12345678", &ACCOUNT_ID_2, 150000000000000, WithdrawReasons::all());
		assert_eq!(Balances::free_balance(ACCOUNT_ID_2), 200000000000000);
		assert_eq!(Balances::usable_balance(ACCOUNT_ID_2.clone()), 50000000000000);

		assert_noop!(
			Balances::reserve(&ACCOUNT_ID_2, 80000000000000),
			pallet_balances::Error::<Test>::LiquidityRestrictions
		);
	});
}

#[test]
fn test_Fix_point_ledger() {
	new_test_ext().execute_with(|| {
		AtoPointLedger::<Test>::insert(1, 100);
		AtoPointLedger::<Test>::insert(2, 200);
		AtoPointTotal::<Test>::put(300);
		AtochaPot::fix_point_ledger(Origin::root(), vec![(1,50), (2,100), (3,50)]);

		assert_eq!(AtoPointLedger::<Test>::get(1), 150);
		assert_eq!(AtoPointLedger::<Test>::get(2), 300);
		assert_eq!(AtoPointLedger::<Test>::get(3), 50);
		assert_eq!(AtoPointTotal::<Test>::get(), Some(300 + 50 + 100 + 50));
	});
}

#[test]
fn test_fix_ato_finance_ledger() {
	new_test_ext().execute_with(|| {
		AtoFinanceLedger::<Test>::insert("a".as_bytes().to_vec(), PotLedgerData{
			owner: 1,
			total: 1000,
			funds: 1000,
			sponsor_list: vec![]
		});
		AtochaPot::fix_ato_finance_ledger(Origin::root(), vec![
			("a".as_bytes().to_vec(), PotLedgerData {
				owner: 1,
				total: 2000,
				funds: 2000,
				sponsor_list: vec![]
			}),
			("b".as_bytes().to_vec(), PotLedgerData {
				owner: 2,
				total: 3000,
				funds: 3000,
				sponsor_list: vec![]
			}),
			("c".as_bytes().to_vec(), PotLedgerData {
				owner: 3,
				total: 4000,
				funds: 4000,
				sponsor_list: vec![]
			}),
		]);

		assert_eq!(AtoFinanceLedger::<Test>::iter_keys().count(), 3);
		assert_eq!(AtoFinanceLedger::<Test>::get("a".as_bytes().to_vec()).unwrap(), PotLedgerData{
			owner: 1,
			total: 1000,
			funds: 1000,
			sponsor_list: vec![]
		});
		assert_eq!(AtoFinanceLedger::<Test>::get("b".as_bytes().to_vec()).unwrap(), PotLedgerData {
			owner: 2,
			total: 3000,
			funds: 3000,
			sponsor_list: vec![]
		});
		assert_eq!(AtoFinanceLedger::<Test>::get("c".as_bytes().to_vec()).unwrap(), PotLedgerData {
			owner: 3,
			total: 4000,
			funds: 4000,
			sponsor_list: vec![]
		});

	});
}
