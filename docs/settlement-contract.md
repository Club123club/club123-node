# 结算模块（Settlement Module / Pallet）

Document ID: DOC-CONT-SETTLEMENT-002
Layer: CONT
Status: Active

本文定义：

- 模块职责
- 资金流模型
- 结算生命周期
- 访问控制模型
- 费用模型
- 安全设计
- 升级策略
- 应急机制
- 审计保证

---

## 1. 模块角色（Module Role）

**Settlement 模块负责：**

- 暂管收款方资金（按地址临时托管；链上不区分商户与个人）
- 执行收款方提现
- 收取平台费用
- 执行结算限额
- 发出可审计事件

**不负责：** 质押逻辑 · 铸币 · 用户钱包 · 任意外部调用 · 跨链桥接

---

## 2. 架构位置（Architectural Position）

```
User Payment
     ↓
Treasury Address
     ↓
Settlement Module
     ↓
Payee Address（收款方地址，链上不区分个人/商户）
```

### 两种可选模型：

**Model A — 直接入模块充值（Direct-to-Module Deposit）**

用户直接向 Settlement 模块支付。

| Pros | Cons |
|------|------|
| 完全透明 | 支付跟踪更复杂 |
| 链上账本原生 | |

**Model B — Treasury + Settlement**

资金先入 Treasury，Settlement 模块处理收款方 payout。

> **推荐用于：** 大型平台 · 多产品生态

---

## 3. 核心职责（Core Responsibilities）

**Settlement 模块必须：** 按地址跟踪收款方余额 · 跟踪平台费用计提 · 执行提现规则 · 防止重复提现 · 发出对账用事件

---

## 4. 数据结构（Data Structures）

本模块作为 Substrate Runtime 中的一个 Pallet，所有状态都存储在链上存储中。下面的数据结构为**逻辑结构**，具体类型通过 `Config` 中的关联类型确定。

### 4.1 收款方余额（Payee Balance）

链上按地址（`AccountId`）记账，不区分个人与商户。

```rust
/// 收款方余额（按 AccountId 记账）
PayeeBalance: map AccountId => Balance;
```

其中：

- **`AccountId`**：来自 Runtime 通用账户类型。
- **`Balance`**：通常为 `BalanceOf<T>`，由本链选定的记账单位决定。

### 4.2 收款方配置（Payee Configuration）

```rust
/// 收款方配置
struct PayeeConfig {
    active: bool,
    /// 单笔提现上限
    withdrawal_limit: Balance,
    /// 当日日累计提现上限
    daily_limit: Balance,
    /// 费用（万分比）
    fee_bps: u16,
}

/// 每个收款地址对应一份配置
PayeeConfigs: map AccountId => Option<PayeeConfig>;
```

### 4.3 平台费用累计（Platform Fee Accumulator）

```rust
/// 平台费用累计余额
PlatformFeeBalance: Balance;
```

---

（其余章节 5–18 及最终摘要与 `cera-chain/docs/05-smart-contract/settlement-contract.md` 一致。）

> 完整设计见：`cera-chain/docs/05-smart-contract/settlement-contract.md`。本目录下的 `pallets/settlement` 为该文档的 FRAME v2 实现。
