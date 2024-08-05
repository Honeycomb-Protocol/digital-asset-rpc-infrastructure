use sea_orm_migration::prelude::*;

use crate::model::table::MerkleTree;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MerkleTree::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("canopy_depth"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MerkleTree::Table)
                    .drop_column(Alias::new("canopy_depth"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
