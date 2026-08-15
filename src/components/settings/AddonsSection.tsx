import React, { useEffect, useState } from "react";
import { platformOpenExternal } from '../../platform/browser';
import {
  addonKey,
  addonLogo,
  addonName,
  addonTypes,
  addonVersion,
} from "../../core/addons";
import { coreInvoke } from "../../core/engine";
import { fetchJson } from "../../core/httpClient";
import type { AddonDescriptor } from "../../core/types";
import { t } from "../../i18n";
import { ExtensionIcon, SettingsSection } from "./SettingsUI";
import { FONT } from "./settingsStyles";
import type { Prefs } from "./settingsTypes";

const UPTIME_STATUS_URL = "https://uptime.ibbylabs.dev/v1/status";
const UPTIME_REFRESH_MS = 2 * 60 * 1000;

type UptimeService = {
  name: string;
  url: string;
  uptimePercent?: number;
  uptimeWindows?: { h24?: number | null };
  last?: {
    state?: string;
    up?: boolean;
    latency?: number | null;
    checkedAt?: string;
  };
};

function UptimeBadge({ service }: { service: UptimeService | null }) {
  if (!service) return null;
  const state = service.last?.state?.toUpperCase() ?? "UNKNOWN";
  const colors = state === "UP"
    ? { dot: "#69D38C", text: "#9BE8B2" }
    : state === "DOWN"
    ? { dot: "#FF7171", text: "#FF9B9B" }
    : state === "DEGRADED"
    ? { dot: "#F6C65B", text: "#F7D98C" }
    : { dot: "rgba(255,255,255,0.45)", text: "rgba(255,255,255,0.55)" };
  const label = state === "UP"
    ? t("addons.uptime_up")
    : state === "DOWN"
    ? t("addons.uptime_down")
    : state === "DEGRADED"
    ? t("addons.uptime_degraded")
    : t("addons.uptime_unknown");
  const latency = service.last?.latency;
  const uptime24h = service.uptimeWindows?.h24;
  const title = [
    service.name,
    latency != null ? `${latency} ms` : null,
    uptime24h != null ? `${uptime24h}% ${t("addons.uptime_24h")}` : null,
  ].filter(Boolean).join(" · ");
  return (
    <span
      title={title}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: "0.3125rem",
        color: colors.text,
        fontSize: "0.6875rem",
        fontWeight: 650,
        fontFamily: FONT,
        whiteSpace: "nowrap",
      }}
    >
      <span
        style={{
          width: "0.4375rem",
          height: "0.4375rem",
          borderRadius: "50%",
          background: colors.dot,
          boxShadow: `0 0 0.375rem ${colors.dot}`,
        }}
      />
      {label}
      {latency != null && ` · ${latency}ms`}
    </span>
  );
}

function AddonToggle(
  { enabled, onChange }: { enabled: boolean; onChange: () => void },
) {
  const [hovered, setHovered] = useState(false);
  return (
    <button
      onClick={onChange}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: "2.75rem",
        height: "1.625rem",
        borderRadius: "0.8125rem",
        border: "none",
        cursor: "pointer",
        padding: 0,
        flexShrink: 0,
        background: enabled
          ? (hovered ? "#e0e0e0" : "#ffffff")
          : (hovered ? "rgba(255,255,255,0.18)" : "rgba(255,255,255,0.14)"),
        transition: "background 0.18s",
        position: "relative",
        outline: "none",
      }}
    >
      <div
        style={{
          position: "absolute",
          top: "0.1875rem",
          left: enabled ? 21 : 3,
          width: "1.25rem",
          height: "1.25rem",
          borderRadius: "50%",
          background: enabled ? "#111" : "rgba(255,255,255,0.6)",
          transition: "left 0.18s",
        }}
      />
    </button>
  );
}

function IconBtn({
  title,
  disabled,
  destructive,
  onClick,
  children,
}: {
  title: string;
  disabled?: boolean;
  destructive?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  const [hovered, setHovered] = useState(false);
  const activeHover = hovered && !disabled;
  return (
    <button
      title={title}
      onClick={onClick}
      disabled={disabled}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        width: "1.875rem",
        height: "1.875rem",
        borderRadius: "0.4375rem",
        border: "none",
        padding: 0,
        background: activeHover
          ? (destructive ? "rgba(255,60,60,0.12)" : "rgba(255,255,255,0.08)")
          : "transparent",
        color: disabled
          ? "rgba(255,255,255,0.15)"
          : activeHover
          ? (destructive ? "rgba(255,100,100,0.9)" : "rgba(255,255,255,0.9)")
          : (destructive ? "rgba(255,255,255,0.22)" : "rgba(255,255,255,0.38)"),
        cursor: disabled ? "default" : "pointer",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        transition: "background 0.12s, color 0.12s",
        flexShrink: 0,
        outline: "none",
      }}
    >
      {children}
    </button>
  );
}

function Tag(
  { children, muted }: { children: React.ReactNode; muted?: boolean },
) {
  return (
    <span
      style={{
        fontSize: "0.6875rem",
        fontWeight: 500,
        fontFamily: FONT,
        color: muted ? "rgba(255,255,255,0.32)" : "rgba(255,255,255,0.55)",
        background: muted ? "rgba(255,255,255,0.04)" : "rgba(255,255,255,0.07)",
        border: `1px solid ${
          muted ? "rgba(255,255,255,0.06)" : "rgba(255,255,255,0.09)"
        }`,
        borderRadius: "0.3125rem",
        padding: "0.125rem 0.4375rem",
      }}
    >
      {children}
    </span>
  );
}

function AddonTile({
  addon,
  enabled,
  isFirst,
  isLast,
  uptimeService,
  onRemove,
  onToggle,
  onMoveUp,
  onMoveDown,
  onRefresh,
}: {
  addon: AddonDescriptor;
  enabled: boolean;
  isFirst: boolean;
  isLast: boolean;
  uptimeService: UptimeService | null;
  onRemove: () => void;
  onToggle: () => void;
  onMoveUp: () => void;
  onMoveDown: () => void;
  onRefresh: () => void;
}) {
  const [imgFailed, setImgFailed] = useState(false);
  const name = addonName(addon);
  const logo = addonLogo(addon);
  const version = addonVersion(addon);
  const types = addonTypes(addon);
  const description = addon.manifest?.description ?? addon.description ?? null;
  const resourceCount =
    (addon.manifest?.resources ?? addon.resources ?? []).length;
  const isConfigurable = addon.manifest?.configurable ??
    addon.behaviorHints?.configurable ?? false;

  const handleConfigure = () => {
    const base = (addon.transportUrl ?? "").replace(/\/manifest\.json$/, "");
    if (base) platformOpenExternal(`${base}/configure`).catch(() => {});
  };

  return (
    <div
      style={{
        background: "#1A1A1A",
        border: "1px solid rgba(255,255,255,0.09)",
        borderRadius: "0.75rem",
        overflow: "hidden",
        opacity: enabled ? 1 : 0.5,
        transition: "opacity 0.2s",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "0.6875rem",
          padding: "0.8125rem 0.8125rem 0.75rem",
        }}
      >
        {logo && !imgFailed
          ? (
            <img
              src={logo}
              alt=""
              onError={() => setImgFailed(true)}
              style={{
                width: "2.75rem",
                height: "2.75rem",
                borderRadius: "0.625rem",
                objectFit: "contain",
                background: "rgba(255,255,255,0.05)",
                flexShrink: 0,
                border: "1px solid rgba(255,255,255,0.07)",
              }}
            />
          )
          : (
            <div
              style={{
                width: "2.75rem",
                height: "2.75rem",
                borderRadius: "0.625rem",
                flexShrink: 0,
                background: "rgba(255,255,255,0.06)",
                border: "1px solid rgba(255,255,255,0.08)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                color: "rgba(255,255,255,0.45)",
              }}
            >
              <ExtensionIcon />
            </div>
          )}

        <div style={{ flex: 1, minWidth: 0 }}>
          <p
            style={{
              color: "rgba(255,255,255,0.92)",
              fontSize: "0.875rem",
              fontWeight: 600,
              margin: 0,
              fontFamily: FONT,
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap",
            }}
          >
            {name}
          </p>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "0.5rem",
              marginTop: "0.125rem",
            }}
          >
            {version && (
              <p
                style={{
                  color: "rgba(255,255,255,0.30)",
                  fontSize: "0.6875rem",
                  margin: 0,
                  fontFamily: FONT,
                }}
              >
                {version}
              </p>
            )}
            <UptimeBadge service={uptimeService} />
          </div>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 1 }}>
          {isConfigurable && (
            <IconBtn title={t("common.configure")} onClick={handleConfigure}>
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
            </IconBtn>
          )}
          <IconBtn title={t("common.refresh")} onClick={onRefresh}>
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="23 4 23 10 17 10" />
              <polyline points="1 20 1 14 7 14" />
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15" />
            </svg>
          </IconBtn>
          <IconBtn
            title={t("common.move_up")}
            disabled={isFirst}
            onClick={onMoveUp}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="18 15 12 9 6 15" />
            </svg>
          </IconBtn>
          <IconBtn
            title={t("common.move_down")}
            disabled={isLast}
            onClick={onMoveDown}
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </IconBtn>
          <IconBtn title={t("common.forget")} destructive onClick={onRemove}>
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6" />
              <path d="M10 11v6" />
              <path d="M14 11v6" />
              <path d="M9 6V4h6v2" />
            </svg>
          </IconBtn>
        </div>

        <AddonToggle enabled={enabled} onChange={onToggle} />
      </div>

      {(description || types.length > 0 || resourceCount > 0) && (
        <>
          <div
            style={{
              height: 1,
              background: "rgba(255,255,255,0.055)",
              margin: "0 0.8125rem",
            }}
          />
          <div style={{ padding: "0.625rem 0.8125rem 0.75rem" }}>
            {description && (
              <p
                style={{
                  color: "rgba(255,255,255,0.38)",
                  fontSize: "0.75rem",
                  lineHeight: 1.55,
                  margin: "0 0 0.5625rem",
                  fontFamily: FONT,
                  display: "-webkit-box",
                  WebkitLineClamp: 2,
                  WebkitBoxOrient: "vertical",
                  overflow: "hidden",
                }}
              >
                {description}
              </p>
            )}
            <div
              style={{ display: "flex", flexWrap: "wrap", gap: "0.3125rem" }}
            >
              <Tag>{enabled ? "Active" : "Inactive"}</Tag>
              {resourceCount > 0 && (
                <Tag muted>
                  {resourceCount} resource{resourceCount !== 1 ? "s" : ""}
                </Tag>
              )}
              {isConfigurable && <Tag muted>Configurable</Tag>}
              {types.slice(0, 4).map((type) => (
                <Tag key={type} muted>{type}</Tag>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

export function AddonsSection({
  prefs: _prefs,
  setPref: _setPref,
  addonUrl,
  setAddonUrl,
  installedAddons,
  disabledAddonKeys,
  installLoading,
  installError,
  onInstall,
  onRemove,
  onToggle,
  onReorder,
  onDispatch,
}: {
  prefs: Prefs;
  setPref: <K extends keyof Prefs>(k: K, v: Prefs[K]) => void;
  addonUrl: string;
  setAddonUrl: (v: string) => void;
  installedAddons: AddonDescriptor[];
  disabledAddonKeys: string[];
  installLoading: boolean;
  installError: string | null;
  onInstall: () => void;
  onRemove: (a: AddonDescriptor) => void;
  onToggle: (a: AddonDescriptor) => void;
  onReorder: (a: AddonDescriptor, dir: "up" | "down") => void;
  onDispatch: (actionJson: string) => void;
}) {
  const [addonUptimes, setAddonUptimes] = useState<
    Record<string, UptimeService>
  >({});
  useEffect(() => {
    let active = true;
    const refresh = async () => {
      try {
        const status = await fetchJson(UPTIME_STATUS_URL) as {
          services?: UptimeService[];
        };
        const matches = await coreInvoke<
          Array<{ id: string; service: UptimeService }>
        >(
          "addonUptimeMatchPlan",
          JSON.stringify({
            addons: installedAddons.map((addon) => ({
              id: addonKey(addon),
              url: addon.transportUrl,
            })),
            services: status.services ?? [],
          }),
        );
        if (active) {
          setAddonUptimes(
            Object.fromEntries(
              (matches ?? []).map((match) => [match.id, match.service]),
            ),
          );
        }
      } catch {
        if (active) setAddonUptimes({});
      }
    };
    void refresh();
    const interval = window.setInterval(
      () => void refresh(),
      UPTIME_REFRESH_MS,
    );
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [installedAddons]);
  return (
    <>
      <SettingsSection
        title={t("addons.install")}
        subtitle={t("auto.installed_add_ons_and_settings")}
      >
        <div
          style={{
            padding: "0.875rem 1rem",
            borderBottom: "1px solid rgba(255,255,255,0.055)",
          }}
        >
          <input
            type="text"
            value={addonUrl}
            onChange={(e) => setAddonUrl(e.target.value)}
            placeholder={t("addons.install_placeholder")}
            onKeyDown={(e) => e.key === "Enter" && onInstall()}
            disabled={installLoading}
            style={{
              width: "100%",
              boxSizing: "border-box",
              background: "rgba(255,255,255,0.05)",
              border: "1px solid rgba(255,255,255,0.10)",
              borderRadius: "0.5rem",
              padding: "0.6875rem 0.8125rem",
              color: "#FFFFFF",
              fontSize: "0.8125rem",
              fontFamily: FONT,
              outline: "none",
              marginBottom: installError ? 8 : 10,
              opacity: installLoading ? 0.65 : 1,
            }}
          />
          {installError && (
            <p
              style={{
                color: "#FF6B6B",
                fontSize: "0.75rem",
                lineHeight: 1.45,
                margin: "0 0 0.625rem",
                fontFamily: FONT,
              }}
            >
              {installError}
            </p>
          )}
          <button
            onClick={onInstall}
            disabled={!addonUrl.trim() || installLoading}
            style={{
              background: addonUrl.trim() && !installLoading
                ? "#FFFFFF"
                : "rgba(255,255,255,0.10)",
              color: addonUrl.trim() && !installLoading
                ? "#000000"
                : "rgba(255,255,255,0.35)",
              border: "none",
              borderRadius: "0.5rem",
              padding: "0.5rem 1.125rem",
              fontSize: "0.8125rem",
              fontWeight: 500,
              fontFamily: FONT,
              cursor: addonUrl.trim() && !installLoading
                ? "pointer"
                : "default",
              transition: "background 0.12s, color 0.12s",
            }}
          >
            {installLoading ? "Installing…" : t("addons.install")}
          </button>
        </div>
      </SettingsSection>

      {installedAddons.length > 0
        ? (
          <SettingsSection
            title={`${t("addons.installed")} (${installedAddons.length})`}
            subtitle={t("auto.installed_add_ons_and_settings")}
          >
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "0.5rem",
                padding: "0.5rem",
              }}
            >
              {installedAddons.map((addon, idx) => (
                <AddonTile
                  key={addonKey(addon)}
                  addon={addon}
                  enabled={!disabledAddonKeys.includes(addonKey(addon))}
                  isFirst={idx === 0}
                  isLast={idx === installedAddons.length - 1}
                  uptimeService={addonUptimes[addonKey(addon)] ?? null}
                  onRemove={() => onRemove(addon)}
                  onToggle={() => onToggle(addon)}
                  onMoveUp={() => onReorder(addon, "up")}
                  onMoveDown={() => onReorder(addon, "down")}
                  onRefresh={() =>
                    onDispatch(
                      JSON.stringify({
                        type: "addonsRefreshRequested",
                        forceRefresh: true,
                      }),
                    )}
                />
              ))}
            </div>
          </SettingsSection>
        )
        : (
          <div style={{ padding: "0.875rem 1rem" }}>
            <p
              style={{
                color: "rgba(255,255,255,0.32)",
                fontSize: "0.8125rem",
                margin: 0,
                fontFamily: FONT,
              }}
            >
              {t("addons.no_addons")}
            </p>
          </div>
        )}
    </>
  );
}
