import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { t } from '../../i18n';
import { ChoiceTile, InputTile, SettingsSection, SliderTile, ToggleTile, langOptions, streamSourceOptions } from './SettingsUI';
import { styles } from './settingsStyles';
import type { Prefs } from './settingsTypes';

export function PlaybackSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  const [mpvScriptsDir, setMpvScriptsDir] = useState<string | null>(null);
  const [scriptsDirCopied, setScriptsDirCopied] = useState(false);
  useEffect(() => {
    invoke<string | null>('get_data_dir').then((dir) => {
      if (dir) setMpvScriptsDir(`${dir}/mpv/scripts`);
    }).catch(() => {});
  }, []);

  const scriptsDirCopiedTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => () => { if (scriptsDirCopiedTimerRef.current) clearTimeout(scriptsDirCopiedTimerRef.current); }, []);
  const copyScriptsDir = async () => {
    if (!mpvScriptsDir) return;
    try {
      await navigator.clipboard.writeText(mpvScriptsDir);
      setScriptsDirCopied(true);
      if (scriptsDirCopiedTimerRef.current) clearTimeout(scriptsDirCopiedTimerRef.current);
      scriptsDirCopiedTimerRef.current = setTimeout(() => setScriptsDirCopied(false), 2000);
    } catch { /* ignore */ }
  };

  return (
    <>
    <SettingsSection title={t('auto.playback_dbc1ddba')} subtitle={t('auto.player_behavior_and_defaults')}>
      <ToggleTile
        title={t('settings.picture_in_picture') || 'Resim İçinde Resim (PiP)'}
        subtitle={t('settings.picture_in_picture_desc') || 'Oynatmayı küçük pencerede sürdür'}
        checked={prefs.pictureInPicture}
        onToggle={(v) => setPref('pictureInPicture', v)}
      />
      <ToggleTile
        title={t('settings.hdr_playback') || 'HDR Oynatma'}
        subtitle={t('settings.hdr_playback_desc') || 'Destekleniyorsa HDR içerikleri HDR modunda oynat'}
        checked={prefs.hdrPlayback}
        onToggle={(v) => setPref('hdrPlayback', v)}
      />
      {true && (
        <>
        <InputTile
          title={t('settings.mpv_custom_options')}
          subtitle={t('settings.mpv_custom_options_desc')}
          value={prefs.mpvCustomOptions}
          placeholder={'# one option per line\nsub-scale=1.2\nvolume-max=200'}
          multiline
          onChange={(v) => setPref('mpvCustomOptions', v)}
        />
        {mpvScriptsDir && (
          <div style={{ width: '100%', minHeight: 62, borderRadius: 0, border: 'none', borderBottom: '1px solid rgba(255,255,255,0.055)', display: 'flex', alignItems: 'center', padding: '13px 16px', boxSizing: 'border-box', gap: 14 }}>
            <span style={{ ...styles.rowIcon, color: 'var(--primary-accent-color)' }}>
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>
            </span>
            <div style={{ flex: 1, minWidth: 0 }}>
              <p style={styles.rowTitle}>{t('settings.mpv_scripts_dir') || 'MPV Komut Dosyaları Klasörü'}</p>
              <p style={{ ...styles.rowSubtitle, fontFamily: 'monospace', fontSize: 11, wordBreak: 'break-all' }}>{mpvScriptsDir}</p>
            </div>
            <button
              style={{ background: scriptsDirCopied ? 'rgba(84,209,122,0.15)' : 'rgba(255,255,255,0.08)', border: '1px solid rgba(255,255,255,0.12)', color: scriptsDirCopied ? '#54D17A' : 'rgba(255,255,255,0.7)', fontSize: 12, fontWeight: 700, cursor: 'pointer', padding: '6px 14px', borderRadius: 999, fontFamily: 'sans-serif', flexShrink: 0, transition: 'background 0.15s, color 0.15s' }}
              onClick={() => void copyScriptsDir()}
            >
              {scriptsDirCopied ? '✓ Kopyalandı' : 'Kopyala'}
            </button>
          </div>
        )}
        </>
      )}
      <ChoiceTile
        title={t('auto.playback_speed')}
        subtitle={t('settings.playback_speed_desc')}
        options={[{ value: '0.75', label: '0.75x' }, { value: '1.0', label: '1.00x' }, { value: '1.25', label: '1.25x' }, { value: '1.5', label: '1.50x' }]}
        selected={prefs.playbackSpeed}
        onSelect={(v) => setPref('playbackSpeed', v)}
      />
      <ChoiceTile
        title={t('auto.forward_rewind')}
        subtitle={t('settings.forward_rewind_desc')}
        options={[{ value: '10', label: '10s' }, { value: '15', label: '15s' }, { value: '30', label: '30s' }]}
        selected={prefs.seekSeconds}
        onSelect={(v) => setPref('seekSeconds', v)}
      />
      <ToggleTile title={t('settings.hold_to_speed')} subtitle={t('settings.hold_to_speed_desc')} checked={prefs.holdToSpeedEnabled} onToggle={(v) => setPref('holdToSpeedEnabled', v)} />
      <ChoiceTile
        title={t('settings.hold_speed')}
        subtitle={t('settings.hold_speed_desc')}
        options={[{ value: '1.25', label: '1.25x' }, { value: '1.5', label: '1.50x' }, { value: '1.75', label: '1.75x' }, { value: '2.0', label: '2.00x' }, { value: '2.5', label: '2.50x' }, { value: '3.0', label: '3.00x' }]}
        selected={prefs.holdSpeed}
        onSelect={(v) => setPref('holdSpeed', v)}
      />
    </SettingsSection>
    <SettingsSection title={t('settings.stream_settings')} subtitle={t('settings.stream_source_selection_desc')}>
      <ChoiceTile
        title={t('settings.stream_source_selection')}
        subtitle={t('settings.stream_source_selection_desc')}
        options={streamSourceOptions()}
        selected={prefs.streamSourceSelectionMode}
        onSelect={(v) => setPref('streamSourceSelectionMode', v)}
      />
      {prefs.streamSourceSelectionMode === 'regex' && (
        <InputTile
          title={t('settings.regex_pattern')}
          subtitle={t('settings.regex_pattern_desc')}
          value={prefs.streamSourceRegexPattern}
          placeholder={t('settings.regex_pattern_placeholder')}
          onChange={(v) => setPref('streamSourceRegexPattern', v)}
        />
      )}
      <ToggleTile title={t('settings.auto_play_next_episode')} subtitle={t('settings.auto_play_next_episode_desc')} checked={prefs.autoPlayNextEpisode} onToggle={(v) => setPref('autoPlayNextEpisode', v)} />
      <ToggleTile title={t('settings.try_binge_group')} subtitle={t('settings.try_binge_group_desc')} checked={prefs.tryBingeGroup} onToggle={(v) => setPref('tryBingeGroup', v)} />
      <SliderTile title={t('settings.next_episode_threshold')} subtitle={t('settings.next_episode_threshold_desc')} value={Number(prefs.nextEpisodeThresholdPercent)} min={50} max={95} step={5} onChange={(v) => setPref('nextEpisodeThresholdPercent', String(v))} />
      <SliderTile title={t('settings.watched_threshold')} subtitle={t('settings.watched_threshold_desc')} value={Number(prefs.watchedThresholdPercent)} min={60} max={100} step={5} onChange={(v) => setPref('watchedThresholdPercent', String(v))} />
    </SettingsSection>
    <SettingsSection title={t('settings.advanced')} subtitle={t('settings.buffer_cache_desc')}>
      <ChoiceTile title={t('settings.buffer_cache')} subtitle={t('settings.buffer_cache_desc')} options={[{ value: '100', label: '100 MB' }, { value: '500', label: '500 MB' }, { value: '1000', label: '1 GB' }, { value: '2000', label: '2 GB' }]} selected={prefs.playerBufferCacheMb} onSelect={(v) => setPref('playerBufferCacheMb', v)} />
      <ChoiceTile title={t('settings.forward_buffer')} subtitle={t('settings.forward_buffer_desc')} options={[{ value: '30', label: '30s' }, { value: '60', label: '60s' }, { value: '120', label: '120s' }, { value: '300', label: '300s' }, { value: '600', label: '600s' }]} selected={prefs.playerForwardBufferSeconds} onSelect={(v) => setPref('playerForwardBufferSeconds', v)} />
      <ChoiceTile title={t('settings.back_buffer')} subtitle={t('settings.back_buffer_desc')} options={[{ value: '0', label: '0s' }, { value: '15', label: '15s' }, { value: '30', label: '30s' }, { value: '60', label: '60s' }, { value: '120', label: '120s' }, { value: '300', label: '300s' }]} selected={prefs.playerBackBufferSeconds} onSelect={(v) => setPref('playerBackBufferSeconds', v)} />
    </SettingsSection>
    <SettingsSection title={t('settings.decoder')} subtitle={t('settings.dv_fallback_desc')}>
      <ChoiceTile title={t('settings.dv_fallback')} subtitle={t('settings.dv_fallback_desc')} options={[{ value: 'auto', label: t('settings.dv_fallback_auto') }, { value: 'convert_dv81', label: t('settings.dv_fallback_convert_dv81') }, { value: 'off', label: t('settings.dv_fallback_off') }]} selected={prefs.dolbyVisionFallbackMode} onSelect={(v) => setPref('dolbyVisionFallbackMode', v)} />
      {prefs.dolbyVisionFallbackMode === 'convert_dv81' && (
        <>
          <ChoiceTile title={t('settings.dv_rpu_mode')} subtitle={t('settings.dv_rpu_mode_desc')} options={[{ value: '2', label: t('settings.dv_rpu_mode_2') }, { value: '1', label: t('settings.dv_rpu_mode_1') }, { value: '4', label: t('settings.dv_rpu_mode_4') }]} selected={prefs.dvRpuMode} onSelect={(v) => setPref('dvRpuMode', v)} />
          <ToggleTile title={t('settings.dv_zero_level5')} subtitle={t('settings.dv_zero_level5_desc')} checked={prefs.dvZeroLevel5} onToggle={(v) => setPref('dvZeroLevel5', v)} />
          <ChoiceTile title={t('settings.dv_hdr10plus_mode')} subtitle={t('settings.dv_hdr10plus_mode_desc')} options={[{ value: 'auto', label: t('settings.dv_hdr10plus_mode_auto') }, { value: 'always', label: t('settings.dv_hdr10plus_mode_always') }, { value: 'never', label: t('settings.dv_hdr10plus_mode_never') }]} selected={prefs.dvHdr10PlusMode} onSelect={(v) => setPref('dvHdr10PlusMode', v)} />
        </>
      )}
      <ChoiceTile title={t('settings.audio_decoder_mode')} subtitle={t('settings.audio_decoder_mode_desc')} options={[{ value: 'hw_prefer', label: t('settings.audio_decoder_hw_prefer') }, { value: 'hw_only', label: t('settings.audio_decoder_hw_only') }, { value: 'sw_only', label: t('settings.audio_decoder_sw_only') }]} selected={prefs.audioDecoderMode} onSelect={(v) => setPref('audioDecoderMode', v)} />
      <ToggleTile title={t('settings.tunneled_playback')} subtitle={t('settings.tunneled_playback_desc')} checked={prefs.tunneledPlayback} onToggle={(v) => setPref('tunneledPlayback', v)} />
      <ToggleTile title={t('settings.fps_counter')} subtitle={t('settings.fps_counter_desc')} checked={prefs.showFpsCounter} onToggle={(v) => setPref('showFpsCounter', v)} />
    </SettingsSection>
    <SettingsSection title={t('settings.skip_segments')} subtitle={t('settings.use_introdb_desc')}>
      <ToggleTile title={t('settings.use_introdb')} subtitle={t('settings.use_introdb_desc')} checked={prefs.useIntroDb} onToggle={(v) => setPref('useIntroDb', v)} />
      <ToggleTile title={t('settings.use_aniskip')} subtitle={t('settings.use_aniskip_desc')} checked={prefs.useAniSkip} onToggle={(v) => setPref('useAniSkip', v)} />
      {(prefs.useIntroDb || prefs.useAniSkip) && (
        <ToggleTile title={t('settings.auto_skip')} subtitle={t('settings.auto_skip_desc')} checked={prefs.autoSkipIntro} onToggle={(v) => setPref('autoSkipIntro', v)} />
      )}
    </SettingsSection>
    <SettingsSection title={t('settings.preferences')} subtitle={t('settings.preferred_audio_language_desc')}>
      <ChoiceTile title={t('settings.preferred_audio_language')} subtitle={t('settings.preferred_audio_language_desc')} options={langOptions()} selected={prefs.preferredAudioLanguage} onSelect={(v) => setPref('preferredAudioLanguage', v)} />
      <ChoiceTile title={t('settings.secondary_audio_language')} subtitle={t('settings.secondary_audio_language_desc')} options={langOptions()} selected={prefs.secondaryAudioLanguage} onSelect={(v) => setPref('secondaryAudioLanguage', v)} />
      <ChoiceTile title={t('settings.preferred_subtitle_language')} subtitle={t('settings.preferred_subtitle_language_desc')} options={langOptions()} selected={prefs.preferredSubtitleLanguage} onSelect={(v) => setPref('preferredSubtitleLanguage', v)} />
      <ChoiceTile title={t('settings.secondary_subtitle_language')} subtitle={t('settings.secondary_subtitle_language_desc')} options={langOptions()} selected={prefs.secondarySubtitleLanguage} onSelect={(v) => setPref('secondarySubtitleLanguage', v)} />
    </SettingsSection>
    <SettingsSection title={t('settings.subtitle.customize')} subtitle={t('auto.subtitle_language_size_and_readability')}>
      <ToggleTile title={t('auto.auto_enable_subtitles_db2311e6')} subtitle={t('auto.enable_subtitles_automatically_when_availabl')} checked={prefs.autoEnableSubtitles} onToggle={(v) => setPref('autoEnableSubtitles', v)} />
      <ChoiceTile title={t('auto.subtitle_size_7fc78c82')} subtitle={t('auto.tune_readability_on_tv_and_mobile')} options={[{ value: '50', label: '50%' }, { value: '75', label: '75%' }, { value: '100', label: '100%' }, { value: '125', label: '125%' }, { value: '150', label: '150%' }, { value: '200', label: '200%' }]} selected={prefs.subtitleSize} onSelect={(v) => setPref('subtitleSize', v)} />
      <ChoiceTile title={t('settings.subtitle_text')} subtitle={t('settings.subtitle_text_desc')} options={[{ value: '#FFFFFF', label: t('auto.white') }, { value: '#000000', label: t('auto.black') }, { value: '#FFE45C', label: t('auto.yellow') }, { value: '#FF5D5D', label: t('auto.red') }, { value: '#3F7CFF', label: t('auto.blue') }, { value: '#54D17A', label: t('auto.green') }, { value: '#FF8A3D', label: t('auto.orange') }, { value: '#C084FC', label: t('auto.purple') }]} selected={prefs.subtitleColor} onSelect={(v) => setPref('subtitleColor', v)} />
      <ChoiceTile title={t('settings.subtitle.outline_opacity')} subtitle={t('settings.subtitle_text_desc')} options={[{ value: '1.0', label: '100%' }, { value: '0.75', label: '75%' }, { value: '0.5', label: '50%' }, { value: '0.25', label: '25%' }, { value: '0.0', label: '0%' }]} selected={prefs.subtitleTextOpacity} onSelect={(v) => setPref('subtitleTextOpacity', v)} />
      <ChoiceTile title={t('settings.subtitle_background')} subtitle={t('settings.subtitle_background_desc')} options={[{ value: '#000000', label: t('auto.black') }, { value: '#FFFFFF', label: t('auto.white') }, { value: '#FFE45C', label: t('auto.yellow') }, { value: '#FF5D5D', label: t('auto.red') }, { value: '#3F7CFF', label: t('auto.blue') }]} selected={prefs.subtitleBackgroundColor} onSelect={(v) => setPref('subtitleBackgroundColor', v)} />
      <ChoiceTile title={t('auto.background_transparency')} subtitle={t('settings.subtitle_background_desc')} options={[{ value: '1.0', label: '100%' }, { value: '0.75', label: '75%' }, { value: '0.5', label: '50%' }, { value: '0.25', label: '25%' }, { value: '0.0', label: '0%' }]} selected={prefs.subtitleBackgroundOpacity} onSelect={(v) => setPref('subtitleBackgroundOpacity', v)} />
      <ChoiceTile title={t('settings.subtitle_outline')} subtitle={t('settings.subtitle_outline_desc')} options={[{ value: '#000000', label: t('auto.black') }, { value: '#FFFFFF', label: t('auto.white') }, { value: '#FFE45C', label: t('auto.yellow') }, { value: '#FF5D5D', label: t('auto.red') }, { value: '#3F7CFF', label: t('auto.blue') }]} selected={prefs.subtitleOutlineColor} onSelect={(v) => setPref('subtitleOutlineColor', v)} />
      <ChoiceTile title={t('settings.subtitle.outline_opacity')} subtitle={t('settings.subtitle_outline_desc')} options={[{ value: '1.0', label: '100%' }, { value: '0.75', label: '75%' }, { value: '0.5', label: '50%' }, { value: '0.25', label: '25%' }, { value: '0.0', label: '0%' }]} selected={prefs.subtitleOutlineOpacity} onSelect={(v) => setPref('subtitleOutlineOpacity', v)} />
      <ToggleTile title={t('settings.subtitle_shadow') || 'Altyazı Gölgesi'} subtitle={t('settings.subtitle_shadow_desc') || 'Altyazı metnine gölge efekti ekle'} checked={prefs.subtitleShadow} onToggle={(v) => setPref('subtitleShadow', v)} />
    </SettingsSection>
    <div style={styles.settingsGroup}>
      <div style={styles.groupHeading}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ background: '#FF9500', color: '#000', fontSize: 10, fontWeight: 900, fontFamily: 'sans-serif', padding: '2px 7px', borderRadius: 4, letterSpacing: '0.06em', flexShrink: 0 }}>EXPERIMENTAL</span>
          <p style={{ ...styles.groupTitle, margin: 0 }}>Experimental</p>
        </div>
        <p style={styles.groupSubtitle}>Features that may change or be removed. Use with caution.</p>
      </div>
      <div style={styles.settingsCard}>
        <ToggleTile
          title="Seek Thumbnails"
          subtitle="Show a thumbnail preview when hovering over the seekbar. Uses a hidden mpv instance — may increase memory and CPU usage."
          checked={prefs.seekThumbnailEnabled}
          onToggle={(v) => {
            void setPref('seekThumbnailEnabled', v);
            void invoke('player_set_seek_thumbnail_enabled', { enabled: v });
          }}
        />
      </div>
    </div>
    </>
  );
}
