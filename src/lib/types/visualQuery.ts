export interface VQTable {
  id: string;
  schema: string;
  name: string;
  alias: string;
  x: number;
  y: number;
  columns: VQColumn[];
  selectedColumns: string[];
}

export interface VQColumn {
  name: string;
  dataType: string;
  isPK: boolean;
  isFK: boolean;
}

export type JoinType = 'INNER JOIN' | 'LEFT JOIN' | 'RIGHT JOIN' | 'FULL JOIN' | 'CROSS JOIN';

export interface VQJoin {
  id: string;
  sourceTableId: string;
  sourceColumn: string;
  targetTableId: string;
  targetColumn: string;
  joinType: JoinType;
}

export interface VQWhereClause {
  id: string;
  tableId: string;
  column: string;
  operator: string;
  value: string;
  connector: 'AND' | 'OR';
}

export interface VQOrderBy {
  tableId: string;
  column: string;
  direction: 'ASC' | 'DESC';
}

export interface VQGroupBy {
  tableId: string;
  column: string;
}

export interface VQState {
  tables: VQTable[];
  joins: VQJoin[];
  where: VQWhereClause[];
  orderBy: VQOrderBy[];
  groupBy: VQGroupBy[];
  distinct: boolean;
  limit: number | null;
}
