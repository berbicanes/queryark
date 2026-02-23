export interface ParameterInfo {
  name: string;
  style: 'positional' | 'named' | 'numbered';
  index: number;
  defaultValue: string;
}

/**
 * Detect query parameters ($1/$2, :name, ?) in SQL,
 * ignoring string literals, comments, and PostgreSQL :: type casts.
 */
export function detectParameters(sql: string): ParameterInfo[] {
  const params: ParameterInfo[] = [];
  const seen = new Set<string>();
  let i = 0;

  while (i < sql.length) {
    const ch = sql[i];

    // Skip single-quoted strings
    if (ch === "'") {
      i++;
      while (i < sql.length && sql[i] !== "'") {
        if (sql[i] === "'" && sql[i + 1] === "'") i += 2;
        else i++;
      }
      i++; // skip closing quote
      continue;
    }

    // Skip double-quoted identifiers
    if (ch === '"') {
      i++;
      while (i < sql.length && sql[i] !== '"') i++;
      i++;
      continue;
    }

    // Skip line comments
    if (ch === '-' && sql[i + 1] === '-') {
      while (i < sql.length && sql[i] !== '\n') i++;
      continue;
    }

    // Skip block comments
    if (ch === '/' && sql[i + 1] === '*') {
      i += 2;
      while (i < sql.length - 1 && !(sql[i] === '*' && sql[i + 1] === '/')) i++;
      i += 2;
      continue;
    }

    // PostgreSQL :: type cast — skip
    if (ch === ':' && sql[i + 1] === ':') {
      i += 2;
      // Skip the type name
      while (i < sql.length && /[a-zA-Z0-9_[\]]/.test(sql[i])) i++;
      continue;
    }

    // Numbered: $1, $2, ...
    if (ch === '$' && i + 1 < sql.length && /[0-9]/.test(sql[i + 1])) {
      let num = '';
      let j = i + 1;
      while (j < sql.length && /[0-9]/.test(sql[j])) {
        num += sql[j];
        j++;
      }
      const name = `$${num}`;
      if (!seen.has(name)) {
        seen.add(name);
        params.push({ name, style: 'numbered', index: params.length, defaultValue: '' });
      }
      i = j;
      continue;
    }

    // Named: :name (but not ::)
    if (ch === ':' && i + 1 < sql.length && /[a-zA-Z_]/.test(sql[i + 1])) {
      let name = '';
      let j = i + 1;
      while (j < sql.length && /[a-zA-Z0-9_]/.test(sql[j])) {
        name += sql[j];
        j++;
      }
      const fullName = `:${name}`;
      if (!seen.has(fullName)) {
        seen.add(fullName);
        params.push({ name: fullName, style: 'named', index: params.length, defaultValue: '' });
      }
      i = j;
      continue;
    }

    // Positional: ? (but not inside operator context like ?| ?&)
    if (ch === '?' && (i === 0 || !/[?|&]/.test(sql[i - 1])) && (i + 1 >= sql.length || !/[?|&]/.test(sql[i + 1]))) {
      // Count existing positional params
      const posCount = params.filter(p => p.style === 'positional').length;
      const name = `?${posCount + 1}`;
      params.push({ name, style: 'positional', index: params.length, defaultValue: '' });
      i++;
      continue;
    }

    i++;
  }

  return params;
}

/**
 * Replace parameter placeholders with literal values for execution.
 */
export function substituteParameters(sql: string, values: Record<string, string>): string {
  let result = '';
  let i = 0;
  let positionalIdx = 0;

  while (i < sql.length) {
    const ch = sql[i];

    // Skip single-quoted strings
    if (ch === "'") {
      let end = i + 1;
      while (end < sql.length && sql[end] !== "'") {
        if (sql[end] === "'" && sql[end + 1] === "'") end += 2;
        else end++;
      }
      result += sql.slice(i, end + 1);
      i = end + 1;
      continue;
    }

    // Skip double-quoted identifiers
    if (ch === '"') {
      let end = i + 1;
      while (end < sql.length && sql[end] !== '"') end++;
      result += sql.slice(i, end + 1);
      i = end + 1;
      continue;
    }

    // Skip line comments
    if (ch === '-' && sql[i + 1] === '-') {
      let end = i;
      while (end < sql.length && sql[end] !== '\n') end++;
      result += sql.slice(i, end);
      i = end;
      continue;
    }

    // Skip block comments
    if (ch === '/' && sql[i + 1] === '*') {
      let end = i + 2;
      while (end < sql.length - 1 && !(sql[end] === '*' && sql[end + 1] === '/')) end++;
      result += sql.slice(i, end + 2);
      i = end + 2;
      continue;
    }

    // PostgreSQL :: type cast — pass through
    if (ch === ':' && sql[i + 1] === ':') {
      result += '::';
      i += 2;
      while (i < sql.length && /[a-zA-Z0-9_[\]]/.test(sql[i])) {
        result += sql[i];
        i++;
      }
      continue;
    }

    // Numbered: $1, $2, ...
    if (ch === '$' && i + 1 < sql.length && /[0-9]/.test(sql[i + 1])) {
      let num = '';
      let j = i + 1;
      while (j < sql.length && /[0-9]/.test(sql[j])) {
        num += sql[j];
        j++;
      }
      const name = `$${num}`;
      const val = values[name];
      result += val !== undefined ? escapeValue(val) : name;
      i = j;
      continue;
    }

    // Named: :name
    if (ch === ':' && i + 1 < sql.length && /[a-zA-Z_]/.test(sql[i + 1])) {
      let name = '';
      let j = i + 1;
      while (j < sql.length && /[a-zA-Z0-9_]/.test(sql[j])) {
        name += sql[j];
        j++;
      }
      const fullName = `:${name}`;
      const val = values[fullName];
      result += val !== undefined ? escapeValue(val) : fullName;
      i = j;
      continue;
    }

    // Positional: ?
    if (ch === '?' && (i === 0 || !/[?|&]/.test(sql[i - 1])) && (i + 1 >= sql.length || !/[?|&]/.test(sql[i + 1]))) {
      positionalIdx++;
      const name = `?${positionalIdx}`;
      const val = values[name];
      result += val !== undefined ? escapeValue(val) : '?';
      i++;
      continue;
    }

    result += ch;
    i++;
  }

  return result;
}

/**
 * Escape a user-provided value for safe SQL interpolation.
 * Numbers pass through as-is; strings get single-quoted with escaped quotes.
 * NULL keyword passes through unquoted.
 */
function escapeValue(val: string): string {
  if (val.toUpperCase() === 'NULL') return 'NULL';
  if (/^-?(\d+\.?\d*|\.\d+)$/.test(val)) return val;
  return "'" + val.replace(/'/g, "''") + "'";
}
