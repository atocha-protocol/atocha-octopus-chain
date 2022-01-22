#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use sp_runtime::traits::Scale;
pub struct PointExchange<T>(PhantomData<T>);

// IPointExchange
// <T::AccountId, BalanceOf<T>, PuzzleSubjectHash, T::BlockNumber, DispatchResult>
impl<T: Config> IPointExchange<T::AccountId, T::BlockNumber, ExchangeEra, PointToken, BalanceOf<T>, ExchangeInfo<PointToken, BalanceOf<T>, Perbill>> for PointExchange<T> {
	fn apply_exchange(who: T::AccountId) -> DispatchResult {
		let current_era = Self::get_current_era();
		ensure!(current_era > Zero::zero(), Error::<T>::LastExchangeRewardClearing);

		if PointExchangeInfo::<T>::contains_key(current_era.saturating_sub(1)) {
			let previous_reward_era = LastExchangeRewardEra::<T>::get();
			ensure!(previous_reward_era.is_some(), Error::<T>::LastExchangeRewardClearing);
			ensure!(previous_reward_era.unwrap() == current_era.saturating_sub(1), Error::<T>::LastExchangeRewardClearing);
		}

		// Get use point value .
		let apply_point = PointManager::<T>::get_total_points(&who);
		ensure!(apply_point > 0, Error::<T>::TooFewPoints);
		// Update and sort list.
		ensure!(Self::update_apply_list_point(), Error::<T>::ExchangeRewardEnded);

		let mut apply_list = Self::get_reward_list(current_era);
		if apply_list.is_empty() {
			apply_list.push((who, apply_point, None));
			LastUpdateBlockInfoOfPointExchage::<T>::put(<frame_system::Pallet<T>>::block_number());
			PointExchangeInfo::<T>::insert(current_era, apply_list);
			return Ok(());
		}

		ensure!( !apply_list.iter().any(|x|&x.0 == &who), Error::<T>::ExchangeApplyAlreadyExists );

		// Update old list point.
		if let Some((original_who, original_point, origin_info)) = apply_list.pop() {
			ensure!(apply_point > original_point, Error::<T>::TooFewPoints);
			apply_list.push((who, apply_point, None));
			if (apply_list.len() < Self::get_max_reward_count() as usize) {
				apply_list.push((original_who, original_point, origin_info));
			}
			println!("RUN2 : {:?}", &apply_list);
			apply_list.sort_by(|(_, point_a, _),(_, point_b, _)|{
				point_b.cmp(point_a)
			});
			println!("RUN3 : {:?}", &apply_list);
			LastUpdateBlockInfoOfPointExchage::<T>::put(<frame_system::Pallet<T>>::block_number());
			PointExchangeInfo::<T>::insert(current_era, apply_list);
		}

		Ok(())
	}

	fn update_apply_list_point() -> bool  {
		let mut apply_list = PointExchangeInfo::<T>::get(&Self::get_current_era());
		let have_final_info = apply_list.iter().any(|(_, _, info_data)|{
			if info_data.is_some() {
				return true;
			}
			false
		});
		let current_bn = <frame_system::Pallet<T>>::block_number();
		if let Some(last_update_bn) = LastUpdateBlockInfoOfPointExchage::<T>::get() {
			if last_update_bn == current_bn {
				return true;
			}
		}
		if !have_final_info {
			let new_apply_list:  Vec<(
				T::AccountId,
				PointToken,
				Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>
			)> = apply_list.into_iter().map(|(who, _, info_data)|{
				let new_point = PointManager::<T>::get_total_points(&who);
				(who, new_point, info_data)
			}).collect();
			LastUpdateBlockInfoOfPointExchage::<T>::put(current_bn);
			PointExchangeInfo::<T>::insert(&Self::get_current_era(), new_apply_list);
			return true;
		}
		false
	}

	fn execute_exchange(era: ExchangeEra, mint_balance: BalanceOf<T>) -> DispatchResult {
		ensure!(era < Self::get_current_era(), Error::<T>::EraNotEnded );
		ensure!(PointExchangeInfo::<T>::contains_key(era), Error::<T>::ExchangeListIsEmpty);
		println!(" RUN A 1 ");
		if let Some(last_exec_era) = LastExchangeRewardEra::<T>::get() {
			ensure!(last_exec_era < era, Error::<T>::ExchangeRewardEnded);
		}
		Self::update_apply_list_point();
		// count total point.
		let exchange_list = PointExchangeInfo::<T>::get(era);
		println!(" RUN A 2 = {:?}", exchange_list);
		ensure!(!exchange_list.iter().any(|(_, _, info_data)|{info_data.is_some()}), Error::<T>::ExchangeRewardEnded);
		println!(" RUN A 3 ");
		// let total_point = exchange_list.into_iter().map(|(_, exchange_point, info_data)|{
		// 	// ensure!(info_data.is_none, Error::<T>::ExchangeRewardEnded);
		// 	exchange_point
		// }).collect::<Vec<PointToken>>();

		let mut total_point: PointToken = Zero::zero();
		for x in exchange_list.clone().into_iter() {
			total_point = total_point.saturating_add(x.1);
		}
		println!("total_point = {:?}", total_point);
		//
		let mut sum_proportion: Perbill = Perbill::from_percent(0);
		let mut all_pay: BalanceOf<T> = Zero::zero();
		for (idx, (who, apply_point, mut info_data)) in exchange_list.clone().into_iter().enumerate() {
			let mut current_proportion = Perbill::from_percent(0);;
			if idx == exchange_list.len().saturating_sub(1) {
				current_proportion = Perbill::from_percent(100) - sum_proportion ;
				let take_token = mint_balance - all_pay;
				info_data = Some(ExchangeInfo {
					proportion: current_proportion.clone(),
					pay_point: apply_point,
					take_token: take_token,
				});
				all_pay += take_token;
			} else {
				current_proportion = Perbill::from_rational(apply_point, total_point);
				let take_token = current_proportion * mint_balance;
				info_data = Some(ExchangeInfo {
					proportion: current_proportion.clone(),
					pay_point: apply_point,
					take_token: take_token,
				});
				all_pay += take_token;
			}
			sum_proportion = sum_proportion + current_proportion;
			println!("current_proportion = {:?}, {:?}, {:?}, {:?}", &current_proportion, &sum_proportion, all_pay, info_data);
		}
		assert_eq!(mint_balance, all_pay);
		assert_eq!(sum_proportion, Perbill::from_percent(100));

		// min balance.

		Ok(())
	}

	fn get_current_era() -> ExchangeEra {
		let current_bn = <frame_system::Pallet<T>>::block_number();
		(current_bn / Self::get_era_length()).unique_saturated_into()
	}

	fn get_max_reward_count() -> u32 {
		3
	}

	fn get_era_length() -> T::BlockNumber {
		10u32.into()
	}

	fn get_reward_list(era: ExchangeEra) -> Vec<(T::AccountId, PointToken, Option<ExchangeInfo<PointToken, BalanceOf<T>, Perbill>>)> {
		// get max_reward_count .
		let mut apply_list = PointExchangeInfo::<T>::get(&Self::get_current_era());
		if apply_list.len() <= Self::get_max_reward_count() as usize {
			return apply_list;
		}
		apply_list.split_off(Self::get_max_reward_count() as usize);
		apply_list
	}

	fn get_history_depth() -> u32 {
		3
	}
}
