import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Type, X } from 'lucide-react';
import { t } from '../../i18n';
import { ActionTile, ChoiceTile, InputTile, SettingsSection, SliderTile, ToggleTile, langOptions, streamSourceOptions, subtitleFontOptions } from './SettingsUI';
import { styles, FONT } from './settingsStyles';
import type { Prefs } from './settingsTypes';
import { listCustomFonts, pickAndAddCustomFont, removeCustomFont, type CustomFont } from '../../core/customFonts';

const isWindows = navigator.userAgent.includes('Windows');

export function PlaybackSection({ prefs, setPref }: { prefs: Prefs; setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void }) {
  const [mpvScriptsDir, setMpvScriptsDir] = useState<string | null>(null);
  const [scriptsDirCopied, setScriptsDirCopied] = useState(false);
  useEffect(() => {
    invoke<string | null>('get_data_dir').then((dir) => {
      if (dir) setMpvScriptsDir(`${dir}/mpv/scripts`);
    }).catch(() => {});
  }, []);

  const [customFonts, setCustomFonts] = useState<CustomFont[]>([]);
  const [fontUploadError, setFontUploadError] = useState<string | null>(null);
  const refreshCustomFonts = () => { void listCustomFonts().then(setCustomFonts); };
  useEffect(() => { refreshCustomFonts(); }, []);
  const uploadCustomFont = async () => {
    setFontUploadError(null);
    try {
      const added = await pickAndAddCustomFont();
      if (added) refreshCustomFonts();
    } catch (e) {
      setFontUploadError(String(e));
    }
  };
  const removeFont = async (fileName: string) => {
    await removeCustomFont(fileName).catch(() => {});
    refreshCustomFonts();
  };

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
        title={t('settings.p2p_enabled')}
        subtitle={t('settings.p2p_enabled_desc')}
        checked={prefs.p2pEnabled}
        onToggle={(v) => setPref('p2pEnabled', v)}
      />
      <ChoiceTile
        title={t('settings.anime_upscaling')}
        subtitle={t('settings.anime_upscaling_desc')}
        options={[
          { value: 'auto', label: t('settings.auto') },
          { value: 'off', label: t('settings.off') },
        ]}
        selected={prefs.animeUpscalingMode}
        onSelect={(v) => setPref('animeUpscalingMode', v)}
      />
      <ChoiceTile
        title={t('settings.anime_upscaling_quality')}
        subtitle={t('settings.anime_upscaling_quality_desc')}
        options={[
          { value: 'anime4k_s', label: t('settings.anime4k_s') },
          { value: 'anime4k_m', label: t('settings.anime4k_m') },
          { value: 'anime4k_l', label: t('settings.anime4k_l') },
        ]}
        selected={prefs.animeUpscalingQuality}
        onSelect={(v) => setPref('animeUpscalingQuality', v)}
      />
      <ChoiceTile
        title={t('settings.frame_interpolation')}
        subtitle={t('settings.frame_interpolation_desc')}
        options={[
          { value: 'off', label: t('settings.off') },
          { value: 'display_resample', label: t('settings.frame_interpolation_display_resample') },
          { value: 'smooth', label: t('settings.frame_interpolation_smooth') },
        ]}
        selected={prefs.frameInterpolationMode}
        onSelect={(v) => setPref('frameInterpolationMode', v)}
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
          <div style={{ width: '100%', minHeight: '3.625rem', borderBottom: '1px solid rgba(255,255,255,0.055)', display: 'flex', alignItems: 'center', padding: '0.75rem 1rem', boxSizing: 'border-box', gap: '0.75rem' }}>
            <span style={{ ...styles.rowIcon, color: 'var(--primary-accent-color)' }}>
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>
            </span>
            <div style={{ flex: 1, minWidth: 0 }}>
              <p style={styles.rowTitle}>{t('settings.mpv_scripts_dir')}</p>
              <p style={{ ...styles.rowSubtitle, fontFamily: 'monospace', fontSize: '0.6875rem', wordBreak: 'break-all' }}>{mpvScriptsDir}</p>
            </div>
            <button
              style={{ background: scriptsDirCopied ? 'rgba(84,209,122,0.12)' : 'rgba(255,255,255,0.07)', border: '1px solid rgba(255,255,255,0.10)', color: scriptsDirCopied ? '#54D17A' : 'rgba(255,255,255,0.65)', fontSize: '0.75rem', fontWeight: 500, cursor: 'pointer', padding: '0.375rem 0.8125rem', borderRadius: '0.5rem', fontFamily: FONT, flexShrink: 0, transition: 'background 0.12s, color 0.12s' }}
              onClick={() => void copyScriptsDir()}
            >
              {scriptsDirCopied ? '✓ Copied' : 'Copy'}
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
      <ToggleTile
        title={t('settings.seek_thumbnails')}
        subtitle={t('settings.seek_thumbnails_desc')}
        checked={prefs.seekThumbnailEnabled}
        onToggle={(v) => {
          void setPref('seekThumbnailEnabled', v);
          void invoke('player_set_seek_thumbnail_enabled', { enabled: v });
        }}
      />
      <ToggleTile
        title="HDR output"
        subtitle="Use HDR output when the display and renderer support it. Falls back to SDR when unavailable."
        checked={prefs.hdrEnabled}
        onToggle={(v) => setPref('hdrEnabled', v)}
      />    </SettingsSection>
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
      <ToggleTile title={t('settings.auto_retry_next_source')} subtitle={t('settings.auto_retry_next_source_desc')} checked={prefs.autoRetryNextSource} onToggle={(v) => setPref('autoRetryNextSource', v)} />
      <ToggleTile title={t('settings.auto_play_next_episode')} subtitle={t('settings.auto_play_next_episode_desc')} checked={prefs.autoPlayNextEpisode} onToggle={(v) => setPref('autoPlayNextEpisode', v)} />
      {prefs.autoPlayNextEpisode && (
        <ChoiceTile
          title={t('settings.auto_play_countdown')}
          subtitle={t('settings.auto_play_countdown_desc')}
          options={[{ value: '5', label: '5s' }, { value: '7', label: '7s' }, { value: '10', label: '10s' }, { value: '15', label: '15s' }]}
          selected={prefs.autoPlayCountdownSecs}
          onSelect={(v) => setPref('autoPlayCountdownSecs', v)}
        />
      )}
      <ToggleTile title={t('settings.try_binge_group')} subtitle={t('settings.try_binge_group_desc')} checked={prefs.tryBingeGroup} onToggle={(v) => setPref('tryBingeGroup', v)} />
      <SliderTile title={t('settings.next_episode_threshold')} subtitle={t('settings.next_episode_threshold_desc')} value={Number(prefs.nextEpisodeThresholdPercent)} min={50} max={95} step={5} onChange={(v) => setPref('nextEpisodeThresholdPercent', String(v))} />
      <SliderTile title={t('settings.watched_threshold')} subtitle={t('settings.watched_threshold_desc')} value={Number(prefs.watchedThresholdPercent)} min={60} max={100} step={5} onChange={(v) => setPref('watchedThresholdPercent', String(v))} />
    </SettingsSection>
    <SettingsSection title={t('settings.advanced')} subtitle={t('settings.buffer_cache_desc')}>
      <ChoiceTile title={t('settings.buffer_cache')} subtitle={t('settings.buffer_cache_desc')} options={[{ value: '100', label: '100 MB' }, { value: '500', label: '500 MB' }, { value: '1000', label: '1 GB' }, { value: '2000', label: '2 GB' }, { value: '-1', label: t('settings.buffer_cache_infinite') }]} selected={prefs.playerBufferCacheMb} onSelect={(v) => setPref('playerBufferCacheMb', v)} />
      <ChoiceTile title={t('settings.forward_buffer')} subtitle={t('settings.forward_buffer_desc')} options={[{ value: '30', label: '30s' }, { value: '60', label: '60s' }, { value: '120', label: '120s' }, { value: '300', label: '300s' }, { value: '600', label: '600s' }]} selected={prefs.playerForwardBufferSeconds} onSelect={(v) => setPref('playerForwardBufferSeconds', v)} />
      <ChoiceTile title={t('settings.back_buffer')} subtitle={t('settings.back_buffer_desc')} options={[{ value: '0', label: '0s' }, { value: '15', label: '15s' }, { value: '30', label: '30s' }, { value: '60', label: '60s' }, { value: '120', label: '120s' }, { value: '300', label: '300s' }]} selected={prefs.playerBackBufferSeconds} onSelect={(v) => setPref('playerBackBufferSeconds', v)} />
      <ChoiceTile
        title={t('settings.render_backend')}
        subtitle={t('settings.render_backend_desc')}
        options={[
          { value: 'opengl', label: 'OpenGL' },
          { value: 'vulkan', label: 'Vulkan' },
          ...(isWindows ? [{ value: 'd3d11', label: 'D3D11' }] : []),
        ]}
        selected={prefs.renderBackend}
        onSelect={(v) => setPref('renderBackend', v)}
      />
      <ChoiceTile
        title={t('settings.player_engine')}
        subtitle={t('settings.player_engine_desc')}
        options={[
          { value: 'mpv', label: 'mpv' },
          { value: 'libvlc', label: 'libVLC' },
        ]}
        selected={prefs.playerEngine}
        onSelect={(v) => setPref('playerEngine', v)}
      />    </SettingsSection>
    <SettingsSection title={t('settings.skip_segments')} subtitle={t('settings.use_introdb_desc')}>
      <ToggleTile title={t('settings.skip_intro_outro_recap')} subtitle={t('settings.skip_intro_outro_recap_desc')} checked={prefs.useIntroDb || prefs.useAniSkip || prefs.useSkipDb || prefs.useTheIntroDb} onToggle={(v) => { setPref('useIntroDb', v); setPref('useAniSkip', v); setPref('useSkipDb', v); setPref('useTheIntroDb', v); }} />
      <ToggleTile title={t('settings.use_animeskip')} subtitle={t('settings.use_animeskip_desc')} checked={prefs.useAnimeSkip} onToggle={(v) => setPref('useAnimeSkip', v)} />
      {prefs.useAnimeSkip && (
        <InputTile
          title={t('settings.animeskip_client_id')}
          subtitle={t('settings.animeskip_client_id_desc')}
          value={prefs.animeSkipClientId}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('animeSkipClientId', v)}
        />
      )}
      <ToggleTile title={t('settings.use_chapter_skip')} subtitle={t('settings.use_chapter_skip_desc')} checked={prefs.useChapterSkip} onToggle={(v) => setPref('useChapterSkip', v)} />
      {(prefs.useIntroDb || prefs.useAniSkip || prefs.useSkipDb || prefs.useTheIntroDb || prefs.useAnimeSkip) && (
        <ToggleTile title={t('settings.auto_skip')} subtitle={t('settings.auto_skip_desc')} checked={prefs.autoSkipIntro} onToggle={(v) => setPref('autoSkipIntro', v)} />
      )}
      {prefs.useIntroDb && (
        <ToggleTile title={t('settings.introdb_submit')} subtitle={t('settings.introdb_submit_desc')} checked={prefs.introDbSubmitEnabled} onToggle={(v) => setPref('introDbSubmitEnabled', v)} />
      )}
      {prefs.useIntroDb && prefs.introDbSubmitEnabled && (
        <InputTile
          title={t('settings.introdb_api_key')}
          subtitle={t('settings.introdb_api_key_desc')}
          value={prefs.introDbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('introDbApiKey', v)}
        />
      )}
      {prefs.useSkipDb && (
        <ToggleTile title={t('settings.skipdb_submit')} subtitle={t('settings.skipdb_submit_desc')} checked={prefs.skipDbSubmitEnabled} onToggle={(v) => setPref('skipDbSubmitEnabled', v)} />
      )}
      {prefs.useSkipDb && prefs.skipDbSubmitEnabled && (
        <InputTile
          title={t('settings.skipdb_api_key')}
          subtitle={t('settings.skipdb_api_key_desc')}
          value={prefs.skipDbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('skipDbApiKey', v)}
        />
      )}
      {prefs.useTheIntroDb && (
        <ToggleTile title={t('settings.theintrodb_submit')} subtitle={t('settings.theintrodb_submit_desc')} checked={prefs.theIntroDbSubmitEnabled} onToggle={(v) => setPref('theIntroDbSubmitEnabled', v)} />
      )}
      {prefs.useTheIntroDb && prefs.theIntroDbSubmitEnabled && (
        <InputTile
          title={t('settings.theintrodb_api_key')}
          subtitle={t('settings.theintrodb_api_key_desc')}
          value={prefs.theIntroDbApiKey}
          placeholder={t('settings.api_key_placeholder')}
          onChange={(v) => setPref('theIntroDbApiKey', v)}
        />
      )}
    </SettingsSection>
    <SettingsSection title={t('settings.preferences')} subtitle={t('settings.preferred_audio_language_desc')}>
      <ChoiceTile title={t('settings.preferred_audio_language')} subtitle={t('settings.preferred_audio_language_desc')} options={langOptions()} selected={prefs.preferredAudioLanguage} onSelect={(v) => setPref('preferredAudioLanguage', v)} />
      <ChoiceTile title={t('settings.secondary_audio_language')} subtitle={t('settings.secondary_audio_language_desc')} options={langOptions()} selected={prefs.secondaryAudioLanguage} onSelect={(v) => setPref('secondaryAudioLanguage', v)} />
      <ChoiceTile title={t('settings.preferred_subtitle_language')} subtitle={t('settings.preferred_subtitle_language_desc')} options={langOptions()} selected={prefs.preferredSubtitleLanguage} onSelect={(v) => setPref('preferredSubtitleLanguage', v)} />
      <ChoiceTile title={t('settings.secondary_subtitle_language')} subtitle={t('settings.secondary_subtitle_language_desc')} options={langOptions()} selected={prefs.secondarySubtitleLanguage} onSelect={(v) => setPref('secondarySubtitleLanguage', v)} />
      <ToggleTile title={t('settings.anime_japanese_audio')} subtitle={t('settings.anime_japanese_audio_desc')} checked={prefs.animePreferJapaneseAudio} onToggle={(v) => setPref('animePreferJapaneseAudio', v)} />
    </SettingsSection>
    <SettingsSection title={t('settings.subtitle.customize')} subtitle={t('auto.subtitle_language_size_and_readability')}>
      <ToggleTile title={t('auto.auto_enable_subtitles_db2311e6')} subtitle={t('auto.enable_subtitles_automatically_when_availabl')} checked={prefs.autoEnableSubtitles} onToggle={(v) => setPref('autoEnableSubtitles', v)} />
      <ChoiceTile title={t('settings.subtitle_position')} subtitle={t('settings.subtitle_position_desc')} options={[{ value: '100', label: t('settings.subtitle_position_bottom') }, { value: '90', label: t('settings.subtitle_position_low') }, { value: '80', label: t('settings.subtitle_position_middle') }, { value: '70', label: t('settings.subtitle_position_high') }]} selected={prefs.subtitlePosition} onSelect={(v) => setPref('subtitlePosition', v)} />
      <ChoiceTile title={t('auto.subtitle_size_7fc78c82')} subtitle={t('auto.tune_readability_on_tv_and_mobile')} options={[{ value: '50', label: '50%' }, { value: '75', label: '75%' }, { value: '100', label: '100%' }, { value: '125', label: '125%' }, { value: '150', label: '150%' }, { value: '200', label: '200%' }]} selected={prefs.subtitleSize} onSelect={(v) => setPref('subtitleSize', v)} />
      <ChoiceTile title={t('settings.subtitle_text')} subtitle={t('settings.subtitle_text_desc')} options={[{ value: '#FFFFFF', label: t('auto.white') }, { value: '#000000', label: t('auto.black') }, { value: '#FFE45C', label: t('auto.yellow') }, { value: '#FF5D5D', label: t('auto.red') }, { value: '#3F7CFF', label: t('auto.blue') }, { value: '#54D17A', label: t('auto.green') }, { value: '#FF8A3D', label: t('auto.orange') }, { value: '#C084FC', label: t('auto.purple') }]} selected={prefs.subtitleColor} onSelect={(v) => setPref('subtitleColor', v)} />
      <ChoiceTile title={t('settings.subtitle.outline_opacity')} subtitle={t('settings.subtitle_text_desc')} options={[{ value: '1.0', label: '100%' }, { value: '0.75', label: '75%' }, { value: '0.5', label: '50%' }, { value: '0.25', label: '25%' }, { value: '0.0', label: '0%' }]} selected={prefs.subtitleTextOpacity} onSelect={(v) => setPref('subtitleTextOpacity', v)} />
      <ChoiceTile title={t('settings.subtitle_background')} subtitle={t('settings.subtitle_background_desc')} options={[{ value: '#000000', label: t('auto.black') }, { value: '#FFFFFF', label: t('auto.white') }, { value: '#FFE45C', label: t('auto.yellow') }, { value: '#FF5D5D', label: t('auto.red') }, { value: '#3F7CFF', label: t('auto.blue') }]} selected={prefs.subtitleBackgroundColor} onSelect={(v) => setPref('subtitleBackgroundColor', v)} />
      <ChoiceTile title={t('auto.background_transparency')} subtitle={t('settings.subtitle_background_desc')} options={[{ value: '1.0', label: '100%' }, { value: '0.75', label: '75%' }, { value: '0.5', label: '50%' }, { value: '0.25', label: '25%' }, { value: '0.0', label: '0%' }]} selected={prefs.subtitleBackgroundOpacity} onSelect={(v) => setPref('subtitleBackgroundOpacity', v)} />
      <ChoiceTile title={t('settings.subtitle_outline')} subtitle={t('settings.subtitle_outline_desc')} options={[{ value: '#000000', label: t('auto.black') }, { value: '#FFFFFF', label: t('auto.white') }, { value: '#FFE45C', label: t('auto.yellow') }, { value: '#FF5D5D', label: t('auto.red') }, { value: '#3F7CFF', label: t('auto.blue') }]} selected={prefs.subtitleOutlineColor} onSelect={(v) => setPref('subtitleOutlineColor', v)} />
      <ChoiceTile title={t('settings.subtitle.outline_opacity')} subtitle={t('settings.subtitle_outline_desc')} options={[{ value: '1.0', label: '100%' }, { value: '0.75', label: '75%' }, { value: '0.5', label: '50%' }, { value: '0.25', label: '25%' }, { value: '0.0', label: '0%' }]} selected={prefs.subtitleOutlineOpacity} onSelect={(v) => setPref('subtitleOutlineOpacity', v)} />
      <ChoiceTile title={t('settings.subtitle_outline_size')} subtitle={t('settings.subtitle_outline_size_desc')} options={[{ value: '0', label: t('settings.subtitle_outline_none') }, { value: '1', label: '1' }, { value: '2', label: '2' }, { value: '3', label: '3' }, { value: '4', label: '4' }, { value: '5', label: '5' }]} selected={prefs.subtitleOutlineSize} onSelect={(v) => setPref('subtitleOutlineSize', v)} />
      <ToggleTile title={t('settings.subtitle_bold')} subtitle={t('settings.subtitle_bold_desc')} checked={prefs.subtitleBold} onToggle={(v) => setPref('subtitleBold', v)} />
      <ToggleTile title={t('settings.subtitle_force_style')} subtitle={t('settings.subtitle_force_style_desc')} checked={prefs.subtitleForceStyle} onToggle={(v) => setPref('subtitleForceStyle', v)} />
      <ToggleTile title={t('settings.subtitle_shadow') || 'Altyazı Gölgesi'} subtitle={t('settings.subtitle_shadow_desc') || 'Altyazı metnine gölge efekti ekle'} checked={prefs.subtitleShadow} onToggle={(v) => setPref('subtitleShadow', v)} />
      <ChoiceTile
        title={t('settings.subtitle_font')}
        subtitle={t('settings.subtitle_font_desc')}
        options={subtitleFontOptions(customFonts.map((f) => f.family))}
        selected={prefs.subtitleFont}
        onSelect={(v) => setPref('subtitleFont', v)}
      />
      <ActionTile
        title={t('settings.upload_custom_font')}
        subtitle={fontUploadError ?? t('settings.upload_custom_font_desc')}
        icon={<Type size={18} />}
        onClick={() => void uploadCustomFont()}
        accent={fontUploadError ? '#FF5D5D' : '#FFFFFF'}
      />
      {customFonts.map((font) => (
        <div key={font.fileName} style={{ width: '100%', minHeight: '2.75rem', borderBottom: '1px solid rgba(255,255,255,0.055)', display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '0.5rem 1rem 0.5rem 2.875rem', boxSizing: 'border-box', gap: '0.75rem' }}>
          <span style={{ fontSize: '0.8125rem', color: 'rgba(255,255,255,0.75)', fontFamily: FONT, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{font.family}</span>
          <button
            type="button"
            onClick={() => void removeFont(font.fileName)}
            style={{ background: 'none', border: 'none', color: 'rgba(255,255,255,0.4)', cursor: 'pointer', padding: '0.25rem', display: 'flex', flexShrink: 0 }}
            title={t('auto.remove')}
          >
            <X size={15} />
          </button>
        </div>
      ))}
    </SettingsSection>
    </>
  );
}
