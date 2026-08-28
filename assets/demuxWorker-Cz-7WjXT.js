(function() {
	var O = Object.defineProperty, I = (e, n, r) => () => {
		if (r) throw r[0];
		try {
			return e && (n = e(e = 0)), n;
		} catch (t) {
			throw r = [t], t;
		}
	}, L = (e, n) => {
		let r = {};
		for (var t in e) O(r, t, {
			get: e[t],
			enumerable: !0
		});
		return n || O(r, Symbol.toStringTag, { value: "Module" }), r;
	}, D = L({
		IncrementalMkvRemuxer: () => h,
		core_invoke: () => B,
		default: () => X,
		engine_complete_effect: () => F,
		engine_dispatch: () => j,
		engine_init: () => z,
		engine_snapshot: () => U,
		fluxa_core_version: () => C,
		initSync: () => G,
		remux_mkv_to_webm: () => P
	});
	function B(e, n) {
		let r, t;
		try {
			const s = w(e, i.__wbindgen_malloc, i.__wbindgen_realloc), c = f, o = w(n, i.__wbindgen_malloc, i.__wbindgen_realloc), _ = f, a = i.core_invoke(s, c, o, _);
			return r = a[0], t = a[1], u(a[0], a[1]);
		} finally {
			i.__wbindgen_free(r, t, 1);
		}
	}
	function F(e, n) {
		const r = w(n, i.__wbindgen_malloc, i.__wbindgen_realloc), t = f, s = i.engine_complete_effect(e, r, t);
		let c;
		return s[0] !== 0 && (c = u(s[0], s[1]), i.__wbindgen_free(s[0], s[1] * 1, 1)), c;
	}
	function j(e, n) {
		const r = w(n, i.__wbindgen_malloc, i.__wbindgen_realloc), t = f, s = i.engine_dispatch(e, r, t);
		let c;
		return s[0] !== 0 && (c = u(s[0], s[1]), i.__wbindgen_free(s[0], s[1] * 1, 1)), c;
	}
	function z(e) {
		const n = w(e, i.__wbindgen_malloc, i.__wbindgen_realloc), r = f;
		return i.engine_init(n, r);
	}
	function U(e) {
		const n = i.engine_snapshot(e);
		let r;
		return n[0] !== 0 && (r = u(n[0], n[1]), i.__wbindgen_free(n[0], n[1] * 1, 1)), r;
	}
	function C() {
		let e, n;
		try {
			const r = i.fluxa_core_version();
			return e = r[0], n = r[1], u(r[0], r[1]);
		} finally {
			i.__wbindgen_free(e, n, 1);
		}
	}
	function P(e) {
		const n = S(e, i.__wbindgen_malloc), r = f, t = i.remux_mkv_to_webm(n, r);
		if (t[3]) throw $(t[2]);
		var s = A(t[0], t[1]).slice();
		return i.__wbindgen_free(t[0], t[1] * 1, 1), s;
	}
	function W() {
		return {
			__proto__: null,
			"./fluxa_core_bg.js": {
				__proto__: null,
				__wbg___wbindgen_is_undefined_67b456be8673d3d7: function(e) {
					return e === void 0;
				},
				__wbg___wbindgen_throw_1506f2235d1bdba0: function(e, n) {
					throw new Error(u(e, n));
				},
				__wbg_getTime_00b3f7db575e4ef5: function(e) {
					return e.getTime();
				},
				__wbg_getTimezoneOffset_08e2892156231088: function(e) {
					return e.getTimezoneOffset();
				},
				__wbg_new_0_445c13a750296eb6: function() {
					return /* @__PURE__ */ new Date();
				},
				__wbg_new_6d75fd236f920a62: function(e) {
					return new Date(e);
				},
				__wbg_new_with_year_month_day_hr_min_sec_c556132f181b08c9: function(e, n, r, t, s, c) {
					return new Date(e >>> 0, n, r, t, s, c);
				},
				__wbg_now_e7c6795a7f81e10f: function(e) {
					return e.now();
				},
				__wbg_performance_3fcf6e32a7e1ed0a: function(e) {
					return e.performance;
				},
				__wbg_static_accessor_GLOBAL_9d53f2689e622ca1: function() {
					const e = typeof global > "u" ? null : global;
					return m(e) ? 0 : p(e);
				},
				__wbg_static_accessor_GLOBAL_THIS_a1a35cec07001a8a: function() {
					const e = typeof globalThis > "u" ? null : globalThis;
					return m(e) ? 0 : p(e);
				},
				__wbg_static_accessor_SELF_4c59f6c7ea29a144: function() {
					const e = typeof self > "u" ? null : self;
					return m(e) ? 0 : p(e);
				},
				__wbg_static_accessor_WINDOW_e70ae9f2eb052253: function() {
					const e = typeof window > "u" ? null : window;
					return m(e) ? 0 : p(e);
				},
				__wbindgen_cast_0000000000000001: function(e) {
					return e;
				},
				__wbindgen_cast_0000000000000002: function(e, n) {
					return u(e, n);
				},
				__wbindgen_init_externref_table: function() {
					const e = i.__wbindgen_externrefs, n = e.grow(4);
					e.set(0, void 0), e.set(n + 0, void 0), e.set(n + 1, null), e.set(n + 2, !0), e.set(n + 3, !1);
				}
			}
		};
	}
	function p(e) {
		const n = i.__externref_table_alloc();
		return i.__wbindgen_externrefs.set(n, e), n;
	}
	function A(e, n) {
		return e = e >>> 0, l().subarray(e / 1, e / 1 + n);
	}
	function u(e, n) {
		return N(e >>> 0, n);
	}
	function l() {
		return (b === null || b.byteLength === 0) && (b = new Uint8Array(i.memory.buffer)), b;
	}
	function m(e) {
		return e == null;
	}
	function S(e, n) {
		const r = n(e.length * 1, 1) >>> 0;
		return l().set(e, r / 1), f = e.length, r;
	}
	function w(e, n, r) {
		if (r === void 0) {
			const _ = g.encode(e), a = n(_.length, 1) >>> 0;
			return l().subarray(a, a + _.length).set(_), f = _.length, a;
		}
		let t = e.length, s = n(t, 1) >>> 0;
		const c = l();
		let o = 0;
		for (; o < t; o++) {
			const _ = e.charCodeAt(o);
			if (_ > 127) break;
			c[s + o] = _;
		}
		if (o !== t) {
			o !== 0 && (e = e.slice(o)), s = r(s, t, t = o + e.length * 3, 1) >>> 0;
			const _ = l().subarray(s + o, s + t), a = g.encodeInto(e, _);
			o += a.written, s = r(s, t, o, 1) >>> 0;
		}
		return f = o, s;
	}
	function $(e) {
		const n = i.__wbindgen_externrefs.get(e);
		return i.__externref_table_dealloc(e), n;
	}
	function N(e, n) {
		return v += n, v >= R && (d = new TextDecoder("utf-8", {
			ignoreBOM: !0,
			fatal: !0
		}), d.decode(), v = n), d.decode(l().subarray(e, e + n));
	}
	function M(e, n) {
		return i = e.exports, b = null, i.__wbindgen_start(), i;
	}
	async function q(e, n) {
		if (typeof Response == "function" && e instanceof Response) {
			if (!e.ok) throw new Error(`failed to fetch Wasm: ${e.status} ${e.statusText} fetching '${e.url}'`);
			if (typeof WebAssembly.instantiateStreaming == "function") try {
				return await WebAssembly.instantiateStreaming(e, n);
			} catch (s) {
				if (r(e.type) && e.headers.get("Content-Type") !== "application/wasm") console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", s);
				else throw s;
			}
			const t = await e.arrayBuffer();
			return await WebAssembly.instantiate(t, n);
		} else {
			const t = await WebAssembly.instantiate(e, n);
			return t instanceof WebAssembly.Instance ? {
				instance: t,
				module: e
			} : t;
		}
		function r(t) {
			switch (t) {
				case "basic":
				case "cors":
				case "default": return !0;
			}
			return !1;
		}
	}
	function G(e) {
		if (i !== void 0) return i;
		e !== void 0 && (Object.getPrototypeOf(e) === Object.prototype ? {module: e} = e : console.warn("using deprecated parameters for `initSync()`; pass a single object instead"));
		const n = W();
		return e instanceof WebAssembly.Module || (e = new WebAssembly.Module(e)), M(new WebAssembly.Instance(e, n), e);
	}
	async function X(e) {
		if (i !== void 0) return i;
		e !== void 0 && (Object.getPrototypeOf(e) === Object.prototype ? {module_or_path: e} = e : console.warn("using deprecated parameters for the initialization function; pass a single object instead")), e === void 0 && (e = new URL("/fluxa/assets/fluxa_core_bg-C9pmX7AK.wasm", "" + self.location.href));
		const n = W();
		(typeof e == "string" || typeof Request == "function" && e instanceof Request || typeof URL == "function" && e instanceof URL) && (e = fetch(e));
		const { instance: r, module: t } = await q(await e, n);
		return M(r, t);
	}
	var h, T, b, d, R, v, g, f, i, H = I((() => {
		h = class {
			__destroy_into_raw() {
				const e = this.__wbg_ptr;
				return this.__wbg_ptr = 0, T.unregister(this), e;
			}
			free() {
				const e = this.__destroy_into_raw();
				i.__wbg_incrementalmkvremuxer_free(e, 0);
			}
			finish() {
				const e = i.incrementalmkvremuxer_finish(this.__wbg_ptr);
				var n = A(e[0], e[1]).slice();
				return i.__wbindgen_free(e[0], e[1] * 1, 1), n;
			}
			constructor() {
				const e = i.incrementalmkvremuxer_new();
				return this.__wbg_ptr = e, T.register(this, this.__wbg_ptr, this), this;
			}
			push(e) {
				const n = S(e, i.__wbindgen_malloc), r = f, t = i.incrementalmkvremuxer_push(this.__wbg_ptr, n, r);
				var s = A(t[0], t[1]).slice();
				return i.__wbindgen_free(t[0], t[1] * 1, 1), s;
			}
		}, Symbol.dispose && (h.prototype[Symbol.dispose] = h.prototype.free), T = typeof FinalizationRegistry > "u" ? {
			register: () => {},
			unregister: () => {}
		} : new FinalizationRegistry((e) => i.__wbg_incrementalmkvremuxer_free(e, 1)), b = null, d = new TextDecoder("utf-8", {
			ignoreBOM: !0,
			fatal: !0
		}), d.decode(), R = 2146435072, v = 0, g = new TextEncoder(), "encodeInto" in g || (g.encodeInto = function(e, n) {
			const r = g.encode(e);
			return n.set(r), {
				read: e.length,
				written: r.length
			};
		}), f = 0;
	}));
	let k = null;
	function K() {
		return k || (k = Promise.resolve().then(() => (H(), D)).then(async (e) => (await e.default(), e))), k;
	}
	function x(e, n = []) {
		self.postMessage(e, n);
	}
	self.onmessage = async (e) => {
		const { id: n, url: r } = e.data;
		try {
			const t = new (await (K())).IncrementalMkvRemuxer(), s = await fetch(r);
			if (!s.ok || !s.body) throw new Error(`fetch failed: ${s.status}`);
			const c = s.body.getReader();
			for (;;) {
				const { done: _, value: a } = await c.read();
				if (_) break;
				const y = t.push(a);
				if (y.length > 0) {
					const E = y.buffer.slice(y.byteOffset, y.byteOffset + y.byteLength);
					x({
						id: n,
						type: "segment",
						webmBytes: E
					}, [E]);
				}
			}
			const o = t.finish();
			if (o.length > 0) {
				const _ = o.buffer.slice(o.byteOffset, o.byteOffset + o.byteLength);
				x({
					id: n,
					type: "segment",
					webmBytes: _
				}, [_]);
			}
			x({
				id: n,
				type: "done"
			});
		} catch (t) {
			x({
				id: n,
				type: "error",
				error: t instanceof Error ? t.message : String(t)
			});
		}
	};
})();
