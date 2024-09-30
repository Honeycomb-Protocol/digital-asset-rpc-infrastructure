use sea_orm::{EntityTrait, EnumIter, Related, RelationDef, RelationTrait};

use crate::dao::{asset, token_accounts};

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    AssetOfTokenAccount,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::AssetOfTokenAccount => token_accounts::Entity::belongs_to(asset::Entity)
                .from(token_accounts::Column::Mint)
                .to(asset::Column::Id)
                .into(),
        }
    }
}

impl Related<asset::Entity> for token_accounts::Entity {
    fn to() -> RelationDef {
        Relation::AssetOfTokenAccount.def()
    }
}

