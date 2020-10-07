//! resources for identifying what plays belong to which team
//! TODO separate team/player ambiguity

use std::collections::HashMap;

pub type TeamId = usize;

/// units on the debug team can be controlled and attacked by any team
pub const DEBUG_TEAM: TeamId = 0;

#[derive(Clone, Debug, PartialEq)]
pub enum TeamRelation {
    Same,
    Allied,
    Enemy,
}

/// a lookup from (team, team): relation
pub struct TeamRelationshipLookup(pub HashMap<(TeamId, TeamId), TeamRelation>);

impl Default for TeamRelationshipLookup {
    fn default() -> Self {
        TeamRelationshipLookup(HashMap::new())
    }
}

impl TeamRelationshipLookup {
    pub fn is_foe(self, t1: TeamId, t2: TeamId) -> bool {
        let rel = self.0.get(&(t1, t2)).expect("Invalid teams");
        rel == &TeamRelation::Enemy
    }

    pub fn is_own(self, t1: TeamId, t2: TeamId) -> bool {
        let rel = self.0.get(&(t1, t2)).expect("Invalid teams");
        rel == &TeamRelation::Same
    }
}
