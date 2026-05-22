#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

// 1. CẬP NHẬT: Thêm trạng thái Refunded (Đã hoàn tiền)
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    AwaitingFunds, 
    Funded,        
    WorkSubmitted, 
    Completed,     
    Refunded,      // MỚI: Khách hàng đã lấy lại tiền do quá hạn
}

// 2. CẬP NHẬT: Thêm khóa Deadline để lưu thời gian
#[contracttype]
pub enum DataKey {
    Client,
    Freelancer,
    Token,  
    Amount, 
    State,  
    Deadline,      // MỚI: Lưu thời hạn chót nộp bài (Unix Timestamp)
}

#[contract]
pub struct FreelanceEscrow;

#[contractimpl]
impl FreelanceEscrow {
    /// BƯỚC 1: Khởi tạo giao kèo
    /// CẬP NHẬT: Thêm tham số `duration_seconds` (Thời gian làm việc tính bằng giây)
    pub fn init(env: Env, client: Address, freelancer: Address, token: Address, amount: i128, duration_seconds: u64) {
        client.require_auth(); 

        // Tính toán hạn chót: Thời gian hiện tại của mạng lưới + Khoảng thời gian cho phép
        let deadline = env.ledger().timestamp() + duration_seconds;

        env.storage().instance().set(&DataKey::Client, &client);
        env.storage().instance().set(&DataKey::Freelancer, &freelancer);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Amount, &amount);
        env.storage().instance().set(&DataKey::Deadline, &deadline); // Lưu deadline
        env.storage().instance().set(&DataKey::State, &Status::AwaitingFunds);
    }

    /// BƯỚC 2: Khóa tiền (Giữ nguyên)
    pub fn fund_contract(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth(); 

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::AwaitingFunds, "Hop dong da duoc nap tien roi");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&client, &env.current_contract_address(), &amount);

        env.storage().instance().set(&DataKey::State, &Status::Funded);
    }

    /// BƯỚC 3: Nộp công việc (Giữ nguyên)
    pub fn submit_work(env: Env) {
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        freelancer.require_auth(); 

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::Funded, "Chua co tien, hoac cong viec da duoc nop");

        env.storage().instance().set(&DataKey::State, &Status::WorkSubmitted);
    }

    /// BƯỚC 4: Nghiệm thu & Giải ngân (Giữ nguyên)
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
    }

    /// BƯỚC 5 (MỚI): Hoàn tiền nếu quá hạn (Timeout Refund)
    /// Client gọi hàm này để đòi lại tiền nếu Freelancer lặn mất tăm sau Deadline.
    pub fn claim_refund(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth(); // Chỉ Client mới được đòi tiền

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        // Ràng buộc 1: Chỉ hoàn tiền nếu Client đã nạp tiền (Funded) nhưng Freelancer CHƯA nộp bài
        assert!(state == Status::Funded, "Khong the hoan tien o trang thai nay");

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        // Ràng buộc 2: Thời gian hiện tại của Blockchain phải lớn hơn Deadline
        assert!(env.ledger().timestamp() > deadline, "Chua qua han chot, hay kien nhan");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        // Chuyển tiền từ Hợp đồng ngược về ví của Client
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &client, &amount);

        // Cập nhật trạng thái để khóa mọi thao tác tiếp theo
        env.storage().instance().set(&DataKey::State, &Status::Refunded);
    }

    /// HÀM PHỤ: Kiểm tra trạng thái
    pub fn get_status(env: Env) -> Status {
        env.storage().instance().get(&DataKey::State).unwrap_or(Status::AwaitingFunds)
    }
}