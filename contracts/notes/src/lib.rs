#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

// 1. Thêm các trạng thái Tranh chấp và Hoàn tiền
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    AwaitingFunds,
    Funded,
    WorkSubmitted,
    InDispute,     // MỚI: Đang trong quá trình tranh chấp
    Completed,
    Refunded,      // MỚI: Đã hoàn tiền cho Client do quá hạn
}

// 2. Thêm từ khóa để lưu Thời hạn và Người phân xử
#[contracttype]
pub enum DataKey {
    Client,
    Freelancer,
    Arbitrator, // MỚI: Người phân xử trung gian
    Token,
    Amount,
    State,
    Deadline,   // MỚI: Hạn chót nộp bài (Unix Timestamp)
}

#[contract]
pub struct FreelanceEscrow;

#[contractimpl]
impl FreelanceEscrow {
    /// BƯỚC 1: Khởi tạo (Cập nhật: Thêm Arbitrator và Thời gian thực hiện dự án)
    pub fn init(
        env: Env, 
        client: Address, 
        freelancer: Address, 
        arbitrator: Address, 
        token: Address, 
        amount: i128,
        duration_seconds: u64 // Khoảng thời gian làm việc (ví dụ: 604800 giây = 7 ngày)
    ) {
        client.require_auth();

        // Tính toán thời điểm hạn chót bằng cách lấy thời gian hiện tại của Ledger + thời gian làm việc
        let deadline = env.ledger().timestamp() + duration_seconds;

        env.storage().instance().set(&DataKey::Client, &client);
        env.storage().instance().set(&DataKey::Freelancer, &freelancer);
        env.storage().instance().set(&DataKey::Arbitrator, &arbitrator);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Amount, &amount);
        env.storage().instance().set(&DataKey::Deadline, &deadline);
        env.storage().instance().set(&DataKey::State, &Status::AwaitingFunds);
    }

    /// BƯỚC 2: Khóa tiền (Giữ nguyên logic cũ)
    pub fn fund_contract(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth();

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::AwaitingFunds, "Hop dong da duoc nap tien");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&client, &env.current_contract_address(), &amount);

        env.storage().instance().set(&DataKey::State, &Status::Funded);
    }

    /// BƯỚC 3: Nộp công việc (Giữ nguyên logic cũ)
    pub fn submit_work(env: Env) {
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        freelancer.require_auth();

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::Funded, "Trang thai khong hop le de nop bai");

        env.storage().instance().set(&DataKey::State, &Status::WorkSubmitted);
    }

    /// BƯỚC 4: Nghiệm thu thông thường (Giữ nguyên logic cũ)
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

    /// GIẢI PHÁP 1: Rút tiền do Quá hạn (Timeout Refund)
    /// Nếu Freelancer nhận tiền nhưng lặn mất tăm, Client được quyền tự rút tiền về sau Deadline
    pub fn claim_timeout_refund(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth();

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        // Chỉ cho phép rút nếu tiền đã nạp và Freelancer CHƯA nộp bài
        assert!(state == Status::Funded, "Khong the hoan tien o trang thai nay");

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        // KIỂM TRA THỜI GIAN: Thời gian hiện tại của Blockchain phải lớn hơn Deadline
        assert!(env.ledger().timestamp() > deadline, "Chua den han chot, hay kien nhan");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        // Trả lại tiền cho Client
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &client, &amount);

        env.storage().instance().set(&DataKey::State, &Status::Refunded);
    }

    /// GIẢI PHÁP 2.1: Kích hoạt Tranh chấp (Raise Dispute)
    /// Khi sản phẩm không đạt yêu cầu hoặc có xích mích, Client hoặc Freelancer có quyền khóa hợp đồng lại
    pub fn raise_dispute(env: Env) {
        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        // Chỉ tranh chấp khi tiền đã khóa hoặc bài đã nộp
        assert!(state == Status::Funded || state == Status::WorkSubmitted, "Khong the mo tranh chap");

        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();

        // Xác thực: Hoặc Client hoặc Freelancer ký tên thì mới được mở tranh chấp
        if client.has_auth() {
            client.require_auth();
        } else {
            freelancer.require_auth();
        }

        env.storage().instance().set(&DataKey::State, &Status::InDispute);
    }

    /// GIẢI PHÁP 2.2: Phán quyết Tranh chấp (Resolve Dispute)
    /// Chỉ có Người phân xử (Arbitrator) mới có quyền gọi hàm này để chia tiền theo tỷ lệ
    pub fn resolve_dispute(env: Env, amount_to_freelancer: i128) {
        let arbitrator: Address = env.storage().instance().get(&DataKey::Arbitrator).unwrap();
        arbitrator.require_auth(); // BẮT BUỘC: Phải có chữ ký của Người phân xử

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::InDispute, "Hop dong khong o trang thai tranh chap");

        let total_amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();
        assert!(amount_to_freelancer <= total_amount && amount_to_freelancer >= 0, "So tien phan chia khong hop le");

        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        
        let token_client = token::Client::new(&env, &token);
        let amount_to_client = total_amount - amount_to_freelancer;

        // 1. Chia tiền cho Freelancer nếu có
        if amount_to_freelancer > 0 {
            token_client.transfer(&env.current_contract_address(), &freelancer, &amount_to_freelancer);
        }
        // 2. Chia số tiền còn lại cho Client
        if amount_to_client > 0 {
            token_client.transfer(&env.current_contract_address(), &client, &amount_to_client);
        }

        env.storage().instance().set(&DataKey::State, &Status::Completed);
    }

    pub fn get_status(env: Env) -> Status {
        env.storage().instance().get(&DataKey::State).unwrap_or(Status::AwaitingFunds)
    }
}