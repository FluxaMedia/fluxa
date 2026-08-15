import type React from 'react';

export const FONT = "'Montserrat', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";

export const S: Record<string, React.CSSProperties> = {
  root: {
    position: 'fixed', inset: 0, zIndex: 9999, background: '#0C0C0C',
    backgroundImage: `linear-gradient(rgba(3, 8, 12, 0.62), rgba(3, 8, 12, 0.78)), url(${import.meta.env.BASE_URL}welcome-background.png)`,
    backgroundPosition: 'center', backgroundSize: 'cover',
    color: '#FFFFFF', overflow: 'auto', fontFamily: FONT,
    display: 'flex', flexDirection: 'column',
  },
  topBar: {
    position: 'absolute', top: 0, left: 0, right: 0, height: '4.5rem',
    display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0 2.75rem',
  },
  logo: { margin: 0, fontSize: '1.375rem', fontWeight: 700, letterSpacing: '0.125rem', fontFamily: FONT },
  kicker: {
    margin: '0.125rem 0 0', color: 'rgba(255,255,255,0.28)', fontSize: '0.625rem',
    fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.09em', fontFamily: FONT,
  },
  backBtn: {
    display: 'flex', alignItems: 'center', gap: '0.375rem', border: 'none',
    background: 'transparent', color: 'rgba(255,255,255,0.55)', cursor: 'pointer',
    padding: '0.375rem 0.5rem', fontFamily: FONT,
  },
  main: {
    flex: 1, display: 'flex', flexDirection: 'column',
    alignItems: 'center', justifyContent: 'center', padding: '5rem 2.5rem 3.75rem',
  },
  hero: { textAlign: 'center', maxWidth: '35rem', marginBottom: '3rem' },
  headline: {
    margin: '0 0 1rem', fontSize: '2.625rem', fontWeight: 700, lineHeight: 1.1,
    letterSpacing: '-0.03em', fontFamily: FONT,
  },
  subheadline: {
    margin: 0, color: 'rgba(255,255,255,0.50)', fontSize: '1rem', lineHeight: 1.6, fontFamily: FONT,
  },
  actions: {
    display: 'flex', flexDirection: 'column', alignItems: 'center',
    gap: '0.75rem', width: '100%', maxWidth: '20rem',
  },
  primaryBtn: {
    width: '100%', padding: '0.875rem 1.5rem', borderRadius: '0.5rem', border: 'none',
    background: '#FFFFFF', color: '#000000', fontSize: '0.9375rem', fontWeight: 600,
    fontFamily: FONT, cursor: 'pointer',
  },
  secondaryBtn: {
    width: '100%', padding: '0.8125rem 1.5rem', borderRadius: '0.5rem',
    border: '1px solid rgba(255,255,255,0.12)', background: 'transparent',
    color: 'rgba(255,255,255,0.70)', fontSize: '0.9375rem', fontWeight: 500, fontFamily: FONT, cursor: 'pointer',
  },
  note: {
    marginTop: '2rem', color: 'rgba(255,255,255,0.25)', fontSize: '0.75rem',
    textAlign: 'center', maxWidth: '22.5rem', lineHeight: 1.5, fontFamily: FONT,
  },
  authMain: {
    flex: 1, display: 'flex', alignItems: 'center',
    justifyContent: 'center', padding: '6.25rem 1.5rem 3.75rem',
  },
  card: { width: '100%', maxWidth: '25rem' },
  nuvioBtn: {
    width: '100%', padding: '0.8125rem 1.25rem', borderRadius: '0.5rem',
    border: '1px solid rgba(255,255,255,0.14)', background: 'rgba(255,255,255,0.05)',
    color: '#FFFFFF', fontSize: '0.875rem', fontWeight: 500, fontFamily: FONT, cursor: 'pointer',
    display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '0.625rem', marginBottom: 0,
  },
  tabs: {
    display: 'flex', marginBottom: '1.5rem',
    borderBottom: '1px solid rgba(255,255,255,0.08)',
  },
  tabBtn: {
    flex: 1, background: 'transparent', border: 'none',
    borderBottom: '0.125rem solid transparent', marginBottom: '-0.0625rem',
    padding: '0.625rem 0 0.8125rem', color: 'rgba(255,255,255,0.40)',
    fontSize: '0.875rem', fontWeight: 500, fontFamily: FONT, cursor: 'pointer',
    transition: 'color 0.15s, border-color 0.15s',
  },
  tabBtnActive: { color: '#FFFFFF', borderBottomColor: '#FFFFFF' },
  form: { display: 'flex', flexDirection: 'column', gap: '1rem' },
  fieldWrap: { display: 'flex', flexDirection: 'column', gap: '0.375rem' },
  label: {
    fontSize: '0.6875rem', fontWeight: 600, color: 'rgba(255,255,255,0.40)',
    fontFamily: FONT, letterSpacing: '0.07em', textTransform: 'uppercase',
  },
  input: {
    width: '100%', padding: '0.6875rem 0.875rem', borderRadius: '0.4375rem',
    border: '1px solid rgba(255,255,255,0.10)', background: 'rgba(255,255,255,0.05)',
    color: '#FFFFFF', fontSize: '0.875rem', fontFamily: FONT, outline: 'none',
    boxSizing: 'border-box', transition: 'border-color 0.15s',
  },
  inputError: { borderColor: 'rgba(255,80,80,0.55)' },
  passwordWrap: { position: 'relative' },
  passwordInput: { paddingRight: '2.625rem' },
  eyeBtn: {
    position: 'absolute', right: '0.75rem', top: '50%', transform: 'translateY(-50%)',
    background: 'transparent', border: 'none', color: 'rgba(255,255,255,0.35)',
    cursor: 'pointer', padding: '0.125rem', display: 'flex', alignItems: 'center',
  },
  errorText: { margin: 0, fontSize: '0.6875rem', color: 'rgba(255,100,100,0.85)', fontFamily: FONT },
  globalError: {
    margin: '0 0 1rem', padding: '0.625rem 0.875rem', borderRadius: '0.4375rem',
    background: 'rgba(255,60,60,0.08)', border: '1px solid rgba(255,60,60,0.20)',
    color: 'rgba(255,140,140,0.90)', fontSize: '0.8125rem', fontFamily: FONT,
  },
  forgotBtn: {
    background: 'transparent', border: 'none', color: 'rgba(255,255,255,0.35)',
    fontSize: '0.75rem', fontFamily: FONT, cursor: 'pointer', padding: 0,
  },
  submitBtn: {
    width: '100%', padding: '0.8125rem 1.5rem', borderRadius: '0.5rem', border: 'none',
    background: '#FFFFFF', color: '#000000', fontSize: '0.9375rem', fontWeight: 600,
    fontFamily: FONT, cursor: 'pointer', transition: 'opacity 0.15s',
  },
  divider: {
    display: 'flex', alignItems: 'center', gap: '0.75rem', margin: '1.25rem 0',
  },
  dividerLine: { flex: 1, height: 1, background: 'rgba(255,255,255,0.08)' },
  dividerText: { color: 'rgba(255,255,255,0.25)', fontSize: '0.75rem', fontFamily: FONT },
  localBtn: {
    width: '100%', padding: '0.8125rem 1.5rem', borderRadius: '0.5rem',
    border: '1px solid rgba(255,255,255,0.10)', background: 'transparent',
    color: 'rgba(255,255,255,0.50)', fontSize: '0.875rem', fontWeight: 500,
    fontFamily: FONT, cursor: 'pointer', transition: 'opacity 0.15s',
  },
  eyebrow: {
    margin: '0 0 0.625rem', fontSize: '0.6875rem', fontWeight: 600,
    color: 'rgba(255,255,255,0.35)', textTransform: 'uppercase',
    letterSpacing: '0.09em', fontFamily: FONT,
  },
  cardTitle: {
    margin: '0 0 0.5rem', fontSize: '1.625rem', fontWeight: 700,
    letterSpacing: '-0.02em', fontFamily: FONT,
  },
  cardSubtitle: {
    margin: 0, fontSize: '0.875rem', color: 'rgba(255,255,255,0.45)',
    lineHeight: 1.55, fontFamily: FONT,
  },
};
