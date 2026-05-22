#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

// Định nghĩa các trạng thái của hợp đồng
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    AwaitingFunds, // Khởi tạo, chờ Client nạp tiền
    Funded,        // Tiền đã bị khóa an toàn, Freelancer bắt đầu làm
    WorkSubmitted, // Freelancer báo cáo đã làm xong, chờ duyệt
    Completed,     // Client duyệt, tiền đã chuyển cho Freelancer
}

// Các từ khóa để lưu trữ dữ liệu
#[contracttype]
pub enum DataKey {
    Client,
    Freelancer,
    Token,   // Loại tiền sử dụng (ví dụ: USDC, XLM)
    Amount,  // Số tiền giao dịch
    State,   // Trạng thái hiện tại
}

#[contract]
pub struct FreelanceEscrow;

#[contractimpl]
impl FreelanceEscrow {
    /// BƯỚC 1: Khởi tạo giao kèo
    /// Khai báo ai là khách, ai là thợ, thanh toán bằng token gì, bao nhiêu tiền.
    pub fn init(env: Env, client: Address, freelancer: Address, token: Address, amount: i128) {
        client.require_auth(); // Client phải tự tay ký giao dịch này

        env.storage().instance().set(&DataKey::Client, &client);
        env.storage().instance().set(&DataKey::Freelancer, &freelancer);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Amount, &amount);
        env.storage().instance().set(&DataKey::State, &Status::AwaitingFunds);
    }

    /// BƯỚC 2: Khóa tiền (Client nạp tiền vào Hợp đồng)
    /// Hợp đồng đóng vai trò như "Két sắt" không ai mở được lúc này.
    pub fn fund_contract(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth(); // Chỉ Client mới được nạp tiền

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::AwaitingFunds, "Hop dong da duoc nap tien roi");

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        // Lệnh thực hiện chuyển tiền từ ví Client sang địa chỉ của Smart Contract này
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&client, &env.current_contract_address(), &amount);

        // Chuyển trạng thái để báo hiệu Freelancer có thể bắt đầu làm việc
        env.storage().instance().set(&DataKey::State, &Status::Funded);
    }

    /// BƯỚC 3: Nộp công việc
    /// Freelancer xác nhận đã làm xong và yêu cầu Client kiểm tra.
    pub fn submit_work(env: Env) {
        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        freelancer.require_auth(); // Bắt buộc là Freelancer trong giao kèo mới được gọi hàm này

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::Funded, "Chua co tien, hoac cong viec da duoc nop");

        env.storage().instance().set(&DataKey::State, &Status::WorkSubmitted);
    }

    /// BƯỚC 4: Nghiệm thu & Giải ngân
    /// Client hài lòng, Hợp đồng tự động nhả tiền từ "két sắt" chuyển thẳng cho Freelancer.
    pub fn approve_and_release(env: Env) {
        let client: Address = env.storage().instance().get(&DataKey::Client).unwrap();
        client.require_auth(); // Client phải tự tay ký duyệt

        let state: Status = env.storage().instance().get(&DataKey::State).unwrap();
        assert!(state == Status::WorkSubmitted, "Freelancer chua nop bai");

        let freelancer: Address = env.storage().instance().get(&DataKey::Freelancer).unwrap();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let amount: i128 = env.storage().instance().get(&DataKey::Amount).unwrap();

        // Hợp đồng tự động chuyển tiền sang ví Freelancer
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &freelancer, &amount);

        // Cập nhật trạng thái hoàn thành
        env.storage().instance().set(&DataKey::State, &Status::Completed);
    }

    /// HÀM PHỤ: Kiểm tra trạng thái hiện tại của giao kèo
    /// Ai cũng có thể xem để biết giao dịch đang ở bước nào (Tính minh bạch)
    pub fn get_status(env: Env) -> Status {
        env.storage().instance().get(&DataKey::State).unwrap_or(Status::AwaitingFunds)
    }
}