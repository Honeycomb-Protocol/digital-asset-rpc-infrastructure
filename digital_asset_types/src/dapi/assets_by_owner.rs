use crate::dao::scopes;
use crate::dao::PageOptions;
use crate::feature_flag::FeatureFlags;
use crate::rpc::filter::AssetSorting;
use crate::rpc::options::Options;
use crate::rpc::response::AssetList;
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;

use super::common::{build_asset_response, create_pagination, create_sorting};

pub async fn get_assets_by_owner(
    db: &DatabaseConnection,
    owner_address: Vec<u8>,
    sort_by: AssetSorting,
    page_options: &PageOptions,
    feature_flags: &FeatureFlags,
    options: &Options,
) -> Result<AssetList, DbErr> {
    let pagination = create_pagination(&page_options)?;
    let (sort_direction, sort_column) = create_sorting(sort_by);

    let enable_grand_total_query =
        feature_flags.enable_grand_total_query && options.show_grand_total;

    let (assets, grand_total) = scopes::asset::get_assets_by_owner(
        db,
        owner_address,
        sort_column,
        sort_direction,
        &pagination,
        page_options.limit,
        enable_grand_total_query,
        options.show_unverified_collections,
    )
    .await?;
    Ok(build_asset_response(
        assets,
        page_options.limit,
        grand_total,
        &pagination,
        options,
    ))
}
