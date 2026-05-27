//! Reinforcement-learning control seam for waremax.
//!
//! Exposes the task-allocation decision of the discrete-event simulator as a
//! Gym-style environment ([`RlEnv`]) driven by an external agent. The simulator
//! is left untouched: an [`RlPolicy`] is injected as the task-allocation policy
//! and blocks at each decision waiting for an action over channels, while the
//! simulation runs on a worker thread. The handshake is strict ping-pong, so the
//! run is deterministic given a seed and an action sequence.
//!
//! See [`crate::env::RlEnv`] for the entry point.

pub mod env;
pub mod observation;
pub mod policy;
pub mod protocol;
pub mod reward;

pub use env::{RlEnv, StepResult};
pub use observation::{Observation, MAX_ROBOTS, ROBOT_FEATS, TASK_FEATS};
pub use policy::RlPolicy;
pub use protocol::{ActionMsg, FinalMetrics, Message, StepInfo};
pub use reward::{RewardConfig, RewardMode, RewardSnapshot};
