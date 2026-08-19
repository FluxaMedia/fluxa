import { useState } from 'react';
import { t } from '../i18n';
import { contrastOn, UtilButton } from './CollectionEditorPrimitives';

interface Props {
  accent: string;
  onDismiss: () => void;
  onImport: (json: string) => void;
}

export function ImportDialog({ accent, onDismiss, onImport }: Props) {
  const [json, setJson] = useState('');

  async function handlePasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      setJson(text);
    } catch {}
  }

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 100,
        background: 'rgba(0,0,0,0.75)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '1.5rem',
      }}
      onClick={(e) => e.target === e.currentTarget && onDismiss()}
    >
      <div
        style={{
          background: '#111620',
          borderRadius: '0.75rem',
          padding: '1.5rem',
          width: '100%',
          maxWidth: '30rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.875rem',
          boxSizing: 'border-box',
        }}
      >
        <span style={{ color: '#fff', fontSize: '1.125rem', fontWeight: 900 }}>{t('library.import_collections')}</span>
        <textarea
          value={json}
          onChange={(e) => setJson(e.target.value)}
          placeholder={t('library.paste_json')}
          style={{
            width: '100%',
            minHeight: '10rem',
            maxHeight: '22.5rem',
            padding: '0.75rem',
            background: 'rgba(255,255,255,0.05)',
            border: `1px solid ${accent}55`,
            borderRadius: '0.5rem',
            color: '#fff',
            fontSize: '0.8125rem',
            fontFamily: 'monospace',
            resize: 'vertical',
            outline: 'none',
            boxSizing: 'border-box',
          }}
        />
        <UtilButton label={t('library.paste_from_clipboard')} accent={accent} onClick={() => void handlePasteFromClipboard()} fullWidth />
        <div style={{ display: 'flex', gap: '0.625rem', justifyContent: 'flex-end' }}>
          <button
            onClick={onDismiss}
            style={{
              background: 'none',
              border: 'none',
              color: 'rgba(255,255,255,0.6)',
              fontSize: '0.875rem',
              cursor: 'pointer',
              padding: '0.5rem 0.75rem',
            }}
          >
            {t('auto.cancel')}
          </button>
          <button
            disabled={!json.trim()}
            onClick={() => {
              if (json.trim()) onImport(json.trim());
            }}
            style={{
              background: json.trim() ? accent : 'rgba(255,255,255,0.12)',
              border: 'none',
              borderRadius: '0.5rem',
              color: json.trim() ? contrastOn(accent) : 'rgba(255,255,255,0.3)',
              fontSize: '0.875rem',
              fontWeight: 700,
              cursor: json.trim() ? 'pointer' : 'default',
              padding: '0.5rem 1rem',
            }}
          >
            {t('library.import_collections')}
          </button>
        </div>
      </div>
    </div>
  );
}
