import React, { type CSSProperties, type ReactNode } from 'react';

interface Props {
  rootStyle: CSSProperties;
  animations: string;
  density: string;
  reduceMotion: string;
  reducedEffects: string;
  navigation: ReactNode;
  globalControls: ReactNode;
  contentStyle: CSSProperties;
  content: ReactNode;
  notices: ReactNode;
  dialogs: ReactNode;
  playback: ReactNode;
}

export function AppShell({
  rootStyle,
  animations,
  density,
  reduceMotion,
  reducedEffects,
  navigation,
  globalControls,
  contentStyle,
  content,
  notices,
  dialogs,
  playback,
}: Props) {
  return (
    <div
      style={rootStyle}
      data-animations={animations}
      data-density={density}
      data-reduce-motion={reduceMotion}
      data-reduced-effects={reducedEffects}
    >
      {navigation}
      {globalControls}
      <div className="app-content" style={contentStyle}>{content}</div>
      {notices}
      {dialogs}
      {playback}
    </div>
  );
}
