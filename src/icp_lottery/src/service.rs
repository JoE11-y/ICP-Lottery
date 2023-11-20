use crate::types::*;
use candid::{Decode, Encode, Principal};
use ic_cdk::api::call::call;
use ic_cdk::api::{time, trap};
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Storable;
use icrc_ledger_types::icrc1::transfer::NumTokens;
use rand_chacha::rand_core::SeedableRng;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct Lottery {
    pub id: u32,
    pub winner: Principal,
    pub no_of_tickets_sold: u32,
    pub no_of_players: u32,
    pub winning_ticket: u32,
    pub amount_in_lottery: NumTokens,
    pub lottery_start_time: u64,
    pub lottery_end_time: u64,
    pub lottery_state: LotteryState,
    ticket_ids: HashMap<u32, Principal>, //keeps track of ticketIds to their owners
    players_tickets: HashMap<Principal, u32>, // keeps track of noOfTickets each player has bought
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
pub struct LotteryData {
    pub id: u32,
    pub winner: Principal,
    pub no_of_tickets_sold: u32,
    pub no_of_players: u32,
    pub winning_ticket: u32,
    pub amount_in_lottery: NumTokens,
    pub lottery_start_time: u64,
    pub lottery_end_time: u64,
    pub lottery_state: LotteryState,
}

// a trait that must be implemented for a struct that is stored in a stable struct
impl Storable for Lottery {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

impl Lottery {
    // Improved input validation and error messages
    pub fn register_tickets(&mut self, no_of_tickets: &u32) -> Result<(), LotteryError> {
        let caller = ic_cdk::caller();
        if *no_of_tickets == 0 {
            return Err(LotteryError::InvalidTicketCount);
        }

        let old_ticket_count = self.no_of_tickets_sold;
        let mut new_ticket_id = old_ticket_count;
        let new_total = *no_of_tickets + old_ticket_count;

        while new_ticket_id < new_total {
            self.ticket_ids.insert(new_ticket_id, caller);
            new_ticket_id += 1;
        }

        // Improved handling for player_existing_tickets
        if let Some(n) = self.players_tickets.get_mut(&caller) {
            *n += no_of_tickets;
        } else {
            self.players_tickets.insert(caller, *no_of_tickets);
        }

        Ok(())
    }

    // Improved random number generation with more descriptive error handling
    async fn make_rng(&self) -> Result<rand_chacha::ChaCha20Rng, String> {
        let raw_rand: Vec<u8> = match call(Principal::management_canister(), "raw_rand", ()).await {
            Ok((res,)) => res,
            Err((_, err)) => return Err(format!("Failed to get seed: {}", err)),
        };

        // Improved error handling for seed conversion
        let seed: Salt = raw_rand[..]
            .try_into()
            .map_err(|_| "Failed to create seed from raw_rand output")?;

        Ok(rand_chacha::ChaCha20Rng::from_seed(seed))
    }
    
}


    pub async fn get_winning_ticket(&mut self) -> Result<(), String> {
        let rand = self.make_rng().await;
        let no_of_tickets = self.no_of_tickets_sold as u128;
        let num = rand.get_word_pos() % no_of_tickets;
        self.winning_ticket = num as u32;
        self.lottery_state = LotteryState::Payout;
        Ok(())
    }

    pub fn check_winner(&mut self) -> Result<(), String> {
        let winning_ticket = self.winning_ticket;
        let winner = self
            .ticket_ids
            .get(&winning_ticket)
            .expect("Ticket out of bounds");

        if winner.clone() != ic_cdk::caller() {
            return Err(format!("Not winner"));
        }
        self.winner = winner.clone();
        self.lottery_state = LotteryState::Closed;
        Ok(())
    }

    pub fn get_player_ticket_count(&self, user: &Principal) -> u32 {
        match self.players_tickets.get(user) {
            Some(tickets) => tickets.clone(),
            None => 0,
        }
    }

    pub fn get_lottery(&self) -> LotteryData {
        LotteryData {
            id: self.id,
            winner: self.winner,
            no_of_tickets_sold: self.no_of_tickets_sold,
            no_of_players: self.no_of_players,
            winning_ticket: self.winning_ticket,
            amount_in_lottery: self.amount_in_lottery.clone(),
            lottery_start_time: self.lottery_start_time,
            lottery_end_time: self.lottery_end_time,
            lottery_state: self.lottery_state.clone(),
        }
    }

    pub fn check_state(&self, state: LotteryState) -> Result<(), String> {
        if self.lottery_state != state {
            return Err(format!("Invalid State"));
        }
        Ok(())
    }

    pub fn check_lottery_ended(&self) -> Result<(), String> {
        if self.lottery_end_time > time() {
            return Err(format!("Time Not Reached"));
        }
        Ok(())
    }

    pub fn check_lottery_ongoing(&self) -> Result<(), String> {
        if self.lottery_end_time < time() {
            return Err(format!("Time Elapsed"));
        }
        Ok(())
    }
}

impl LotteryConf {
    pub fn init(&mut self, args: InitArgs) {
        // lottery duration in minutes
        self.ticket_price = args.ticket_price;
        self.lottery_duration = args.lottery_duration.clone() * 60 * 1000000000;
    }

    pub fn calc_ticket_price(&self, no_of_tickets: &u32) -> NumTokens {
        self.ticket_price.clone() * *no_of_tickets
    }

    pub fn increment_pool(&mut self, amount: &NumTokens) {
        self.lottery_pool += amount.clone();
    }

    pub fn decrement_pool(&mut self, amount: &NumTokens) {
        self.lottery_pool -= amount.clone();
    }

    pub fn gen_lottery(&mut self) -> Lottery {
        // get and update next lottery id
        let id = self.next_lottery_id;
        self.next_lottery_id += 1;

        // update lottery state to active
        self.state = ConfState::Active;

        // return lottery instance
        Lottery {
            id,
            winner: Principal::anonymous(),
            no_of_tickets_sold: 0,
            no_of_players: 0,
            winning_ticket: 0,
            amount_in_lottery: 0u128.into(),
            lottery_start_time: time(),
            lottery_end_time: time() + self.lottery_duration,
            lottery_state: LotteryState::Active,
            ticket_ids: HashMap::new(),
            players_tickets: HashMap::new(),
        }
    }

    pub fn reset_state(&mut self) {
        self.state = ConfState::Inactive;
    }

    pub fn check_state(&self, state: ConfState) -> Result<(), String> {
        if self.state != state {
            return Err(format!("Invalid State"));
        }
        Ok(())
    }


    // Updated to minimize unnecessary computations for prize calculation
    pub fn get_prize(&self) -> NumTokens {
        self.lottery_pool.clone() / 2
    
    }
}
