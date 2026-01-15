#[cfg(test)]
mod tests {
    use crate::activities::activities_model::Activity;
    use crate::assets::assets_model::Asset;
    use crate::market_data::market_data_model::{OwnershipStatus, QuoteSummary};
    use crate::market_data::search_ownership::{
        enrich_search_results_with_ownership, get_previously_owned_from_activities,
    };
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use std::collections::{HashMap, HashSet};

    fn create_test_asset(symbol: &str, name: &str) -> Asset {
        Asset {
            id: format!("id_{}", symbol),
            symbol: symbol.to_string(),
            name: Some(name.to_string()),
            isin: None,
            asset_type: Some("EQUITY".to_string()),
            symbol_mapping: None,
            asset_class: None,
            asset_sub_class: None,
            notes: None,
            countries: None,
            categories: None,
            classes: None,
            attributes: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            currency: "USD".to_string(),
            data_source: "MANUAL".to_string(),
            sectors: None,
            url: None,
        }
    }

    fn create_test_quote_summary(
        symbol: &str,
        score: f64,
        ownership_status: OwnershipStatus,
    ) -> QuoteSummary {
        QuoteSummary {
            symbol: symbol.to_string(),
            exchange: "NASDAQ".to_string(),
            short_name: symbol.to_string(),
            quote_type: "EQUITY".to_string(),
            index: "symbol".to_string(),
            score,
            type_display: "Equity".to_string(),
            long_name: symbol.to_string(),
            ownership_status,
        }
    }

    fn create_test_activity(account_id: &str, asset_id: &str) -> Activity {
        Activity {
            id: "activity_id".to_string(),
            account_id: account_id.to_string(),
            asset_id: asset_id.to_string(),
            activity_type: "BUY".to_string(),
            activity_date: DateTime::from_timestamp(0, 0).unwrap(),
            quantity: Decimal::from(10),
            unit_price: Decimal::from(100),
            currency: "USD".to_string(),
            fee: Decimal::ZERO,
            amount: Some(Decimal::from(1000)),
            is_draft: false,
            comment: None,
            created_at: DateTime::from_timestamp(0, 0).unwrap(),
            updated_at: DateTime::from_timestamp(0, 0).unwrap(),
        }
    }

    #[test]
    fn test_enrich_search_results_adds_local_assets_not_in_external_results() {
        // Arrange
        let external_results = vec![create_test_quote_summary("AAPL", 0.9, OwnershipStatus::None)];
        let owned_assets = vec![create_test_asset("TSLA", "Tesla Inc")];
        let currently_owned: HashMap<String, OwnershipStatus> =
            vec![("TSLA".to_string(), OwnershipStatus::CurrentlyOwned)]
                .into_iter()
                .collect();
        let previously_owned: HashSet<String> = HashSet::new();

        // Act
        let result = enrich_search_results_with_ownership(
            external_results,
            owned_assets,
            &currently_owned,
            &previously_owned,
        );

        // Assert
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].symbol, "TSLA");
        assert_eq!(result[0].ownership_status, OwnershipStatus::CurrentlyOwned);
        assert_eq!(result[1].symbol, "AAPL");
    }

    #[test]
    fn test_enrich_search_results_updates_existing_external_results() {
        // Arrange
        let external_results = vec![create_test_quote_summary("AAPL", 0.9, OwnershipStatus::None)];
        let owned_assets = vec![create_test_asset("AAPL", "Apple Inc")];
        let currently_owned: HashMap<String, OwnershipStatus> =
            vec![("AAPL".to_string(), OwnershipStatus::CurrentlyOwned)]
                .into_iter()
                .collect();
        let previously_owned: HashSet<String> = HashSet::new();

        // Act
        let result = enrich_search_results_with_ownership(
            external_results,
            owned_assets,
            &currently_owned,
            &previously_owned,
        );

        // Assert
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].symbol, "AAPL");
        assert_eq!(result[0].ownership_status, OwnershipStatus::CurrentlyOwned);
    }

    #[test]
    fn test_enrich_search_results_sorts_by_ownership_priority() {
        // Arrange
        let external_results = vec![
            create_test_quote_summary("MSFT", 0.8, OwnershipStatus::None),
            create_test_quote_summary("GOOGL", 0.85, OwnershipStatus::None),
        ];
        let owned_assets = vec![
            create_test_asset("AAPL", "Apple Inc"),
            create_test_asset("TSLA", "Tesla Inc"),
        ];
        let currently_owned: HashMap<String, OwnershipStatus> =
            vec![("AAPL".to_string(), OwnershipStatus::CurrentlyOwned)]
                .into_iter()
                .collect();
        let previously_owned: HashSet<String> =
            vec!["TSLA".to_string()].into_iter().collect();

        // Act
        let result = enrich_search_results_with_ownership(
            external_results,
            owned_assets,
            &currently_owned,
            &previously_owned,
        );

        // Assert - Currently owned should be first, then previously owned, then none
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].symbol, "AAPL");
        assert_eq!(result[0].ownership_status, OwnershipStatus::CurrentlyOwned);
        assert_eq!(result[1].symbol, "TSLA");
        assert_eq!(result[1].ownership_status, OwnershipStatus::PreviouslyOwned);
        assert_eq!(result[2].ownership_status, OwnershipStatus::None);
        assert_eq!(result[3].ownership_status, OwnershipStatus::None);
    }

    #[test]
    fn test_enrich_search_results_handles_empty_inputs() {
        // Arrange
        let external_results: Vec<QuoteSummary> = vec![];
        let owned_assets: Vec<Asset> = vec![];
        let currently_owned: HashMap<String, OwnershipStatus> = HashMap::new();
        let previously_owned: HashSet<String> = HashSet::new();

        // Act
        let result = enrich_search_results_with_ownership(
            external_results,
            owned_assets,
            &currently_owned,
            &previously_owned,
        );

        // Assert
        assert!(result.is_empty());
    }

    #[test]
    fn test_enrich_search_results_sorts_by_score_within_same_ownership_status() {
        // Arrange
        let external_results = vec![
            create_test_quote_summary("GOOGL", 0.85, OwnershipStatus::None),
            create_test_quote_summary("MSFT", 0.8, OwnershipStatus::None),
        ];
        let owned_assets: Vec<Asset> = vec![];
        let currently_owned: HashMap<String, OwnershipStatus> = HashMap::new();
        let previously_owned: HashSet<String> = HashSet::new();

        // Act
        let result = enrich_search_results_with_ownership(
            external_results,
            owned_assets,
            &currently_owned,
            &previously_owned,
        );

        // Assert - Within same ownership, higher score should come first
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].symbol, "GOOGL");
        assert_eq!(result[0].score, 0.85);
        assert_eq!(result[1].symbol, "MSFT");
        assert_eq!(result[1].score, 0.8);
    }

    #[test]
    fn test_get_previously_owned_from_activities_returns_all_assets() {
        // Arrange
        let activities = vec![
            create_test_activity("account1", "AAPL"),
            create_test_activity("account2", "TSLA"),
            create_test_activity("account1", "MSFT"),
        ];

        // Act
        let result = get_previously_owned_from_activities(&activities);

        // Assert - should return ALL assets from ALL accounts
        assert_eq!(result.len(), 3);
        assert!(result.contains("AAPL"));
        assert!(result.contains("TSLA"));
        assert!(result.contains("MSFT"));
    }

    #[test]
    fn test_get_previously_owned_from_activities_handles_empty_list() {
        // Arrange
        let activities: Vec<Activity> = vec![];

        // Act
        let result = get_previously_owned_from_activities(&activities);

        // Assert
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_previously_owned_from_activities_returns_unique_assets() {
        // Arrange
        let activities = vec![
            create_test_activity("account1", "AAPL"),
            create_test_activity("account2", "AAPL"), // Same asset, different account
            create_test_activity("account1", "TSLA"),
        ];

        // Act
        let result = get_previously_owned_from_activities(&activities);

        // Assert
        assert_eq!(result.len(), 2);
        assert!(result.contains("AAPL"));
        assert!(result.contains("TSLA"));
    }
}
