//! resources for identifying what plays belong to which team
//! TODO separate team/player ambiguity

use std::collections::HashMap;

use itertools::Itertools;

pub type TeamId = usize;
pub type PlayerId = usize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TeamRelation {
    Same,
    Allied,
    Enemy,
}

#[derive(Default, Debug)]
pub struct TeamsResource {
    /// a lookup from (team, team): relation
    pub team_relationship_lookup: HashMap<(TeamId, TeamId), TeamRelation>,
    /// stores what team a player is on
    pub player_team_lookup: HashMap<PlayerId, TeamId>
}


impl TeamsResource {
    pub fn is_foe(&self, p1: PlayerId, p2: PlayerId) -> bool {
        self.get_relation(p1, p2) == TeamRelation::Enemy
    }
    
    pub fn is_own(&self, p1: PlayerId, p2: PlayerId) -> bool {
        self.get_relation(p1, p2) == TeamRelation::Same
    }
    
    fn get_relation(&self, p1: PlayerId, p2: PlayerId) -> TeamRelation {
        let t1 = self.player_team_lookup.get(&p1).expect("Invalid player").clone();
        let t2 = self.player_team_lookup.get(&p2).expect("Invalid player").clone();
        self.team_relationship_lookup.get(&(t1, t2)).expect("Invalid teams").clone()
    }

    pub fn add_player(&mut self, p: PlayerId, t: TeamId) {
        self.player_team_lookup.entry(p).or_insert(t);
    }

    // TODO make foolproof and better
    pub fn free_for_all(&mut self) {
        for (t1, t2) in self.player_team_lookup.keys().cloned().tuple_combinations::<(TeamId, TeamId)>() {
            let rel = if t1 == t2 {
                TeamRelation::Same
            } else {
                TeamRelation::Enemy
            };
            self.team_relationship_lookup.insert((t1, t2), rel);
        }
    }
}
