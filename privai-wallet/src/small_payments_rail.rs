use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use privai_chain::Hash32;
use privai_chain::Nullifier;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RailContext {
    pub rail_seed: [u8; 32],
    pub tickets: HashMap<Hash32, LocalTicketPool>,
}

impl RailContext {
    pub fn new(rail_seed: [u8; 32]) -> Self {
        Self {
            rail_seed,
            tickets: HashMap::new(),
        }
    }

    pub fn get_or_create_pool(&mut self, merchant_commit: Hash32) -> &mut LocalTicketPool {
        self.tickets
            .entry(merchant_commit)
            .or_insert_with(|| LocalTicketPool::new(self.rail_seed, merchant_commit))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalTicket {
    pub ticket_id: Hash32,
    pub ticket_nullifier: Nullifier,
    pub ticket_auth: [u8; 32],
    pub is_used: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalTicketPool {
    pub merchant_commit: Hash32,
    pub next_index: u64,
    pub generated_tickets: Vec<LocalTicket>,
}

impl LocalTicketPool {
    pub fn new(_rail_seed: [u8; 32], merchant_commit: Hash32) -> Self {
        Self {
            merchant_commit,
            next_index: 0,
            generated_tickets: Vec::new(),
        }
    }

    pub fn generate_next_ticket(&mut self, rail_seed: &[u8; 32]) -> LocalTicket {
        let index_bytes = self.next_index.to_le_bytes();

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"nxms_privai_ticket_id_v0");
        hasher.update(rail_seed);
        hasher.update(&self.merchant_commit);
        hasher.update(&index_bytes);
        let ticket_id: Hash32 = hasher.finalize().into();

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"nxms_privai_ticket_nullifier_v0");
        hasher.update(rail_seed);
        hasher.update(&self.merchant_commit);
        hasher.update(&index_bytes);
        let ticket_nullifier = Nullifier(hasher.finalize().into());

        let mut hasher = blake3::Hasher::new();
        hasher.update(b"nxms_privai_ticket_auth_v0");
        hasher.update(rail_seed);
        hasher.update(&self.merchant_commit);
        hasher.update(&index_bytes);
        let ticket_auth: [u8; 32] = hasher.finalize().into();

        let ticket = LocalTicket {
            ticket_id,
            ticket_nullifier,
            ticket_auth,
            is_used: false,
        };

        self.generated_tickets.push(ticket.clone());
        self.next_index += 1;

        ticket
    }

    pub fn mark_used(&mut self, ticket_nullifier: &Nullifier) -> bool {
        for ticket in &mut self.generated_tickets {
            if &ticket.ticket_nullifier == ticket_nullifier {
                ticket.is_used = true;
                return true;
            }
        }
        false
    }
}
