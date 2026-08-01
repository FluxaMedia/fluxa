import { useEffect, useMemo, useRef, useState } from 'react';
import { Search as SearchIcon, X, Clock } from 'lucide-react';
import { t, getLanguage } from '../i18n';
import { addRecentSearch, loadRecentSearches, clearRecentSearches, removeRecentSearch, type RecentSearch } from '../core/searchHistory';
import { setSearchPartialHandler } from '../core/catalogEffects';
import { appPrefs, prefBool } from '../core/appPrefs';
import { coreInvoke } from '../core/engine';
import type { AppState, Meta } from '../core/types';

interface Props {
  query: string;
  onSearch: (query: string) => void;
  onBack?: () => void;
  focusSignal?: number;
  state: Pick<AppState, 'home' | 'search' | 'settings'>;
  onDispatch: (actionJson: string) => void;
  onNavigateDetail: (meta: Meta) => void;
}

const SUGGESTION_DEBOUNCE_MS = 200;
const MAX_SUGGESTIONS = 6;

export function GlobalSearchBar({ query, onSearch, onBack, focusSignal, state, onDispatch, onNavigateDetail }: Props) {
  const [expanded, setExpanded] = useState(false);
  const [inputValue, setInputValue] = useState('');
  const [recentSearches, setRecentSearches] = useState<RecentSearch[]>([]);
  const [focused, setFocused] = useState(false);
  const [activeIndex, setActiveIndex] = useState(-1);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);
  const [partialResults, setPartialResults] = useState<Meta[]>([]);
  const partialQueryRef = useRef('');

  useEffect(() => {
    setInputValue(query);
  }, [query]);

  useEffect(() => {
    if (focusSignal) open();
  }, [focusSignal]);

  useEffect(() => {
    if (!expanded) return;
    loadRecentSearches().then(setRecentSearches);
  }, [expanded]);

  useEffect(() => {
    const trimmed = inputValue.trim();
    if (trimmed.length < 2) return;
    clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      partialQueryRef.current = trimmed;
      setPartialResults([]);
      onDispatch(JSON.stringify({ type: 'searchRequested', query: trimmed, language: getLanguage() }));
    }, SUGGESTION_DEBOUNCE_MS);
    return () => clearTimeout(debounceRef.current);
  }, [inputValue, onDispatch]);

  useEffect(() => {
    setSearchPartialHandler((q, items) => {
      if (q !== partialQueryRef.current) return;
      setPartialResults((current) => {
        const seen = new Set(current.map((meta) => meta.id));
        const added = (items as Meta[]).filter((meta) => !seen.has(meta.id));
        return added.length > 0 ? [...current, ...added] : current;
      });
    });
    return () => setSearchPartialHandler(null);
  }, []);

  const [localSuggestions, setLocalSuggestions] = useState<Meta[]>([]);
  useEffect(() => {
    let active = true;
    const needle = inputValue.trim().toLowerCase();
    void coreInvoke<Meta[]>('searchSuggestionsPlan', JSON.stringify({ categories: state.home.categories, needle, limit: MAX_SUGGESTIONS }))
      .then((items) => { if (active) setLocalSuggestions(items ?? []); });
    return () => { active = false; };
  }, [state.home.categories, inputValue]);

  const [networkSuggestions, setNetworkSuggestions] = useState<Meta[]>([]);
  useEffect(() => {
    let active = true;
    const trimmed = inputValue.trim();
    const needle = trimmed.toLowerCase();
    if (needle.length < 2) {
      setNetworkSuggestions([]);
      return () => { active = false; };
    }
    let request: Record<string, unknown> | null = null;
    if (partialQueryRef.current === trimmed && partialResults.length > 0) {
      request = { items: partialResults, needle, limit: MAX_SUGGESTIONS };
    } else if ((state.search.query ?? '').trim().toLowerCase() === needle) {
      request = { categories: state.search.categories, needle, limit: MAX_SUGGESTIONS };
    }
    if (!request) {
      setNetworkSuggestions([]);
      return () => { active = false; };
    }
    void coreInvoke<Meta[]>('searchSuggestionsPlan', JSON.stringify(request))
      .then((items) => { if (active) setNetworkSuggestions(items ?? []); });
    return () => { active = false; };
  }, [partialResults, state.search.categories, state.search.query, inputValue]);

  const suggestions = networkSuggestions.length > 0 ? networkSuggestions : localSuggestions;
  const showDropdown = focused && (recentSearches.length > 0 || suggestions.length > 0);
  const showingRecent = !inputValue.trim();
  const listItems = showingRecent ? recentSearches : (inputValue.trim().length >= 2 ? suggestions : []);

  useEffect(() => {
    setActiveIndex(-1);
  }, [inputValue, expanded, showingRecent, suggestions.length]);

  const open = () => {
    setExpanded(true);
    setTimeout(() => inputRef.current?.focus(), 30);
  };

  const close = () => {
    setExpanded(false);
    setInputValue('');
    onSearch('');
    onBack?.();
  };

  const saveRecentSearch = (value: string, meta?: Meta) => {
    void addRecentSearch(value, recentSearches, meta).then(setRecentSearches);
  };

  const submit = (value: string) => {
    const trimmed = value.trim();
    if (!trimmed) return;
    saveRecentSearch(trimmed);
    onSearch(trimmed);
    inputRef.current?.blur();
  };

  const clearOrClose = () => {
    if (inputValue) {
      setInputValue('');
      onSearch('');
      inputRef.current?.focus();
    } else {
      close();
    }
  };

  const handleKey = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown' && listItems.length > 0) {
      e.preventDefault();
      setActiveIndex((i) => Math.min(i + 1, listItems.length - 1));
      return;
    }
    if (e.key === 'ArrowUp' && listItems.length > 0) {
      e.preventDefault();
      setActiveIndex((i) => Math.max(i - 1, 0));
      return;
    }
    if (e.key === 'Enter') {
      if (activeIndex >= 0 && activeIndex < listItems.length) {
        if (showingRecent) {
          handleRecentClick(recentSearches[activeIndex]);
        } else {
          handleSuggestionClick(suggestions[activeIndex]);
        }
      } else if (inputValue.trim().length >= 1) {
        submit(inputValue);
      }
      return;
    }
    if (e.key === 'Escape') {
      clearOrClose();
    }
  };

  const handleSuggestionClick = (meta: Meta) => {
    const openDetail = prefBool(appPrefs(state), 'searchSuggestionsOpenDetail', false);
    if (openDetail) {
      saveRecentSearch(meta.name, meta);
      setExpanded(false);
      setInputValue('');
      onNavigateDetail(meta);
      return;
    }
    submit(meta.name);
  };

  const handleRecentClick = (recent: RecentSearch) => {
    const openDetail = prefBool(appPrefs(state), 'searchSuggestionsOpenDetail', false);
    if (openDetail && recent.meta) {
      setExpanded(false);
      setInputValue('');
      onNavigateDetail(recent.meta);
      return;
    }
    setExpanded(false);
    setInputValue(recent.query);
    saveRecentSearch(recent.query);
    onSearch(recent.query);
  };

  const handleClearHistory = () => {
    void clearRecentSearches().then(setRecentSearches);
  };

  const handleRemoveRecent = (value: string) => {
    void removeRecentSearch(value, recentSearches).then(setRecentSearches);
  };

  if (!expanded) {
    return (
      <button
        onClick={open}
        title={t('auto.search')}
        style={{
          width: '2.625rem',
          height: '2.625rem',
          borderRadius: '50%',
          background: 'rgba(10,12,20,0.88)',
          border: '1px solid rgba(255,255,255,0.12)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          cursor: 'pointer',
          boxShadow: '0 0.25rem 1.25rem rgba(0,0,0,0.3)',
          pointerEvents: 'auto',
          padding: 0,
          transition: 'background 0.15s, border-color 0.15s',
          flexShrink: 0,
        }}
        onMouseEnter={(e) => {
          (e.currentTarget as HTMLButtonElement).style.background = 'rgba(10,12,20,0.97)';
          (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(255,255,255,0.22)';
        }}
        onMouseLeave={(e) => {
          (e.currentTarget as HTMLButtonElement).style.background = 'rgba(10,12,20,0.88)';
          (e.currentTarget as HTMLButtonElement).style.borderColor = 'rgba(255,255,255,0.12)';
        }}
      >
        <SearchIcon size={18} color="rgba(255,255,255,0.7)" />
      </button>
    );
  }

  return (
    <div style={{ position: 'relative', width: '22.5rem', pointerEvents: 'auto' }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.625rem',
          width: '100%',
          height: '2.625rem',
          background: 'rgba(10,12,20,0.97)',
          border: '1px solid rgba(255,255,255,0.25)',
          borderRadius: '62.4375rem',
          padding: '0 1rem',
          boxShadow: '0 0.5rem 2rem rgba(0,0,0,0.4), 0 0 0 1px rgba(232,93,63,0.15)',
        }}
      >
        <SearchIcon size={18} color="rgba(255,255,255,0.48)" style={{ flexShrink: 0 }} />
        <input
          ref={inputRef}
          type="text"
          value={inputValue}
          placeholder={t('search.placeholder_expanded')}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={handleKey}
          onFocus={() => setFocused(true)}
          onBlur={() => { setFocused(false); if (!inputValue) close(); }}
          style={{
            flex: 1,
            background: 'transparent',
            border: 'none',
            outline: 'none',
            color: '#FFFFFF',
            fontSize: '0.9375rem',
            fontWeight: 500,
          }}
        />
        <button
          style={{ background: 'transparent', border: 'none', cursor: 'pointer', padding: 0, display: 'flex', alignItems: 'center', color: 'rgba(255,255,255,0.4)', flexShrink: 0 }}
          onMouseDown={(e) => { e.preventDefault(); clearOrClose(); }}
        >
          <X size={17} />
        </button>
      </div>

      {showDropdown && (
        <div
          style={{
            position: 'absolute',
            top: 'calc(100% + 0.5rem)',
            left: 0,
            right: 0,
            background: 'rgba(10,12,20,0.98)',
            border: '1px solid rgba(255,255,255,0.14)',
            borderRadius: '1rem',
            boxShadow: '0 0.75rem 2.5rem rgba(0,0,0,0.55)',
            padding: '0.75rem',
            maxHeight: '26rem',
            overflowY: 'auto',
          }}
        >
          {!inputValue.trim() && recentSearches.length > 0 && (
            <>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0.25rem 0.5rem 0.5rem' }}>
                <span style={dropdownStyles.sectionLabel}>{t('search.recent_searches')}</span>
                <button style={dropdownStyles.clearBtn} onMouseDown={(e) => { e.preventDefault(); handleClearHistory(); }}>
                  {t('search.clear_history')}
                </button>
              </div>
              {recentSearches.map((item, index) => (
                <div
                  key={item.query}
                  style={{ ...dropdownStyles.row, padding: '0 0.25rem 0 0.5rem', background: activeIndex === index ? 'rgba(255,255,255,0.06)' : 'transparent' }}
                  onMouseEnter={(e) => { setActiveIndex(index); (e.currentTarget as HTMLDivElement).style.background = 'rgba(255,255,255,0.06)'; }}
                  onMouseLeave={(e) => { (e.currentTarget as HTMLDivElement).style.background = 'transparent'; }}
                >
                  <button
                    style={dropdownStyles.rowMain}
                    onMouseDown={(e) => { e.preventDefault(); handleRecentClick(item); }}
                  >
                    <Clock size={15} color="rgba(255,255,255,0.4)" style={{ flexShrink: 0 }} />
                    <span style={dropdownStyles.rowText}>{item.query}</span>
                  </button>
                  <button
                    title={t('common.remove')}
                    style={dropdownStyles.rowRemove}
                    onMouseDown={(e) => { e.preventDefault(); e.stopPropagation(); handleRemoveRecent(item.query); }}
                  >
                    <X size={13} />
                  </button>
                </div>
              ))}
            </>
          )}

          {inputValue.trim().length >= 2 && suggestions.length > 0 && (
            <>
              <div style={{ padding: '0.25rem 0.5rem 0.5rem' }}>
                <span style={dropdownStyles.sectionLabel}>{t('search.suggestions')}</span>
              </div>
              {suggestions.map((meta, index) => (
                <button
                  key={meta.id}
                  style={{ ...dropdownStyles.row, background: activeIndex === index ? 'rgba(255,255,255,0.06)' : 'transparent' }}
                  onMouseDown={(e) => { e.preventDefault(); handleSuggestionClick(meta); }}
                  onMouseEnter={(e) => { setActiveIndex(index); (e.currentTarget as HTMLButtonElement).style.background = 'rgba(255,255,255,0.06)'; }}
                  onMouseLeave={(e) => { (e.currentTarget as HTMLButtonElement).style.background = 'transparent'; }}
                >
                  <SearchIcon size={15} color="rgba(255,255,255,0.4)" style={{ flexShrink: 0 }} />
                  <span style={dropdownStyles.rowText}>{meta.name}</span>
                </button>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  );
}

const dropdownStyles: Record<string, React.CSSProperties> = {
  sectionLabel: {
    color: 'rgba(255,255,255,0.5)',
    fontSize: '0.75rem',
    fontWeight: 700,
    textTransform: 'uppercase',
    letterSpacing: '0.0313rem',
  },
  clearBtn: {
    background: 'transparent',
    border: 'none',
    color: 'rgba(255,255,255,0.5)',
    fontSize: '0.75rem',
    fontWeight: 700,
    cursor: 'pointer',
    padding: 0,
  },
  row: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.625rem',
    width: '100%',
    height: '2.375rem',
    padding: '0 0.5rem',
    borderRadius: '0.625rem',
    border: 'none',
    background: 'transparent',
    cursor: 'pointer',
    transition: 'background 0.12s',
  },
  rowText: {
    color: '#FFFFFF',
    fontSize: '0.875rem',
    fontWeight: 600,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  rowMain: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.625rem',
    flex: 1,
    minWidth: 0,
    height: '100%',
    border: 'none',
    background: 'transparent',
    cursor: 'pointer',
    padding: 0,
  },
  rowRemove: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
    width: '1.625rem',
    height: '1.625rem',
    borderRadius: '0.375rem',
    border: 'none',
    background: 'transparent',
    color: 'rgba(255,255,255,0.4)',
    cursor: 'pointer',
    padding: 0,
  },
};
