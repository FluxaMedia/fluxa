use super::EFFECT_EXPIRY;
use super::HeadlessEngine;
use super::state::GenerationKey;
use crate::runtime::{EffectEnvelope, EffectKind};
use serde_json::Value;
use web_time::Instant;

impl HeadlessEngine {
    pub(super) fn effect<P: serde::Serialize>(
        &mut self,
        kind: EffectKind,
        generation: u64,
        payload: P,
    ) -> EffectEnvelope {
        let payload = serde_json::to_value(&payload).unwrap_or(Value::Null);
        let id = format!("fx-{}", self.next_effect_id);
        self.next_effect_id += 1;
        let envelope = EffectEnvelope::new(id.clone(), kind, generation, payload);
        self.register_effect(id, envelope.clone());
        envelope
    }

    fn register_effect(&mut self, id: String, envelope: EffectEnvelope) {
        self.pending_effects.push(envelope);
        self.effect_created_at.insert(id, Instant::now());
    }

    pub(super) fn take_pending_effect(&mut self, id: &str) -> Option<EffectEnvelope> {
        let index = self
            .pending_effects
            .iter()
            .position(|effect| effect.id == id)?;
        let effect = self.pending_effects.remove(index);
        self.delivered_effect_ids.remove(id);
        self.effect_created_at.remove(id);
        Some(effect)
    }

    // Drops any pending effect old enough that it's almost certainly been abandoned by
    // the platform rather than genuinely still in flight. Called opportunistically on
    // every dispatch/complete_effect so no background timer is needed.
    pub(super) fn expire_stale_pending_effects(&mut self, now: Instant) {
        let stale_ids: Vec<String> = self
            .pending_effects
            .iter()
            .filter(|effect| {
                self.effect_created_at
                    .get(&effect.id)
                    .is_some_and(|created_at| now.duration_since(*created_at) > EFFECT_EXPIRY)
            })
            .map(|effect| effect.id.clone())
            .collect();
        for id in &stale_ids {
            self.pending_effects.retain(|effect| &effect.id != id);
            self.delivered_effect_ids.remove(id);
            self.effect_created_at.remove(id);
        }
    }

    pub(super) fn bump_generation(&mut self, key: GenerationKey) -> u64 {
        self.state.runtime.bump(key)
    }

    // When a dispatch/complete_effect handler produces no new effects directly, we
    // fall back to draining whatever's still pending so the platform doesn't lose
    // track of multi-effect work spread across several calls. But anything already
    // handed to the platform is presumably still in flight (e.g. an addon fetch that
    // hasn't finished) — redelivering it here would make the platform start a second,
    // duplicate execution of the same effect. Only ever drain genuinely undelivered ones.
    pub(super) fn resolve_visible_effects(
        &mut self,
        effects: Vec<EffectEnvelope>,
    ) -> Vec<EffectEnvelope> {
        let visible = if effects.is_empty() {
            self.undelivered_pending_effects()
        } else {
            effects
        };
        for effect in &visible {
            self.delivered_effect_ids.insert(effect.id.clone());
        }
        visible
    }

    pub(super) fn undelivered_pending_effects(&self) -> Vec<EffectEnvelope> {
        self.pending_effects
            .iter()
            .filter(|effect| !self.delivered_effect_ids.contains(&effect.id))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless_engine::contracts::{EffectResultInput, EffectStatus};
    use serde_json::json;

    #[test]
    fn typed_effects_use_their_schedule_metadata() {
        let mut engine = HeadlessEngine::default();

        let effect = engine.effect(
            EffectKind::FetchCatalogPage,
            3,
            json!({ "url": "https://example.com/catalog" }),
        );

        assert_eq!(effect.group_id.as_deref(), Some("addon"));
        assert_eq!(effect.priority, 50);
        assert_eq!(effect.cache_policy.as_deref(), Some("default"));
        assert_eq!(effect.timeout_ms, Some(15_000));
        assert!(effect.dedupe_key.is_some());
    }

    #[test]
    fn completed_effect_removes_the_runtime_entry() {
        let mut engine = HeadlessEngine::default();
        let effect = engine.effect(EffectKind::FetchCatalogPage, 1, json!({}));

        let effects = engine.complete_effect(EffectResultInput {
            effect_id: effect.id,
            status: EffectStatus::Ok,
            value: serde_json::Value::Null,
            error: serde_json::Value::Null,
        });

        assert!(effects.is_empty());
        assert!(engine.pending_effects.is_empty());
        assert!(engine.delivered_effect_ids.is_empty());
        assert!(engine.effect_created_at.is_empty());
    }
}
