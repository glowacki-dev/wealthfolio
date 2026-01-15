//! Helper module for enriching symbol search results with ownership status.
//!
//! This module contains shared logic used by both Tauri commands and web API handlers
//! to combine external search results with local asset ownership information.

use std::collections::{HashMap, HashSet};

use super::market_data_model::{OwnershipStatus, QuoteSummary};
use crate::assets::assets_model::Asset;

/// Returns a priority value for sorting by ownership status.
/// Lower values = higher priority in search results.
fn ownership_priority(status: &OwnershipStatus) -> u8 {
    match status {
        OwnershipStatus::CurrentlyOwned => 0,
        OwnershipStatus::PreviouslyOwned => 1,
        OwnershipStatus::None => 2,
    }
}

/// Enriches external search results with local ownership status and adds owned assets.
///
/// # Arguments
/// * `results` - External search results from market data providers
/// * `owned_assets` - Local assets that match the search query
/// * `currently_owned` - Set of symbols currently held in any account
/// * `previously_owned` - Set of symbols previously held in any account
///
/// # Returns
/// Enriched and sorted search results with ownership status set
pub fn enrich_search_results_with_ownership(
    mut results: Vec<QuoteSummary>,
    owned_assets: Vec<Asset>,
    currently_owned: &HashMap<String, OwnershipStatus>,
    previously_owned: &HashSet<String>,
) -> Vec<QuoteSummary> {
    // Collect symbols already in external results
    let external_symbols: HashSet<String> =
        results.iter().map(|r| r.symbol.clone()).collect();

    // Process local assets
    for asset in owned_assets {
        let ownership = if currently_owned.contains_key(&asset.symbol) {
            OwnershipStatus::CurrentlyOwned
        } else if previously_owned.contains(&asset.symbol) {
            OwnershipStatus::PreviouslyOwned
        } else {
            OwnershipStatus::None
        };

        // Check if already in external results
        if external_symbols.contains(&asset.symbol) {
            // Update ownership status on existing result
            if let Some(r) = results.iter_mut().find(|r| r.symbol == asset.symbol) {
                r.ownership_status = ownership.clone();
            }
        } else {
            // Add new result for local asset
            let summary = QuoteSummary::from_asset(&asset, 1000.0, ownership);
            results.push(summary);
        }
    }

    // Sort by ownership priority, then by score
    results.sort_by(|a, b| {
        let pa = ownership_priority(&a.ownership_status);
        let pb = ownership_priority(&b.ownership_status);

        match (pa, pb) {
            (0, 0) | (1, 1) | (2, 2) => b.score.total_cmp(&a.score),
            _ => pa.cmp(&pb),
        }
    });

    results
}

/// Computes the set of previously owned symbols from all activities.
///
/// # Arguments
/// * `all_activities` - All activities from the activity service
///
/// # Returns
/// HashSet of asset IDs that were ever involved in activities across all accounts
pub fn get_previously_owned_from_activities(
    all_activities: &[crate::activities::activities_model::Activity],
) -> HashSet<String> {
    all_activities
        .iter()
        .map(|a| a.asset_id.clone())
        .collect()
}
