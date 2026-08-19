import React, { useState } from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DiscoverScreen } from './DiscoverScreen';
import type { AppState, DiscoverCatalog } from '../core/types';

vi.mock('../core/engine', () => ({
  coreInvoke: async (method: string, argsJson: string) => {
    const args = JSON.parse(argsJson);
    if (method === 'discoverContentTypes') return ['movie', 'series'];
    if (method === 'discoverSelectionPlan') {
      const catalogs = args.catalogs.filter((catalog: DiscoverCatalog) => catalog.type === args.contentType);
      const selectedCatalog = catalogs.find((catalog: DiscoverCatalog) => catalog.key === args.selectedCatalogKey) ?? catalogs[0] ?? null;
      return {
        catalogs,
        selectedCatalogKey: selectedCatalog?.key ?? null,
        selectedCatalog,
        selectedExtra: null,
        extraValue: null,
        key: `${selectedCatalog?.key ?? ''}||`,
      };
    }
    if (method === 'mergeDiscoverPages') return { items: args.baseItems, appendedItems: [], exhausted: true };
    return null;
  },
}));

const MOVIE_CATALOGS: DiscoverCatalog[] = [{ key: 'cinemeta-movie-top', label: 'Cinemeta: Popular', type: 'movie', extras: [] }];

const SERIES_CATALOGS: DiscoverCatalog[] = [{ key: 'cinemeta-series-top', label: 'Cinemeta: Popular', type: 'series', extras: [] }];

function baseState(): AppState {
  return {
    navigation: { route: 'discover' },
    home: {} as AppState['home'],
    detail: {} as AppState['detail'],
    search: {} as AppState['search'],
    player: {} as AppState['player'],
    library: {} as AppState['library'],
    discover: { catalogs: MOVIE_CATALOGS, results: [], isLoading: false, catalogsLoading: false },
    addons: {
      installed: [
        {
          manifest: { catalogs: [{ type: 'movie' }, { type: 'series' }] },
        },
      ] as unknown as AppState['addons']['installed'],
    },
    settings: {},
  } as unknown as AppState;
}

function Harness({ catalogFetchDelayMs }: { catalogFetchDelayMs: number }) {
  const [state, setState] = useState<AppState>(baseState());

  const onDispatch = (actionJson: string) => {
    const action = JSON.parse(actionJson) as { type: string; contentType?: string };
    if (action.type === 'discoverCatalogFiltersRequested') {
      const forType = action.contentType;
      setTimeout(() => {
        setState((prev) => ({
          ...prev,
          discover: {
            ...prev.discover,
            catalogs: forType === 'series' ? SERIES_CATALOGS : MOVIE_CATALOGS,
          },
        }));
      }, catalogFetchDelayMs);
    }
  };

  return <DiscoverScreen state={state} onDispatch={onDispatch} onNavigateDetail={() => {}} onBack={() => {}} />;
}

function openPopoverMenu() {
  return document.body.querySelector('.ui-popover') as HTMLElement;
}

async function switchToSeries(user: ReturnType<typeof userEvent.setup>) {
  const typeDropdown = screen.getByText('Movies').closest('button')!;
  await user.click(typeDropdown);
  const menu = openPopoverMenu();
  await user.click(within(menu).getByText('Series'));
}

describe('DiscoverScreen: switching content type', () => {
  it('shows the series catalog once the async catalog fetch resolves, never leaving the dropdown stuck empty', async () => {
    const user = userEvent.setup();
    render(<Harness catalogFetchDelayMs={50} />);

    await switchToSeries(user);

    await waitFor(() => {
      expect(screen.getByText('Cinemeta: Popular')).toBeInTheDocument();
    });

    const catalogDropdown = screen.getByText('Cinemeta: Popular').closest('button')!;
    await user.click(catalogDropdown);
    const menu = openPopoverMenu();
    expect(within(menu).getByText('Cinemeta: Popular')).toBeInTheDocument();
  });

  it('never lets a stale movie catalog key survive the switch to Series (the actual bug)', async () => {
    const user = userEvent.setup();
    render(<Harness catalogFetchDelayMs={10} />);

    await switchToSeries(user);

    await waitFor(() => {
      expect(screen.queryByText('No content found')).not.toBeInTheDocument();
    });
    expect(screen.getByText('Cinemeta: Popular')).toBeInTheDocument();
  });
});
