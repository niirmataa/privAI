//! Main compute session runtime loop.
//! 
//! Connects the off-chain Miner Agent (`metering.rs`) with the 
//! blockchain clock and NXMS transport.

use crate::metering::MeteringAgent;
use privai_chain::compute_lease::{ComputeLeasePolicy, HeartbeatStatus};
use privai_chain::primitives::Hash32;
use std::time::{SystemTime, UNIX_EPOCH};

/// Trait representing the connection to the L1 chain clock.
pub trait BlockClock {
    /// Waits until the next block is produced and returns its hash and height.
    fn wait_for_next_block(&self) -> (Hash32, u64);
}

/// Trait representing the secure transport layer (Tor/P2P/NXMS).
pub trait NetworkTransport {
    /// Blocks until a challenge is received from the User for the given window.
    fn wait_for_challenge(&self, expected_block_hash: Hash32) -> Result<Hash32, &'static str>;
    /// Sends the signed telemetry record back to the user.
    fn send_telemetry(&self, record_bytes: Vec<u8>) -> Result<(), &'static str>;
}

/// Trait representing the hardware performance benchmark logic.
pub trait HardwareBenchmark {
    /// Runs the benchmark suite and returns the execution time in milliseconds.
    fn run_benchmark(&self) -> u32;
}

/// The main loop running on the Miner's machine after a lease is accepted.
pub struct ComputeSessionRunner<C: BlockClock, T: NetworkTransport, B: HardwareBenchmark> {
    pub session_id: Hash32,
    pub policy: ComputeLeasePolicy,
    pub agent: MeteringAgent,
    clock: C,
    transport: T,
    benchmark: B,
}

impl<C: BlockClock, T: NetworkTransport, B: HardwareBenchmark> ComputeSessionRunner<C, T, B> {
    pub fn new(
        session_id: Hash32, 
        policy: ComputeLeasePolicy, 
        agent: MeteringAgent,
        clock: C,
        transport: T,
        benchmark: B,
    ) -> Self {
        Self { session_id, policy, agent, clock, transport, benchmark }
    }

    /// Block listener loop matching the V0 Core Draft.
    /// Executes every `window_duration_blocks` (e.g. 60 blocks).
    pub fn run_session_loop(&mut self) -> Result<(), &'static str> {
        let total_windows = self.policy.total_windows;
        
        for window_idx in 0..total_windows {
            // 1. Wait for block clock to tick (e.g., waiting 60 blocks per window)
            // In reality, this would loop until `start_height + (window_idx * duration)` is reached.
            let (block_hash, _height) = self.clock.wait_for_next_block();
            
            // 2. Receive challenge from User based on the unpredictable block_hash
            // If the user doesn't send it within timeout, availability fails.
            let challenge_result = self.transport.wait_for_challenge(block_hash);
            
            let (challenge_hash, passed, performance_pass) = match challenge_result {
                Ok(ch) => {
                    // 3. Verify availability & performance
                    let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    let benchmark_time = self.benchmark.run_benchmark();
                    let end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    
                    let ping_latency = (end_time - start_time) as u32;
                    let perf_pass = benchmark_time <= self.policy.benchmark_floor_ms;
                    
                    (ch, true, Some(perf_pass))
                }
                Err(_) => {
                    // Challenge timeout or network failure
                    ([0u8; 32], false, None)
                }
            };
            
            // 4. Record window in the hash-chain
            let record = self.agent.record_window(
                window_idx,
                0, // response_time_ms
                challenge_hash,
                0, // ping_latency
                passed,
                performance_pass,
                if passed { HeartbeatStatus::Active } else { HeartbeatStatus::Missed },
                None, None, None, vec![]
            )?;
            
            // 5. Send record to User
            // In a real system, you would serialize `record` here.
            let _ = self.transport.send_telemetry(vec![]);
        }
        
        Ok(())
    }
}
