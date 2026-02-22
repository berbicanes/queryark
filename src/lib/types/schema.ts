// Generic models (all database types)
export interface ContainerInfo {
  name: string;
  container_type: string;
}

export interface ItemInfo {
  name: string;
  container: string;
  item_type: string;
  item_count: number | null;
}

export interface FieldInfo {
  name: string;
  data_type: string;
  is_nullable: boolean;
  is_primary: boolean;
  default_value: string | null;
  ordinal_position: number;
}

// SQL-specific models
export interface SchemaInfo {
  name: string;
}

export interface TableInfo {
  name: string;
  schema: string;
  table_type: string;
  row_count: number | null;
}

export interface ColumnInfo {
  name: string;
  data_type: string;
  is_nullable: boolean;
  column_default: string | null;
  is_primary_key: boolean;
  ordinal_position: number;
}

export interface IndexInfo {
  name: string;
  columns: string[];
  is_unique: boolean;
  is_primary: boolean;
  index_type: string;
}

export interface ForeignKeyInfo {
  name: string;
  columns: string[];
  referenced_table: string;
  referenced_schema: string;
  referenced_columns: string[];
  on_update: string;
  on_delete: string;
}
