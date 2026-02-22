<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorState, Compartment } from '@codemirror/state';
  import { EditorView, keymap, lineNumbers, highlightActiveLineGutter, highlightSpecialChars, drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightActiveLine, Decoration, type DecorationSet } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap, indentWithTab, toggleComment } from '@codemirror/commands';
  import { sql, PostgreSQL, MySQL, SQLite, MSSQL, Cassandra } from '@codemirror/lang-sql';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { autocompletion, completionKeymap, closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { lintKeymap } from '@codemirror/lint';
  import { syntaxHighlighting, defaultHighlightStyle, indentOnInput, bracketMatching, foldGutter, foldKeymap } from '@codemirror/language';
  import { StateField, StateEffect, RangeSet } from '@codemirror/state';
  import { format as formatSqlString } from 'sql-formatter';
  import type { DatabaseType } from '$lib/types/connection';

  let {
    value = $bindable(''),
    onexecute,
    dialect = 'PostgreSQL',
    schemaNamespace = {},
    errorRange = null,
  }: {
    value: string;
    onexecute?: () => void;
    dialect?: DatabaseType;
    schemaNamespace?: Record<string, readonly string[]>;
    errorRange?: { from: number; to: number } | null;
  } = $props();

  let editorContainer: HTMLDivElement;
  let view: EditorView;
  let isUpdatingFromProp = false;

  // Compartments for dynamic reconfiguration
  const sqlCompartment = new Compartment();
  const errorCompartment = new Compartment();

  const appTheme = EditorView.theme({
    '&': {
      height: '100%',
      fontSize: '13px',
      backgroundColor: 'var(--bg-primary)',
    },
    '.cm-content': {
      fontFamily: 'var(--font-mono)',
      caretColor: 'var(--accent)',
      padding: '8px 0',
    },
    '.cm-gutters': {
      backgroundColor: 'var(--bg-secondary)',
      color: 'var(--text-muted)',
      border: 'none',
      borderRight: '1px solid var(--border-color)',
    },
    '.cm-activeLineGutter': {
      backgroundColor: 'var(--bg-tertiary)',
    },
    '.cm-activeLine': {
      backgroundColor: 'rgba(69, 69, 90, 0.3)',
    },
    '.cm-cursor': {
      borderLeftColor: 'var(--accent)',
      borderLeftWidth: '2px',
    },
    '.cm-selectionBackground': {
      backgroundColor: 'rgba(122, 162, 247, 0.2) !important',
    },
    '&.cm-focused .cm-selectionBackground': {
      backgroundColor: 'rgba(122, 162, 247, 0.3) !important',
    },
    '.cm-matchingBracket': {
      backgroundColor: 'rgba(122, 162, 247, 0.25)',
      outline: '1px solid rgba(122, 162, 247, 0.5)',
    },
    '.cm-scroller': {
      overflow: 'auto',
    },
    '.cm-tooltip': {
      backgroundColor: 'var(--bg-secondary)',
      border: '1px solid var(--border-color)',
    },
    '.cm-tooltip-autocomplete': {
      backgroundColor: 'var(--bg-secondary)',
    },
    '.cm-tooltip.cm-tooltip-autocomplete > ul > li': {
      color: 'var(--text-primary)',
    },
    '.cm-tooltip.cm-tooltip-autocomplete > ul > li[aria-selected]': {
      backgroundColor: 'var(--bg-active)',
      color: 'var(--text-primary)',
    },
    '.cm-panels': {
      backgroundColor: 'var(--bg-secondary)',
      color: 'var(--text-primary)',
    },
    '.cm-panel input': {
      backgroundColor: 'var(--bg-primary)',
      color: 'var(--text-primary)',
      border: '1px solid var(--border-color)',
    },
    '.cm-panel button': {
      backgroundColor: 'var(--bg-tertiary)',
      color: 'var(--text-primary)',
      border: '1px solid var(--border-color)',
    },
    '.cm-searchMatch': {
      backgroundColor: 'rgba(249, 226, 175, 0.2)',
      outline: '1px solid rgba(249, 226, 175, 0.4)',
    },
    '.cm-searchMatch.cm-searchMatch-selected': {
      backgroundColor: 'rgba(249, 226, 175, 0.4)',
    },
    '.cm-foldGutter span': {
      color: 'var(--text-muted)',
    },
  }, { dark: true });

  // Error highlight decoration
  const errorMark = Decoration.mark({ class: 'cm-error-highlight' });

  // State effect to set/clear error decorations
  const setErrorEffect = StateEffect.define<{ from: number; to: number } | null>();

  const errorField = StateField.define<DecorationSet>({
    create() {
      return Decoration.none;
    },
    update(decorations, tr) {
      for (const effect of tr.effects) {
        if (effect.is(setErrorEffect)) {
          if (effect.value) {
            const { from, to } = effect.value;
            if (from >= 0 && to <= tr.state.doc.length && from < to) {
              return RangeSet.of([errorMark.range(from, to)]);
            }
          }
          return Decoration.none;
        }
      }
      // Clear decorations on document changes
      if (tr.docChanged) return Decoration.none;
      return decorations;
    },
    provide: f => EditorView.decorations.from(f),
  });

  function getSqlDialect() {
    switch (dialect) {
      case 'MySQL':
      case 'MariaDB':
        return MySQL;
      case 'SQLite':
        return SQLite;
      case 'MSSQL':
        return MSSQL;
      case 'Cassandra':
      case 'ScyllaDB':
        return Cassandra;
      default:
        return PostgreSQL;
    }
  }

  function getSqlLanguage(schema?: Record<string, readonly string[]>) {
    return sql({ dialect: getSqlDialect(), schema });
  }

  function getFormatterLanguage(): string {
    switch (dialect) {
      case 'MySQL':
      case 'MariaDB':
        return 'mysql';
      case 'SQLite':
        return 'sqlite';
      case 'MSSQL':
        return 'tsql';
      case 'PostgreSQL':
      case 'CockroachDB':
      case 'Redshift':
        return 'postgresql';
      default:
        return 'sql';
    }
  }

  function formatSql() {
    if (!view) return;
    const doc = view.state.doc.toString();
    if (!doc.trim()) return;
    try {
      const formatted = formatSqlString(doc, {
        language: getFormatterLanguage() as any,
        tabWidth: 2,
        keywordCase: 'upper',
      });
      isUpdatingFromProp = true;
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: formatted },
      });
      value = formatted;
      isUpdatingFromProp = false;
    } catch {
      // Formatting failed — leave content unchanged
    }
  }

  const executeKeymap = keymap.of([
    {
      key: 'Ctrl-Enter',
      mac: 'Cmd-Enter',
      run: () => {
        onexecute?.();
        return true;
      }
    },
    {
      key: 'Ctrl-/',
      mac: 'Cmd-/',
      run: toggleComment
    },
    {
      key: 'Ctrl-Shift-f',
      mac: 'Cmd-Shift-f',
      run: () => {
        formatSql();
        return true;
      }
    }
  ]);

  onMount(() => {
    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged && !isUpdatingFromProp) {
        value = update.state.doc.toString();
      }
    });

    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightSpecialChars(),
        history(),
        foldGutter(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        indentOnInput(),
        bracketMatching(),
        closeBrackets(),
        autocompletion(),
        rectangularSelection(),
        crosshairCursor(),
        highlightActiveLine(),
        highlightSelectionMatches(),
        executeKeymap,
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...searchKeymap,
          ...historyKeymap,
          ...foldKeymap,
          ...completionKeymap,
          ...lintKeymap,
          indentWithTab,
        ]),
        sqlCompartment.of(getSqlLanguage(schemaNamespace)),
        errorField,
        oneDark,
        appTheme,
        updateListener,
      ],
    });

    view = new EditorView({
      state,
      parent: editorContainer,
    });
  });

  // Hot-swap SQL language when schema namespace changes
  $effect(() => {
    const ns = schemaNamespace;
    if (view) {
      view.dispatch({
        effects: sqlCompartment.reconfigure(getSqlLanguage(ns)),
      });
    }
  });

  // Update error highlighting when errorRange changes
  $effect(() => {
    const range = errorRange;
    if (view) {
      view.dispatch({
        effects: setErrorEffect.of(range),
      });
    }
  });

  // Sync prop changes to editor
  $effect(() => {
    if (view && value !== view.state.doc.toString()) {
      isUpdatingFromProp = true;
      view.dispatch({
        changes: {
          from: 0,
          to: view.state.doc.length,
          insert: value,
        },
      });
      isUpdatingFromProp = false;
    }
  });

  onDestroy(() => {
    if (view) {
      view.destroy();
    }
  });
</script>

<div bind:this={editorContainer} class="sql-editor"></div>

<style>
  .sql-editor {
    height: 100%;
    overflow: hidden;
  }

  .sql-editor :global(.cm-editor) {
    height: 100%;
  }

  .sql-editor :global(.cm-scroller) {
    font-family: var(--font-mono);
  }

  .sql-editor :global(.cm-error-highlight) {
    text-decoration: underline wavy var(--error);
    text-underline-offset: 3px;
    background-color: rgba(243, 139, 168, 0.1);
  }
</style>
