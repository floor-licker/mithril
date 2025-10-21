# Mithril Anywhere - Implementation Plan

## Immediate Action Items

### Decision Points (Make These First)

#### 1. Development Approach
- [ ] **Independent repo** (faster, more control) or **Fork + PR** (easier to merge later)?
  - Recommendation: Start with independent repo, easier to experiment

#### 2. Initial Scope
- [ ] **Full Phase 1+2** (abstraction layer + Ethereum) or **Phase 1 only** (abstraction layer)?
  - Recommendation: Phase 1 only first (4 weeks), validate approach before Ethereum work

#### 3. Engagement Strategy
- [ ] Contact IOG Mithril team **now** or **after prototype**?
  - Recommendation: Send courtesy heads-up now, formal discussion after Phase 1

#### 4. Public vs Private
- [ ] Build in **public** from day 1 or **private** until working?
  - Recommendation: Public from day 1 for community feedback and credibility

---

## Week 1: Setup and Foundation

### Day 1-2: Project Setup

**Create Repository Structure**
```bash
# Option A: Independent repo (recommended)
mkdir mithril-universal
cd mithril-universal
git init
cargo new --lib mithril-chain-abstraction

# Option B: Fork and branch
git clone https://github.com/input-output-hk/mithril.git mithril-universal
cd mithril-universal
git checkout -b feature/universal-chain-support
```

**Initial Repository Structure**
```
mithril-universal/
├── README.md
├── Cargo.toml (workspace)
├── .github/
│   └── workflows/
│       └── ci.yml
├── mithril-chain-abstraction/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── chain_observer.rs
│   │   ├── types.rs
│   │   └── errors.rs
│   └── tests/
└── docs/
    ├── DESIGN.md (copy from your design doc)
    └── CONTRIBUTING.md
```

**Dependencies Setup**
```bash
cd mithril-chain-abstraction

# If independent repo, add Mithril dependencies
cargo add mithril-common --git https://github.com/input-output-hk/mithril
cargo add mithril-stm --git https://github.com/input-output-hk/mithril

# Core dependencies
cargo add async-trait
cargo add serde --features derive
cargo add tokio --features full
cargo add anyhow
cargo add thiserror
```

### Day 3-5: Core Abstractions

**Task 1: Define Universal Chain Observer Trait**

Create `mithril-chain-abstraction/src/chain_observer.rs`:

```rust
use async_trait::async_trait;
use std::collections::HashMap;

/// Identifier for a blockchain
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChainId(String);

impl ChainId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Epoch information for a chain
#[derive(Debug, Clone)]
pub struct EpochInfo {
    pub chain_id: ChainId,
    pub epoch_number: u64,
    pub start_time: i64,
    pub end_time: Option<i64>,
}

/// Stake distribution for an epoch
#[derive(Debug, Clone)]
pub struct StakeDistribution {
    pub epoch: u64,
    pub validators: HashMap<ValidatorId, u64>,
    pub total_stake: u64,
}

/// Unique validator identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidatorId(String);

impl ValidatorId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

/// State commitment representing chain state
#[derive(Debug, Clone)]
pub struct StateCommitment {
    pub chain_id: ChainId,
    pub epoch: u64,
    pub commitment_type: CommitmentType,
    pub value: Vec<u8>,
    pub block_number: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitmentType {
    StateRoot,
    AccountsHash,
    ImmutableFileSet,
    ParachainHead,
}

/// Universal chain observer trait
#[async_trait]
pub trait UniversalChainObserver: Send + Sync {
    /// Get the chain identifier
    fn chain_id(&self) -> ChainId;
    
    /// Get current epoch information
    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError>;
    
    /// Get validator set and stakes for an epoch
    async fn get_stake_distribution(
        &self,
        epoch: u64,
    ) -> Result<StakeDistribution, ChainObserverError>;
    
    /// Compute state commitment for an epoch
    async fn compute_state_commitment(
        &self,
        epoch: u64,
    ) -> Result<StateCommitment, ChainObserverError>;
    
    /// Check if validator is active in this epoch
    async fn is_validator_active(
        &self,
        validator_id: &ValidatorId,
        epoch: u64,
    ) -> Result<bool, ChainObserverError>;
}
```

**Task 2: Error Types**

Create `mithril-chain-abstraction/src/errors.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainObserverError {
    #[error("Failed to connect to chain: {0}")]
    ConnectionError(String),
    
    #[error("Failed to query epoch data: {0}")]
    EpochQueryError(String),
    
    #[error("Failed to get stake distribution: {0}")]
    StakeDistributionError(String),
    
    #[error("Failed to compute state commitment: {0}")]
    StateCommitmentError(String),
    
    #[error("Invalid data received: {0}")]
    InvalidData(String),
    
    #[error("Chain-specific error: {0}")]
    ChainSpecific(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

**Task 3: Write Tests**

Create `mithril-chain-abstraction/tests/chain_observer_test.rs`:

```rust
use mithril_chain_abstraction::*;

// Mock implementation for testing
struct MockChainObserver {
    chain_id: ChainId,
}

#[async_trait::async_trait]
impl UniversalChainObserver for MockChainObserver {
    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }
    
    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
        Ok(EpochInfo {
            chain_id: self.chain_id.clone(),
            epoch_number: 100,
            start_time: 1234567890,
            end_time: None,
        })
    }
    
    // ... implement other methods
}

#[tokio::test]
async fn test_mock_observer() {
    let observer = MockChainObserver {
        chain_id: ChainId::new("test-chain"),
    };
    
    let epoch = observer.get_current_epoch().await.unwrap();
    assert_eq!(epoch.epoch_number, 100);
}
```

### Day 6-7: Cardano Adapter

**Task 4: Create Cardano Adapter**

This proves the abstraction works by wrapping existing Cardano implementation.

Create `mithril-chain-abstraction/src/adapters/cardano.rs`:

```rust
use async_trait::async_trait;
use std::sync::Arc;
use mithril_cardano_node_chain::ChainObserver as CardanoChainObserver;
use crate::*;

/// Adapter that wraps Cardano's existing ChainObserver
pub struct CardanoChainObserverAdapter {
    cardano_observer: Arc<dyn CardanoChainObserver>,
    chain_id: ChainId,
}

impl CardanoChainObserverAdapter {
    pub fn new(cardano_observer: Arc<dyn CardanoChainObserver>, network: &str) -> Self {
        Self {
            cardano_observer,
            chain_id: ChainId::new(format!("cardano-{}", network)),
        }
    }
}

#[async_trait]
impl UniversalChainObserver for CardanoChainObserverAdapter {
    fn chain_id(&self) -> ChainId {
        self.chain_id.clone()
    }
    
    async fn get_current_epoch(&self) -> Result<EpochInfo, ChainObserverError> {
        let epoch = self.cardano_observer
            .get_current_epoch()
            .await
            .map_err(|e| ChainObserverError::EpochQueryError(e.to_string()))?
            .ok_or_else(|| ChainObserverError::EpochQueryError("No epoch found".to_string()))?;
        
        Ok(EpochInfo {
            chain_id: self.chain_id.clone(),
            epoch_number: epoch,
            start_time: 0, // Calculate from epoch
            end_time: None,
        })
    }
    
    async fn get_stake_distribution(
        &self,
        _epoch: u64,
    ) -> Result<StakeDistribution, ChainObserverError> {
        let stake_dist = self.cardano_observer
            .get_current_stake_distribution()
            .await
            .map_err(|e| ChainObserverError::StakeDistributionError(e.to_string()))?
            .ok_or_else(|| ChainObserverError::StakeDistributionError("No stake distribution".to_string()))?;
        
        let validators = stake_dist
            .into_iter()
            .map(|(party_id, stake)| (ValidatorId::new(party_id), stake))
            .collect::<HashMap<_, _>>();
        
        let total_stake = validators.values().sum();
        
        Ok(StakeDistribution {
            epoch: _epoch,
            validators,
            total_stake,
        })
    }
    
    async fn compute_state_commitment(
        &self,
        epoch: u64,
    ) -> Result<StateCommitment, ChainObserverError> {
        // For Cardano, this would be the immutable file set
        // This is simplified - real implementation would compute digest
        Ok(StateCommitment {
            chain_id: self.chain_id.clone(),
            epoch,
            commitment_type: CommitmentType::ImmutableFileSet,
            value: vec![], // TODO: compute actual digest
            block_number: 0,
            metadata: HashMap::new(),
        })
    }
    
    async fn is_validator_active(
        &self,
        _validator_id: &ValidatorId,
        _epoch: u64,
    ) -> Result<bool, ChainObserverError> {
        // Check if validator is in stake distribution
        Ok(true) // Simplified
    }
}
```

---

## Week 2: Testing and Documentation

### Day 8-10: Integration Tests

**Create Integration Test Suite**

```bash
# Add test dependencies
cd mithril-chain-abstraction
cargo add --dev tokio-test
cargo add --dev mockall
```

Create comprehensive tests:
```rust
// Test that Cardano adapter works with existing infrastructure
// Test that traits can be mocked
// Test error handling
// Test edge cases
```

### Day 11-12: Documentation

**Write Comprehensive Docs**

1. **API Documentation**
   - Add docstrings to all public items
   - Include examples in docs
   - Run `cargo doc --open` to verify

2. **Usage Guide**
   ```markdown
   # How to Implement a Chain Observer
   
   ## Step 1: Define Your Chain's Types
   ## Step 2: Implement the Trait
   ## Step 3: Test Your Implementation
   ```

3. **Architecture Document**
   - Explain design decisions
   - Show how it integrates with Mithril
   - Provide examples

### Day 13-14: CI/CD and Polish

**Setup GitHub Actions**

`.github/workflows/ci.yml`:
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

**Polish**
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy --fix`
- [ ] Add examples/ directory with usage examples
- [ ] Write good README.md
- [ ] Add LICENSE (Apache 2.0 to match Mithril)

---

## Week 3: Validation and Feedback

### Day 15-17: Internal Validation

**Checklist**
- [ ] All tests pass
- [ ] Documentation is complete
- [ ] CI/CD is green
- [ ] Cardano adapter proves abstraction works
- [ ] Code is clean and well-commented

**Create Example Usage**
```rust
// examples/basic_usage.rs
use mithril_chain_abstraction::*;

#[tokio::main]
async fn main() {
    // Show how to use the trait
    // Demonstrate with mock implementation
}
```

### Day 18-21: Community Feedback

**Public Announcement**

1. **Tweet Thread**
   ```
   1/ Building Mithril Anywhere: extending Mithril's fast-sync to work
   with any proof-of-stake blockchain. Starting with the abstraction layer.
   
   2/ The insight: Mithril's STM cryptography is already chain-agnostic.
   We just need clean interfaces for different chains to plug in.
   
   3/ Here's the UniversalChainObserver trait. Any chain that implements
   this can use Mithril's stake-based threshold signatures.
   [code snippet]
   
   4/ Proved it works by wrapping Cardano's existing observer. Zero
   changes to Cardano functionality, but now extensible.
   
   5/ Next up: Ethereum implementation. Building the beacon chain client
   to fetch validator sets and state roots. Target: <2 hour ETH sync.
   
   6/ This is all open source, building in public. Looking for feedback
   from blockchain devs, especially Ethereum validators.
   Link: [GitHub repo]
   ```

2. **Reddit Post**
   - r/cardano: "Extending Mithril beyond Cardano"
   - r/ethereum: "Bringing Cardano's fast-sync tech to Ethereum"
   - r/crypto: "Universal fast-sync for PoS blockchains"

3. **Discord/Telegram**
   - IOG Discord #mithril channel
   - Ethereum R&D Discord
   - Various validator communities

**Collect Feedback**
- GitHub issues
- Direct messages
- Community discussions
- Technical reviews

---

## Week 4: Refinement and Planning

### Day 22-25: Incorporate Feedback

- Refactor based on community input
- Fix any issues discovered
- Improve documentation
- Add requested features

### Day 26-28: Phase 2 Planning

**Prepare for Ethereum Integration**

1. **Research**
   - [ ] Study Ethereum Beacon API thoroughly
   - [ ] Identify Rust beacon chain libraries
   - [ ] Review Lighthouse and Prysm architectures
   - [ ] Understand validator activation/exit flows

2. **Design**
   - [ ] Draft Ethereum-specific types
   - [ ] Design beacon chain client API
   - [ ] Plan state root extraction strategy
   - [ ] Design validator sampling approach

3. **Outreach**
   - [ ] Identify 10 potential Ethereum validators for testnet
   - [ ] Draft validator recruitment message
   - [ ] Plan testnet launch strategy

---

## Critical Success Factors

### Week 1 Success
- [ ] Clean trait definition that's actually chain-agnostic
- [ ] Cardano adapter proves it works
- [ ] Tests demonstrate flexibility

### Week 2 Success
- [ ] Documentation explains the "why" not just "what"
- [ ] Examples show how to implement for new chains
- [ ] CI/CD catches regressions

### Week 3 Success
- [ ] Positive reception from community
- [ ] At least 3 people star the repo
- [ ] No major design flaws identified

### Week 4 Success
- [ ] Refined, production-quality abstraction layer
- [ ] Clear path to Ethereum integration
- [ ] Team assembled (even if just 1-2 people)

---

## Resources Needed

### Required
- Development machine with Rust toolchain
- GitHub account
- Time: 20-40 hours/week for 4 weeks
- Access to Cardano testnet node (for testing adapter)

### Helpful
- Twitter account for build-in-public updates
- Domain for documentation site (optional)
- Discord presence in relevant communities
- Ethereum testnet node access (for Phase 2 prep)

### Optional
- Budget for infrastructure ($50-100/month)
- Graphics/design help for diagrams
- Technical writer for docs
- Grant funding for full-time work

---

## Decision Matrix

### Should I start this week?

**YES if:**
- You have 20+ hours/week available
- You're comfortable with Rust and async
- You want to build in public
- You're okay with uncertainty

**WAIT if:**
- You need IOG approval first
- You want a team assembled
- You need grant funding secured
- You're uncertain about the technical approach

### Should I go independent or fork?

**Independent repo if:**
- You want maximum flexibility
- You're willing to maintain sync with upstream
- You want to move fast
- You're okay with potential future merge complexity

**Fork if:**
- You want easier merge path later
- You don't mind slower pace
- You prefer official contribution from start
- You have IOG relationship

---

## Next Physical Steps

### Right Now (Next 30 Minutes)

1. **Make decisions above**
   - Independent or fork?
   - Contact IOG or build first?
   - Public or private initially?

2. **Create repository**
   ```bash
   mkdir mithril-universal
   cd mithril-universal
   git init
   # Setup initial structure
   ```

3. **Create project board**
   - GitHub Projects or Trello
   - Add tasks from Week 1
   - Set target dates

### Tomorrow (Day 1)

1. **Setup Cargo workspace**
2. **Create mithril-chain-abstraction crate**
3. **Define initial trait in chain_observer.rs**
4. **Write first test**
5. **Commit and push**

### This Week (Days 2-7)

Follow Week 1 plan above, day by day.

---

## Questions to Answer First

1. **Scope**: Just abstraction layer or include Ethereum?
   - My recommendation: Abstraction layer first

2. **Timeline**: 4 weeks part-time or 2 weeks full-time?
   - Be realistic about available hours

3. **Team**: Solo or recruit help?
   - Start solo, recruit after Week 1 success

4. **Funding**: Bootstrap or seek grant?
   - Can bootstrap Phase 1, may need funding for Phase 2

5. **IOG**: Engage now or after prototype?
   - My recommendation: Courtesy heads-up now, detailed discussion after Week 2

---

## Ready to Start?

If you're ready, the very first command is:

```bash
mkdir mithril-universal && cd mithril-universal && git init
```

Then create the Cargo workspace and start implementing the trait.

Would you like me to:
1. Generate the initial boilerplate code files?
2. Draft the message to IOG Mithril team?
3. Create a detailed day-by-day task list?
4. Something else?

