use sea_orm_migration::prelude::*;
// use sea_query::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Consider below link for sqlite datatype and rust type.
        // https://www.sea-ql.org/SeaORM/docs/generate-entity/entity-structure/
        manager
            .create_table(
                Table::create()
                    .table(MidTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MidTable::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MidTable::MeshId)
                            .integer()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(MidTable::Claimed).integer())
                    .col(ColumnDef::new(MidTable::FirstTimestamp).date_time())
                    .col(ColumnDef::new(MidTable::FirstIp).string())
                    .col(ColumnDef::new(MidTable::FirstMac).string())
                    .col(ColumnDef::new(MidTable::LastTimestamp).date_time())
                    .col(ColumnDef::new(MidTable::LastIp).string())
                    .col(ColumnDef::new(MidTable::LastMac).string())
                    .to_owned(),
            )
            .await
            .unwrap();

        manager
            .create_index(
                Index::create()
                    .table(MidTable::Table)
                    .col(MidTable::MeshId)
                    .name("mesh_id")
                    .to_owned(),
            )
            .await
            .unwrap();

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Simply drop the table.
        manager
            .drop_table(Table::drop().table(MidTable::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
/// Annotate You must see above link!!
#[derive(Iden)]
enum MidTable {
    Table,
    Id,
    MeshId,
    Claimed,
    FirstTimestamp,
    FirstIp,
    FirstMac,
    LastTimestamp,
    LastIp,
    LastMac,
}
