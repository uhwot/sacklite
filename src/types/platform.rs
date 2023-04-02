use strum_macros::IntoStaticStr;

use super::npticket::{NpTicket, KeyId};

#[derive(Debug, IntoStaticStr)]
pub enum LinkedUserId {
    PSN(u64),
    RPCN(u64),
}

impl LinkedUserId {
    pub fn from_npticket(npticket: &NpTicket) -> Self {
        let user_id = npticket.body.user_id;
        match npticket.footer.key_id {
            KeyId::PSN => Self::PSN(user_id),
            KeyId::RPCN => Self::RPCN(user_id),
        }
    }

    pub fn id(&self) -> u64 {
        match self {
            Self::PSN(id) => *id,
            Self::RPCN(id) => *id,
        }
    }

    pub fn to_string(&self) -> String {
        let platform_str: &str = self.into();
        format!("{}:{}", platform_str, self.id())
    }
}