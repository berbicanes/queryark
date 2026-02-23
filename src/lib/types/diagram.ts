export interface DiagramTable {
  id: string;              // "schema.tableName"
  schema: string;
  name: string;
  x: number;
  y: number;
  columns: DiagramColumn[];
}

export interface DiagramColumn {
  name: string;
  dataType: string;
  isPK: boolean;
  isFK: boolean;
  isNullable: boolean;
}

export interface DiagramRelationship {
  id: string;
  sourceTable: string;
  sourceColumns: string[];
  targetTable: string;
  targetColumns: string[];
  fkName: string;
  onDelete: string;
  onUpdate: string;
}
