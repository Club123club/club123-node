# pallet-settlement

与 `docs/settlement-contract.md`（及 `cera-chain/docs/05-smart-contract/settlement-contract.md`）对应的结算模块，使用 **FRAME v2**（`#[pallet]`）实现。

## 结构

- **Storage**: `PayeeBalance`, `PayeeConfigs`, `PlatformFeeBalance`, `DailyWithdrawal`, `LastWithdrawalDay`, `Paused`
- **Call**: `deposit_for_payee`, `request_settlement`, `execute_settlement`, `withdraw_platform_fee`, `set_payee_config`, `pause`, `unpause`, `emergency_withdraw`
- **Event**: `Deposit`, `SettlementRequested`, `SettlementExecuted`, `PayeeUpdated`, `PlatformFeeWithdrawn`, `Paused`, `Unpaused`, `EmergencyWithdrawn`

## Config

- `Currency`: 结算用货币（runtime 中设为 `Balances`）
- `BlocksPerDay`: 每“天”对应的区块数，用于日限额重置
- `TreasuryAccount`: 结算模块资金账户（部署时改为实际 Treasury 账户）
- `WeightInfo`: 各 dispatchable 的权重

## 说明

- 入账/执行结算/提平台费：当前仅做链上账本与事件；实际从 Treasury 转出需由链下或 runtime 侧配合。
- 权限暂用 `ensure_root`（sudo）；后续可改为 `AdminOrigin` / `OperatorOrigin` / `DepositorOrigin` / `PauserOrigin`。
