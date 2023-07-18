use std::collections::HashMap;

use serde_json::{json, Value};

pub fn get_pix_keys() -> Value {
    let ret = json!({
        "query": r#"query customerKeys {
            viewer {
                name
                maskedTaxId
                savingsAccount {
                    id
                    dict {
                        keys(onlyActive: true) {
                            id
                            kind
                            value
                            formattedValue
                            itemDeepLink
                            badge
                        }
                    }
                }
            }
        }"#
    });
    ret
}

pub fn get_create_pix_qr_code(
    amount: f32,
    pix: String,
    account_id: String,
    transaction_id: String,
) -> HashMap<String, Value> {
    let query = format!(
        "mutation createPaymentRequest($createPaymentRequestInput: CreatePaymentRequestInput) {{
            createPaymentRequest(input: $createPaymentRequestInput) {{
              paymentRequest {{
                id
                amount
                message
                url
                transactionId
                pixAlias
                brcode
              }}
            }}
          }}",
    );

    let variables = json!({
        "createPaymentRequestInput": {
            "amount": amount,
            "pixAlias": pix,
            "savingsAccountId":account_id,
            "transactionId": transaction_id
        }
    });

    let mut payload = HashMap::new();
    payload.insert("query".to_string(), Value::String(query));
    payload.insert(
        "variables".to_string(),
        serde_json::to_value(variables).unwrap(),
    );
    payload
}

pub fn feed_items_query(cursor: &str) -> Value {
    let query = r#"query feed_items($cursor: String) {
        viewer {
            savingsAccount {
                feedItems(cursor: $cursor) {
                    pageInfo {
                        hasNextPage
                    }
                    edges {
                        cursor
                        node {
                            id
                            title
                            detail
                            postDate
                            displayDate
                            footer
                            strikethrough
                            showClock
                            iconKey
                            detailsDeeplink
                            tags
                            kind
                        }
                    }
                }
            }
        }
    }"#;

    json!({
        "query": query,
        "variables": {
            "cursor": cursor
        }
    })
}
