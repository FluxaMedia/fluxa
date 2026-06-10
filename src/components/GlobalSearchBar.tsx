import { useEffect, useRef, useState } from 'react';
import { MagnifyingGlass as SearchIcon, X } from '@phosphor-icons/react';
import { t } from '../i18n';

interface Props {
  query: string;
  onSearch: (query: string) => void;
  onBack?: () => void;
}

export function GlobalSearchBar({ query, onSearch, onBack }: Props) {
  const [expanded, setExpanded] = useState(false);
  const [inputValue, setInputValue] = useState('');
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setInputValue(query);
  }, [query]);

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

  const handleKey = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && inputValue.trim().length >= 1) {
      onSearch(inputValue.trim());
      inputRef.current?.blur();
    }
    if (e.key === 'Escape') {
      if (inputValue) {
        setInputValue('');
        onSearch('');
      } else {
        close();
      }
    }
  };

  if (!expanded) {
    return (
      <button
        onClick={open}
        title={t('auto.search')}
        style={{
          width: 42,
          height: 42,
          borderRadius: '50%',
          background: 'rgba(10,12,20,0.88)',
          border: '1px solid rgba(255,255,255,0.12)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          cursor: 'pointer',
          boxShadow: '0 4px 20px rgba(0,0,0,0.3)',
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
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 10,
        width: 360,
        height: 42,
        background: 'rgba(10,12,20,0.97)',
        border: '1px solid rgba(255,255,255,0.25)',
        borderRadius: 999,
        padding: '0 16px',
        boxShadow: '0 8px 32px rgba(0,0,0,0.4), 0 0 0 1px rgba(232,93,63,0.15)',
        pointerEvents: 'auto',
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
        onBlur={() => { if (!inputValue) close(); }}
        style={{
          flex: 1,
          background: 'transparent',
          border: 'none',
          outline: 'none',
          color: '#FFFFFF',
          fontSize: 15,
          fontWeight: 500,
          fontFamily: 'sans-serif',
        }}
      />
      <button
        style={{ background: 'transparent', border: 'none', cursor: 'pointer', padding: 0, display: 'flex', alignItems: 'center', color: 'rgba(255,255,255,0.4)', flexShrink: 0 }}
        onMouseDown={(e) => { e.preventDefault(); close(); }}
      >
        <X size={17} />
      </button>
    </div>
  );
}
