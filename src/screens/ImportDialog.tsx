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
        padding: 24,
      }}
      onClick={(e) => e.target === e.currentTarget && onDismiss()}
    >
      <div
        style={{
          background: '#111620',
          borderRadius: 12,
          padding: 24,
          width: '100%',
          maxWidth: 480,
          display: 'flex',
          flexDirection: 'column',
          gap: 14,
          boxSizing: 'border-box',
        }}
      >
        <span style={{ color: '#fff', fontSize: 18, fontWeight: 900, fontFamily: 'sans-serif' }}>
          {t('library.import_collections')}
        </span>
        <textarea
          value={json}
          onChange={(e) => setJson(e.target.value)}
          placeholder={t('library.paste_json')}
          style={{
            width: '100%',
            minHeight: 160,
            maxHeight: 360,
            padding: 12,
            background: 'rgba(255,255,255,0.05)',
            border: `1px solid ${accent}55`,
            borderRadius: 8,
            color: '#fff',
            fontSize: 13,
            fontFamily: 'monospace',
            resize: 'vertical',
            outline: 'none',
            boxSizing: 'border-box',
          }}
        />
        <UtilButton
          label={t('library.paste_from_clipboard')}
          accent={accent}
          onClick={() => void handlePasteFromClipboard()}
          fullWidth
        />
        <div style={{ display: 'flex', gap: 10, justifyContent: 'flex-end' }}>
          <button
            onClick={onDismiss}
            style={{
              background: 'none',
              border: 'none',
              color: 'rgba(255,255,255,0.6)',
              fontFamily: 'sans-serif',
              fontSize: 14,
              cursor: 'pointer',
              padding: '8px 12px',
            }}
          >
            {t('auto.cancel')}
          </button>
          <button
            disabled={!json.trim()}
            onClick={() => { if (json.trim()) onImport(json.trim()); }}
            style={{
              background: json.trim() ? accent : 'rgba(255,255,255,0.12)',
              border: 'none',
              borderRadius: 8,
              color: json.trim() ? contrastOn(accent) : 'rgba(255,255,255,0.3)',
              fontFamily: 'sans-serif',
              fontSize: 14,
              fontWeight: 700,
              cursor: json.trim() ? 'pointer' : 'default',
              padding: '8px 16px',
            }}
          >
            {t('library.import_collections')}
          </button>
        </div>
      </div>
    </div>
  );
}
