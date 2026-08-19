import React, { type ReactNode } from 'react';
import { appStyles } from '../appConstants';

interface Props {
  ready: boolean;
  profilesChecked: boolean;
  welcomeCompleted: boolean;
  profileReady: boolean;
  loading: ReactNode;
  welcome: ReactNode;
  profile: ReactNode;
  children: ReactNode;
}

export function AppBootstrap({ ready, profilesChecked, welcomeCompleted, profileReady, loading, welcome, profile, children }: Props) {
  if (!ready || !profilesChecked) {
    return <div style={appStyles.loading}>{loading}</div>;
  }
  if (!welcomeCompleted) return <>{welcome}</>;
  if (!profileReady) return <>{profile}</>;
  return <>{children}</>;
}
