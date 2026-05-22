# Trustless Freelance Escrow

## Description
The Trustless Freelance Escrow is a decentralized application (dApp) built on the Stellar network using Soroban smart contracts. Traditional freelance platforms charge high fees and act as centralized middlemen holding the funds. This project solves that by replacing the middleman with an immutable, automated smart contract. It ensures a secure, trustless environment where a client's funds are safely locked and only released to the freelancer when the work is delivered, protecting both parties from scams, non-payment, or non-delivery.

## Features
* **Secure Fund Locking:** Clients deposit XLM into the smart contract acting as an impenetrable vault until conditions are met.
* **Timeout & Auto-Refunds:** Utilizes on-chain timestamps (`env.ledger().timestamp()`). If the freelancer fails to submit work before the agreed-upon deadline, the client can securely claim a full refund.
* **Dispute Resolution (Arbitrator Role):** In case of a disagreement, either the client or the freelancer can freeze the contract by raising a dispute. A designated third-party Arbitrator can then review the situation and distribute the funds fairly.
* **Strict State Machine:** Enforces a rigid workflow (`AwaitingFunds` ➔ `Funded` ➔ `WorkSubmitted` ➔ `Completed`/`Refunded`/`InDispute`) to prevent unauthorized actions and reentrancy attacks.
* **On-chain Event Emission:** Broadcasts contract events at every step (e.g., `funds_locked`, `work_submitted`) allowing the front-end to listen and update the UI in real-time.

## Contract
**Network:** Stellar Testnet  
**Contract ID:** CBAN4BN5BPF4ERFKMCWCHVJ5JFZMYLCXYSBSKZYDA3UCDFXEPGP3WPUK

https://stellar.expert/explorer/testnet/contract/CBAN4BN5BPF4ERFKMCWCHVJ5JFZMYLCXYSBSKZYDA3UCDFXEPGP3WPUK

### Contract Screenshot

<img width="1856" height="874" alt="image" src="https://github.com/user-attachments/assets/e21caef0-8bbc-42d5-b711-e6ded1125192" />


## Future Scopes
* **Milestone-based Payments:** Upgrading the contract to support partial fund releases for long-term projects divided into multiple phases.
* **Stablecoin Integration:** Expanding token support so users can transact using Stellar-based stablecoins like USDC or EURC to avoid cryptocurrency volatility.
* **Decentralized Arbitrator DAO:** Transitioning the single Arbitrator role into a multi-signature DAO panel where a jury votes on dispute resolutions to ensure absolute neutrality.
* **On-chain Reputation System:** Building a rating system tied to wallet addresses to help clients find reliable freelancers based on successful smart contract completions.

## Profile
* **Name:** Nguyen Ba Duy
* **GitHub:** https://github.com/johannguyen015
* **Skills:** Blockchain Development, Rust, Soroban Smart Contracts, Stellar SDK, JavaScript/HTML/CSS Frontend Integration, Web3 System Architecture.
