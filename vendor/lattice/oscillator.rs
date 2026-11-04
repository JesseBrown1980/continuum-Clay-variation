//! Local transition lattice above representation functions and estate catalogs.
//!
//! There is no single global forward chain followed by one late unwind. Every
//! representation edge exists in all three tagged directions at every sector
//! and level. Lockstep completion of those local plays advances the shell.

use crate::trit::Trit;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RepresentationNode {
    NullOccupancy,
    FirstOccupancy,
    BinaryAlphabet,
    TaggedTriad,
    GlyphCube27,
    BinaryCommit,
}

impl RepresentationNode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NullOccupancy => "0_null_occupancy",
            Self::FirstOccupancy => "1_first_occupancy",
            Self::BinaryAlphabet => "2_binary_alphabet",
            Self::TaggedTriad => "3_tagged_neutral_alphabet",
            Self::GlyphCube27 => "27_three_axis_composition",
            Self::BinaryCommit => "binary_exact_framed_commit",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RepresentationEdge {
    NullFirst,
    FirstBinary,
    BinaryTriad,
    TriadGlyph27,
    Glyph27BinaryCommit,
}

impl RepresentationEdge {
    pub const ALL: [Self; 5] = [
        Self::NullFirst,
        Self::FirstBinary,
        Self::BinaryTriad,
        Self::TriadGlyph27,
        Self::Glyph27BinaryCommit,
    ];

    pub const fn ordinal(self) -> u8 {
        match self {
            Self::NullFirst => 0,
            Self::FirstBinary => 1,
            Self::BinaryTriad => 2,
            Self::TriadGlyph27 => 3,
            Self::Glyph27BinaryCommit => 4,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::NullFirst => "0_1",
            Self::FirstBinary => "1_2",
            Self::BinaryTriad => "2_3",
            Self::TriadGlyph27 => "3_27",
            Self::Glyph27BinaryCommit => "27_binary",
        }
    }

    pub const fn endpoints(self) -> (RepresentationNode, RepresentationNode) {
        match self {
            Self::NullFirst => (
                RepresentationNode::NullOccupancy,
                RepresentationNode::FirstOccupancy,
            ),
            Self::FirstBinary => (
                RepresentationNode::FirstOccupancy,
                RepresentationNode::BinaryAlphabet,
            ),
            Self::BinaryTriad => (
                RepresentationNode::BinaryAlphabet,
                RepresentationNode::TaggedTriad,
            ),
            Self::TriadGlyph27 => (
                RepresentationNode::TaggedTriad,
                RepresentationNode::GlyphCube27,
            ),
            Self::Glyph27BinaryCommit => (
                RepresentationNode::GlyphCube27,
                RepresentationNode::BinaryCommit,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransitionKey {
    pub level: u8,
    pub sector: u16,
    pub edge: RepresentationEdge,
    pub direction: Trit,
}

impl TransitionKey {
    pub const fn inverse(self) -> Self {
        let direction = match self.direction {
            Trit::NegativeZero => Trit::PositiveZero,
            Trit::CenterZero => Trit::CenterZero,
            Trit::PositiveZero => Trit::NegativeZero,
        };
        Self { direction, ..self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionRole {
    Fischer,
    Mtp,
    Hrm,
    Mcp,
    QuantCatalog,
    EngineCatalog,
}

impl TransitionRole {
    pub const ALL: [Self; 6] = [
        Self::Fischer,
        Self::Mtp,
        Self::Hrm,
        Self::Mcp,
        Self::QuantCatalog,
        Self::EngineCatalog,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleBinding {
    pub role: TransitionRole,
    pub key: TransitionKey,
}

pub fn bind_roles(key: TransitionKey) -> [RoleBinding; 6] {
    TransitionRole::ALL.map(|role| RoleBinding { role, key })
}

pub fn level_transition_keys(level: u8, sectors: usize) -> Result<Vec<TransitionKey>, String> {
    let sector_limit =
        u16::try_from(sectors).map_err(|_| "sector count exceeds u16 key width".to_string())?;
    if sector_limit == 0 {
        return Err("transition lattice requires at least one sector".to_string());
    }
    let mut keys = Vec::with_capacity(sectors * RepresentationEdge::ALL.len() * Trit::ALL.len());
    for sector in 0..sector_limit {
        for edge in RepresentationEdge::ALL {
            for direction in Trit::ALL {
                keys.push(TransitionKey {
                    level,
                    sector,
                    edge,
                    direction,
                });
            }
        }
    }
    Ok(keys)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarrierAdvance {
    pub completed_level: u8,
    pub completed_sectors: usize,
    pub completed_local_keys: usize,
    pub next_level: u8,
    pub next_sectors: usize,
}

pub fn advance_completed_barrier(
    completed_level: u8,
    completed_sectors: usize,
    completed_local_keys: usize,
) -> Result<BarrierAdvance, String> {
    if completed_sectors == 0 || completed_sectors % 3 != 0 {
        return Err(
            "completed shell sector count must be a positive multiple of three".to_string(),
        );
    }
    let required_keys = completed_sectors
        .checked_mul(RepresentationEdge::ALL.len())
        .and_then(|value| value.checked_mul(Trit::ALL.len()))
        .ok_or_else(|| "transition lattice width overflow".to_string())?;
    if completed_local_keys != required_keys {
        return Err(format!(
            "barrier cannot advance before all local keys complete: expected {required_keys}, got {completed_local_keys}"
        ));
    }
    let next_sectors = completed_sectors
        .checked_mul(3)
        .and_then(|value| value.checked_mul(3))
        .ok_or_else(|| "sector recurrence overflow".to_string())?;
    let next_level = completed_level
        .checked_add(1)
        .ok_or_else(|| "level recurrence overflow".to_string())?;
    Ok(BarrierAdvance {
        completed_level,
        completed_sectors,
        completed_local_keys,
        next_level,
        next_sectors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_level_sector_and_edge_has_all_three_local_directions() {
        for (level, sectors) in [(0, 3), (1, 27), (2, 243)] {
            let keys = level_transition_keys(level, sectors).unwrap();
            assert_eq!(keys.len(), sectors * 5 * 3);
            for sector in 0..sectors {
                for edge in RepresentationEdge::ALL {
                    let directions: Vec<Trit> = keys
                        .iter()
                        .filter(|key| usize::from(key.sector) == sector && key.edge == edge)
                        .map(|key| key.direction)
                        .collect();
                    assert_eq!(directions, Trit::ALL);
                }
            }
        }
    }

    #[test]
    fn local_reverse_and_hold_are_active_at_every_edge() {
        let keys = level_transition_keys(0, 3).unwrap();
        for sector in 0..3 {
            for edge in RepresentationEdge::ALL {
                for direction in Trit::ALL {
                    assert!(keys.contains(&TransitionKey {
                        level: 0,
                        sector,
                        edge,
                        direction,
                    }));
                }
            }
        }
    }

    #[test]
    fn inverse_is_a_local_key_involution() {
        for key in level_transition_keys(2, 243).unwrap() {
            assert_eq!(key.inverse().inverse(), key);
            assert_eq!(key.inverse().level, key.level);
            assert_eq!(key.inverse().sector, key.sector);
            assert_eq!(key.inverse().edge, key.edge);
        }
    }

    #[test]
    fn every_participating_role_receives_the_identical_key() {
        let key = TransitionKey {
            level: 1,
            sector: 12,
            edge: RepresentationEdge::TriadGlyph27,
            direction: Trit::NegativeZero,
        };
        let bindings = bind_roles(key);
        assert_eq!(bindings.len(), TransitionRole::ALL.len());
        assert!(bindings.iter().all(|binding| binding.key == key));
        assert_eq!(
            bindings
                .iter()
                .filter(|binding| binding.role == TransitionRole::Fischer)
                .count(),
            1
        );
    }

    #[test]
    fn lockstep_barrier_requires_every_local_play_before_times_nine() {
        let first_keys = level_transition_keys(0, 3).unwrap().len();
        assert!(advance_completed_barrier(0, 3, first_keys - 1).is_err());
        let first = advance_completed_barrier(0, 3, first_keys).unwrap();
        assert_eq!(first.completed_local_keys, 45);
        assert_eq!(first.next_sectors, 27);

        let second_keys = level_transition_keys(first.next_level, first.next_sectors)
            .unwrap()
            .len();
        let second =
            advance_completed_barrier(first.next_level, first.next_sectors, second_keys).unwrap();
        assert_eq!(second.completed_local_keys, 405);
        assert_eq!(second.next_sectors, 243);
    }
}
