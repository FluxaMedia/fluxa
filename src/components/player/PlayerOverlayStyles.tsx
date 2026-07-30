export function PlayerOverlayStyles() {
  return <style>{`
    @keyframes fluxa-seek-spin { to { transform: rotate(360deg); } }
    @keyframes fluxa-skip-in { from { opacity: 0; transform: translateX(0.75rem); } to { opacity: 1; transform: translateX(0); } }
    @keyframes fluxa-nextep-in { from { opacity: 0; transform: translateX(0.75rem); } to { opacity: 1; transform: translateX(0); } }
    .fluxa-ibtn { opacity: 0.8; transition: opacity 0.15s, background 0.12s; }
    .fluxa-ibtn:hover { opacity: 1; background: rgba(255,255,255,0.09) !important; }
    .fluxa-skip-btn { animation: fluxa-skip-in 0.2s cubic-bezier(0.16, 1, 0.3, 1); transition: filter 0.12s, transform 0.12s; }
    .fluxa-skip-btn:hover { filter: brightness(1.06); transform: translateY(-0.0625rem); }
    .fluxa-skip-btn:active { transform: translateY(0); }
    .fluxa-skip-btn:focus-visible { outline: 0.125rem solid rgba(255,255,255,0.85); outline-offset: 0.125rem; }
    .fluxa-cursor-hidden, .fluxa-cursor-hidden * { cursor: none !important; }
    .fluxa-seek-track { transition: height 0.15s ease; }
    .fluxa-seek-dot { transition: width 0.15s, height 0.15s; }
  `}</style>;
}
