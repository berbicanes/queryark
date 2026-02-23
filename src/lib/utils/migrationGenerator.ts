import type { DatabaseType } from '$lib/types/connection';
import type { TableDiffResult } from '$lib/types/diff';
import { quoteIdentifier } from '$lib/utils/sqlHelpers';

export function generateMigration(
  diff: TableDiffResult,
  targetSchema: string,
  targetTable: string,
  dbType: DatabaseType
): string {
  const lines: string[] = [];
  const qualifiedTable = `${quoteIdentifier(targetSchema, dbType)}.${quoteIdentifier(targetTable, dbType)}`;

  lines.push(`-- Migration generated for ${qualifiedTable}`);
  lines.push(`-- Source: ${diff.sourceTable} → Target: ${diff.targetTable}`);
  lines.push('');

  // Column additions
  for (const col of diff.columns) {
    if (col.status === 'added' && col.target) {
      const nullable = col.target.is_nullable ? '' : ' NOT NULL';
      const def = col.target.column_default ? ` DEFAULT ${col.target.column_default}` : '';
      lines.push(`ALTER TABLE ${qualifiedTable} ADD COLUMN ${quoteIdentifier(col.name, dbType)} ${col.target.data_type}${nullable}${def};`);
    }
  }

  // Column removals
  for (const col of diff.columns) {
    if (col.status === 'removed') {
      lines.push(`ALTER TABLE ${qualifiedTable} DROP COLUMN ${quoteIdentifier(col.name, dbType)};`);
    }
  }

  // Column modifications
  for (const col of diff.columns) {
    if (col.status === 'changed' && col.source && col.target) {
      if (col.source.data_type !== col.target.data_type) {
        switch (dbType) {
          case 'PostgreSQL':
          case 'CockroachDB':
          case 'Redshift':
            lines.push(`ALTER TABLE ${qualifiedTable} ALTER COLUMN ${quoteIdentifier(col.name, dbType)} TYPE ${col.target.data_type};`);
            break;
          case 'MySQL':
          case 'MariaDB': {
            const nullable = col.target.is_nullable ? '' : ' NOT NULL';
            const def = col.target.column_default ? ` DEFAULT ${col.target.column_default}` : '';
            lines.push(`ALTER TABLE ${qualifiedTable} MODIFY COLUMN ${quoteIdentifier(col.name, dbType)} ${col.target.data_type}${nullable}${def};`);
            break;
          }
          case 'MSSQL':
            lines.push(`ALTER TABLE ${qualifiedTable} ALTER COLUMN ${quoteIdentifier(col.name, dbType)} ${col.target.data_type};`);
            break;
          case 'SQLite':
            lines.push(`-- SQLite does not support ALTER COLUMN TYPE. Manual migration required for column "${col.name}".`);
            break;
          default:
            lines.push(`ALTER TABLE ${qualifiedTable} ALTER COLUMN ${quoteIdentifier(col.name, dbType)} TYPE ${col.target.data_type};`);
        }
      }
      if (col.source.is_nullable !== col.target.is_nullable) {
        if (dbType === 'MySQL' || dbType === 'MariaDB') {
          // Already handled by MODIFY COLUMN above if type also changed; handle standalone
          if (col.source.data_type === col.target.data_type) {
            const nullable = col.target.is_nullable ? '' : ' NOT NULL';
            const def = col.target.column_default ? ` DEFAULT ${col.target.column_default}` : '';
            lines.push(`ALTER TABLE ${qualifiedTable} MODIFY COLUMN ${quoteIdentifier(col.name, dbType)} ${col.target.data_type}${nullable}${def};`);
          }
        } else if (dbType === 'SQLite') {
          lines.push(`-- SQLite does not support ALTER COLUMN NULL constraint. Manual migration required for column "${col.name}".`);
        } else {
          if (col.target.is_nullable) {
            lines.push(`ALTER TABLE ${qualifiedTable} ALTER COLUMN ${quoteIdentifier(col.name, dbType)} DROP NOT NULL;`);
          } else {
            lines.push(`ALTER TABLE ${qualifiedTable} ALTER COLUMN ${quoteIdentifier(col.name, dbType)} SET NOT NULL;`);
          }
        }
      }
      if (col.source.column_default !== col.target.column_default) {
        if (dbType === 'SQLite') {
          lines.push(`-- SQLite does not support ALTER COLUMN DEFAULT. Manual migration required for column "${col.name}".`);
        } else if (col.target.column_default) {
          lines.push(`ALTER TABLE ${qualifiedTable} ALTER COLUMN ${quoteIdentifier(col.name, dbType)} SET DEFAULT ${col.target.column_default};`);
        } else {
          lines.push(`ALTER TABLE ${qualifiedTable} ALTER COLUMN ${quoteIdentifier(col.name, dbType)} DROP DEFAULT;`);
        }
      }
    }
  }

  // Index changes
  for (const idx of diff.indexes) {
    if (idx.status === 'removed' && idx.source) {
      if (dbType === 'MySQL' || dbType === 'MariaDB') {
        lines.push(`DROP INDEX ${quoteIdentifier(idx.name, dbType)} ON ${qualifiedTable};`);
      } else {
        lines.push(`DROP INDEX ${quoteIdentifier(idx.name, dbType)};`);
      }
    }
  }
  for (const idx of diff.indexes) {
    if ((idx.status === 'added' || idx.status === 'changed') && idx.target) {
      if (idx.status === 'changed' && idx.source) {
        if (dbType === 'MySQL' || dbType === 'MariaDB') {
          lines.push(`DROP INDEX ${quoteIdentifier(idx.name, dbType)} ON ${qualifiedTable};`);
        } else {
          lines.push(`DROP INDEX ${quoteIdentifier(idx.name, dbType)};`);
        }
      }
      const unique = idx.target.is_unique ? 'UNIQUE ' : '';
      const cols = idx.target.columns.map(c => quoteIdentifier(c, dbType)).join(', ');
      lines.push(`CREATE ${unique}INDEX ${quoteIdentifier(idx.name, dbType)} ON ${qualifiedTable} (${cols});`);
    }
  }

  // Foreign key changes
  for (const fk of diff.foreignKeys) {
    if (fk.status === 'removed' && fk.source) {
      lines.push(`ALTER TABLE ${qualifiedTable} DROP CONSTRAINT ${quoteIdentifier(fk.name, dbType)};`);
    }
  }
  for (const fk of diff.foreignKeys) {
    if ((fk.status === 'added' || fk.status === 'changed') && fk.target) {
      if (fk.status === 'changed' && fk.source) {
        lines.push(`ALTER TABLE ${qualifiedTable} DROP CONSTRAINT ${quoteIdentifier(fk.name, dbType)};`);
      }
      const cols = fk.target.columns.map(c => quoteIdentifier(c, dbType)).join(', ');
      const refTable = `${quoteIdentifier(fk.target.referenced_schema, dbType)}.${quoteIdentifier(fk.target.referenced_table, dbType)}`;
      const refCols = fk.target.referenced_columns.map(c => quoteIdentifier(c, dbType)).join(', ');
      let constraint = `ALTER TABLE ${qualifiedTable} ADD CONSTRAINT ${quoteIdentifier(fk.name, dbType)} FOREIGN KEY (${cols}) REFERENCES ${refTable} (${refCols})`;
      if (fk.target.on_update && fk.target.on_update !== 'NO ACTION') constraint += ` ON UPDATE ${fk.target.on_update}`;
      if (fk.target.on_delete && fk.target.on_delete !== 'NO ACTION') constraint += ` ON DELETE ${fk.target.on_delete}`;
      lines.push(`${constraint};`);
    }
  }

  if (lines.length <= 3) {
    lines.push('-- No changes detected');
  }

  return lines.join('\n');
}
