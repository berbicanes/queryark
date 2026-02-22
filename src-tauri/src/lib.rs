mod commands;
mod db;
mod error;
mod models;

use db::pool::PoolManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(PoolManager::new())
        .invoke_handler(tauri::generate_handler![
            // Connection management
            commands::connection::connect_db,
            commands::connection::disconnect_db,
            commands::connection::test_connection,
            commands::connection::ping_connection,
            // Query execution
            commands::query::execute_query,
            // Generic schema browsing (all databases)
            commands::schema::get_database_category,
            commands::schema::get_containers,
            commands::schema::get_items,
            commands::schema::get_item_fields,
            commands::schema::get_item_data,
            commands::schema::get_item_count,
            // SQL-specific schema (relational + analytics + CQL)
            commands::schema::get_schemas,
            commands::schema::get_tables,
            commands::schema::get_columns,
            commands::schema::get_indexes,
            commands::schema::get_foreign_keys,
            commands::schema::get_table_data,
            commands::schema::get_row_count,
            commands::schema::update_cell,
            commands::schema::insert_row,
            commands::schema::delete_rows,
            commands::schema::get_table_stats,
            commands::schema::get_routines,
            commands::schema::get_sequences,
            commands::schema::get_enums,
            // Document operations (MongoDB, DynamoDB)
            commands::document::insert_document,
            commands::document::update_document,
            commands::document::delete_documents,
            // Key-value operations (Redis)
            commands::keyvalue::get_value,
            commands::keyvalue::set_value,
            commands::keyvalue::delete_keys,
            commands::keyvalue::get_key_type,
            commands::keyvalue::scan_keys,
            // Graph operations (Neo4j)
            commands::graph::get_labels,
            commands::graph::get_relationship_types,
            commands::graph::get_node_properties,
            commands::graph::get_nodes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
