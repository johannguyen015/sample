#![no_std]
#![allow(deprecated)] // Ẩn cảnh báo về Events::publish ở bản SDK mới

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    AwaitingFunds, 
    Funded,        
    WorkSubmitted, 
    InDispute,     
    Completed,     
    Refunded,      
}

#[contracttype]
pub enum DataKey {
    Client,
    Freelancer,
    Arbitrator,    
    Token,         
    Amount,        
    State,         
    Deadline,      
}

#[contract]
pub struct FreelanceEscrow;

#[contractimpl]
impl FreelanceEscrow {
    
    pub fn init(
        env: Env, 
        client: Address, 
        freelancer: Address, 
        arbitrator: Address, 
        token: Address, 
        amount: i128, 
        duration_seconds: u64
    ) {
        client.require_auth(); 

        let deadline = env.ledger().timestamp() + duration_seconds;

        env.storage().instance().set(&DataKey::Client, &client);
        env.storage().instance().set(&DataKey::Freelancer, &freelancer);
        env.storage().instance().set(&DataKey::Arbitrator, &arbitrator);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Amount, &amount);
        env.storage().instance().set(&DataKey::Deadline, &deadline);
        env.storage().instance().set(&DataKey::State, &Status::AwaitingFunds);

        env.events().publish(
            (Symbol::new(&env, "escrow_initialized"), client),
            (freelancer, amount, deadline)
        );
    }

    pub fn fund_contract(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth(); 

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::AwaitingFunds, "Trang thai phai la AwaitingFunds");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&client, &env.current_contract_address(), &amount);

        env.storage().instance().set(&DataKey::State, &Status::Funded);

        env.events().publish(
            (Symbol::new(&env, "funds_locked"), client), 
            amount
        );
    }

    pub fn submit_work(env: Env) {
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        freelancer.require_auth(); 

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::Funded, "Hop dong chua duoc nap tien hoac da dong");

        env.storage().instance().set(&DataKey::State, &Status::WorkSubmitted);

        env.events().publish(
            (Symbol::new(&env, "work_submitted"), freelancer), 
            true
        );
    }

    pub fn approve_and_release(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth(); 

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::WorkSubmitted, "Freelancer chua nop bai");

        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &freelancer, &amount);

        env.storage().instance().set(&DataKey::State, &Status::Completed);

        env.events().publish(
            (Symbol::new(&env, "escrow_completed"), client), 
            amount
        );
    }

    pub fn claim_timeout_refund(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth(); 

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::Funded, "Khong the hoan tien o trang thai nay");

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        assert!(env.ledger().timestamp() > deadline, "Chua qua han chot thuc hien");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &client, &amount);

        env.storage().instance().set(&DataKey::State, &Status::Refunded);

        env.events().publish(
            (Symbol::new(&env, "timeout_refund_triggered"), client), 
            amount
        );
    }

    pub fn raise_dispute(env: Env, caller: Address) {
        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::Funded || state == Status::WorkSubmitted, "Khong dung thoi diem de mo tranh chap");

        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();

        assert!(caller == client || caller == freelancer, "Chi Client hoac Freelancer moi duoc mo tranh chap");
        caller.require_auth(); 

        env.storage().instance().set(&DataKey::State, &Status::InDispute);

        env.events().publish(
            (Symbol::new(&env, "dispute_raised"),), 
            true
        );
    }

    pub fn resolve_dispute(env: Env, amount_to_freelancer: i128) {
        let arbitrator: Address = env.storage().instance().get(&DataKey::Arbitrator).unwrap();
        arbitrator.require_auth(); 

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::InDispute, "Hop dong phai dang trong trang thai tranh chap");

        let total_amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();
        assert!(amount_to_freelancer >= 0 && amount_to_freelancer <= total_amount, "So tien chia khong hop le");

        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        
        let token_client = token::Client::new(&env, &token);
        let amount_to_client = total_amount - amount_to_freelancer;

        if amount_to_freelancer > 0 {
            token_client.transfer(&env.current_contract_address(), &freelancer, &amount_to_freelancer);
        }
        if amount_to_client > 0 {
            token_client.transfer(&env.current_contract_address(), &client, &amount_to_client);
        }

        env.storage().instance().set(&DataKey::State, &Status::Completed);

        env.events().publish(
            (Symbol::new(&env, "dispute_resolved_by_arbiter"), arbitrator), 
            amount_to_freelancer
        );
    }

    pub fn get_status(env: Env) -> Status {
        env.storage().instance().get(&DataKey::State).unwrap_or(Status::AwaitingFunds)
    }
}