use super::common::FGRoundBadge;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FGCompletedEpisodeDto {
    pub kudos: Option<isize>,
    pub fame: Option<isize>,
    pub crowns: Option<isize>,
    pub current_crown_shards: Option<isize>,

    pub rounds: Vec<FGCompletedEpisodeDtoRound>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FGCompletedEpisodeDtoRound {
    pub round_order: isize,
    pub round_id_str: String,
    pub round_display_name: String,
    pub qualified: bool,
    pub position: isize,
    pub team_score: isize,
    pub kudos: isize,
    pub fame: isize,
    pub bonus_tier: isize,
    pub bonus_kudos: isize,
    pub bonus_fame: isize,
    pub badge_id: FGRoundBadge,
}

pub fn generate_fg_completed_episode_dto_round() -> FGCompletedEpisodeDtoRound {
    FGCompletedEpisodeDtoRound {
        round_order: isize::default(),
        round_id_str: String::default(),
        round_display_name: String::default(),
        qualified: false,
        position: isize::default(),
        team_score: isize::default(),
        kudos: isize::default(),
        fame: isize::default(),
        bonus_tier: isize::default(),
        bonus_kudos: isize::default(),
        bonus_fame: isize::default(),
        badge_id: FGRoundBadge::Fail,
    }
}
