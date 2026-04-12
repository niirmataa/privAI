//! Protocol versioning for privAI V0.
//!
//! 9 version domains. Each versioned independently.
//! Hard rule: No silent downgrade from FullPrivacy.
//!
//! Activation:
//! - Chain-activated: by height or epoch
//! - Session/handshake-negotiated: NXMS transport, mailbox
//! - Declared per offer/session: lease, meter, discovery

use serde::{Deserialize, Serialize};

// ── Version Domain Identifiers ─────────────────────────────────────────

/// Protocol version domains.
///
/// Each domain is versioned independently.
/// A new format in any domain must declare version impact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum VersionDomain {
    /// Consensus, block format. Chain-activated (by height/epoch).
    ChainProtocol = 0x01,
    /// Transaction format. Chain-activated.
    TxVersion = 0x02,
    /// Escrow/lease policy rules. Chain-activated.
    EscrowPolicy = 0x03,
    /// ZK proof system. Chain-activated.
    ProofSystem = 0x04,
    /// NXMS transport. Session/handshake-negotiated.
    NxmsTransport = 0x05,
    /// Mailbox protocol. Session/handshake-negotiated.
    MailboxProtocol = 0x06,
    /// Compute lease session protocol. Declared per offer/session.
    ComputeLease = 0x07,
    /// Metering format. Declared per offer/session.
    MeterProtocol = 0x08,
    /// Discovery protocol. Declared per offer/session.
    DiscoveryProtocol = 0x09,
}

impl VersionDomain {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::ChainProtocol),
            0x02 => Some(Self::TxVersion),
            0x03 => Some(Self::EscrowPolicy),
            0x04 => Some(Self::ProofSystem),
            0x05 => Some(Self::NxmsTransport),
            0x06 => Some(Self::MailboxProtocol),
            0x07 => Some(Self::ComputeLease),
            0x08 => Some(Self::MeterProtocol),
            0x09 => Some(Self::DiscoveryProtocol),
            _ => None,
        }
    }

    /// How this domain is activated.
    pub fn activation(&self) -> VersionActivation {
        match self {
            Self::ChainProtocol | Self::TxVersion | Self::EscrowPolicy | Self::ProofSystem => {
                VersionActivation::ChainActivated
            }
            Self::NxmsTransport | Self::MailboxProtocol => VersionActivation::Handshake,
            Self::ComputeLease | Self::MeterProtocol | Self::DiscoveryProtocol => {
                VersionActivation::DeclaredPerSession
            }
        }
    }

    /// Domain name string (for hashing, logging, display).
    pub fn name(&self) -> &'static str {
        match self {
            Self::ChainProtocol => "chain_protocol_version",
            Self::TxVersion => "tx_version",
            Self::EscrowPolicy => "escrow_policy_version",
            Self::ProofSystem => "proof_system_id",
            Self::NxmsTransport => "nxms_transport_version",
            Self::MailboxProtocol => "mailbox_protocol_version",
            Self::ComputeLease => "compute_lease_protocol_version",
            Self::MeterProtocol => "meter_protocol_version",
            Self::DiscoveryProtocol => "discovery_protocol_version",
        }
    }

    /// All 9 domains.
    pub fn all() -> &'static [VersionDomain] {
        &[
            Self::ChainProtocol,
            Self::TxVersion,
            Self::EscrowPolicy,
            Self::ProofSystem,
            Self::NxmsTransport,
            Self::MailboxProtocol,
            Self::ComputeLease,
            Self::MeterProtocol,
            Self::DiscoveryProtocol,
        ]
    }
}

// ── Activation Mode ────────────────────────────────────────────────────

/// How a version domain is activated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VersionActivation {
    /// Activated by chain height or epoch.
    ChainActivated,
    /// Negotiated during session/handshake.
    Handshake,
    /// Declared per offer/session.
    DeclaredPerSession,
}

// ── Version Number ─────────────────────────────────────────────────────

/// A version number within a domain.
///
/// Higher = newer. Lower = older.
/// Downgrade from FullPrivacy is never allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Version(pub u16);

impl Version {
    pub const V1: Version = Version(1);

    pub fn new(v: u16) -> Self {
        Self(v)
    }
}

// ── Version Registry ───────────────────────────────────────────────────

/// Registry of active protocol versions.
///
/// Each node maintains its own registry.
/// Version negotiation happens per domain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionRegistry {
    /// Current versions per domain.
    pub versions: Vec<(VersionDomain, Version)>,
}

impl VersionRegistry {
    /// Create a new registry with all domains set to V1.
    pub fn new_v1() -> Self {
        let versions: Vec<(VersionDomain, Version)> = VersionDomain::all()
            .iter()
            .map(|d| (*d, Version::V1))
            .collect();
        Self { versions }
    }

    /// Get the version for a domain.
    pub fn get(&self, domain: VersionDomain) -> Option<Version> {
        self.versions
            .iter()
            .find(|(d, _)| *d == domain)
            .map(|(_, v)| *v)
    }

    /// Set the version for a domain.
    pub fn set(&mut self, domain: VersionDomain, version: Version) {
        if let Some(entry) = self.versions.iter_mut().find(|(d, _)| *d == domain) {
            entry.1 = version;
        } else {
            self.versions.push((domain, version));
        }
    }

    /// Check if a downgrade from FullPrivacy is attempted.
    ///
    /// Hard rule: No silent downgrade from FullPrivacy.
    /// Returns true if the proposed version is LOWER than current.
    pub fn is_downgrade(&self, domain: VersionDomain, proposed: Version) -> bool {
        match self.get(domain) {
            Some(current) => proposed < current,
            None => false,
        }
    }

    /// Validate a version negotiation (handshake).
    ///
    /// Returns Ok(proposed) if upgrade or same.
    /// Returns Err if downgrade.
    pub fn negotiate(
        &self,
        domain: VersionDomain,
        proposed: Version,
    ) -> Result<Version, VersionError> {
        if self.is_downgrade(domain, proposed) {
            return Err(VersionError::DowngradeRejected {
                domain,
                current: self.get(domain).unwrap_or(Version::V1),
                proposed,
            });
        }
        Ok(proposed)
    }

    /// List all domains with their current versions.
    pub fn list(&self) -> &[(VersionDomain, Version)] {
        &self.versions
    }
}

// ── Errors ─────────────────────────────────────────────────────────────

/// Version-related errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionError {
    /// Attempted downgrade from FullPrivacy.
    DowngradeRejected {
        domain: VersionDomain,
        current: Version,
        proposed: Version,
    },
    /// Unknown version domain.
    UnknownDomain { domain_id: u8 },
    /// Version not found in registry.
    VersionNotFound { domain: VersionDomain },
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DowngradeRejected {
                domain,
                current,
                proposed,
            } => write!(
                f,
                "downgrade rejected for {}: current={}, proposed={}",
                domain.name(),
                current.0,
                proposed.0
            ),
            Self::UnknownDomain { domain_id } => {
                write!(f, "unknown version domain: 0x{:02x}", domain_id)
            }
            Self::VersionNotFound { domain } => {
                write!(f, "version not found for {}", domain.name())
            }
        }
    }
}

impl std::error::Error for VersionError {}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_nine_domains_exist() {
        assert_eq!(VersionDomain::all().len(), 9);
    }

    #[test]
    fn domain_roundtrip() {
        for domain in VersionDomain::all() {
            let id = *domain as u8;
            let back = VersionDomain::from_u8(id);
            assert_eq!(back, Some(*domain), "roundtrip failed for {:?}", domain);
        }
    }

    #[test]
    fn unknown_domain_returns_none() {
        assert!(VersionDomain::from_u8(0x00).is_none());
        assert!(VersionDomain::from_u8(0x0A).is_none());
        assert!(VersionDomain::from_u8(0xFF).is_none());
    }

    #[test]
    fn activation_types_are_correct() {
        assert_eq!(
            VersionDomain::ChainProtocol.activation(),
            VersionActivation::ChainActivated
        );
        assert_eq!(
            VersionDomain::TxVersion.activation(),
            VersionActivation::ChainActivated
        );
        assert_eq!(
            VersionDomain::EscrowPolicy.activation(),
            VersionActivation::ChainActivated
        );
        assert_eq!(
            VersionDomain::ProofSystem.activation(),
            VersionActivation::ChainActivated
        );
        assert_eq!(
            VersionDomain::NxmsTransport.activation(),
            VersionActivation::Handshake
        );
        assert_eq!(
            VersionDomain::MailboxProtocol.activation(),
            VersionActivation::Handshake
        );
        assert_eq!(
            VersionDomain::ComputeLease.activation(),
            VersionActivation::DeclaredPerSession
        );
        assert_eq!(
            VersionDomain::MeterProtocol.activation(),
            VersionActivation::DeclaredPerSession
        );
        assert_eq!(
            VersionDomain::DiscoveryProtocol.activation(),
            VersionActivation::DeclaredPerSession
        );
    }

    #[test]
    fn domain_names_are_stable() {
        assert_eq!(
            VersionDomain::ChainProtocol.name(),
            "chain_protocol_version"
        );
        assert_eq!(VersionDomain::TxVersion.name(), "tx_version");
        assert_eq!(VersionDomain::EscrowPolicy.name(), "escrow_policy_version");
        assert_eq!(VersionDomain::ProofSystem.name(), "proof_system_id");
        assert_eq!(
            VersionDomain::NxmsTransport.name(),
            "nxms_transport_version"
        );
        assert_eq!(
            VersionDomain::MailboxProtocol.name(),
            "mailbox_protocol_version"
        );
        assert_eq!(
            VersionDomain::ComputeLease.name(),
            "compute_lease_protocol_version"
        );
        assert_eq!(
            VersionDomain::MeterProtocol.name(),
            "meter_protocol_version"
        );
        assert_eq!(
            VersionDomain::DiscoveryProtocol.name(),
            "discovery_protocol_version"
        );
    }

    #[test]
    fn registry_new_v1_has_all_domains() {
        let registry = VersionRegistry::new_v1();
        for domain in VersionDomain::all() {
            assert_eq!(registry.get(*domain), Some(Version::V1));
        }
    }

    #[test]
    fn registry_set_and_get() {
        let mut registry = VersionRegistry::new_v1();
        registry.set(VersionDomain::MeterProtocol, Version::new(2));
        assert_eq!(
            registry.get(VersionDomain::MeterProtocol),
            Some(Version::new(2))
        );
        assert_eq!(
            registry.get(VersionDomain::ChainProtocol),
            Some(Version::V1)
        );
    }

    #[test]
    fn no_downgrade_rejected() {
        let mut registry = VersionRegistry::new_v1();
        registry.set(VersionDomain::NxmsTransport, Version::new(3));

        // Upgrade is OK
        assert!(registry
            .negotiate(VersionDomain::NxmsTransport, Version::new(4))
            .is_ok());

        // Same version is OK
        assert!(registry
            .negotiate(VersionDomain::NxmsTransport, Version::new(3))
            .is_ok());

        // Downgrade is REJECTED
        let result = registry.negotiate(VersionDomain::NxmsTransport, Version::new(2));
        assert!(result.is_err());
        match result {
            Err(VersionError::DowngradeRejected {
                domain,
                current,
                proposed,
            }) => {
                assert_eq!(domain, VersionDomain::NxmsTransport);
                assert_eq!(current, Version::new(3));
                assert_eq!(proposed, Version::new(2));
            }
            _ => panic!("expected DowngradeRejected"),
        }
    }

    #[test]
    fn version_ordering() {
        assert!(Version::V1 < Version::new(2));
        assert!(Version::new(2) < Version::new(3));
        assert!(Version::new(3) > Version::new(1));
    }

    #[test]
    fn domain_name_stability_across_versions() {
        // Domain names must never change — they are used in hashing and logging
        let registry_v1 = VersionRegistry::new_v1();
        let mut registry_v2 = VersionRegistry::new_v1();
        registry_v2.set(VersionDomain::ChainProtocol, Version::new(2));

        // Names are the same regardless of version
        assert_eq!(
            VersionDomain::ChainProtocol.name(),
            "chain_protocol_version"
        );
    }
}
