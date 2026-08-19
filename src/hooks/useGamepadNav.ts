import { useEffect } from 'react';
import type { NavRoute } from '../components/NavSidebar';
import { onGamepadAction } from '../platform/gamepadInput';
import { focusNearestCard, isNavCard } from '../core/spatialNav';
import { isTextEntryTarget } from '../platform/webosKeys';

export function useGamepadNav({
  nativePlayerActive,
  goBack,
}: {
  nativePlayerActive: boolean;
  navigateRoute: (route: NavRoute) => void;
  goBack: () => void;
}) {
  useEffect(() => {
    return onGamepadAction((action) => {
      if (nativePlayerActive) return;
      if (isTextEntryTarget(document.activeElement)) return;

      if (action === 'back') { goBack(); return; }
      if (action === 'enter') {
        const active = document.activeElement;
        if (active instanceof HTMLElement) active.click();
        return;
      }
      const direction = action === 'up' || action === 'down' || action === 'left' || action === 'right' ? action : null;
      if (!direction) return;
      const current = isNavCard(document.activeElement) ? document.activeElement : null;
      if (current) focusNearestCard(current, direction);
    });
  }, [nativePlayerActive, goBack]);
}
