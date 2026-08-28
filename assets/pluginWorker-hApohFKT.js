(function() {
	var Mu = Object.create, vt = Object.defineProperty, Gu = Object.getOwnPropertyDescriptor, Hu = Object.getOwnPropertyNames, Lu = Object.getPrototypeOf, Xn = Object.prototype.hasOwnProperty, Se = (e, t, n) => () => {
		if (n) throw n[0];
		try {
			return e && (t = e(e = 0)), t;
		} catch (r) {
			throw n = [r], r;
		}
	}, L = (e, t) => () => (t || (e((t = { exports: {} }).exports, t), e = null), t.exports), Me = (e, t) => {
		let n = {};
		for (var r in e) vt(n, r, {
			get: e[r],
			enumerable: !0
		});
		return t || vt(n, Symbol.toStringTag, { value: "Module" }), n;
	}, Gs = (e, t, n, r) => {
		if (t && typeof t == "object" || typeof t == "function") for (var s = Hu(t), i = 0, o = s.length, a; i < o; i++) a = s[i], !Xn.call(e, a) && a !== n && vt(e, a, {
			get: ((u) => t[u]).bind(null, a),
			enumerable: !(r = Gu(t, a)) || r.enumerable
		});
		return e;
	}, Ou = (e, t, n) => (n = e != null ? Mu(Lu(e)) : {}, Gs(t || !e || !e.__esModule || !Xn.call(e, "default") ? vt(n, "default", {
		value: e,
		enumerable: !0
	}) : n, e)), Ku = (e) => Xn.call(e, "module.exports") ? e["module.exports"] : Gs(vt({}, "__esModule", { value: !0 }), e);
	function _e(e) {
		return function(...t) {
			let n = e(...t);
			if (n && typeof n == "object" && n instanceof Promise) throw new Error("Function unexpectedly returned a Promise");
			return n;
		};
	}
	var je, qn, $t, Xe, Et, $n = Se((() => {
		je = {
			JS_EVAL_TYPE_GLOBAL: 0,
			JS_EVAL_TYPE_MODULE: 1,
			JS_EVAL_TYPE_DIRECT: 2,
			JS_EVAL_TYPE_INDIRECT: 3,
			JS_EVAL_TYPE_MASK: 3,
			JS_EVAL_FLAG_STRICT: 8,
			JS_EVAL_FLAG_STRIP: 16,
			JS_EVAL_FLAG_COMPILE_ONLY: 32,
			JS_EVAL_FLAG_BACKTRACE_BARRIER: 64
		}, qn = {
			BaseObjects: 1,
			Date: 2,
			Eval: 4,
			StringNormalize: 8,
			RegExp: 16,
			RegExpCompiler: 32,
			JSON: 64,
			Proxy: 128,
			MapSet: 256,
			TypedArrays: 512,
			Promise: 1024,
			BigInt: 2048,
			BigFloat: 4096,
			BigDecimal: 8192,
			OperatorOverloading: 16384,
			BignumExt: 32768
		}, $t = {
			Pending: 0,
			Fulfilled: 1,
			Rejected: 2
		}, Xe = {
			JS_GPN_STRING_MASK: 1,
			JS_GPN_SYMBOL_MASK: 2,
			JS_GPN_PRIVATE_MASK: 4,
			JS_GPN_ENUM_ONLY: 16,
			JS_GPN_SET_ENUM: 32,
			QTS_GPN_NUMBER_MASK: 64,
			QTS_STANDARD_COMPLIANT_NUMBER: 128
		}, Et = {
			IsStrictlyEqual: 0,
			IsSameValue: 1,
			IsSameValueZero: 2
		};
	}));
	function xt(...e) {
		en && console.log("quickjs-emscripten:", ...e);
	}
	function* Hs(e) {
		return yield e;
	}
	function Ju(e) {
		return Hs(zn(e));
	}
	function Ls(e, t) {
		return (...n) => zn(t.call(e, sn, ...n));
	}
	function Pu(e, t) {
		return zn(t.call(e, sn));
	}
	function zn(e) {
		function t(n) {
			return n.done ? n.value : n.value instanceof Promise ? n.value.then((r) => t(e.next(r)), (r) => t(e.throw(r))) : t(e.next(n.value));
		}
		return t(e.next());
	}
	function er(e, t) {
		let n;
		try {
			e.dispose();
		} catch (r) {
			n = r;
		}
		if (t && n) throw Object.assign(t, {
			message: `${t.message}
 Then, failed to dispose scope: ${n.message}`,
			disposeError: n
		}), t;
		if (t || n) throw t || n;
	}
	function Wu(e) {
		let t = e ? Array.from(e) : [];
		function n() {
			return t.forEach((s) => s.alive ? s.dispose() : void 0);
		}
		function r() {
			return t.some((s) => s.alive);
		}
		return Object.defineProperty(t, on, {
			configurable: !0,
			enumerable: !1,
			value: n
		}), Object.defineProperty(t, "dispose", {
			configurable: !0,
			enumerable: !1,
			value: n
		}), Object.defineProperty(t, "alive", {
			configurable: !0,
			enumerable: !1,
			get: r
		}), t;
	}
	function zt(e) {
		return !!(e && (typeof e == "object" || typeof e == "function") && "alive" in e && typeof e.alive == "boolean" && "dispose" in e && typeof e.dispose == "function");
	}
	function Os(e) {
		if (!e) return 0;
		let t = 0;
		for (let [n, r] of Object.entries(e)) {
			if (!(n in qn)) throw new ir(n);
			r && (t |= qn[n]);
		}
		return t;
	}
	function Ks(e) {
		if (typeof e == "number") return e;
		if (e === void 0) return 0;
		let { type: t, strict: n, strip: r, compileOnly: s, backtraceBarrier: i } = e, o = 0;
		return t === "global" && (o |= je.JS_EVAL_TYPE_GLOBAL), t === "module" && (o |= je.JS_EVAL_TYPE_MODULE), n && (o |= je.JS_EVAL_FLAG_STRICT), r && (o |= je.JS_EVAL_FLAG_STRIP), s && (o |= je.JS_EVAL_FLAG_COMPILE_ONLY), i && (o |= je.JS_EVAL_FLAG_BACKTRACE_BARRIER), o;
	}
	function Yu(e) {
		if (typeof e == "number") return e;
		if (e === void 0) return 0;
		let { strings: t, symbols: n, quickjsPrivate: r, onlyEnumerable: s, numbers: i, numbersAsStrings: o } = e, a = 0;
		return t && (a |= Xe.JS_GPN_STRING_MASK), n && (a |= Xe.JS_GPN_SYMBOL_MASK), r && (a |= Xe.JS_GPN_PRIVATE_MASK), s && (a |= Xe.JS_GPN_ENUM_ONLY), i && (a |= Xe.QTS_GPN_NUMBER_MASK), o && (a |= Xe.QTS_STANDARD_COMPLIANT_NUMBER), a;
	}
	function Uu(...e) {
		let t = [];
		for (let n of e) n !== void 0 && (t = t.concat(n));
		return t;
	}
	function tr(e) {
		return e >> 8;
	}
	function Js(e, t) {
		t.interruptHandler && e.setInterruptHandler(t.interruptHandler), t.maxStackSizeBytes !== void 0 && e.setMaxStackSize(t.maxStackSizeBytes), t.memoryLimitBytes !== void 0 && e.setMemoryLimit(t.memoryLimitBytes);
	}
	function Ps(e, t) {
		t.moduleLoader && e.setModuleLoader(t.moduleLoader), t.shouldInterrupt && e.setInterruptHandler(t.shouldInterrupt), t.memoryLimitBytes !== void 0 && e.setMemoryLimit(t.memoryLimitBytes), t.maxStackSizeBytes !== void 0 && e.setMaxStackSize(t.maxStackSizeBytes);
	}
	var Ws, Ys, en, Us, tn, nr, nn, rn, rr, sr, Vs, Zs, ir, or, ar, ur, Fe, sn, Te, on, lr, j, Ge, Ar, ce, an, js, Xs, qe, qs, cr, $s, hr, dr, Tt, zs, gr, ei, fr, pr, ti, ni, ri, Zu = Se((() => {
		$n(), Ws = Object.defineProperty, Ys = (e, t) => {
			for (var n in t) Ws(e, n, {
				get: t[n],
				enumerable: !0
			});
		}, en = !1, Us = {}, Ys(Us, {
			QuickJSAsyncifyError: () => rr,
			QuickJSAsyncifySuspended: () => sr,
			QuickJSEmptyGetOwnPropertyNames: () => ar,
			QuickJSEmscriptenModuleError: () => Zs,
			QuickJSHostRefInvalid: () => Fe,
			QuickJSHostRefRangeExceeded: () => ur,
			QuickJSMemoryLeakDetected: () => Vs,
			QuickJSNotImplemented: () => rn,
			QuickJSPromisePending: () => or,
			QuickJSUnknownIntrinsic: () => ir,
			QuickJSUnwrapError: () => tn,
			QuickJSUseAfterFree: () => nn,
			QuickJSWrongOwner: () => nr
		}), tn = class extends Error {
			constructor(e, t) {
				let n = typeof e == "object" && e && "message" in e ? String(e.message) : String(e);
				super(n), this.cause = e, this.context = t, this.name = "QuickJSUnwrapError";
			}
		}, nr = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSWrongOwner";
			}
		}, nn = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSUseAfterFree";
			}
		}, rn = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSNotImplemented";
			}
		}, rr = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSAsyncifyError";
			}
		}, sr = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSAsyncifySuspended";
			}
		}, Vs = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSMemoryLeakDetected";
			}
		}, Zs = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSEmscriptenModuleError";
			}
		}, ir = class extends TypeError {
			constructor() {
				super(...arguments), this.name = "QuickJSUnknownIntrinsic";
			}
		}, or = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSPromisePending";
			}
		}, ar = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSEmptyGetOwnPropertyNames";
			}
		}, ur = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSHostRefRangeExceeded";
			}
		}, Fe = class extends Error {
			constructor() {
				super(...arguments), this.name = "QuickJSHostRefInvalid";
			}
		}, sn = Hs, sn.of = Ju, Te = class {
			[Symbol.dispose]() {
				return this.dispose();
			}
		}, on = Symbol.dispose ?? Symbol.for("Symbol.dispose"), lr = Te.prototype, lr[on] || (lr[on] = function() {
			return this.dispose();
		}), j = class Tu extends Te {
			constructor(t, n, r, s) {
				super(), this._value = t, this.copier = n, this.disposer = r, this._owner = s, this._alive = !0, this._constructorStack = en ? (/* @__PURE__ */ new Error("Lifetime constructed")).stack : void 0;
			}
			get alive() {
				return this._alive;
			}
			get value() {
				return this.assertAlive(), this._value;
			}
			get owner() {
				return this._owner;
			}
			get dupable() {
				return !!this.copier;
			}
			dup() {
				if (this.assertAlive(), !this.copier) throw new Error("Non-dupable lifetime");
				return new Tu(this.copier(this._value), this.copier, this.disposer, this._owner);
			}
			consume(t) {
				this.assertAlive();
				let n = t(this);
				return this.dispose(), n;
			}
			map(t) {
				return this.assertAlive(), t(this);
			}
			tap(t) {
				return t(this), this;
			}
			dispose() {
				this.assertAlive(), this.disposer && this.disposer(this._value), this._alive = !1;
			}
			assertAlive() {
				if (!this.alive) throw this._constructorStack ? new nn(`Lifetime not alive
${this._constructorStack}
Lifetime used`) : new nn("Lifetime not alive");
			}
		}, Ge = class extends j {
			constructor(e, t) {
				super(e, void 0, void 0, t);
			}
			get dupable() {
				return !0;
			}
			dup() {
				return this;
			}
			dispose() {}
		}, Ar = class extends j {
			constructor(e, t, n, r) {
				super(e, t, n, r);
			}
			dispose() {
				this._alive = !1;
			}
		}, ce = class jn extends Te {
			constructor() {
				super(...arguments), this._disposables = new j(/* @__PURE__ */ new Set()), this.manage = (t) => (this._disposables.value.add(t), t);
			}
			static withScope(t) {
				let n = new jn(), r;
				try {
					return t(n);
				} catch (s) {
					throw r = s, s;
				} finally {
					er(n, r);
				}
			}
			static withScopeMaybeAsync(t, n) {
				return Pu(void 0, function* (r) {
					let s = new jn(), i;
					try {
						return yield* r.of(n.call(t, r, s));
					} catch (o) {
						throw i = o, o;
					} finally {
						er(s, i);
					}
				});
			}
			static async withScopeAsync(t) {
				let n = new jn(), r;
				try {
					return await t(n);
				} catch (s) {
					throw r = s, s;
				} finally {
					er(n, r);
				}
			}
			get alive() {
				return this._disposables.alive;
			}
			dispose() {
				let t = Array.from(this._disposables.value.values()).reverse();
				for (let n of t) n.alive && n.dispose();
				this._disposables.dispose();
			}
		}, an = class Bu extends Te {
			static success(t) {
				return new js(t);
			}
			static fail(t, n) {
				return new Xs(t, n);
			}
			static is(t) {
				return t instanceof Bu;
			}
		}, js = class extends an {
			constructor(e) {
				super(), this.value = e;
			}
			get alive() {
				return zt(this.value) ? this.value.alive : !0;
			}
			dispose() {
				zt(this.value) && this.value.dispose();
			}
			unwrap() {
				return this.value;
			}
			unwrapOr(e) {
				return this.value;
			}
		}, Xs = class extends an {
			constructor(e, t) {
				super(), this.error = e, this.onUnwrap = t;
			}
			get alive() {
				return zt(this.error) ? this.error.alive : !0;
			}
			dispose() {
				zt(this.error) && this.error.dispose();
			}
			unwrap() {
				throw this.onUnwrap(this), this.error;
			}
			unwrapOr(e) {
				return e;
			}
		}, qe = an, qs = class extends Te {
			constructor(e) {
				super(), this.resolve = (t) => {
					this.resolveHandle.alive && (this.context.unwrapResult(this.context.callFunction(this.resolveHandle, this.context.undefined, t || this.context.undefined)).dispose(), this.disposeResolvers(), this.onSettled());
				}, this.reject = (t) => {
					this.rejectHandle.alive && (this.context.unwrapResult(this.context.callFunction(this.rejectHandle, this.context.undefined, t || this.context.undefined)).dispose(), this.disposeResolvers(), this.onSettled());
				}, this.dispose = () => {
					this.handle.alive && this.handle.dispose(), this.disposeResolvers();
				}, this.context = e.context, this.owner = e.context.runtime, this.handle = e.promiseHandle, this.settled = new Promise((t) => {
					this.onSettled = t;
				}), this.resolveHandle = e.resolveHandle, this.rejectHandle = e.rejectHandle;
			}
			get alive() {
				return this.handle.alive || this.resolveHandle.alive || this.rejectHandle.alive;
			}
			disposeResolvers() {
				this.resolveHandle.alive && this.resolveHandle.dispose(), this.rejectHandle.alive && this.rejectHandle.dispose();
			}
		}, cr = class {
			constructor(e) {
				this.module = e;
			}
			toPointerArray(e) {
				let t = new Int32Array(e.map((s) => s.value)), n = t.length * t.BYTES_PER_ELEMENT, r = this.module._malloc(n);
				return new Uint8Array(this.module.HEAPU8.buffer, r, n).set(new Uint8Array(t.buffer)), new j(r, void 0, (s) => this.module._free(s));
			}
			newTypedArray(e, t) {
				let n = new e(new Array(t).fill(0)), r = n.length * n.BYTES_PER_ELEMENT, s = this.module._malloc(r), i = new e(this.module.HEAPU8.buffer, s, t);
				return i.set(n), new j({
					typedArray: i,
					ptr: s
				}, void 0, (o) => this.module._free(o.ptr));
			}
			newMutablePointerArray(e) {
				return this.newTypedArray(Int32Array, e);
			}
			newHeapCharPointer(e) {
				let t = this.module.lengthBytesUTF8(e), n = t + 1, r = this.module._malloc(n);
				return this.module.stringToUTF8(e, r, n), new j({
					ptr: r,
					strlen: t
				}, void 0, (s) => this.module._free(s.ptr));
			}
			newHeapBufferPointer(e) {
				let t = e.byteLength, n = this.module._malloc(t);
				return this.module.HEAPU8.set(e, n), new j({
					pointer: n,
					numBytes: t
				}, void 0, (r) => this.module._free(r.pointer));
			}
			consumeHeapCharPointer(e) {
				let t = this.module.UTF8ToString(e);
				return this.module._free(e), t;
			}
		}, Object.freeze({
			BaseObjects: !0,
			Date: !0,
			Eval: !0,
			StringNormalize: !0,
			RegExp: !0,
			JSON: !0,
			Proxy: !0,
			MapSet: !0,
			TypedArrays: !0,
			Promise: !0
		}), $s = class extends Te {
			constructor(e, t) {
				super(), this.handle = e, this.context = t, this._isDone = !1, this.owner = t.runtime;
			}
			[Symbol.iterator]() {
				return this;
			}
			next(e) {
				if (!this.alive || this._isDone) return {
					done: !0,
					value: void 0
				};
				let t = this._next ?? (this._next = this.context.getProp(this.handle, "next"));
				return this.callIteratorMethod(t, e);
			}
			return(e) {
				if (!this.alive) return {
					done: !0,
					value: void 0
				};
				let t = this.context.getProp(this.handle, "return");
				if (t === this.context.undefined && e === void 0) return this.dispose(), {
					done: !0,
					value: void 0
				};
				let n = this.callIteratorMethod(t, e);
				return t.dispose(), this.dispose(), n;
			}
			throw(e) {
				if (!this.alive) return {
					done: !0,
					value: void 0
				};
				let t = e instanceof j ? e : this.context.newError(e), n = this.context.getProp(this.handle, "throw"), r = this.callIteratorMethod(n, e);
				return t.alive && t.dispose(), n.dispose(), this.dispose(), r;
			}
			get alive() {
				return this.handle.alive;
			}
			dispose() {
				this._isDone = !0, this.handle.dispose(), this._next?.dispose();
			}
			callIteratorMethod(e, t) {
				let n = t ? this.context.callFunction(e, this.handle, t) : this.context.callFunction(e, this.handle);
				if (n.error) return this.dispose(), { value: n };
				let r = this.context.getProp(n.value, "done").consume((i) => this.context.dump(i)), s = this.context.getProp(n.value, "value");
				return n.value.dispose(), r && this.dispose(), {
					value: qe.success(s),
					done: r
				};
			}
		}, hr = -2147483648, dr = 2147483647, Tt = 0, zs = class {
			constructor() {
				this.nextId = hr, this.freelist = [], this.groups = /* @__PURE__ */ new Map();
			}
			put(e) {
				let t = this.allocateId(), n = tr(t), r = this.groups.get(n);
				return r || (r = /* @__PURE__ */ new Map(), this.groups.set(n, r)), r.set(t, e), t;
			}
			get(e) {
				if (e === Tt) throw new Fe("no host reference id defined");
				let t = tr(e), n = this.groups.get(t);
				if (!n) throw new Fe(`host reference id ${e} is not defined`);
				let r = n.get(e);
				if (!r) throw new Fe(`host reference id ${e} is not defined`);
				return r;
			}
			delete(e) {
				if (e === Tt) throw new Fe("no host reference id defined");
				let t = tr(e), n = this.groups.get(t);
				if (!n) throw new Fe(`host reference id ${e} is not defined`);
				n.delete(e), n.size === 0 && this.groups.delete(t), this.freelist.push(e);
			}
			allocateId() {
				if (this.freelist.length > 0) return this.freelist.shift();
				if (this.nextId === Tt && this.nextId++, this.nextId > dr) throw new ur(`HostRefMap: too many host refs created without disposing. Max simultaneous host refs: ${dr - hr}`);
				return this.nextId++;
			}
		}, gr = class extends Te {
			constructor(e, t, n) {
				if (n === Tt) throw new Fe("cannot create HostRef with undefined id");
				super(), this.runtime = e, this.handle = t, this.id = n;
			}
			get alive() {
				return this.handle.alive;
			}
			dispose() {
				this.handle.dispose();
			}
			get value() {
				return this.runtime.hostRefs.get(this.id);
			}
		}, ei = class extends cr {
			constructor(e) {
				super(e.module), this.scope = new ce(), this.copyJSValue = (t) => this.ffi.QTS_DupValuePointer(this.ctx.value, t), this.freeJSValue = (t) => {
					this.ffi.QTS_FreeValuePointer(this.ctx.value, t);
				}, e.ownedLifetimes?.forEach((t) => this.scope.manage(t)), this.owner = e.owner, this.module = e.module, this.ffi = e.ffi, this.rt = e.rt, this.ctx = this.scope.manage(e.ctx);
			}
			get alive() {
				return this.scope.alive;
			}
			dispose() {
				return this.scope.dispose();
			}
			[Symbol.dispose]() {
				return this.dispose();
			}
			manage(e) {
				return this.scope.manage(e);
			}
			consumeJSCharPointer(e) {
				let t = this.module.UTF8ToString(e);
				return this.ffi.QTS_FreeCString(this.ctx.value, e), t;
			}
			heapValueHandle(e, t) {
				let n = t ? (r) => {
					t(), this.freeJSValue(r);
				} : this.freeJSValue;
				return new j(e, this.copyJSValue, n, this.owner);
			}
			staticHeapValueHandle(e) {
				return this.manage(this.heapValueHandle(e)), new Ge(e, this.owner);
			}
		}, fr = class extends Te {
			constructor(e) {
				super(), this._undefined = void 0, this._null = void 0, this._false = void 0, this._true = void 0, this._global = void 0, this._BigInt = void 0, this._Symbol = void 0, this._SymbolIterator = void 0, this._SymbolAsyncIterator = void 0, this.cToHostCallbacks = { callFunction: (t, n, r, s, i) => {
					if (t !== this.ctx.value) throw new Error("QuickJSContext instance received C -> JS call with mismatched ctx");
					let o = this.getFunction(i);
					return ce.withScopeMaybeAsync(this, function* (a, u) {
						let l = u.manage(new Ar(n, this.memory.copyJSValue, this.memory.freeJSValue, this.runtime)), p = new Array(r);
						for (let f = 0; f < r; f++) {
							let g = this.ffi.QTS_ArgvGetJSValueConstPointer(s, f);
							p[f] = u.manage(new Ar(g, this.memory.copyJSValue, this.memory.freeJSValue, this.runtime));
						}
						try {
							let f = yield* a(o.apply(l, p));
							if (f) {
								if ("error" in f && f.error) throw this.runtime.debugLog("throw error", f.error), f.error;
								let g = u.manage(f instanceof j ? f : f.value);
								return this.ffi.QTS_DupValuePointer(this.ctx.value, g.value);
							}
							return 0;
						} catch (f) {
							return this.errorToHandle(f).consume((g) => this.ffi.QTS_Throw(this.ctx.value, g.value));
						}
					});
				} }, this.runtime = e.runtime, this.module = e.module, this.ffi = e.ffi, this.rt = e.rt, this.ctx = e.ctx, this.memory = new ei({
					...e,
					owner: this.runtime
				}), e.callbacks.setContextCallbacks(this.ctx.value, this.cToHostCallbacks), this.dump = this.dump.bind(this), this.getString = this.getString.bind(this), this.getNumber = this.getNumber.bind(this), this.resolvePromise = this.resolvePromise.bind(this), this.uint32Out = this.memory.manage(this.memory.newTypedArray(Uint32Array, 1));
			}
			get alive() {
				return this.memory.alive;
			}
			dispose() {
				this.memory.dispose();
			}
			get undefined() {
				if (this._undefined) return this._undefined;
				let e = this.ffi.QTS_GetUndefined();
				return this._undefined = new Ge(e);
			}
			get null() {
				if (this._null) return this._null;
				let e = this.ffi.QTS_GetNull();
				return this._null = new Ge(e);
			}
			get true() {
				if (this._true) return this._true;
				let e = this.ffi.QTS_GetTrue();
				return this._true = new Ge(e);
			}
			get false() {
				if (this._false) return this._false;
				let e = this.ffi.QTS_GetFalse();
				return this._false = new Ge(e);
			}
			get global() {
				if (this._global) return this._global;
				let e = this.ffi.QTS_GetGlobalObject(this.ctx.value);
				return this._global = this.memory.staticHeapValueHandle(e), this._global;
			}
			newNumber(e) {
				return this.memory.heapValueHandle(this.ffi.QTS_NewFloat64(this.ctx.value, e));
			}
			newString(e) {
				let t = this.memory.newHeapCharPointer(e).consume((n) => this.ffi.QTS_NewString(this.ctx.value, n.value.ptr));
				return this.memory.heapValueHandle(t);
			}
			newUniqueSymbol(e) {
				let t = (typeof e == "symbol" ? e.description : e) ?? "", n = this.memory.newHeapCharPointer(t).consume((r) => this.ffi.QTS_NewSymbol(this.ctx.value, r.value.ptr, 0));
				return this.memory.heapValueHandle(n);
			}
			newSymbolFor(e) {
				let t = (typeof e == "symbol" ? e.description : e) ?? "", n = this.memory.newHeapCharPointer(t).consume((r) => this.ffi.QTS_NewSymbol(this.ctx.value, r.value.ptr, 1));
				return this.memory.heapValueHandle(n);
			}
			getWellKnownSymbol(e) {
				return this._Symbol ?? (this._Symbol = this.memory.manage(this.getProp(this.global, "Symbol"))), this.getProp(this._Symbol, e);
			}
			newBigInt(e) {
				if (!this._BigInt) {
					let r = this.getProp(this.global, "BigInt");
					this.memory.manage(r), this._BigInt = new Ge(r.value, this.runtime);
				}
				let t = this._BigInt, n = String(e);
				return this.newString(n).consume((r) => this.unwrapResult(this.callFunction(t, this.undefined, r)));
			}
			newObject(e) {
				e && this.runtime.assertOwned(e);
				let t = e ? this.ffi.QTS_NewObjectProto(this.ctx.value, e.value) : this.ffi.QTS_NewObject(this.ctx.value);
				return this.memory.heapValueHandle(t);
			}
			newArray() {
				let e = this.ffi.QTS_NewArray(this.ctx.value);
				return this.memory.heapValueHandle(e);
			}
			newArrayBuffer(e) {
				let t = new Uint8Array(e), n = this.memory.newHeapBufferPointer(t), r = this.ffi.QTS_NewArrayBuffer(this.ctx.value, n.value.pointer, t.length);
				return this.memory.heapValueHandle(r);
			}
			newPromise(e) {
				let t = ce.withScope((n) => {
					let r = n.manage(this.memory.newMutablePointerArray(2)), s = this.ffi.QTS_NewPromiseCapability(this.ctx.value, r.value.ptr), i = this.memory.heapValueHandle(s), [o, a] = Array.from(r.value.typedArray).map((u) => this.memory.heapValueHandle(u));
					return new qs({
						context: this,
						promiseHandle: i,
						resolveHandle: o,
						rejectHandle: a
					});
				});
				return e && typeof e == "function" && (e = new Promise(e)), e && Promise.resolve(e).then(t.resolve, (n) => n instanceof j ? t.reject(n) : this.newError(n).consume(t.reject)), t;
			}
			newFunction(e, t) {
				let n = typeof e == "function" ? e : t;
				if (!n) throw new TypeError("Expected a function");
				return this.newFunctionWithOptions({
					name: typeof e == "string" ? e : void 0,
					length: n.length,
					isConstructor: !1,
					fn: n
				});
			}
			newConstructorFunction(e, t) {
				let n = typeof e == "function" ? e : t;
				if (!n) throw new TypeError("Expected a function");
				return this.newFunctionWithOptions({
					name: typeof e == "string" ? e : void 0,
					length: n.length,
					isConstructor: !0,
					fn: n
				});
			}
			newFunctionWithOptions(e) {
				let { name: t, length: n, isConstructor: r, fn: s } = e, i = this.runtime.hostRefs.put(s);
				try {
					return this.memory.heapValueHandle(this.ffi.QTS_NewFunction(this.ctx.value, t ?? "", n, r, i));
				} catch (o) {
					throw this.runtime.hostRefs.delete(i), o;
				}
			}
			newError(e) {
				let t = this.memory.heapValueHandle(this.ffi.QTS_NewError(this.ctx.value));
				return e && typeof e == "object" ? (e.name !== void 0 && this.newString(e.name).consume((n) => this.setProp(t, "name", n)), e.message !== void 0 && this.newString(e.message).consume((n) => this.setProp(t, "message", n))) : typeof e == "string" ? this.newString(e).consume((n) => this.setProp(t, "message", n)) : e !== void 0 && this.newString(String(e)).consume((n) => this.setProp(t, "message", n)), t;
			}
			newHostRef(e) {
				let t = this.runtime.hostRefs.put(e);
				try {
					let n = this.memory.heapValueHandle(this.ffi.QTS_NewHostRef(this.ctx.value, t));
					return new gr(this.runtime, n, t);
				} catch (n) {
					throw this.runtime.hostRefs.delete(t), n;
				}
			}
			toHostRef(e) {
				let t = this.ffi.QTS_GetHostRefId(e.value);
				if (t !== 0) return this.runtime.hostRefs.get(t), new gr(this.runtime, e.dup(), t);
			}
			unwrapHostRef(e) {
				let t = this.ffi.QTS_GetHostRefId(e.value);
				if (t === 0) throw new Fe("handle is not a HostRef");
				return this.runtime.hostRefs.get(t);
			}
			typeof(e) {
				return this.runtime.assertOwned(e), this.memory.consumeHeapCharPointer(this.ffi.QTS_Typeof(this.ctx.value, e.value));
			}
			getNumber(e) {
				return this.runtime.assertOwned(e), this.ffi.QTS_GetFloat64(this.ctx.value, e.value);
			}
			getString(e) {
				return this.runtime.assertOwned(e), this.memory.consumeJSCharPointer(this.ffi.QTS_GetString(this.ctx.value, e.value));
			}
			getSymbol(e) {
				this.runtime.assertOwned(e);
				let t = this.memory.consumeJSCharPointer(this.ffi.QTS_GetSymbolDescriptionOrKey(this.ctx.value, e.value));
				return this.ffi.QTS_IsGlobalSymbol(this.ctx.value, e.value) ? Symbol.for(t) : Symbol(t);
			}
			getBigInt(e) {
				this.runtime.assertOwned(e);
				let t = this.getString(e);
				return BigInt(t);
			}
			getArrayBuffer(e) {
				this.runtime.assertOwned(e);
				let t = this.ffi.QTS_GetArrayBufferLength(this.ctx.value, e.value), n = this.ffi.QTS_GetArrayBuffer(this.ctx.value, e.value);
				if (!n) throw new Error("Couldn't allocate memory to get ArrayBuffer");
				return new j(this.module.HEAPU8.subarray(n, n + t), void 0, () => this.module._free(n));
			}
			getPromiseState(e) {
				this.runtime.assertOwned(e);
				let t = this.ffi.QTS_PromiseState(this.ctx.value, e.value);
				if (t < 0) return {
					type: "fulfilled",
					value: e,
					notAPromise: !0
				};
				if (t === $t.Pending) return {
					type: "pending",
					get error() {
						return new or("Cannot unwrap a pending promise");
					}
				};
				let n = this.ffi.QTS_PromiseResult(this.ctx.value, e.value), r = this.memory.heapValueHandle(n);
				if (t === $t.Fulfilled) return {
					type: "fulfilled",
					value: r
				};
				if (t === $t.Rejected) return {
					type: "rejected",
					error: r
				};
				throw r.dispose(), /* @__PURE__ */ new Error(`Unknown JSPromiseStateEnum: ${t}`);
			}
			resolvePromise(e) {
				this.runtime.assertOwned(e);
				let t = ce.withScope((n) => {
					let r = n.manage(this.getProp(this.global, "Promise")), s = n.manage(this.getProp(r, "resolve"));
					return this.callFunction(s, r, e);
				});
				return t.error ? Promise.resolve(t) : new Promise((n) => {
					ce.withScope((r) => {
						let s = r.manage(this.newFunction("resolve", (u) => {
							n(this.success(u && u.dup()));
						})), i = r.manage(this.newFunction("reject", (u) => {
							n(this.fail(u && u.dup()));
						})), o = r.manage(t.value), a = r.manage(this.getProp(o, "then"));
						this.callFunction(a, o, s, i).unwrap().dispose();
					});
				});
			}
			isEqual(e, t, n = Et.IsStrictlyEqual) {
				if (e === t) return !0;
				this.runtime.assertOwned(e), this.runtime.assertOwned(t);
				let r = this.ffi.QTS_IsEqual(this.ctx.value, e.value, t.value, n);
				if (r === -1) throw new rn("WASM variant does not expose equality");
				return !!r;
			}
			eq(e, t) {
				return this.isEqual(e, t, Et.IsStrictlyEqual);
			}
			sameValue(e, t) {
				return this.isEqual(e, t, Et.IsSameValue);
			}
			sameValueZero(e, t) {
				return this.isEqual(e, t, Et.IsSameValueZero);
			}
			getProp(e, t) {
				this.runtime.assertOwned(e);
				let n;
				return typeof t == "number" && t >= 0 ? n = this.ffi.QTS_GetPropNumber(this.ctx.value, e.value, t) : n = this.borrowPropertyKey(t).consume((r) => this.ffi.QTS_GetProp(this.ctx.value, e.value, r.value)), this.memory.heapValueHandle(n);
			}
			getLength(e) {
				if (this.runtime.assertOwned(e), !(this.ffi.QTS_GetLength(this.ctx.value, this.uint32Out.value.ptr, e.value) < 0)) return this.uint32Out.value.typedArray[0];
			}
			getOwnPropertyNames(e, t = {
				strings: !0,
				numbersAsStrings: !0
			}) {
				this.runtime.assertOwned(e), e.value;
				let n = Yu(t);
				if (n === 0) throw new ar("No options set, will return an empty array");
				return ce.withScope((r) => {
					let s = r.manage(this.memory.newMutablePointerArray(1)), i = this.ffi.QTS_GetOwnPropertyNames(this.ctx.value, s.value.ptr, this.uint32Out.value.ptr, e.value, n);
					if (i) return this.fail(this.memory.heapValueHandle(i));
					let o = this.uint32Out.value.typedArray[0], a = s.value.typedArray[0], u = new Uint32Array(this.module.HEAP8.buffer, a, o), l = Array.from(u).map((p) => this.memory.heapValueHandle(p));
					return this.ffi.QTS_FreeVoidPointer(this.ctx.value, a), this.success(Wu(l));
				});
			}
			getIterator(e) {
				let t = this._SymbolIterator ?? (this._SymbolIterator = this.memory.manage(this.getWellKnownSymbol("iterator")));
				return ce.withScope((n) => {
					let r = n.manage(this.getProp(e, t)), s = this.callFunction(r, e);
					return s.error ? s : this.success(new $s(s.value, this));
				});
			}
			setProp(e, t, n) {
				this.runtime.assertOwned(e), this.borrowPropertyKey(t).consume((r) => this.ffi.QTS_SetProp(this.ctx.value, e.value, r.value, n.value));
			}
			defineProp(e, t, n) {
				this.runtime.assertOwned(e), ce.withScope((r) => {
					let s = r.manage(this.borrowPropertyKey(t)), i = n.value || this.undefined, o = !!n.configurable, a = !!n.enumerable, u = !!n.value, l = n.get ? r.manage(this.newFunction(n.get.name, n.get)) : this.undefined, p = n.set ? r.manage(this.newFunction(n.set.name, n.set)) : this.undefined;
					this.ffi.QTS_DefineProp(this.ctx.value, e.value, s.value, i.value, l.value, p.value, o, a, u);
				});
			}
			callFunction(e, t, ...n) {
				this.runtime.assertOwned(e);
				let r, s = n[0];
				s === void 0 || Array.isArray(s) ? r = s ?? [] : r = n;
				let i = this.memory.toPointerArray(r).consume((a) => this.ffi.QTS_Call(this.ctx.value, e.value, t.value, r.length, a.value)), o = this.ffi.QTS_ResolveException(this.ctx.value, i);
				return o ? (this.ffi.QTS_FreeValuePointer(this.ctx.value, i), this.fail(this.memory.heapValueHandle(o))) : this.success(this.memory.heapValueHandle(i));
			}
			callMethod(e, t, n = []) {
				return this.getProp(e, t).consume((r) => this.callFunction(r, e, n));
			}
			evalCode(e, t = "eval.js", n) {
				let r = n === void 0 ? 1 : 0, s = Ks(n), i = this.memory.newHeapCharPointer(e).consume((a) => this.ffi.QTS_Eval(this.ctx.value, a.value.ptr, a.value.strlen, t, r, s)), o = this.ffi.QTS_ResolveException(this.ctx.value, i);
				return o ? (this.ffi.QTS_FreeValuePointer(this.ctx.value, i), this.fail(this.memory.heapValueHandle(o))) : this.success(this.memory.heapValueHandle(i));
			}
			throw(e) {
				return this.errorToHandle(e).consume((t) => this.ffi.QTS_Throw(this.ctx.value, t.value));
			}
			borrowPropertyKey(e) {
				return typeof e == "number" ? this.newNumber(e) : typeof e == "string" ? this.newString(e) : new Ge(e.value, this.runtime);
			}
			getMemory(e) {
				if (e === this.rt.value) return this.memory;
				throw new Error("Private API. Cannot get memory from a different runtime");
			}
			dump(e) {
				this.runtime.assertOwned(e);
				let t = this.typeof(e);
				if (t === "string") return this.getString(e);
				if (t === "number") return this.getNumber(e);
				if (t === "bigint") return this.getBigInt(e);
				if (t === "undefined") return;
				if (t === "symbol") return this.getSymbol(e);
				let n = this.getPromiseState(e);
				if (n.type === "fulfilled" && !n.notAPromise) return e.dispose(), {
					type: n.type,
					value: n.value.consume(this.dump)
				};
				if (n.type === "pending") return e.dispose(), { type: n.type };
				if (n.type === "rejected") return e.dispose(), {
					type: n.type,
					error: n.error.consume(this.dump)
				};
				let r = this.memory.consumeJSCharPointer(this.ffi.QTS_Dump(this.ctx.value, e.value));
				try {
					return JSON.parse(r);
				} catch {
					return r;
				}
			}
			unwrapResult(e) {
				if (e.error) {
					let t = "context" in e.error ? e.error.context : this, n = e.error.consume((r) => this.dump(r));
					if (n && typeof n == "object" && typeof n.message == "string") {
						let { message: r, name: s, stack: i, ...o } = n, a = new tn(n, t);
						typeof s == "string" && (a.name = n.name), a.message = r;
						let u = a.stack;
						throw typeof i == "string" && (a.stack = `${s}: ${r}
${n.stack}Host: ${u}`), Object.assign(a, o), a;
					}
					throw new tn(n);
				}
				return e.value;
			}
			[Symbol.for("nodejs.util.inspect.custom")]() {
				return this.alive ? `${this.constructor.name} { ctx: ${this.ctx.value} rt: ${this.rt.value} }` : `${this.constructor.name} { disposed }`;
			}
			getFunction(e) {
				let t = this.runtime.hostRefs.get(e);
				if (typeof t != "function") throw new Error(`Host reference ${e} is not a function`);
				return t;
			}
			errorToHandle(e) {
				return e instanceof j ? e : this.newError(e);
			}
			encodeBinaryJSON(e) {
				let t = this.ffi.QTS_bjson_encode(this.ctx.value, e.value);
				return this.memory.heapValueHandle(t);
			}
			decodeBinaryJSON(e) {
				let t = this.ffi.QTS_bjson_decode(this.ctx.value, e.value);
				return this.memory.heapValueHandle(t);
			}
			success(e) {
				return qe.success(e);
			}
			fail(e) {
				return qe.fail(e, (t) => this.unwrapResult(t));
			}
		}, pr = class extends Te {
			constructor(e) {
				super(), this.scope = new ce(), this.contextMap = /* @__PURE__ */ new Map(), this.hostRefs = new zs(), this._debugMode = !1, this.cToHostCallbacks = {
					freeHostRef: (t, n) => {
						if (t !== this.rt.value) throw new Error("Runtime pointer mismatch");
						this.hostRefs.delete(n);
					},
					shouldInterrupt: (t) => {
						if (t !== this.rt.value) throw new Error("QuickJSContext instance received C -> JS interrupt with mismatched rt");
						let n = this.interruptHandler;
						if (!n) throw new Error("QuickJSContext had no interrupt handler");
						return n(this) ? 1 : 0;
					},
					loadModuleSource: Ls(this, function* (t, n, r, s) {
						let i = this.moduleLoader;
						if (!i) throw new Error("Runtime has no module loader");
						if (n !== this.rt.value) throw new Error("Runtime pointer mismatch");
						let o = this.contextMap.get(r) ?? this.newContext({ contextPointer: r });
						try {
							let a = yield* t(i(s, o));
							if (typeof a == "object" && "error" in a && a.error) throw this.debugLog("cToHostLoadModule: loader returned error", a.error), a.error;
							let u = typeof a == "string" ? a : "value" in a ? a.value : a;
							return this.memory.newHeapCharPointer(u).value.ptr;
						} catch (a) {
							return this.debugLog("cToHostLoadModule: caught error", a), o.throw(a), 0;
						}
					}),
					normalizeModule: Ls(this, function* (t, n, r, s, i) {
						let o = this.moduleNormalizer;
						if (!o) throw new Error("Runtime has no module normalizer");
						if (n !== this.rt.value) throw new Error("Runtime pointer mismatch");
						let a = this.contextMap.get(r) ?? this.newContext({ contextPointer: r });
						try {
							let u = yield* t(o(s, i, a));
							if (typeof u == "object" && "error" in u && u.error) throw this.debugLog("cToHostNormalizeModule: normalizer returned error", u.error), u.error;
							let l = typeof u == "string" ? u : u.value;
							return a.getMemory(this.rt.value).newHeapCharPointer(l).value.ptr;
						} catch (u) {
							return this.debugLog("normalizeModule: caught error", u), a.throw(u), 0;
						}
					})
				}, e.ownedLifetimes?.forEach((t) => this.scope.manage(t)), this.module = e.module, this.memory = new cr(this.module), this.ffi = e.ffi, this.rt = e.rt, this.callbacks = e.callbacks, this.scope.manage(this.rt), this.callbacks.setRuntimeCallbacks(this.rt.value, this.cToHostCallbacks), this.executePendingJobs = this.executePendingJobs.bind(this), en && this.setDebugMode(!0);
			}
			get alive() {
				return this.scope.alive;
			}
			dispose() {
				return this.scope.dispose();
			}
			newContext(e = {}) {
				let t = Os(e.intrinsics), n = new j(e.contextPointer || this.ffi.QTS_NewContext(this.rt.value, t), void 0, (s) => {
					this.contextMap.delete(s), this.callbacks.deleteContext(s), this.ffi.QTS_FreeContext(s);
				}), r = new fr({
					module: this.module,
					ctx: n,
					ffi: this.ffi,
					rt: this.rt,
					ownedLifetimes: e.ownedLifetimes,
					runtime: this,
					callbacks: this.callbacks
				});
				return this.contextMap.set(n.value, r), r;
			}
			setModuleLoader(e, t) {
				this.moduleLoader = e, this.moduleNormalizer = t, this.ffi.QTS_RuntimeEnableModuleLoader(this.rt.value, this.moduleNormalizer ? 1 : 0);
			}
			removeModuleLoader() {
				this.moduleLoader = void 0, this.ffi.QTS_RuntimeDisableModuleLoader(this.rt.value);
			}
			hasPendingJob() {
				return !!this.ffi.QTS_IsJobPending(this.rt.value);
			}
			setInterruptHandler(e) {
				let t = this.interruptHandler;
				this.interruptHandler = e, t || this.ffi.QTS_RuntimeEnableInterruptHandler(this.rt.value);
			}
			removeInterruptHandler() {
				this.interruptHandler && (this.ffi.QTS_RuntimeDisableInterruptHandler(this.rt.value), this.interruptHandler = void 0);
			}
			executePendingJobs(e = -1) {
				let t = this.memory.newMutablePointerArray(1), n = this.ffi.QTS_ExecutePendingJob(this.rt.value, e ?? -1, t.value.ptr), r = t.value.typedArray[0];
				if (t.dispose(), r === 0) return this.ffi.QTS_FreeValuePointerRuntime(this.rt.value, n), qe.success(0);
				let s = this.contextMap.get(r) ?? this.newContext({ contextPointer: r }), i = s.getMemory(this.rt.value).heapValueHandle(n);
				if (s.typeof(i) === "number") {
					let o = s.getNumber(i);
					return i.dispose(), qe.success(o);
				} else {
					let o = Object.assign(i, { context: s });
					return qe.fail(o, (a) => s.unwrapResult(a));
				}
			}
			setMemoryLimit(e) {
				if (e < 0 && e !== -1) throw new Error("Cannot set memory limit to negative number. To unset, pass -1");
				this.ffi.QTS_RuntimeSetMemoryLimit(this.rt.value, e);
			}
			computeMemoryUsage() {
				let e = this.getSystemContext().getMemory(this.rt.value);
				return e.heapValueHandle(this.ffi.QTS_RuntimeComputeMemoryUsage(this.rt.value, e.ctx.value));
			}
			dumpMemoryUsage() {
				return this.memory.consumeHeapCharPointer(this.ffi.QTS_RuntimeDumpMemoryUsage(this.rt.value));
			}
			setMaxStackSize(e) {
				if (e < 0) throw new Error("Cannot set memory limit to negative number. To unset, pass 0.");
				this.ffi.QTS_RuntimeSetMaxStackSize(this.rt.value, e);
			}
			assertOwned(e) {
				if (e.owner && e.owner.rt !== this.rt) throw new nr(`Handle is not owned by this runtime: ${e.owner.rt.value} != ${this.rt.value}`);
			}
			setDebugMode(e) {
				this._debugMode = e, this.ffi.DEBUG && this.rt.alive && this.ffi.QTS_SetDebugLogEnabled(this.rt.value, e ? 1 : 0);
			}
			isDebugMode() {
				return this._debugMode;
			}
			debugLog(...e) {
				this._debugMode && console.log("quickjs-emscripten:", ...e);
			}
			[Symbol.for("nodejs.util.inspect.custom")]() {
				return this.alive ? `${this.constructor.name} { rt: ${this.rt.value} }` : `${this.constructor.name} { disposed }`;
			}
			getSystemContext() {
				return this.context || (this.context = this.scope.manage(this.newContext())), this.context;
			}
		}, ti = class {
			constructor(e) {
				this.freeHostRef = e.freeHostRef, this.callFunction = e.callFunction, this.shouldInterrupt = e.shouldInterrupt, this.loadModuleSource = e.loadModuleSource, this.normalizeModule = e.normalizeModule;
			}
		}, ni = class {
			constructor(e) {
				this.contextCallbacks = /* @__PURE__ */ new Map(), this.runtimeCallbacks = /* @__PURE__ */ new Map(), this.suspendedCount = 0, this.cToHostCallbacks = new ti({
					freeHostRef: (t, n, r) => {
						let s = this.runtimeCallbacks.get(n);
						if (!s) throw new Error(`QuickJSRuntime(rt = ${n}) not found when trying to free HostRef(id = ${r})`);
						s.freeHostRef(n, r);
					},
					callFunction: (t, n, r, s, i, o) => this.handleAsyncify(t, () => {
						try {
							let a = this.contextCallbacks.get(n);
							if (!a) throw new Error(`QuickJSContext(ctx = ${n}) not found for C function call "${o}"`);
							return a.callFunction(n, r, s, i, o);
						} catch (a) {
							return console.error("[C to host error: returning null]", a), 0;
						}
					}),
					shouldInterrupt: (t, n) => this.handleAsyncify(t, () => {
						try {
							let r = this.runtimeCallbacks.get(n);
							if (!r) throw new Error(`QuickJSRuntime(rt = ${n}) not found for C interrupt`);
							return r.shouldInterrupt(n);
						} catch (r) {
							return console.error("[C to host interrupt: returning error]", r), 1;
						}
					}),
					loadModuleSource: (t, n, r, s) => this.handleAsyncify(t, () => {
						try {
							let i = this.runtimeCallbacks.get(n);
							if (!i) throw new Error(`QuickJSRuntime(rt = ${n}) not found for C module loader`);
							let o = i.loadModuleSource;
							if (!o) throw new Error(`QuickJSRuntime(rt = ${n}) does not support module loading`);
							return o(n, r, s);
						} catch (i) {
							return console.error("[C to host module loader error: returning null]", i), 0;
						}
					}),
					normalizeModule: (t, n, r, s, i) => this.handleAsyncify(t, () => {
						try {
							let o = this.runtimeCallbacks.get(n);
							if (!o) throw new Error(`QuickJSRuntime(rt = ${n}) not found for C module loader`);
							let a = o.normalizeModule;
							if (!a) throw new Error(`QuickJSRuntime(rt = ${n}) does not support module loading`);
							return a(n, r, s, i);
						} catch (o) {
							return console.error("[C to host module loader error: returning null]", o), 0;
						}
					})
				}), this.module = e, this.module.callbacks = this.cToHostCallbacks;
			}
			setRuntimeCallbacks(e, t) {
				this.runtimeCallbacks.set(e, t);
			}
			deleteRuntime(e) {
				this.runtimeCallbacks.delete(e);
			}
			setContextCallbacks(e, t) {
				this.contextCallbacks.set(e, t);
			}
			deleteContext(e) {
				this.contextCallbacks.delete(e);
			}
			handleAsyncify(e, t) {
				if (e) return e.handleSleep((r) => {
					try {
						let s = t();
						if (!(s instanceof Promise)) {
							xt("asyncify.handleSleep: not suspending:", s), r(s);
							return;
						}
						if (this.suspended) throw new rr(`Already suspended at: ${this.suspended.stack}
Attempted to suspend at:`);
						this.suspended = new sr(`(${this.suspendedCount++})`), xt("asyncify.handleSleep: suspending:", this.suspended), s.then((i) => {
							this.suspended = void 0, xt("asyncify.handleSleep: resolved:", i), r(i);
						}, (i) => {
							xt("asyncify.handleSleep: rejected:", i), console.error("QuickJS: cannot handle error in suspended function", i), this.suspended = void 0;
						});
					} catch (s) {
						throw xt("asyncify.handleSleep: error:", s), this.suspended = void 0, s;
					}
				});
				let n = t();
				if (n instanceof Promise) throw new Error("Promise return value not supported in non-asyncify context.");
				return n;
			}
		}, ri = class {
			constructor(e, t) {
				this.module = e, this.ffi = t, this.callbacks = new ni(e);
			}
			newRuntime(e = {}) {
				let t = new j(this.ffi.QTS_NewRuntime(), void 0, (r) => {
					this.ffi.QTS_FreeRuntime(r), this.callbacks.deleteRuntime(r);
				}), n = new pr({
					module: this.module,
					callbacks: this.callbacks,
					ffi: this.ffi,
					rt: t
				});
				return Js(n, e), e.moduleLoader && n.setModuleLoader(e.moduleLoader), n;
			}
			newContext(e = {}) {
				let t = this.newRuntime(), n = t.newContext({
					...e,
					ownedLifetimes: Uu(t, e.ownedLifetimes)
				});
				return t.context = n, n;
			}
			evalCode(e, t = {}) {
				return ce.withScope((n) => {
					let r = n.manage(this.newContext());
					Ps(r.runtime, t);
					let s = r.evalCode(e, "eval.js");
					if (t.memoryLimitBytes !== void 0 && r.runtime.setMemoryLimit(-1), s.error) throw r.dump(n.manage(s.error));
					return r.dump(n.manage(s.value));
				});
			}
			getWasmMemory() {
				let e = this.module.quickjsEmscriptenInit?.(() => {})?.getWasmMemory?.();
				if (!e) throw new Error("Variant does not support getting WebAssembly.Memory");
				return e;
			}
			getFFI() {
				return this.ffi;
			}
		};
	})), si, ii, oi, ju = Se((() => {
		Zu(), si = class extends fr {
			async evalCodeAsync(e, t = "eval.js", n) {
				let r = n === void 0 ? 1 : 0, s = Ks(n), i = 0;
				try {
					i = await this.memory.newHeapCharPointer(e).consume((a) => this.ffi.QTS_Eval_MaybeAsync(this.ctx.value, a.value.ptr, a.value.strlen, t, r, s));
				} catch (a) {
					throw this.runtime.debugLog("QTS_Eval_MaybeAsync threw", a), a;
				}
				let o = this.ffi.QTS_ResolveException(this.ctx.value, i);
				return o ? (this.ffi.QTS_FreeValuePointer(this.ctx.value, i), this.fail(this.memory.heapValueHandle(o))) : this.success(this.memory.heapValueHandle(i));
			}
			newAsyncifiedFunction(e, t) {
				return this.newFunction(e, t);
			}
		}, ii = class extends pr {
			constructor(e) {
				super(e);
			}
			newContext(e = {}) {
				let t = Os(e.intrinsics), n = new j(this.ffi.QTS_NewContext(this.rt.value, t), void 0, (s) => {
					this.contextMap.delete(s), this.callbacks.deleteContext(s), this.ffi.QTS_FreeContext(s);
				}), r = new si({
					module: this.module,
					ctx: n,
					ffi: this.ffi,
					rt: this.rt,
					ownedLifetimes: [],
					runtime: this,
					callbacks: this.callbacks
				});
				return this.contextMap.set(n.value, r), r;
			}
			setModuleLoader(e, t) {
				super.setModuleLoader(e, t);
			}
			setMaxStackSize(e) {
				return super.setMaxStackSize(e);
			}
		}, oi = class extends ri {
			constructor(e, t) {
				super(e, t), this.ffi = t, this.module = e;
			}
			newRuntime(e = {}) {
				let t = new j(this.ffi.QTS_NewRuntime(), void 0, (r) => {
					this.callbacks.deleteRuntime(r), this.ffi.QTS_FreeRuntime(r);
				}), n = new ii({
					module: this.module,
					ffi: this.ffi,
					rt: t,
					callbacks: this.callbacks
				});
				return Js(n, e), e.moduleLoader && n.setModuleLoader(e.moduleLoader), n;
			}
			newContext(e = {}) {
				let t = this.newRuntime(), n = e.ownedLifetimes ? e.ownedLifetimes.concat([t]) : [t], r = t.newContext({
					...e,
					ownedLifetimes: n
				});
				return t.context = r, r;
			}
			evalCode() {
				throw new rn("QuickJSWASMModuleAsyncify.evalCode: use evalCodeAsync instead");
			}
			evalCodeAsync(e, t) {
				return ce.withScopeAsync(async (n) => {
					let r = n.manage(this.newContext());
					Ps(r.runtime, t);
					let s = await r.evalCodeAsync(e, "eval.js");
					if (t.memoryLimitBytes !== void 0 && r.runtime.setMemoryLimit(-1), s.error) throw r.dump(n.manage(s.error));
					return r.dump(n.manage(s.value));
				});
			}
		};
	})), Xu = Me({ QuickJSAsyncWASMModule: () => oi }), qu = Se((() => {
		ju();
	}));
	$n();
	async function $u(e) {
		let t = mr(await e), [n, r, { QuickJSAsyncWASMModule: s }] = await Promise.all([
			t.importModuleLoader().then(mr),
			t.importFFI(),
			Promise.resolve().then(() => (qu(), Xu)).then(mr)
		]), i = await n();
		return i.type = "async", new s(i, new r(i));
	}
	function mr(e) {
		return e && "default" in e && e.default ? e.default && "default" in e.default && e.default.default ? e.default.default : e.default : e;
	}
	function Bt(e) {
		"@babel/helpers - typeof";
		return Bt = typeof Symbol == "function" && typeof Symbol.iterator == "symbol" ? function(t) {
			return typeof t;
		} : function(t) {
			return t && typeof Symbol == "function" && t.constructor === Symbol && t !== Symbol.prototype ? "symbol" : typeof t;
		}, Bt(e);
	}
	var ai = Se((() => {}));
	function zu(e, t) {
		if (Bt(e) != "object" || !e) return e;
		var n = e[Symbol.toPrimitive];
		if (n !== void 0) {
			var r = n.call(e, t || "default");
			if (Bt(r) != "object") return r;
			throw new TypeError("@@toPrimitive must return a primitive value.");
		}
		return (t === "string" ? String : Number)(e);
	}
	var el = Se((() => {
		ai();
	}));
	function tl(e) {
		var t = zu(e, "string");
		return Bt(t) == "symbol" ? t : t + "";
	}
	var nl = Se((() => {
		ai(), el();
	}));
	function rl(e, t, n) {
		return (t = tl(t)) in e ? Object.defineProperty(e, t, {
			value: n,
			enumerable: !0,
			configurable: !0,
			writable: !0
		}) : e[t] = n, e;
	}
	var sl = Se((() => {
		nl();
	})), il = Me({ QuickJSAsyncFFI: () => ui }), ui, ol = Se((() => {
		$n(), ui = class {
			constructor(e) {
				this.module = e, this.DEBUG = !1, this.QTS_Throw = this.module.cwrap("QTS_Throw", "number", ["number", "number"]), this.QTS_NewError = this.module.cwrap("QTS_NewError", "number", ["number"]), this.QTS_RuntimeSetMemoryLimit = this.module.cwrap("QTS_RuntimeSetMemoryLimit", null, ["number", "number"]), this.QTS_RuntimeComputeMemoryUsage = this.module.cwrap("QTS_RuntimeComputeMemoryUsage", "number", ["number", "number"]), this.QTS_RuntimeDumpMemoryUsage = this.module.cwrap("QTS_RuntimeDumpMemoryUsage", "number", ["number"]), this.QTS_RecoverableLeakCheck = this.module.cwrap("QTS_RecoverableLeakCheck", "number", []), this.QTS_BuildIsSanitizeLeak = this.module.cwrap("QTS_BuildIsSanitizeLeak", "number", []), this.QTS_RuntimeSetMaxStackSize = this.module.cwrap("QTS_RuntimeSetMaxStackSize", null, ["number", "number"]), this.QTS_GetUndefined = this.module.cwrap("QTS_GetUndefined", "number", []), this.QTS_GetNull = this.module.cwrap("QTS_GetNull", "number", []), this.QTS_GetFalse = this.module.cwrap("QTS_GetFalse", "number", []), this.QTS_GetTrue = this.module.cwrap("QTS_GetTrue", "number", []), this.QTS_NewHostRef = this.module.cwrap("QTS_NewHostRef", "number", ["number", "number"]), this.QTS_GetHostRefId = this.module.cwrap("QTS_GetHostRefId", "number", ["number"]), this.QTS_NewRuntime = this.module.cwrap("QTS_NewRuntime", "number", []), this.QTS_FreeRuntime = this.module.cwrap("QTS_FreeRuntime", null, ["number"]), this.QTS_NewContext = this.module.cwrap("QTS_NewContext", "number", ["number", "number"]), this.QTS_FreeContext = this.module.cwrap("QTS_FreeContext", null, ["number"]), this.QTS_FreeValuePointer = this.module.cwrap("QTS_FreeValuePointer", null, ["number", "number"]), this.QTS_FreeValuePointerRuntime = this.module.cwrap("QTS_FreeValuePointerRuntime", null, ["number", "number"]), this.QTS_FreeVoidPointer = this.module.cwrap("QTS_FreeVoidPointer", null, ["number", "number"]), this.QTS_FreeCString = this.module.cwrap("QTS_FreeCString", null, ["number", "number"]), this.QTS_DupValuePointer = this.module.cwrap("QTS_DupValuePointer", "number", ["number", "number"]), this.QTS_NewObject = this.module.cwrap("QTS_NewObject", "number", ["number"]), this.QTS_NewObjectProto = this.module.cwrap("QTS_NewObjectProto", "number", ["number", "number"]), this.QTS_NewArray = this.module.cwrap("QTS_NewArray", "number", ["number"]), this.QTS_NewArrayBuffer = this.module.cwrap("QTS_NewArrayBuffer", "number", [
					"number",
					"number",
					"number"
				]), this.QTS_NewFloat64 = this.module.cwrap("QTS_NewFloat64", "number", ["number", "number"]), this.QTS_GetFloat64 = this.module.cwrap("QTS_GetFloat64", "number", ["number", "number"]), this.QTS_NewString = this.module.cwrap("QTS_NewString", "number", ["number", "number"]), this.QTS_GetString = this.module.cwrap("QTS_GetString", "number", ["number", "number"]), this.QTS_GetArrayBuffer = this.module.cwrap("QTS_GetArrayBuffer", "number", ["number", "number"]), this.QTS_GetArrayBufferLength = this.module.cwrap("QTS_GetArrayBufferLength", "number", ["number", "number"]), this.QTS_NewSymbol = this.module.cwrap("QTS_NewSymbol", "number", [
					"number",
					"number",
					"number"
				]), this.QTS_GetSymbolDescriptionOrKey = _e(this.module.cwrap("QTS_GetSymbolDescriptionOrKey", "number", ["number", "number"])), this.QTS_GetSymbolDescriptionOrKey_MaybeAsync = this.module.cwrap("QTS_GetSymbolDescriptionOrKey", "number", ["number", "number"]), this.QTS_IsGlobalSymbol = this.module.cwrap("QTS_IsGlobalSymbol", "number", ["number", "number"]), this.QTS_IsJobPending = this.module.cwrap("QTS_IsJobPending", "number", ["number"]), this.QTS_ExecutePendingJob = _e(this.module.cwrap("QTS_ExecutePendingJob", "number", [
					"number",
					"number",
					"number"
				])), this.QTS_ExecutePendingJob_MaybeAsync = this.module.cwrap("QTS_ExecutePendingJob", "number", [
					"number",
					"number",
					"number"
				]), this.QTS_GetProp = _e(this.module.cwrap("QTS_GetProp", "number", [
					"number",
					"number",
					"number"
				])), this.QTS_GetProp_MaybeAsync = this.module.cwrap("QTS_GetProp", "number", [
					"number",
					"number",
					"number"
				]), this.QTS_GetPropNumber = _e(this.module.cwrap("QTS_GetPropNumber", "number", [
					"number",
					"number",
					"number"
				])), this.QTS_GetPropNumber_MaybeAsync = this.module.cwrap("QTS_GetPropNumber", "number", [
					"number",
					"number",
					"number"
				]), this.QTS_SetProp = _e(this.module.cwrap("QTS_SetProp", null, [
					"number",
					"number",
					"number",
					"number"
				])), this.QTS_SetProp_MaybeAsync = this.module.cwrap("QTS_SetProp", null, [
					"number",
					"number",
					"number",
					"number"
				]), this.QTS_DefineProp = this.module.cwrap("QTS_DefineProp", null, [
					"number",
					"number",
					"number",
					"number",
					"number",
					"number",
					"boolean",
					"boolean",
					"boolean"
				]), this.QTS_GetOwnPropertyNames = _e(this.module.cwrap("QTS_GetOwnPropertyNames", "number", [
					"number",
					"number",
					"number",
					"number",
					"number"
				])), this.QTS_GetOwnPropertyNames_MaybeAsync = this.module.cwrap("QTS_GetOwnPropertyNames", "number", [
					"number",
					"number",
					"number",
					"number",
					"number"
				]), this.QTS_Call = _e(this.module.cwrap("QTS_Call", "number", [
					"number",
					"number",
					"number",
					"number",
					"number"
				])), this.QTS_Call_MaybeAsync = this.module.cwrap("QTS_Call", "number", [
					"number",
					"number",
					"number",
					"number",
					"number"
				]), this.QTS_ResolveException = this.module.cwrap("QTS_ResolveException", "number", ["number", "number"]), this.QTS_Dump = _e(this.module.cwrap("QTS_Dump", "number", ["number", "number"])), this.QTS_Dump_MaybeAsync = this.module.cwrap("QTS_Dump", "number", ["number", "number"]), this.QTS_Eval = _e(this.module.cwrap("QTS_Eval", "number", [
					"number",
					"number",
					"number",
					"string",
					"number",
					"number"
				])), this.QTS_Eval_MaybeAsync = this.module.cwrap("QTS_Eval", "number", [
					"number",
					"number",
					"number",
					"string",
					"number",
					"number"
				]), this.QTS_GetModuleNamespace = this.module.cwrap("QTS_GetModuleNamespace", "number", ["number", "number"]), this.QTS_Typeof = this.module.cwrap("QTS_Typeof", "number", ["number", "number"]), this.QTS_GetLength = this.module.cwrap("QTS_GetLength", "number", [
					"number",
					"number",
					"number"
				]), this.QTS_IsEqual = this.module.cwrap("QTS_IsEqual", "number", [
					"number",
					"number",
					"number",
					"number"
				]), this.QTS_GetGlobalObject = this.module.cwrap("QTS_GetGlobalObject", "number", ["number"]), this.QTS_NewPromiseCapability = this.module.cwrap("QTS_NewPromiseCapability", "number", ["number", "number"]), this.QTS_PromiseState = this.module.cwrap("QTS_PromiseState", "number", ["number", "number"]), this.QTS_PromiseResult = this.module.cwrap("QTS_PromiseResult", "number", ["number", "number"]), this.QTS_TestStringArg = this.module.cwrap("QTS_TestStringArg", null, ["string"]), this.QTS_GetDebugLogEnabled = this.module.cwrap("QTS_GetDebugLogEnabled", "number", ["number"]), this.QTS_SetDebugLogEnabled = this.module.cwrap("QTS_SetDebugLogEnabled", null, ["number", "number"]), this.QTS_BuildIsDebug = this.module.cwrap("QTS_BuildIsDebug", "number", []), this.QTS_BuildIsAsyncify = this.module.cwrap("QTS_BuildIsAsyncify", "number", []), this.QTS_NewFunction = this.module.cwrap("QTS_NewFunction", "number", [
					"number",
					"string",
					"number",
					"boolean",
					"number"
				]), this.QTS_ArgvGetJSValueConstPointer = this.module.cwrap("QTS_ArgvGetJSValueConstPointer", "number", ["number", "number"]), this.QTS_RuntimeEnableInterruptHandler = this.module.cwrap("QTS_RuntimeEnableInterruptHandler", null, ["number"]), this.QTS_RuntimeDisableInterruptHandler = this.module.cwrap("QTS_RuntimeDisableInterruptHandler", null, ["number"]), this.QTS_RuntimeEnableModuleLoader = this.module.cwrap("QTS_RuntimeEnableModuleLoader", null, ["number", "number"]), this.QTS_RuntimeDisableModuleLoader = this.module.cwrap("QTS_RuntimeDisableModuleLoader", null, ["number"]), this.QTS_bjson_encode = this.module.cwrap("QTS_bjson_encode", "number", ["number", "number"]), this.QTS_bjson_decode = this.module.cwrap("QTS_bjson_decode", "number", ["number", "number"]);
			}
		};
	})), al = Me({ default: () => ul });
	async function ul(e = {}) {
		var t, n = e, r = !!globalThis.window, s = !!globalThis.WorkerGlobalScope;
		function i(A) {
			A = { log: A || function() {} };
			for (const h of i.eb) h(A);
			return n.quickJSEmscriptenExtensions = A;
		}
		i.eb = [], n.quickjsEmscriptenInit = i, i.eb.push((A) => {
			A.getWasmMemory = function() {
				return pe;
			};
		});
		var o = "./this.program", a = self.location.href, u = "", l, p;
		if (r || s) {
			try {
				u = new URL(".", a).href;
			} catch {}
			s && (p = (A) => {
				var h = new XMLHttpRequest();
				return h.open("GET", A, !1), h.responseType = "arraybuffer", h.send(null), new Uint8Array(h.response);
			}), l = async (A) => {
				if (A = await fetch(A, { credentials: "same-origin" }), A.ok) return A.arrayBuffer();
				throw Error(A.status + " : " + A.url);
			};
		}
		var f = console.log.bind(console), g = console.error.bind(console), y, B = !1, N, z, W, F, G, P, H, Ne = !1;
		function De() {
			var A = pe.buffer;
			n.HEAP8 = F = new Int8Array(A), new Int16Array(A), n.HEAPU8 = G = new Uint8Array(A), new Uint16Array(A), P = new Int32Array(A), H = new Uint32Array(A), new Float32Array(A), new Float64Array(A), new BigInt64Array(A), new BigUint64Array(A);
		}
		function fe(A) {
			throw n.onAbort?.(A), A = "Aborted(" + A + ")", g(A), B = !0, A = new WebAssembly.RuntimeError(A + ". Build with -sASSERTIONS for more info."), W?.(A), A;
		}
		var Ye;
		async function Wn(A) {
			if (!y) try {
				var h = await l(A);
				return new Uint8Array(h);
			} catch {}
			if (A == Ye && y) A = new Uint8Array(y);
			else if (p) A = p(A);
			else throw "both async and sync fetching of the wasm failed";
			return A;
		}
		async function lt(A, h) {
			try {
				var c = await Wn(A);
				return await WebAssembly.instantiate(c, h);
			} catch (b) {
				g(`failed to asynchronously prepare wasm: ${b}`), fe(b);
			}
		}
		async function te(A) {
			var h = Ye;
			if (!y) try {
				var c = fetch(h, { credentials: "same-origin" });
				return await WebAssembly.instantiateStreaming(c, A);
			} catch (b) {
				g(`wasm streaming compile failed: ${b}`), g("falling back to ArrayBuffer instantiation");
			}
			return lt(h, A);
		}
		class At {
			constructor(h) {
				rl(this, "name", "ExitStatus"), this.message = `Program terminated with exit(${h})`, this.status = h;
			}
		}
		var ct = (A) => {
			for (; 0 < A.length;) A.shift()(n);
		}, Yn = [], jt = [], Un = () => {
			var A = n.preRun.shift();
			jt.push(A);
		}, ht = !0, pe, hu = new TextDecoder(), du = (A, h, c, b) => {
			if (c = h + c, b) return c;
			for (; A[h] && !(h >= c);) ++h;
			return h;
		}, Ue = (A, h, c) => A ? hu.decode(G.subarray(A, du(G, A, h, c))) : "", Xt = 0, Ag = [
			0,
			31,
			60,
			91,
			121,
			152,
			182,
			213,
			244,
			274,
			305,
			335
		], cg = [
			0,
			31,
			59,
			90,
			120,
			151,
			181,
			212,
			243,
			273,
			304,
			334
		], qt = {}, gu = (A) => {
			if (!(A instanceof At || A == "unwind")) throw A;
		}, fu = (A) => {
			throw N = A, ht || 0 < Xt || (n.onExit?.(A), B = !0), new At(A);
		}, Ds = (A) => {
			if (!B) try {
				return A();
			} catch (h) {
				gu(h);
			} finally {
				if (!(ht || 0 < Xt)) try {
					N = A = N, fu(A);
				} catch (h) {
					gu(h);
				}
			}
		}, dt = (A, h, c) => {
			var b = G;
			if (!(0 < c)) return 0;
			var Q = h;
			c = h + c - 1;
			for (var D = 0; D < A.length; ++D) {
				var O = A.codePointAt(D);
				if (127 >= O) {
					if (h >= c) break;
					b[h++] = O;
				} else if (2047 >= O) {
					if (h + 1 >= c) break;
					b[h++] = 192 | O >> 6, b[h++] = 128 | O & 63;
				} else if (65535 >= O) {
					if (h + 2 >= c) break;
					b[h++] = 224 | O >> 12, b[h++] = 128 | O >> 6 & 63, b[h++] = 128 | O & 63;
				} else {
					if (h + 3 >= c) break;
					b[h++] = 240 | O >> 18, b[h++] = 128 | O >> 12 & 63, b[h++] = 128 | O >> 6 & 63, b[h++] = 128 | O & 63, D++;
				}
			}
			return b[h] = 0, h - Q;
		}, _s = {}, pu = () => {
			if (!Fs) {
				var A = {
					USER: "web_user",
					LOGNAME: "web_user",
					PATH: "/",
					PWD: "/",
					HOME: "/home/web_user",
					LANG: (globalThis.navigator?.language ?? "C").replace("-", "_") + ".UTF-8",
					_: o || "./this.program"
				}, h;
				for (h in _s) _s[h] === void 0 ? delete A[h] : A[h] = _s[h];
				var c = [];
				for (h in A) c.push(`${h}=${A[h]}`);
				Fs = c;
			}
			return Fs;
		}, Fs, ks = (A) => {
			for (var h = 0, c = 0; c < A.length; ++c) {
				var b = A.charCodeAt(c);
				127 >= b ? h++ : 2047 >= b ? h += 2 : 55296 <= b && 57343 >= b ? (h += 4, ++c) : h += 3;
			}
			return h;
		}, hg = [
			null,
			[],
			[]
		], Vn = (A) => {
			try {
				A();
			} catch (h) {
				fe(h);
			}
		};
		function dg(A) {
			var h = (...c) => {
				T.Sa.push(A);
				try {
					return A(...c);
				} finally {
					B || (T.Sa.pop(), T.Qa && T.state === T.Ra.Za && T.Sa.length === 0 && (T.state = T.Ra.Ua, Vn(Iu), typeof Fibers < "u" && Fibers.nb()));
				}
			};
			return T.ab.set(A, h), h;
		}
		function gg() {
			return new Promise((A, h) => {
				T.Wa = {
					resolve: A,
					reject: h
				};
			});
		}
		function fg() {
			var A = mu(12 + T.Va), h = A + 12, c = T.Va;
			return H[A >> 2] = h, H[A + 4 >> 2] = h + c, h = T.Sa[0], T.Xa.has(h) || (c = T.fb++, T.Xa.set(h, c), T.$a.set(c, h)), h = T.Xa.get(h), P[A + 8 >> 2] = h, A;
		}
		function pg() {
			var A = T.$a.get(P[T.Qa + 8 >> 2]);
			return A = T.ab.get(A), Ds(A);
		}
		var T = {
			kb(A) {
				var h = /^(qts_host_call_function|qts_host_load_module_source|qts_host_normalize_module|invoke_.*|__asyncjs__.*)$/;
				for (let [c, b] of Object.entries(A)) typeof b == "function" && (b.lb || h.test(c));
			},
			Ra: {
				Ua: 0,
				Za: 1,
				Ya: 2,
				ib: 3
			},
			state: 0,
			Va: 81920,
			Qa: null,
			cb: 0,
			Sa: [],
			Xa: /* @__PURE__ */ new Map(),
			$a: /* @__PURE__ */ new Map(),
			ab: /* @__PURE__ */ new Map(),
			fb: 0,
			Wa: null,
			hb: [],
			Ta(A) {
				if (!B) {
					if (T.state === T.Ra.Ua) {
						var h = !1, c = !1;
						A((b = 0) => {
							if (!B && (T.cb = b, h = !0, c)) {
								T.state = T.Ra.Ya, Vn(() => vu(T.Qa)), typeof MainLoop < "u" && MainLoop.gb && MainLoop.resume(), b = !1;
								try {
									var Q = pg();
								} catch (Ee) {
									Q = Ee, b = !0;
								}
								var D = !1;
								if (!T.Qa) {
									var O = T.Wa;
									O && (T.Wa = null, (b ? O.reject : O.resolve)(Q), D = !0);
								}
								if (b && !D) throw Q;
							}
						}), c = !0, h || (T.state = T.Ra.Za, T.Qa = fg(), typeof MainLoop < "u" && MainLoop.gb && MainLoop.pause(), Vn(() => Cu(T.Qa)));
					} else T.state === T.Ra.Ya ? (T.state = T.Ra.Ua, Vn(Eu), Su(T.Qa), T.Qa = null, T.hb.forEach(Ds)) : fe(`invalid state: ${T.state}`);
					return T.cb;
				}
			},
			jb: (A) => T.Ta(async (h) => {
				h(await A());
			})
		}, mg = (A, h, c, b, Q) => {
			function D(Z) {
				return --Xt, Ve !== 0 && yu(Ve), h === "string" ? Ue(Z) : h === "boolean" ? !!Z : Z;
			}
			var O = {
				string: (Z) => {
					var Ze = 0;
					if (Z != null && Z !== 0) {
						Ze = ks(Z) + 1;
						var xu = Ms(Ze);
						dt(Z, xu, Ze), Ze = xu;
					}
					return Ze;
				},
				array: (Z) => {
					var Ze = Ms(Z.length);
					return F.set(Z, Ze), Ze;
				}
			};
			A = n["_" + A];
			var Ee = [], Ve = 0;
			if (b) for (var me = 0; me < b.length; me++) {
				var xe = O[c[me]];
				xe ? (Ve === 0 && (Ve = wu()), Ee[me] = xe(b[me])) : Ee[me] = b[me];
			}
			return c = T.Qa, b = A(...Ee), Q = Q?.async, Xt += 1, T.Qa != c ? gg().then(D) : (b = D(b), Q ? Promise.resolve(b) : b);
		};
		if (n.wasmMemory ? pe = n.wasmMemory : pe = new WebAssembly.Memory({
			initial: (n.INITIAL_MEMORY || 16777216) / 65536,
			maximum: 32768
		}), De(), n.noExitRuntime && (ht = n.noExitRuntime), n.print && (f = n.print), n.printErr && (g = n.printErr), n.wasmBinary && (y = n.wasmBinary), n.thisProgram && (o = n.thisProgram), n.preInit) for (typeof n.preInit == "function" && (n.preInit = [n.preInit]); 0 < n.preInit.length;) n.preInit.shift()();
		n.cwrap = (A, h, c, b) => {
			var Q = !c || c.every((D) => D === "number" || D === "boolean");
			return h !== "string" && Q && !b ? n["_" + A] : (...D) => mg(A, h, c, D, b);
		}, n.UTF8ToString = Ue, n.stringToUTF8 = (A, h, c) => dt(A, h, c), n.lengthBytesUTF8 = ks, n.Asyncify = T;
		var mu, Su, bu, yu, Ms, wu, Cu, Iu, vu, Eu, Sg = {
			b: (A, h, c, b) => fe(`Assertion failed: ${Ue(A)}, at: ` + [
				h ? Ue(h) : "unknown filename",
				c,
				b ? Ue(b) : "unknown function"
			]),
			q: () => fe(""),
			l: () => {
				ht = !1, Xt = 0;
			},
			m: function(A, h) {
				A = -9007199254740992 > A || 9007199254740992 < A ? NaN : Number(A), A = /* @__PURE__ */ new Date(1e3 * A), P[h >> 2] = A.getSeconds(), P[h + 4 >> 2] = A.getMinutes(), P[h + 8 >> 2] = A.getHours(), P[h + 12 >> 2] = A.getDate(), P[h + 16 >> 2] = A.getMonth(), P[h + 20 >> 2] = A.getFullYear() - 1900, P[h + 24 >> 2] = A.getDay();
				var c = A.getFullYear();
				P[h + 28 >> 2] = (c % 4 !== 0 || c % 100 === 0 && c % 400 !== 0 ? cg : Ag)[A.getMonth()] + A.getDate() - 1 | 0, P[h + 36 >> 2] = -(60 * A.getTimezoneOffset()), c = new Date(A.getFullYear(), 6, 1).getTimezoneOffset();
				var b = new Date(A.getFullYear(), 0, 1).getTimezoneOffset();
				P[h + 32 >> 2] = (c != b && A.getTimezoneOffset() == Math.min(b, c)) | 0;
			},
			j: (A, h) => (qt[A] && (clearTimeout(qt[A].id), delete qt[A]), h && (qt[A] = {
				id: setTimeout(() => {
					delete qt[A], Ds(() => bu(A, performance.now()));
				}, h),
				mb: h
			}), 0),
			n: (A, h, c, b) => {
				var Q = (/* @__PURE__ */ new Date()).getFullYear(), D = new Date(Q, 0, 1).getTimezoneOffset();
				Q = new Date(Q, 6, 1).getTimezoneOffset(), H[A >> 2] = 60 * Math.max(D, Q), P[h >> 2] = +(D != Q), h = (O) => {
					var Ee = Math.abs(O);
					return `UTC${0 <= O ? "-" : "+"}${String(Math.floor(Ee / 60)).padStart(2, "0")}${String(Ee % 60).padStart(2, "0")}`;
				}, A = h(D), h = h(Q), Q < D ? (dt(A, c, 17), dt(h, b, 17)) : (dt(A, b, 17), dt(h, c, 17));
			},
			p: () => Date.now(),
			k: (A) => {
				var h = G.length;
				if (A >>>= 0, 2147483648 < A) return !1;
				for (var c = 1; 4 >= c; c *= 2) {
					var b = h * (1 + .2 / c);
					b = Math.min(b, A + 100663296);
					e: {
						b = (Math.min(2147483648, 65536 * Math.ceil(Math.max(A, b) / 65536)) - pe.buffer.byteLength + 65535) / 65536 | 0;
						try {
							pe.grow(b), De();
							var Q = 1;
							break e;
						} catch {}
						Q = void 0;
					}
					if (Q) return !0;
				}
				return !1;
			},
			d: (A, h) => {
				var c = 0, b = 0, Q;
				for (Q of pu()) {
					var D = h + c;
					H[A + b >> 2] = D, c += dt(Q, D, 1 / 0) + 1, b += 4;
				}
				return 0;
			},
			e: (A, h) => {
				var c = pu();
				H[A >> 2] = c.length, A = 0;
				for (var b of c) A += ks(b) + 1;
				return H[h >> 2] = A, 0;
			},
			c: () => 52,
			o: function() {
				return 70;
			},
			s: (A, h, c, b) => {
				for (var Q = 0, D = 0; D < c; D++) {
					var O = H[h >> 2], Ee = H[h + 4 >> 2];
					h += 8;
					for (var Ve = 0; Ve < Ee; Ve++) {
						var me = A, xe = G[O + Ve], Z = hg[me];
						xe === 0 || xe === 10 ? (me = me === 1 ? f : g, xe = du(Z, 0), xe = hu.decode(Z.buffer ? Z.subarray(0, xe) : new Uint8Array(Z.slice(0, xe))), me(xe), Z.length = 0) : Z.push(xe);
					}
					Q += Ee;
				}
				return H[b >> 2] = Q, 0;
			},
			a: pe,
			r: fu,
			i: function(A, h, c, b, Q) {
				return n.callbacks.callFunction({ handleSleep: T.Ta }, A, h, c, b, Q);
			},
			h: function(A) {
				return n.callbacks.shouldInterrupt(void 0, A);
			},
			g: function(A, h, c) {
				const b = { handleSleep: T.Ta };
				return c = Ue(c), n.callbacks.loadModuleSource(b, A, h, c);
			},
			f: function(A, h, c, b) {
				const Q = { handleSleep: T.Ta };
				return c = Ue(c), b = Ue(b), n.callbacks.normalizeModule(Q, A, h, c, b);
			},
			t: function(A, h) {
				n.callbacks.freeHostRef(void 0, A, h);
			},
			u: function(A, h) {
				T.Va = A || h;
			}
		}, Zn = await (async function() {
			function A(c) {
				var b = Zn = c.exports;
				c = {};
				for (let [Q, D] of Object.entries(b)) typeof D == "function" ? (b = dg(D), c[Q] = b) : c[Q] = D;
				return c = Zn = c, mu = n._malloc = c.w, n._QTS_Throw = c.x, n._QTS_NewError = c.y, n._QTS_RuntimeSetMemoryLimit = c.z, n._QTS_RuntimeComputeMemoryUsage = c.A, n._QTS_RuntimeDumpMemoryUsage = c.B, n._QTS_RecoverableLeakCheck = c.C, n._QTS_BuildIsSanitizeLeak = c.D, n._QTS_RuntimeSetMaxStackSize = c.E, n._QTS_GetUndefined = c.F, n._QTS_GetNull = c.G, n._QTS_GetFalse = c.H, n._QTS_GetTrue = c.I, n._QTS_NewHostRef = c.J, n._QTS_GetHostRefId = c.K, n._QTS_NewRuntime = c.L, n._QTS_FreeRuntime = c.M, Su = n._free = c.N, n._QTS_NewContext = c.O, n._QTS_FreeContext = c.P, n._QTS_FreeValuePointer = c.Q, n._QTS_FreeValuePointerRuntime = c.R, n._QTS_FreeVoidPointer = c.S, n._QTS_FreeCString = c.T, n._QTS_DupValuePointer = c.U, n._QTS_NewObject = c.V, n._QTS_NewObjectProto = c.W, n._QTS_NewArray = c.X, n._QTS_NewArrayBuffer = c.Y, n._QTS_NewFloat64 = c.Z, n._QTS_GetFloat64 = c._, n._QTS_NewString = c.$, n._QTS_GetString = c.aa, n._QTS_GetArrayBuffer = c.ba, n._QTS_GetArrayBufferLength = c.ca, n._QTS_NewSymbol = c.da, n._QTS_GetSymbolDescriptionOrKey = c.ea, n._QTS_IsGlobalSymbol = c.fa, n._QTS_IsJobPending = c.ga, n._QTS_ExecutePendingJob = c.ha, n._QTS_GetProp = c.ia, n._QTS_GetPropNumber = c.ja, n._QTS_SetProp = c.ka, n._QTS_DefineProp = c.la, n._QTS_GetOwnPropertyNames = c.ma, n._QTS_Call = c.na, n._QTS_ResolveException = c.oa, n._QTS_Dump = c.pa, n._QTS_Eval = c.qa, n._QTS_GetModuleNamespace = c.ra, n._QTS_Typeof = c.sa, n._QTS_GetLength = c.ta, n._QTS_IsEqual = c.ua, n._QTS_GetGlobalObject = c.va, n._QTS_NewPromiseCapability = c.wa, n._QTS_PromiseState = c.xa, n._QTS_PromiseResult = c.ya, n._QTS_TestStringArg = c.za, n._QTS_GetDebugLogEnabled = c.Aa, n._QTS_SetDebugLogEnabled = c.Ba, n._QTS_BuildIsDebug = c.Ca, n._QTS_BuildIsAsyncify = c.Da, n._QTS_NewFunction = c.Ea, n._QTS_ArgvGetJSValueConstPointer = c.Fa, n._QTS_RuntimeEnableInterruptHandler = c.Ga, n._QTS_RuntimeDisableInterruptHandler = c.Ha, n._QTS_RuntimeEnableModuleLoader = c.Ia, n._QTS_RuntimeDisableModuleLoader = c.Ja, n._QTS_bjson_encode = c.Ka, n._QTS_bjson_decode = c.La, bu = c.Ma, yu = c.Na, Ms = c.Oa, wu = c.Pa, c._a, c.bb, c.sb, Cu = c.tb, Iu = c.ub, vu = c.vb, Eu = c.wb, Zn;
			}
			var h = { a: Sg };
			return n.instantiateWasm ? new Promise((c) => {
				n.instantiateWasm(h, (b, Q) => {
					c(A(b, Q));
				});
			}) : (Ye ??= n.locateFile ? n.locateFile ? n.locateFile("emscripten-module.wasm", u) : u + "emscripten-module.wasm" : new URL("/fluxa/assets/emscripten-module-B1g2L2eS.wasm", "" + self.location.href).href, A((await te(h)).instance));
		})();
		return (function() {
			function A() {
				if (n.calledRun = !0, !B) {
					if (Ne = !0, Zn.v(), z?.(n), n.onRuntimeInitialized?.(), n.postRun) for (typeof n.postRun == "function" && (n.postRun = [n.postRun]); n.postRun.length;) {
						var h = n.postRun.shift();
						Yn.push(h);
					}
					ct(Yn);
				}
			}
			if (n.preRun) for (typeof n.preRun == "function" && (n.preRun = [n.preRun]); n.preRun.length;) Un();
			ct(jt), n.setStatus ? (n.setStatus("Running..."), setTimeout(() => {
				setTimeout(() => n.setStatus(""), 1), A();
			}, 1)) : A();
		})(), Ne ? t = n : t = new Promise((A, h) => {
			z = A, W = h;
		}), t;
	}
	var ll = Se((() => {
		sl();
	})), Al = {
		type: "async",
		importFFI: () => Promise.resolve().then(() => (ol(), il)).then((e) => e.QuickJSAsyncFFI),
		importModuleLoader: () => Promise.resolve().then(() => (ll(), al)).then((e) => e.default)
	};
	async function cl(e = Al) {
		return $u(e);
	}
	async function hl(e) {
		return (await cl()).newContext(e);
	}
	const un = Symbol("changed"), gt = Symbol("classList"), be = Symbol("CustomElements"), ln = Symbol("content"), Sr = Symbol("dataset"), $e = Symbol("doctype"), br = Symbol("DOMParser"), v = Symbol("end"), Qt = Symbol("EventTarget"), An = Symbol("globals"), ye = Symbol("image"), ft = Symbol("mime"), He = Symbol("MutationObserver"), w = Symbol("next"), li = Symbol("ownerElement"), X = Symbol("prev"), ee = Symbol("private"), pt = Symbol("sheet"), se = Symbol("start"), yr = Symbol("style"), Rt = Symbol("upgrade"), M = Symbol("value");
	var wr;
	const dl = /* @__PURE__ */ new Map([
		[0, 65533],
		[128, 8364],
		[130, 8218],
		[131, 402],
		[132, 8222],
		[133, 8230],
		[134, 8224],
		[135, 8225],
		[136, 710],
		[137, 8240],
		[138, 352],
		[139, 8249],
		[140, 338],
		[142, 381],
		[145, 8216],
		[146, 8217],
		[147, 8220],
		[148, 8221],
		[149, 8226],
		[150, 8211],
		[151, 8212],
		[152, 732],
		[153, 8482],
		[154, 353],
		[155, 8250],
		[156, 339],
		[158, 382],
		[159, 376]
	]), Ai = (wr = String.fromCodePoint) !== null && wr !== void 0 ? wr : ((e) => {
		let t = "";
		return e > 65535 && (e -= 65536, t += String.fromCharCode(e >>> 10 & 1023 | 55296), e = 56320 | e & 1023), t += String.fromCharCode(e), t;
	});
	function gl(e) {
		var t;
		return e >= 55296 && e <= 57343 || e > 1114111 ? 65533 : (t = dl.get(e)) !== null && t !== void 0 ? t : e;
	}
	function ci(e) {
		const t = typeof atob == "function" ? atob(e) : typeof Buffer.from == "function" ? Buffer.from(e, "base64").toString("binary") : new Buffer(e, "base64").toString("binary"), n = t.length & -2, r = new Uint16Array(n / 2);
		for (let s = 0, i = 0; s < n; s += 2) {
			const o = t.charCodeAt(s), a = t.charCodeAt(s + 1);
			r[i++] = o | a << 8;
		}
		return r;
	}
	const fl = ci("QR08ALkAAgH6AYsDNQR2BO0EPgXZBQEGLAbdBxMISQrvCmQLfQurDKQNLw4fD4YPpA+6D/IPAAAAAAAAAAAAAAAAKhBMEY8TmxUWF2EYLBkxGuAa3RsJHDscWR8YIC8jSCSIJcMl6ie3Ku8rEC0CLjoupS7kLgAIRU1hYmNmZ2xtbm9wcnN0dVQAWgBeAGUAaQBzAHcAfgCBAIQAhwCSAJoAoACsALMAbABpAGcAO4DGAMZAUAA7gCYAJkBjAHUAdABlADuAwQDBQHIiZXZlAAJhAAFpeW0AcgByAGMAO4DCAMJAEGRyAADgNdgE3XIAYQB2AGUAO4DAAMBA8CFoYZFj4SFjcgBhZAAAoFMqAAFncIsAjgBvAG4ABGFmAADgNdg43fAlbHlGdW5jdGlvbgCgYSBpAG4AZwA7gMUAxUAAAWNzpACoAHIAAOA12Jzc6SFnbgCgVCJpAGwAZABlADuAwwDDQG0AbAA7gMQAxEAABGFjZWZvcnN1xQDYANoA7QDxAPYA+QD8AAABY3LJAM8AayNzbGFzaAAAoBYidgHTANUAAKDnKmUAZAAAoAYjeQARZIABY3J0AOAA5QDrAGEidXNlAACgNSLuI291bGxpcwCgLCFhAJJjcgAA4DXYBd1wAGYAAOA12Dnd5SF2ZdhiYwDyAOoAbSJwZXEAAKBOIgAHSE9hY2RlZmhpbG9yc3UXARoBHwE6AVIBVQFiAWQBZgGCAakB6QHtAfIBYwB5ACdkUABZADuAqQCpQIABY3B5ACUBKAE1AfUhdGUGYWmg0iJ0KGFsRGlmZmVyZW50aWFsRAAAoEUhbCJleXMAAKAtIQACYWVpb0EBRAFKAU0B8iFvbgxhZABpAGwAO4DHAMdAcgBjAAhhbiJpbnQAAKAwIm8AdAAKYQABZG5ZAV0BaSJsbGEAuGB0I2VyRG90ALdg8gA5AWkAp2NyImNsZQAAAkRNUFRwAXQBeQF9AW8AdAAAoJkiaSJudXMAAKCWIuwhdXMAoJUiaSJtZXMAAKCXIm8AAAFjc4cBlAFrKndpc2VDb250b3VySW50ZWdyYWwAAKAyImUjQ3VybHkAAAFEUZwBpAFvJXVibGVRdW90ZQAAoB0gdSJvdGUAAKAZIAACbG5wdbABtgHNAdgBbwBuAGWgNyIAoHQqgAFnaXQAvAHBAcUB8iJ1ZW50AKBhIm4AdAAAoC8i7yV1ckludGVncmFsAKAuIgABZnLRAdMBAKACIe8iZHVjdACgECJuLnRlckNsb2Nrd2lzZUNvbnRvdXJJbnRlZ3JhbAAAoDMi7yFzcwCgLypjAHIAAOA12J7ccABDoNMiYQBwAACgTSKABURKU1phY2VmaW9zAAsCEgIVAhgCGwIsAjQCOQI9AnMCfwNvoEUh9CJyYWhkAKARKWMAeQACZGMAeQAFZGMAeQAPZIABZ3JzACECJQIoAuchZXIAoCEgcgAAoKEhaAB2AACg5CoAAWF5MAIzAvIhb24OYRRkbAB0oAciYQCUY3IAAOA12AfdAAFhZkECawIAAWNtRQJnAvIjaXRpY2FsAAJBREdUUAJUAl8CYwJjInV0ZQC0YG8AdAFZAloC2WJiJGxlQWN1dGUA3WJyImF2ZQBgYGkibGRlANxi7yFuZACgxCJmJWVyZW50aWFsRAAAoEYhcAR9AgAAAAAAAIECjgIAABoDZgAA4DXYO91EoagAhQKJAm8AdAAAoNwgcSJ1YWwAAKBQIuIhbGUAA0NETFJVVpkCqAK1Au8C/wIRA28AbgB0AG8AdQByAEkAbgB0AGUAZwByAGEA7ADEAW8AdAKvAgAAAACwAqhgbiNBcnJvdwAAoNMhAAFlb7kC0AJmAHQAgAFBUlQAwQLGAs0CciJyb3cAAKDQIekkZ2h0QXJyb3cAoNQhZQDlACsCbgBnAAABTFLWAugC5SFmdAABQVLcAuECciJyb3cAAKD4J+kkZ2h0QXJyb3cAoPon6SRnaHRBcnJvdwCg+SdpImdodAAAAUFU9gL7AnIicm93AACg0iFlAGUAAKCoInAAQQIGAwAAAAALA3Iicm93AACg0SFvJHduQXJyb3cAAKDVIWUlcnRpY2FsQmFyAACgJSJuAAADQUJMUlRhJAM2AzoDWgNxA3oDciJyb3cAAKGTIUJVLAMwA2EAcgAAoBMpcCNBcnJvdwAAoPUhciJldmUAEWPlIWZ00gJDAwAASwMAAFIDaSVnaHRWZWN0b3IAAKBQKWUkZVZlY3RvcgAAoF4p5SJjdG9yQqC9IWEAcgAAoFYpaSJnaHQA1AFiAwAAaQNlJGVWZWN0b3IAAKBfKeUiY3RvckKgwSFhAHIAAKBXKWUAZQBBoKQiciJyb3cAAKCnIXIAcgBvAPcAtAIAAWN0gwOHA3IA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YwAAoN8i4SFycnCgtiEAoD0pgKIqImJjZG9zACsVMBU6FT4VQRVyImNhcAAAoEgqAAFhdTQVNxVwAACgRipwAACgSipvAHQAAKCNInIAAKBFKgDgKiIA/gACYWxydksVURVuFXMVcgByAG2gtyEAoDwpeQCAAWV2dwBYFWUVaRVxAHACXxUAAAAAYxVyAGUA4wAXFXUA4wAZFWUAZQAAoM4iZSJkZ2UAAKDPImUAbgA7gKQApEBlI2Fycm93AAABbHJ7FX8V5SFmdACgtiFpImdodAAAoLchZQDkAG0VAAFjaYsVkRVvAG4AaQBuAPQAkwFuAHQAAKAxImwiY3R5AACgLSOACUFIYWJjZGVmaGlqbG9yc3R1d3oAuBW7Fb8V1RXgFegV+RUKFhUWHxZUFlcWZRbFFtsW7xb7FgUXChdyAPIAtAJhAHIAAKBlKQACZ2xyc8YVyhXOFdAV5yFlcgCgICDlIXRoAKA4IfIA9QxoAHagECAAoKMiawHZFd4VYSJyb3cAAKAPKWEA4wBfAgABYXnkFecV8iFvbg9hNGQAoUYhYW/tFfQVAAFnciEC8RVyAACgyiF0InNlcQAAoHcqgAFnbG0A/xUCFgUWO4CwALBAdABhALRjcCJ0eXYAAKCxKQABaXIOFhIW8yFodACgfykA4DXYId1hAHIAAAFschsWHRYAoMMhAKDCIYACYWVnc3YAKBauAjYWOhY+Fm0AAKHEIm9zLhY0Fm4AZABzoMQi9SFpdACgZiZhIm1tYQDdY2kAbgAAoPIiAKH3AGlvQxZRFmQAZQAAgfcAO29KFksW90BuI3RpbWVzAACgxyJuAPgAUBZjAHkAUmRjAG8CXhYAAAAAYhZyAG4AAKAeI28AcAAAoA0jgAJscHR1dwBuFnEWdRaSFp4W7CFhciRgZgAA4DXYVd0AotkCZW1wc30WhBaJFo0WcQBkoFAibwB0AACgUSJpIm51cwAAoDgi7CF1cwCgFCLxInVhcmUAoKEiYgBsAGUAYgBhAHIAdwBlAGQAZwDlANcAbgCAAWFkaAClFqoWtBZyAHIAbwD3APUMbwB3AG4AYQByAHIAbwB3APMA8xVhI3Jwb29uAAABbHK8FsAWZQBmAPQAHBZpAGcAaAD0AB4WYgHJFs8WawBhAHIAbwD3AJILbwLUFgAAAADYFnIAbgAAoB8jbwBwAACgDCOAAWNvdADhFukW7BYAAXJ55RboFgDgNdi53FVkbAAAoPYp8iFvaxFhAAFkcvMW9xZvAHQAAKDxImkA5qC/JVsSAAFhaP8WAhdyAPIANQNhAPIA1wvhIm5nbGUAoKYpAAFjaQ4XEBd5AF9k5yJyYXJyAKD/JwAJRGFjZGVmZ2xtbm9wcXJzdHV4MRc4F0YXWxcyBF4XaRd5F40XrBe0F78X2RcVGCEYLRg1GEAYAAFEbzUXgRZvAPQA+BUAAWNzPBdCF3UAdABlADuA6QDpQPQhZXIAoG4qAAJhaW95TRdQF1YXWhfyIW9uG2FyAGOgViI7gOoA6kDsIW9uAKBVIk1kbwB0ABdhAAFEcmIXZhdvAHQAAKBSIgDgNdgi3XKhmipuF3QXYQB2AGUAO4DoAOhAZKCWKm8AdAAAoJgqgKGZKmlscwCAF4UXhxfuInRlcnMAoOcjAKATIWSglSpvAHQAAKCXKoABYXBzAJMXlheiF2MAcgATYXQAeQBzogUinxcAAAAAoRdlAHQAAKAFInAAMaADIDMBqRerFwCgBCAAoAUgAAFnc7AXsRdLYXAAAKACIAABZ3C4F7sXbwBuABlhZgAA4DXYVt2AAWFscwDFF8sXzxdyAHOg1SJsAACg4yl1AHMAAKBxKmkAAKG1A2x21RfYF28AbgC1Y/VjAAJjc3V24BfoF/0XEBgAAWlv5BdWF3IAYwAAoFYiaQLuFwAAAADwF+0ADQThIW50AAFnbPUX+Rd0AHIAAKCWKuUhc3MAoJUqgAFhZWkAAxgGGAoYbABzAD1gcwB0AACgXyJ2AESgYSJEAACgeCrwImFyc2wAoOUpAAFEYRkYHRhvAHQAAKBTInIAcgAAoHEpgAFjZGkAJxgqGO0XcgAAoC8hbwD0AIwCAAFhaDEYMhi3YzuA8ADwQAABbXI5GD0YbAA7gOsA60BvAACgrCCAAWNpcABGGEgYSxhsACFgcwD0ACwEAAFlb08YVxhjAHQAYQB0AGkAbwDuABoEbgBlAG4AdABpAGEAbADlADME4Ql1GAAAgRgAAIMYiBgAAAAAoRilGAAAqhgAALsYvhjRGAAA1xgnGWwAbABpAG4AZwBkAG8AdABzAGUA8QBlF3kARGRtImFsZQAAoEAmgAFpbHIAjRiRGJ0Y7CFpZwCgA/tpApcYAAAAAJoYZwAAoAD7aQBnAACgBPsA4DXYI93sIWlnAKAB++whaWcA4GYAagCAAWFsdACvGLIYthh0AACgbSZpAGcAAKAC+24AcwAAoLElbwBmAJJh8AHCGAAAxhhmAADgNdhX3QABYWvJGMwYbADsAGsEdqDUIgCg2SphI3J0aW50AACgDSoAAWFv2hgiGQABY3PeGB8ZsQPnGP0YBRkSGRUZAAAdGbID7xjyGPQY9xj5GAAA+xg7gL0AvUAAoFMhO4C8ALxAAKBVIQCgWSEAoFshswEBGQAAAxkAoFQhAKBWIbQCCxkOGQAAAAAQGTuAvgC+QACgVyEAoFwhNQAAoFghtgEZGQAAGxkAoFohAKBdITgAAKBeIWwAAKBEIHcAbgAAoCIjYwByAADgNdi73IAIRWFiY2RlZmdpamxub3JzdHYARhlKGVoZXhlmGWkZkhmWGZkZnRmgGa0ZxhnLGc8Z4BkjGmygZyIAoIwqgAFjbXAAUBlTGVgZ9SF0ZfVhbQBhAOSgswM6FgCghipyImV2ZQAfYQABaXliGWUZcgBjAB1hM2RvAHQAIWGAoWUibHFzAMYEcBl6GfGhZSLOBAAAdhlsAGEAbgD0AN8EgKF+KmNkbACBGYQZjBljAACgqSpvAHQAb6CAKmyggioAoIQqZeDbIgD+cwAAoJQqcgAA4DXYJN3noGsirATtIWVsAKA3IWMAeQBTZIChdyJFYWoApxmpGasZAKCSKgCgpSoAoKQqAAJFYWVztBm2Gb0ZwhkAoGkicABwoIoq8iFveACgiipxoIgq8aCIKrUZaQBtAACg5yJwAGYAAOA12FjdYQB2AOUAYwIAAWNp0xnWGXIAAKAKIW0AAKFzImVs3BneGQCgjioAoJAqAIM+ADtjZGxxco0E6xn0GfgZ/BkBGgABY2nvGfEZAKCnKnIAAKB6Km8AdAAAoNci0CFhcgCglSl1ImVzdAAAoHwqgAJhZGVscwAKGvQZFhrVBCAa8AEPGgAAFBpwAHIAbwD4AFkZcgAAoHgpcQAAAWxxxAQbGmwAZQBzAPMASRlpAO0A5AQAAWVuJxouGnIjdG5lcXEAAOBpIgD+xQAsGgAFQWFiY2Vma29zeUAaQxpmGmoabRqDGocalhrCGtMacgDyAMwCAAJpbG1yShpOGlAaVBpyAHMA8ABxD2YAvWBpAGwA9AASBQABZHJYGlsaYwB5AEpkAKGUIWN3YBpkGmkAcgAAoEgpAKCtIWEAcgAAoA8h6SFyYyVhgAFhbHIAcxp7Gn8a8iF0c3WgZSZpAHQAAKBlJuwhaXAAoCYg4yFvbgCguSJyAADgNdgl3XMAAAFld4wakRphInJvdwAAoCUpYSJyb3cAAKAmKYACYW1vcHIAnxqjGqcauhq+GnIAcgAAoP8h9CFodACgOyJrAAABbHKsGrMaZSRmdGFycm93AACgqSHpJGdodGFycm93AKCqIWYAAOA12Fnd4iFhcgCgFSCAAWNsdADIGswa0BpyAADgNdi93GEAcwDoAGka8iFvaydhAAFicNca2xr1IWxsAKBDIOghZW4AoBAg4Qr2GgAA/RoAAAgbExsaGwAAIRs7GwAAAAA+G2IbmRuVG6sbAACyG80b0htjAHUAdABlADuA7QDtQAChYyBpeQEbBhtyAGMAO4DuAO5AOGQAAWN4CxsNG3kANWRjAGwAO4ChAKFAAAFmcssCFhsA4DXYJt1yAGEAdgBlADuA7ADsQIChSCFpbm8AJxsyGzYbAAFpbisbLxtuAHQAAKAMKnQAAKAtIuYhaW4AoNwpdABhAACgKSHsIWlnM2GAAWFvcABDG1sbXhuAAWNndABJG0sbWRtyACthgAFlbHAAcQVRG1UbaQBuAOUAyAVhAHIA9AByBWgAMWFmAACgtyJlAGQAtWEAoggiY2ZvdGkbbRt1G3kb4SFyZQCgBSFpAG4AdKAeImkAZQAAoN0pZABvAPQAWxsAoisiY2VscIEbhRuPG5QbYQBsAACguiIAAWdyiRuNG2UAcgDzACMQ4wCCG2EicmhrAACgFyryIW9kAKA8KgACY2dwdJ8boRukG6gbeQBRZG8AbgAvYWYAAOA12FrdYQC5Y3UAZQBzAHQAO4C/AL9AAAFjabUbuRtyAADgNdi+3G4AAKIIIkVkc3bCG8QbyBvQAwCg+SJvAHQAAKD1Inag9CIAoPMiaaBiIOwhZGUpYesB1hsAANkbYwB5AFZkbAA7gO8A70AAA2NmbW9zdeYb7hvyG/Ub+hsFHAABaXnqG+0bcgBjADVhOWRyAADgNdgn3eEhdGg3YnAAZgAA4DXYW93jAf8bAAADHHIAAOA12L/c8iFjeVhk6yFjeVRkAARhY2ZnaGpvcxUcGhwiHCYcKhwtHDAcNRzwIXBhdqC6A/BjAAFleR4cIRzkIWlsN2E6ZHIAAOA12CjdciJlZW4AOGFjAHkARWRjAHkAXGRwAGYAAOA12FzdYwByAADgNdjA3IALQUJFSGFiY2RlZmdoamxtbm9wcnN0dXYAXhxtHHEcdRx5HN8cBx0dHTwd3B3tHfEdAR4EHh0eLB5FHrwewx7hHgkfPR9LH4ABYXJ0AGQcZxxpHHIA8gBvB/IAxQLhIWlsAKAbKeEhcnIAoA4pZ6BmIgCgiyphAHIAAKBiKWMJjRwAAJAcAACVHAAAAAAAAAAAAACZHJwcAACmHKgcrRwAANIc9SF0ZTph7SJwdHl2AKC0KXIAYQDuAFoG4iFkYbtjZwAAoegnZGyhHKMcAKCRKeUAiwYAoIUqdQBvADuAqwCrQHIAgKOQIWJmaGxwc3QAuhy/HMIcxBzHHMoczhxmoOQhcwAAoB8pcwAAoB0p6wCyGnAAAKCrIWwAAKA5KWkAbQAAoHMpbAAAoKIhAKGrKmFl1hzaHGkAbAAAoBkpc6CtKgDgrSoA/oABYWJyAOUc6RztHHIAcgAAoAwpcgBrAACgcicAAWFr8Rz4HGMAAAFla/Yc9xx7YFtgAAFlc/wc/hwAoIspbAAAAWR1Ax0FHQCgjykAoI0pAAJhZXV5Dh0RHRodHB3yIW9uPmEAAWRpFR0YHWkAbAA8YewAowbiAPccO2QAAmNxcnMkHScdLB05HWEAAKA2KXUAbwDyoBwgqhEAAWR1MB00HeghYXIAoGcpcyJoYXIAAKBLKWgAAKCyIQCiZCJmZ3FzRB1FB5Qdnh10AIACYWhscnQATh1WHWUdbB2NHXIicm93AHSgkCFhAOkAzxxhI3Jwb29uAAABZHVeHWId7yF3bgCgvSFwAACgvCHlJGZ0YXJyb3dzAKDHIWkiZ2h0AIABYWhzAHUdex2DHXIicm93APOglCGdBmEAcgBwAG8AbwBuAPMAzgtxAHUAaQBnAGEAcgByAG8A9wBlGugkcmVldGltZXMAoMsi8aFkIk0HAACaHWwAYQBuAPQAXgcAon0qY2Rnc6YdqR2xHbcdYwAAoKgqbwB0AG+gfypyoIEqAKCDKmXg2iIA/nMAAKCTKoACYWRlZ3MAwB3GHcod1h3ZHXAAcAByAG8A+ACmHG8AdAAAoNYicQAAAWdxzx3SHXQA8gBGB2cAdADyAHQcdADyAFMHaQDtAGMHgAFpbHIA4h3mHeod8yFodACgfClvAG8A8gDKBgDgNdgp3UWgdiIAoJEqYQH1Hf4dcgAAAWR1YB35HWygvCEAoGopbABrAACghCVjAHkAWWQAomoiYWNodAweDx4VHhkecgDyAGsdbwByAG4AZQDyAGAW4SFyZACgaylyAGkAAKD6JQABaW8hHiQe5CFvdEBh9SFzdGGgsCPjIWhlAKCwIwACRWFlczMeNR48HkEeAKBoInAAcKCJKvIhb3gAoIkqcaCHKvGghyo0HmkAbQAAoOYiAARhYm5vcHR3elIeXB5fHoUelh6mHqsetB4AAW5yVh5ZHmcAAKDsJ3IAAKD9IXIA6wCwBmcAgAFsbXIAZh52Hnse5SFmdAABYXKIB2weaQBnAGgAdABhAHIAcgBvAPcAkwfhInBzdG8AoPwnaQBnAGgAdABhAHIAcgBvAPcAmgdwI2Fycm93AAABbHKNHpEeZQBmAPQAxhxpImdodAAAoKwhgAFhZmwAnB6fHqIecgAAoIUpAOA12F3ddQBzAACgLSppIm1lcwAAoDQqYQGvHrMecwB0AACgFyLhAIoOZaHKJbkeRhLuIWdlAKDKJWEAcgBsoCgAdAAAoJMpgAJhY2htdADMHs8e1R7bHt0ecgDyAJ0GbwByAG4AZQDyANYWYQByAGSgyyEAoG0pAKAOIHIAaQAAoL8iAANhY2hpcXTrHu8e1QfzHv0eBh/xIXVvAKA5IHIAAOA12MHcbQDloXIi+h4AAPweAKCNKgCgjyoAAWJ19xwBH28AcqAYIACgGiDyIW9rQmEAhDwAO2NkaGlscXJCBhcfxh0gHyQfKB8sHzEfAAFjaRsfHR8AoKYqcgAAoHkqcgBlAOUAkx3tIWVzAKDJIuEhcnIAoHYpdSJlc3QAAKB7KgABUGk1HzkfYQByAACglillocMlAgdfEnIAAAFkdUIfRx9zImhhcgAAoEop6CFhcgCgZikAAWVuTx9WH3IjdG5lcXEAAOBoIgD+xQBUHwAHRGFjZGVmaGlsbm9wc3VuH3Ifoh+rH68ftx+7H74f5h/uH/MfBwj/HwsgxCFvdACgOiIAAmNscHJ5H30fiR+eH3IAO4CvAK9AAAFldIEfgx8AoEImZaAgJ3MAZQAAoCAnc6CmIXQAbwCAoaYhZGx1AJQfmB+cH28AdwDuAHkDZQBmAPQA6gbwAOkO6yFlcgCgriUAAW95ph+qH+0hbWEAoCkqPGThIXNoAKAUIOElc3VyZWRhbmdsZQCgISJyAADgNdgq3W8AAKAnIYABY2RuAMQfyR/bH3IAbwA7gLUAtUBhoiMi0B8AANMf1x9zAPQAKxFpAHIAAKDwKm8AdAA7gLcAt0B1AHMA4qESIh4TAADjH3WgOCIAoCoqYwHqH+0fcAAAoNsq8gB+GnAAbAB1APMACAgAAWRw9x/7H+UhbHMAoKciZgAA4DXYXt0AAWN0AyAHIHIAAOA12MLc8CFvcwCgPiJsobwDECAVIPQiaW1hcACguCJhAPAAEyAADEdMUlZhYmNkZWZnaGlqbG1vcHJzdHV2dzwgRyBmIG0geSCqILgg2iDeIBEhFSEyIUMhTSFQIZwhnyHSIQAiIyKLIrEivyIUIwABZ3RAIEMgAODZIjgD9uBrItIgBwmAAWVsdABNIF8gYiBmAHQAAAFhclMgWCByInJvdwAAoM0h6SRnaHRhcnJvdwCgziEA4NgiOAP24Goi0iBfCekkZ2h0YXJyb3cAoM8hAAFEZHEgdSDhIXNoAKCvIuEhc2gAoK4igAJiY25wdACCIIYgiSCNIKIgbABhAACgByL1IXRlRGFnAADgICLSIACiSSJFaW9wlSCYIJwgniAA4HAqOANkAADgSyI4A3MASWFyAG8A+AAyCnUAcgBhoG4mbADzoG4mmwjzAa8gAACzIHAAO4CgAKBAbQBwAOXgTiI4AyoJgAJhZW91eQDBIMogzSDWINkg8AHGIAAAyCAAoEMqbwBuAEhh5CFpbEZhbgBnAGSgRyJvAHQAAOBtKjgDcAAAoEIqPWThIXNoAKATIACjYCJBYWRxc3jpIO0g+SD+IAIhDCFyAHIAAKDXIXIAAAFocvIg9SBrAACgJClvoJch9wAGD28AdAAA4FAiOAN1AGkA9gC7CAABZWkGIQohYQByAACgKCntAN8I6SFzdPOgBCLlCHIAAOA12CvdAAJFZXN0/wgcISshLiHxoXEiIiEAABMJ8aFxIgAJAAAnIWwAYQBuAPQAEwlpAO0AGQlyoG8iAKBvIoABQWFwADghOyE/IXIA8gBeIHIAcgAAoK4hYQByAACg8ipzogsiSiEAAAAAxwtkoPwiAKD6ImMAeQBaZIADQUVhZGVzdABcIV8hYiFmIWkhkyGWIXIA8gBXIADgZiI4A3IAcgAAoJohcgAAoCUggKFwImZxcwBwIYQhjiF0AAABYXJ1IXohcgByAG8A9wBlIWkAZwBoAHQAYQByAHIAbwD3AD4h8aFwImAhAACKIWwAYQBuAPQAZwlz4H0qOAMAoG4iaQDtAG0JcqBuImkA5aDqIkUJaQDkADoKAAFwdKMhpyFmAADgNdhf3YCBrAA7aW4AriGvIcchrEBuAIChCSJFZHYAtyG6Ib8hAOD5IjgDbwB0AADg9SI4A+EB1gjEIcYhAKD3IgCg9iJpAHagDCLhAagJzyHRIQCg/iIAoP0igAFhb3IA2CHsIfEhcgCAoSYiYXN0AOAh5SHpIWwAbABlAOwAywhsAADg/SrlIADgAiI4A2wiaW50AACgFCrjoYAi9yEAAPohdQDlAJsJY+CvKjgDZaCAIvEAkwkAAkFhaXQHIgoiFyIeInIA8gBsIHIAcgAAoZshY3cRIhQiAOAzKTgDAOCdITgDZyRodGFycm93AACgmyFyAGkA5aDrIr4JgANjaGltcHF1AC8iPCJHIpwhTSJQIloigKGBImNlcgA2Iv0JOSJ1AOUABgoA4DXYw9zvIXJ0bQKdIQAAAABEImEAcgDhAOEhbQBloEEi8aBEIiYKYQDyAMsIcwB1AAABYnBWIlgi5QDUCeUA3wmAAWJjcABgInMieCKAoYQiRWVzAGci7glqIgDgxSo4A2UAdABl4IIi0iBxAPGgiCJoImMAZaCBIvEA/gmAoYUiRWVzAH8iFgqCIgDgxio4A2UAdABl4IMi0iBxAPGgiSKAIgACZ2lscpIilCKaIpwi7AAMCWwAZABlADuA8QDxQOcAWwlpI2FuZ2xlAAABbHKkIqoi5SFmdGWg6iLxAEUJaSJnaHQAZaDrIvEAvgltoL0DAKEjAGVzuCK8InIAbwAAoBYhcAAAoAcggARESGFkZ2lscnMAziLSItYi2iLeIugi7SICIw8j4SFzaACgrSLhIXJyAKAEKXAAAOBNItIg4SFzaACgrCIAAWV04iLlIgDgZSLSIADgPgDSIG4iZmluAACg3imAAUFldADzIvci+iJyAHIAAKACKQDgZCLSIHLgPADSIGkAZQAA4LQi0iAAAUF0BiMKI3IAcgAAoAMp8iFpZQDgtSLSIGkAbQAA4Dwi0iCAAUFhbgAaIx4jKiNyAHIAAKDWIXIAAAFociMjJiNrAACgIylvoJYh9wD/DuUhYXIAoCcpUxJqFAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAVCMAAF4jaSN/I4IjjSOeI8AUAAAAAKYjwCMAANoj3yMAAO8jHiQvJD8kRCQAAWNzVyNsFHUAdABlADuA8wDzQAABaXlhI2cjcgBjoJoiO4D0APRAPmSAAmFiaW9zAHEjdCN3I3EBeiNzAOgAdhTsIWFjUWF2AACgOCrvIWxkAKC8KewhaWdTYQABY3KFI4kjaQByAACgvykA4DXYLN1vA5QjAAAAAJYjAACcI24A22JhAHYAZQA7gPIA8kAAoMEpAAFibaEjjAphAHIAAKC1KQACYWNpdKwjryO6I70jcgDyAFkUAAFpcrMjtiNyAACgvinvIXNzAKC7KW4A5QDZCgCgwCmAAWFlaQDFI8gjyyNjAHIATWFnAGEAyWOAAWNkbgDRI9Qj1iPyIW9uv2MAoLYpdQDzAHgBcABmAADgNdhg3YABYWVsAOQj5yPrI3IAAKC3KXIAcAAAoLkpdQDzAHwBAKMoImFkaW9zdvkj/CMPJBMkFiQbJHIA8gBeFIChXSplZm0AAyQJJAwkcgBvoDQhZgAAoDQhO4CqAKpAO4C6ALpA5yFvZgCgtiJyAACgVipsIm9wZQAAoFcqAKBbKoABY2xvACMkJSQrJPIACCRhAHMAaAA7gPgA+EBsAACgmCJpAGwBMyQ4JGQAZQA7gPUA9UBlAHMAYaCXInMAAKA2Km0AbAA7gPYA9kDiIWFyAKA9I+EKXiQAAHokAAB8JJQkAACYJKkkAAAAALUkEQsAAPAkAAAAAAQleiUAAIMlcgCAoSUiYXN0AGUkbyQBCwCBtgA7bGokayS2QGwAZQDsABgDaQJ1JAAAAAB4JG0AAKDzKgCg/Sp5AD9kcgCAAmNpbXB0AIUkiCSLJJkSjyRuAHQAJWBvAGQALmBpAGwAAKAwIOUhbmsAoDEgcgAA4DXYLd2AAWltbwCdJKAkpCR2oMYD1WNtAGEA9AD+B24AZQAAoA4m9KHAA64kAAC0JGMjaGZvcmsAAKDUItZjAAFhdbgkxCRuAAABY2u9JMIkawBooA8hAKAOIfYAaRpzAACkKwBhYmNkZW1zdNMkIRPXJNsk4STjJOck6yTjIWlyAKAjKmkAcgAAoCIqAAFvdYsW3yQAoCUqAKByKm4AO4CxALFAaQBtAACgJip3AG8AAKAnKoABaXB1APUk+iT+JO4idGludACgFSpmAADgNdhh3W4AZAA7gKMAo0CApHoiRWFjZWlub3N1ABMlFSUYJRslTCVRJVklSSV1JQCgsypwAACgtyp1AOUAPwtjoK8qgKJ6ImFjZW5zACclLSU0JTYlSSVwAHAAcgBvAPgAFyV1AHIAbAB5AGUA8QA/C/EAOAuAAWFlcwA8JUElRSXwInByb3gAoLkqcQBxAACgtSppAG0AAKDoImkA7QBEC20AZQDzoDIgIguAAUVhcwBDJVclRSXwAEAlgAFkZnAATwtfJXElgAFhbHMAZSVpJW0l7CFhcgCgLiPpIW5lAKASI/UhcmYAoBMjdKAdIu8AWQvyIWVsAKCwIgABY2l9JYElcgAA4DXYxdzIY24iY3NwAACgCCAAA2Zpb3BzdZElKxuVJZolnyWkJXIAAOA12C7dcABmAADgNdhi3XIiaW1lAACgVyBjAHIAAOA12MbcgAFhZW8AqiW6JcAldAAAAWVpryW2JXIAbgBpAG8AbgDzABkFbgB0AACgFipzAHQAZaA/APEACRj0AG0LgApBQkhhYmNkZWZoaWxtbm9wcnN0dXgA4yXyJfYl+iVpJpAmpia9JtUm5ib4JlonaCdxJ3UnnietJ7EnyCfiJ+cngAFhcnQA6SXsJe4lcgDyAJkM8gD6AuEhaWwAoBwpYQByAPIA3BVhAHIAAKBkKYADY2RlbnFydAAGJhAmEyYYJiYmKyZaJgABZXUKJg0mAOA9IjEDdABlAFVhaQDjACAN7SJwdHl2AKCzKWcAgKHpJ2RlbAAgJiImJCYAoJIpAKClKeUA9wt1AG8AO4C7ALtAcgAApZIhYWJjZmhscHN0dz0mQCZFJkcmSiZMJk4mUSZVJlgmcAAAoHUpZqDlIXMAAKAgKQCgMylzAACgHinrALka8ACVHmwAAKBFKWkAbQAAoHQpbAAAoKMhAKCdIQABYWleJmImaQBsAACgGilvAG6gNiJhAGwA8wB2C4ABYWJyAG8mciZ2JnIA8gAvEnIAawAAoHMnAAFha3omgSZjAAABZWt/JoAmfWBdYAABZXOFJocmAKCMKWwAAAFkdYwmjiYAoI4pAKCQKQACYWV1eZcmmiajJqUm8iFvbllhAAFkaZ4moSZpAGwAV2HsAA8M4gCAJkBkAAJjbHFzrSawJrUmuiZhAACgNylkImhhcgAAoGkpdQBvAPKgHSCjAWgAAKCzIYABYWNnAMMm0iaUC2wAgKEcIWlwcwDLJs4migxuAOUAoAxhAHIA9ADaC3QAAKCtJYABaWxyANsm3ybjJvMhaHQAoH0pbwBvAPIANgwA4DXYL90AAWFv6ib1JnIAAAFkde8m8SYAoMEhbKDAIQCgbCl2oMED8WOAAWducwD+Jk4nUCdoAHQAAANhaGxyc3QKJxInISc1Jz0nRydyInJvdwB0oJIhYQDpAFYmYSNycG9vbgAAAWR1GiceJ28AdwDuAPAmcAAAoMAh5SFmdAABYWgnJy0ncgByAG8AdwDzAAkMYQByAHAAbwBvAG4A8wATBGklZ2h0YXJyb3dzAACgySFxAHUAaQBnAGEAcgByAG8A9wBZJugkcmVldGltZXMAoMwiZwDaYmkAbgBnAGQAbwB0AHMAZQDxABwYgAFhaG0AYCdjJ2YncgDyAAkMYQDyABMEAKAPIG8idXN0AGGgsSPjIWhlAKCxI+0haWQAoO4qAAJhYnB0fCeGJ4knmScAAW5ygCeDJ2cAAKDtJ3IAAKD+IXIA6wAcDIABYWZsAI8nkieVJ3IAAKCGKQDgNdhj3XUAcwAAoC4qaSJtZXMAAKA1KgABYXCiJ6gncgBnoCkAdAAAoJQp7yJsaW50AKASKmEAcgDyADwnAAJhY2hxuCe8J6EMwCfxIXVvAKA6IHIAAOA12MfcAAFidYAmxCdvAPKgGSCoAYABaGlyAM4n0ifWJ3IAZQDlAE0n7SFlcwCgyiJpAIChuSVlZmwAXAxjEt4n9CFyaQCgzinsInVoYXIAoGgpAKAeIWENBSgJKA0oSyhVKIYoAACLKLAoAAAAAOMo5ygAABApJCkxKW0pcSmHKaYpAACYKgAAAACxKmMidXRlAFthcQB1AO8ABR+ApHsiRWFjZWlucHN5ABwoHignKCooLygyKEEoRihJKACgtCrwASMoAAAlKACguCpvAG4AYWF1AOUAgw1koLAqaQBsAF9hcgBjAF1hgAFFYXMAOCg6KD0oAKC2KnAAAKC6KmkAbQAAoOki7yJsaW50AKATKmkA7QCIDUFkbwB0AGKixSKRFgAAAABTKACgZiqAA0FhY21zdHgAYChkKG8ocyh1KHkogihyAHIAAKDYIXIAAAFocmkoayjrAJAab6CYIfcAzAd0ADuApwCnQGkAO2D3IWFyAKApKW0AAAFpbn4ozQBuAHUA8wDOAHQAAKA2J3IA7+A12DDdIxkAAmFjb3mRKJUonSisKHIAcAAAoG8mAAFoeZkonChjAHkASWRIZHIAdABtAqUoAAAAAKgoaQDkAFsPYQByAGEA7ABsJDuArQCtQAABZ22zKLsobQBhAAChwwNmdroouijCY4CjPCJkZWdsbnByAMgozCjPKNMo1yjaKN4obwB0AACgairxoEMiCw5FoJ4qAKCgKkWgnSoAoJ8qZQAAoEYi7CF1cwCgJCrhIXJyAKByKWEAcgDyAPwMAAJhZWl07Sj8KAEpCCkAAWxz8Sj4KGwAcwBlAHQAbQDpAH8oaABwAACgMyrwImFyc2wAoOQpAAFkbFoPBSllAACgIyNloKoqc6CsKgDgrCoA/oABZmxwABUpGCkfKfQhY3lMZGKgLwBhoMQpcgAAoD8jZgAA4DXYZN1hAAABZHIoKRcDZQBzAHWgYCZpAHQAAKBgJoABY3N1ADYpRilhKQABYXU6KUApcABzoJMiAOCTIgD+cABzoJQiAOCUIgD+dQAAAWJwSylWKQChjyJlcz4NUCllAHQAZaCPIvEAPw0AoZAiZXNIDVspZQB0AGWgkCLxAEkNAKGhJWFmZilbBHIAZQFrKVwEAKChJWEAcgDyAAMNAAJjZW10dyl7KX8pgilyAADgNdjI3HQAbQDuAM4AaQDsAAYpYQByAOYAVw0AAWFyiimOKXIA5qAGJhESAAFhbpIpoylpImdodAAAAWVwmSmgKXAAcwBpAGwAbwDuANkXaADpAKAkcwCvYIACYmNtbnAArin8KY4NJSooKgCkgiJFZGVtbnByc7wpvinCKcgpzCnUKdgp3CkAoMUqbwB0AACgvSpkoIYibwB0AACgwyr1IWx0AKDBKgABRWXQKdIpAKDLKgCgiiLsIXVzAKC/KuEhcnIAoHkpgAFlaXUA4inxKfQpdAAAoYIiZW7oKewpcQDxoIYivSllAHEA8aCKItEpbQAAoMcqAAFicPgp+ikAoNUqAKDTKmMAgKJ7ImFjZW5zAAcqDSoUKhYqRihwAHAAcgBvAPgAIyh1AHIAbAB5AGUA8QCDDfEAfA2AAWFlcwAcKiIqPShwAHAAcgBvAPgAPChxAPEAOShnAACgaiYApoMiMTIzRWRlaGxtbnBzPCo/KkIqRSpHKlIqWCpjKmcqaypzKncqO4C5ALlAO4CyALJAO4CzALNAAKDGKgABb3NLKk4qdAAAoL4qdQBiAACg2CpkoIcibwB0AACgxCpzAAABb3VdKmAqbAAAoMknYgAAoNcq4SFycgCgeyn1IWx0AKDCKgABRWVvKnEqAKDMKgCgiyLsIXVzAKDAKoABZWl1AH0qjCqPKnQAAKGDImVugyqHKnEA8aCHIkYqZQBxAPGgiyJwKm0AAKDIKgABYnCTKpUqAKDUKgCg1iqAAUFhbgCdKqEqrCpyAHIAAKDZIXIAAAFocqYqqCrrAJUab6CZIfcAxQf3IWFyAKAqKWwAaQBnADuA3wDfQOELzyrZKtwq6SrsKvEqAAD1KjQrAAAAAAAAAAAAAEwrbCsAAHErvSsAAAAAAADRK3IC1CoAAAAA2CrnIWV0AKAWI8RjcgDrAOUKgAFhZXkA4SrkKucq8iFvbmVh5CFpbGNhQmRvAPQAIg5sInJlYwAAoBUjcgAA4DXYMd0AAmVpa2/7KhIrKCsuK/IBACsAAAkrZQAAATRm6g0EK28AcgDlAOsNYQBzorgDECsAAAAAEit5AG0A0WMAAWNuFislK2sAAAFhcxsrIStwAHAAcgBvAPgAFw5pAG0AAKA8InMA8AD9DQABYXMsKyEr8AAXDnIAbgA7gP4A/kDsATgrOyswG2QA5QBnAmUAcwCAgdcAO2JkAEMrRCtJK9dAYaCgInIAAKAxKgCgMCqAAWVwcwBRK1MraSvhAAkh4qKkIlsrXysAAAAAYytvAHQAAKA2I2kAcgAAoPEqb+A12GXdcgBrAACg2irhAHgociJpbWUAAKA0IIABYWlwAHYreSu3K2QA5QC+DYADYWRlbXBzdACFK6MrmiunK6wrsCuzK24iZ2xlAACitSVkbHFykCuUK5ornCvvIXduAKC/JeUhZnRloMMl8QACBwCgXCJpImdodABloLkl8QBdDG8AdAAAoOwlaSJudXMAAKA6KuwhdXMAoDkqYgAAoM0p6SFtZQCgOyrlInppdW0AoOIjgAFjaHQAwivKK80rAAFyecYrySsA4DXYydxGZGMAeQBbZPIhb2tnYQABaW/UK9creAD0ANERaCJlYWQAAAFsct4r5ytlAGYAdABhAHIAcgBvAPcAXQbpJGdodGFycm93AKCgIQAJQUhhYmNkZmdobG1vcHJzdHV3CiwNLBEsHSwnLDEsQCxLLFIsYix6LIQsjyzLLOgs7Sz/LAotcgDyAAkDYQByAACgYykAAWNyFSwbLHUAdABlADuA+gD6QPIACQ1yAOMBIywAACUseQBeZHYAZQBtYQABaXkrLDAscgBjADuA+wD7QENkgAFhYmgANyw6LD0scgDyANEO7CFhY3FhYQDyAOAOAAFpckQsSCzzIWh0AKB+KQDgNdgy3XIAYQB2AGUAO4D5APlAYQFWLF8scgAAAWxyWixcLACgvyEAoL4hbABrAACggCUAAWN0Zix2LG8CbCwAAAAAcyxyAG4AZaAcI3IAAKAcI28AcAAAoA8jcgBpAACg+CUAAWFsfiyBLGMAcgBrYTuAqACoQAABZ3CILIssbwBuAHNhZgAA4DXYZt0AA2FkaGxzdZksniynLLgsuyzFLHIAcgBvAPcACQ1vAHcAbgBhAHIAcgBvAPcA2A5hI3Jwb29uAAABbHKvLLMsZQBmAPQAWyxpAGcAaAD0AF0sdQDzAKYOaQAAocUDaGzBLMIs0mNvAG4AxWPwI2Fycm93cwCgyCGAAWNpdADRLOEs5CxvAtcsAAAAAN4scgBuAGWgHSNyAACgHSNvAHAAAKAOI24AZwBvYXIAaQAAoPklYwByAADgNdjK3IABZGlyAPMs9yz6LG8AdAAAoPAi7CFkZWlhaQBmoLUlAKC0JQABYW0DLQYtcgDyAMosbAA7gPwA/EDhIm5nbGUAoKcpgAdBQkRhY2RlZmxub3Byc3oAJy0qLTAtNC2bLZ0toS2/LcMtxy3TLdgt3C3gLfwtcgDyABADYQByAHag6CoAoOkqYQBzAOgA/gIAAW5yOC08LechcnQAoJwpgANla25wcnN0AJkpSC1NLVQtXi1iLYItYQBwAHAA4QAaHG8AdABoAGkAbgDnAKEXgAFoaXIAoSmzJFotbwBwAPQAdCVooJUh7wD4JgABaXVmLWotZwBtAOEAuygAAWJwbi14LXMjZXRuZXEAceCKIgD+AODLKgD+cyNldG5lcQBx4IsiAP4A4MwqAP4AAWhyhi2KLWUAdADhABIraSNhbmdsZQAAAWxyki2WLeUhZnQAoLIiaSJnaHQAAKCzInkAMmThIXNoAKCiIoABZWxyAKcttC24LWKiKCKuLQAAAACyLWEAcgAAoLsicQAAoFoi7CFpcACg7iIAAWJ0vC1eD2EA8gBfD3IAAOA12DPddAByAOkAlS1zAHUAAAFicM0t0C0A4IIi0iAA4IMi0iBwAGYAAOA12GfdcgBvAPAAWQt0AHIA6QCaLQABY3XkLegtcgAA4DXYy9wAAWJw7C30LW4AAAFFZXUt8S0A4IoiAP5uAAABRWV/LfktAOCLIgD+6SJnemFnAKCaKYADY2Vmb3BycwANLhAuJS4pLiMuLi40LukhcmN1YQABZGkULiEuAAFiZxguHC5hAHIAAKBfKmUAcaAnIgCgWSLlIXJwAKAYIXIAAOA12DTdcABmAADgNdho3WWgQCJhAHQA6ABqD2MAcgAA4DXYzNzjCuQRUC4AAFQuAABYLmIuAAAAAGMubS5wLnQuAAAAAIguki4AAJouJxIqEnQAcgDpAB0ScgAA4DXYNd0AAUFhWy5eLnIA8gDnAnIA8gCTB75jAAFBYWYuaS5yAPIA4AJyAPIAjAdhAPAAeh5pAHMAAKD7IoABZHB0APgReS6DLgABZmx9LoAuAOA12GnddQDzAP8RaQBtAOUABBIAAUFhiy6OLnIA8gDuAnIA8gCaBwABY3GVLgoScgAA4DXYzdwAAXB0nS6hLmwAdQDzACUScgDpACASAARhY2VmaW9zdbEuvC7ELsguzC7PLtQu2S5jAAABdXm2LrsudABlADuA/QD9QE9kAAFpecAuwy5yAGMAd2FLZG4AO4ClAKVAcgAA4DXYNt1jAHkAV2RwAGYAAOA12GrdYwByAADgNdjO3AABY23dLt8ueQBOZGwAO4D/AP9AAAVhY2RlZmhpb3N38y73Lv8uAi8MLxAvEy8YLx0vIi9jInV0ZQB6YQABYXn7Lv4u8iFvbn5hN2RvAHQAfGEAAWV0Bi8KL3QAcgDmAB8QYQC2Y3IAAOA12DfdYwB5ADZk5yJyYXJyAKDdIXAAZgAA4DXYa91jAHIAAOA12M/cAAFqbiYvKC8AoA0gagAAoAwg"), pl = ci("AAJhZ2xxBwARABMAFQBtAg0AAAAAAA8AcAAmYG8AcwAnYHQAPmB0ADxg9SFvdCJg");
	var ne;
	(function(e) {
		e[e.VALUE_LENGTH = 49152] = "VALUE_LENGTH", e[e.FLAG13 = 8192] = "FLAG13", e[e.BRANCH_LENGTH = 8064] = "BRANCH_LENGTH", e[e.JUMP_TABLE = 127] = "JUMP_TABLE";
	})(ne || (ne = {}));
	var K;
	(function(e) {
		e[e.NUM = 35] = "NUM", e[e.SEMI = 59] = "SEMI", e[e.EQUALS = 61] = "EQUALS", e[e.ZERO = 48] = "ZERO", e[e.NINE = 57] = "NINE", e[e.LOWER_A = 97] = "LOWER_A", e[e.LOWER_F = 102] = "LOWER_F", e[e.LOWER_X = 120] = "LOWER_X", e[e.LOWER_Z = 122] = "LOWER_Z", e[e.UPPER_A = 65] = "UPPER_A", e[e.UPPER_F = 70] = "UPPER_F", e[e.UPPER_Z = 90] = "UPPER_Z";
	})(K || (K = {}));
	const hi = 32;
	function Cr(e) {
		return e >= K.ZERO && e <= K.NINE;
	}
	function ml(e) {
		return e >= K.UPPER_A && e <= K.UPPER_F || e >= K.LOWER_A && e <= K.LOWER_F;
	}
	function Sl(e) {
		return e >= K.UPPER_A && e <= K.UPPER_Z || e >= K.LOWER_A && e <= K.LOWER_Z || Cr(e);
	}
	function bl(e) {
		return e === K.EQUALS || Sl(e);
	}
	var q;
	(function(e) {
		e[e.EntityStart = 0] = "EntityStart", e[e.NumericStart = 1] = "NumericStart", e[e.NumericDecimal = 2] = "NumericDecimal", e[e.NumericHex = 3] = "NumericHex", e[e.NamedEntity = 4] = "NamedEntity";
	})(q || (q = {}));
	var Be;
	(function(e) {
		e[e.Legacy = 0] = "Legacy", e[e.Strict = 1] = "Strict", e[e.Attribute = 2] = "Attribute";
	})(Be || (Be = {}));
	var yl = class {
		constructor(e, t, n) {
			this.decodeTree = e, this.emitCodePoint = t, this.errors = n, this.state = q.EntityStart, this.consumed = 1, this.result = 0, this.treeIndex = 0, this.excess = 1, this.decodeMode = Be.Strict, this.runConsumed = 0;
		}
		startEntity(e) {
			this.decodeMode = e, this.state = q.EntityStart, this.result = 0, this.treeIndex = 0, this.excess = 1, this.consumed = 1, this.runConsumed = 0;
		}
		write(e, t) {
			switch (this.state) {
				case q.EntityStart: return e.charCodeAt(t) === K.NUM ? (this.state = q.NumericStart, this.consumed += 1, this.stateNumericStart(e, t + 1)) : (this.state = q.NamedEntity, this.stateNamedEntity(e, t));
				case q.NumericStart: return this.stateNumericStart(e, t);
				case q.NumericDecimal: return this.stateNumericDecimal(e, t);
				case q.NumericHex: return this.stateNumericHex(e, t);
				case q.NamedEntity: return this.stateNamedEntity(e, t);
			}
		}
		stateNumericStart(e, t) {
			return t >= e.length ? -1 : (e.charCodeAt(t) | hi) === K.LOWER_X ? (this.state = q.NumericHex, this.consumed += 1, this.stateNumericHex(e, t + 1)) : (this.state = q.NumericDecimal, this.stateNumericDecimal(e, t));
		}
		stateNumericHex(e, t) {
			for (; t < e.length;) {
				const n = e.charCodeAt(t);
				if (Cr(n) || ml(n)) {
					const r = n <= K.NINE ? n - K.ZERO : (n | hi) - K.LOWER_A + 10;
					this.result = this.result * 16 + r, this.consumed++, t++;
				} else return this.emitNumericEntity(n, 3);
			}
			return -1;
		}
		stateNumericDecimal(e, t) {
			for (; t < e.length;) {
				const n = e.charCodeAt(t);
				if (Cr(n)) this.result = this.result * 10 + (n - K.ZERO), this.consumed++, t++;
				else return this.emitNumericEntity(n, 2);
			}
			return -1;
		}
		emitNumericEntity(e, t) {
			var n;
			if (this.consumed <= t) return (n = this.errors) === null || n === void 0 || n.absenceOfDigitsInNumericCharacterReference(this.consumed), 0;
			if (e === K.SEMI) this.consumed += 1;
			else if (this.decodeMode === Be.Strict) return 0;
			return this.emitCodePoint(gl(this.result), this.consumed), this.errors && (e !== K.SEMI && this.errors.missingSemicolonAfterCharacterReference(), this.errors.validateNumericCharacterReference(this.result)), this.consumed;
		}
		stateNamedEntity(e, t) {
			const { decodeTree: n } = this;
			let r = n[this.treeIndex], s = (r & ne.VALUE_LENGTH) >> 14;
			for (; t < e.length;) {
				if (s === 0 && (r & ne.FLAG13) !== 0) {
					const o = (r & ne.BRANCH_LENGTH) >> 7;
					if (this.runConsumed === 0) {
						const a = r & ne.JUMP_TABLE;
						if (e.charCodeAt(t) !== a) return this.result === 0 ? 0 : this.emitNotTerminatedNamedEntity();
						t++, this.excess++, this.runConsumed++;
					}
					for (; this.runConsumed < o;) {
						if (t >= e.length) return -1;
						const a = this.runConsumed - 1, u = n[this.treeIndex + 1 + (a >> 1)], l = a % 2 === 0 ? u & 255 : u >> 8 & 255;
						if (e.charCodeAt(t) !== l) return this.runConsumed = 0, this.result === 0 ? 0 : this.emitNotTerminatedNamedEntity();
						t++, this.excess++, this.runConsumed++;
					}
					this.runConsumed = 0, this.treeIndex += 1 + (o >> 1), r = n[this.treeIndex], s = (r & ne.VALUE_LENGTH) >> 14;
				}
				if (t >= e.length) break;
				const i = e.charCodeAt(t);
				if (i === K.SEMI && s !== 0 && (r & ne.FLAG13) !== 0) return this.emitNamedEntityData(this.treeIndex, s, this.consumed + this.excess);
				if (this.treeIndex = wl(n, r, this.treeIndex + Math.max(1, s), i), this.treeIndex < 0) return this.result === 0 || this.decodeMode === Be.Attribute && (s === 0 || bl(i)) ? 0 : this.emitNotTerminatedNamedEntity();
				if (r = n[this.treeIndex], s = (r & ne.VALUE_LENGTH) >> 14, s !== 0) {
					if (i === K.SEMI) return this.emitNamedEntityData(this.treeIndex, s, this.consumed + this.excess);
					this.decodeMode !== Be.Strict && (r & ne.FLAG13) === 0 && (this.result = this.treeIndex, this.consumed += this.excess, this.excess = 0);
				}
				t++, this.excess++;
			}
			return -1;
		}
		emitNotTerminatedNamedEntity() {
			var e;
			const { result: t, decodeTree: n } = this, r = (n[t] & ne.VALUE_LENGTH) >> 14;
			return this.emitNamedEntityData(t, r, this.consumed), (e = this.errors) === null || e === void 0 || e.missingSemicolonAfterCharacterReference(), this.consumed;
		}
		emitNamedEntityData(e, t, n) {
			const { decodeTree: r } = this;
			return this.emitCodePoint(t === 1 ? r[e] & ~(ne.VALUE_LENGTH | ne.FLAG13) : r[e + 1], n), t === 3 && this.emitCodePoint(r[e + 2], n), n;
		}
		end() {
			var e;
			switch (this.state) {
				case q.NamedEntity: return this.result !== 0 && (this.decodeMode !== Be.Attribute || this.result === this.treeIndex) ? this.emitNotTerminatedNamedEntity() : 0;
				case q.NumericDecimal: return this.emitNumericEntity(0, 2);
				case q.NumericHex: return this.emitNumericEntity(0, 3);
				case q.NumericStart: return (e = this.errors) === null || e === void 0 || e.absenceOfDigitsInNumericCharacterReference(this.consumed), 0;
				case q.EntityStart: return 0;
			}
		}
	};
	function wl(e, t, n, r) {
		const s = (t & ne.BRANCH_LENGTH) >> 7, i = t & ne.JUMP_TABLE;
		if (s === 0) return i !== 0 && r === i ? n : -1;
		if (i) {
			const l = r - i;
			return l < 0 || l >= s ? -1 : e[n + l] - 1;
		}
		const o = s + 1 >> 1;
		let a = 0, u = s - 1;
		for (; a <= u;) {
			const l = a + u >>> 1, p = e[n + (l >> 1)] >> (l & 1) * 8 & 255;
			if (p < r) a = l + 1;
			else if (p > r) u = l - 1;
			else return e[n + o + l];
		}
		return -1;
	}
	var E;
	(function(e) {
		e[e.Tab = 9] = "Tab", e[e.NewLine = 10] = "NewLine", e[e.FormFeed = 12] = "FormFeed", e[e.CarriageReturn = 13] = "CarriageReturn", e[e.Space = 32] = "Space", e[e.ExclamationMark = 33] = "ExclamationMark", e[e.Number = 35] = "Number", e[e.Amp = 38] = "Amp", e[e.SingleQuote = 39] = "SingleQuote", e[e.DoubleQuote = 34] = "DoubleQuote", e[e.Dash = 45] = "Dash", e[e.Slash = 47] = "Slash", e[e.Zero = 48] = "Zero", e[e.Nine = 57] = "Nine", e[e.Semi = 59] = "Semi", e[e.Lt = 60] = "Lt", e[e.Eq = 61] = "Eq", e[e.Gt = 62] = "Gt", e[e.Questionmark = 63] = "Questionmark", e[e.UpperA = 65] = "UpperA", e[e.LowerA = 97] = "LowerA", e[e.UpperF = 70] = "UpperF", e[e.LowerF = 102] = "LowerF", e[e.UpperZ = 90] = "UpperZ", e[e.LowerZ = 122] = "LowerZ", e[e.LowerX = 120] = "LowerX", e[e.OpeningSquareBracket = 91] = "OpeningSquareBracket";
	})(E || (E = {}));
	var m;
	(function(e) {
		e[e.Text = 1] = "Text", e[e.BeforeTagName = 2] = "BeforeTagName", e[e.InTagName = 3] = "InTagName", e[e.InSelfClosingTag = 4] = "InSelfClosingTag", e[e.BeforeClosingTagName = 5] = "BeforeClosingTagName", e[e.InClosingTagName = 6] = "InClosingTagName", e[e.AfterClosingTagName = 7] = "AfterClosingTagName", e[e.BeforeAttributeName = 8] = "BeforeAttributeName", e[e.InAttributeName = 9] = "InAttributeName", e[e.AfterAttributeName = 10] = "AfterAttributeName", e[e.BeforeAttributeValue = 11] = "BeforeAttributeValue", e[e.InAttributeValueDq = 12] = "InAttributeValueDq", e[e.InAttributeValueSq = 13] = "InAttributeValueSq", e[e.InAttributeValueNq = 14] = "InAttributeValueNq", e[e.BeforeDeclaration = 15] = "BeforeDeclaration", e[e.InDeclaration = 16] = "InDeclaration", e[e.InProcessingInstruction = 17] = "InProcessingInstruction", e[e.BeforeComment = 18] = "BeforeComment", e[e.CDATASequence = 19] = "CDATASequence", e[e.InSpecialComment = 20] = "InSpecialComment", e[e.InCommentLike = 21] = "InCommentLike", e[e.BeforeSpecialS = 22] = "BeforeSpecialS", e[e.BeforeSpecialT = 23] = "BeforeSpecialT", e[e.SpecialStartSequence = 24] = "SpecialStartSequence", e[e.InSpecialTag = 25] = "InSpecialTag", e[e.InEntity = 26] = "InEntity";
	})(m || (m = {}));
	function ke(e) {
		return e === E.Space || e === E.NewLine || e === E.Tab || e === E.FormFeed || e === E.CarriageReturn;
	}
	function cn(e) {
		return e === E.Slash || e === E.Gt || ke(e);
	}
	function Cl(e) {
		return e >= E.LowerA && e <= E.LowerZ || e >= E.UpperA && e <= E.UpperZ;
	}
	var we;
	(function(e) {
		e[e.NoValue = 0] = "NoValue", e[e.Unquoted = 1] = "Unquoted", e[e.Single = 2] = "Single", e[e.Double = 3] = "Double";
	})(we || (we = {}));
	const Y = {
		Cdata: new Uint8Array([
			67,
			68,
			65,
			84,
			65,
			91
		]),
		CdataEnd: new Uint8Array([
			93,
			93,
			62
		]),
		CommentEnd: new Uint8Array([
			45,
			45,
			62
		]),
		ScriptEnd: new Uint8Array([
			60,
			47,
			115,
			99,
			114,
			105,
			112,
			116
		]),
		StyleEnd: new Uint8Array([
			60,
			47,
			115,
			116,
			121,
			108,
			101
		]),
		TitleEnd: new Uint8Array([
			60,
			47,
			116,
			105,
			116,
			108,
			101
		]),
		TextareaEnd: new Uint8Array([
			60,
			47,
			116,
			101,
			120,
			116,
			97,
			114,
			101,
			97
		]),
		XmpEnd: new Uint8Array([
			60,
			47,
			120,
			109,
			112
		])
	};
	var di = class {
		constructor({ xmlMode: e = !1, decodeEntities: t = !0 }, n) {
			this.cbs = n, this.state = m.Text, this.buffer = "", this.sectionStart = 0, this.index = 0, this.entityStart = 0, this.baseState = m.Text, this.isSpecial = !1, this.running = !0, this.offset = 0, this.currentSequence = void 0, this.sequenceIndex = 0, this.xmlMode = e, this.decodeEntities = t, this.entityDecoder = new yl(e ? pl : fl, (r, s) => this.emitCodePoint(r, s));
		}
		reset() {
			this.state = m.Text, this.buffer = "", this.sectionStart = 0, this.index = 0, this.baseState = m.Text, this.currentSequence = void 0, this.running = !0, this.offset = 0;
		}
		write(e) {
			this.offset += this.buffer.length, this.buffer = e, this.parse();
		}
		end() {
			this.running && this.finish();
		}
		pause() {
			this.running = !1;
		}
		resume() {
			this.running = !0, this.index < this.buffer.length + this.offset && this.parse();
		}
		stateText(e) {
			e === E.Lt || !this.decodeEntities && this.fastForwardTo(E.Lt) ? (this.index > this.sectionStart && this.cbs.ontext(this.sectionStart, this.index), this.state = m.BeforeTagName, this.sectionStart = this.index) : this.decodeEntities && e === E.Amp && this.startEntity();
		}
		stateSpecialStartSequence(e) {
			const t = this.sequenceIndex === this.currentSequence.length;
			if (!(t ? cn(e) : (e | 32) === this.currentSequence[this.sequenceIndex])) this.isSpecial = !1;
			else if (!t) {
				this.sequenceIndex++;
				return;
			}
			this.sequenceIndex = 0, this.state = m.InTagName, this.stateInTagName(e);
		}
		stateInSpecialTag(e) {
			if (this.sequenceIndex === this.currentSequence.length) {
				if (e === E.Gt || ke(e)) {
					const t = this.index - this.currentSequence.length;
					if (this.sectionStart < t) {
						const n = this.index;
						this.index = t, this.cbs.ontext(this.sectionStart, t), this.index = n;
					}
					this.isSpecial = !1, this.sectionStart = t + 2, this.stateInClosingTagName(e);
					return;
				}
				this.sequenceIndex = 0;
			}
			(e | 32) === this.currentSequence[this.sequenceIndex] ? this.sequenceIndex += 1 : this.sequenceIndex === 0 ? this.currentSequence === Y.TitleEnd ? this.decodeEntities && e === E.Amp && this.startEntity() : this.fastForwardTo(E.Lt) && (this.sequenceIndex = 1) : this.sequenceIndex = +(e === E.Lt);
		}
		stateCDATASequence(e) {
			e === Y.Cdata[this.sequenceIndex] ? ++this.sequenceIndex === Y.Cdata.length && (this.state = m.InCommentLike, this.currentSequence = Y.CdataEnd, this.sequenceIndex = 0, this.sectionStart = this.index + 1) : (this.sequenceIndex = 0, this.state = m.InDeclaration, this.stateInDeclaration(e));
		}
		fastForwardTo(e) {
			for (; ++this.index < this.buffer.length + this.offset;) if (this.buffer.charCodeAt(this.index - this.offset) === e) return !0;
			return this.index = this.buffer.length + this.offset - 1, !1;
		}
		stateInCommentLike(e) {
			e === this.currentSequence[this.sequenceIndex] ? ++this.sequenceIndex === this.currentSequence.length && (this.currentSequence === Y.CdataEnd ? this.cbs.oncdata(this.sectionStart, this.index, 2) : this.cbs.oncomment(this.sectionStart, this.index, 2), this.sequenceIndex = 0, this.sectionStart = this.index + 1, this.state = m.Text) : this.sequenceIndex === 0 ? this.fastForwardTo(this.currentSequence[0]) && (this.sequenceIndex = 1) : e !== this.currentSequence[this.sequenceIndex - 1] && (this.sequenceIndex = 0);
		}
		isTagStartChar(e) {
			return this.xmlMode ? !cn(e) : Cl(e);
		}
		startSpecial(e, t) {
			this.isSpecial = !0, this.currentSequence = e, this.sequenceIndex = t, this.state = m.SpecialStartSequence;
		}
		stateBeforeTagName(e) {
			if (e === E.ExclamationMark) this.state = m.BeforeDeclaration, this.sectionStart = this.index + 1;
			else if (e === E.Questionmark) this.state = m.InProcessingInstruction, this.sectionStart = this.index + 1;
			else if (this.isTagStartChar(e)) {
				const t = e | 32;
				this.sectionStart = this.index, this.xmlMode ? this.state = m.InTagName : t === Y.ScriptEnd[2] ? this.state = m.BeforeSpecialS : t === Y.TitleEnd[2] || t === Y.XmpEnd[2] ? this.state = m.BeforeSpecialT : this.state = m.InTagName;
			} else e === E.Slash ? this.state = m.BeforeClosingTagName : (this.state = m.Text, this.stateText(e));
		}
		stateInTagName(e) {
			cn(e) && (this.cbs.onopentagname(this.sectionStart, this.index), this.sectionStart = -1, this.state = m.BeforeAttributeName, this.stateBeforeAttributeName(e));
		}
		stateBeforeClosingTagName(e) {
			ke(e) || (e === E.Gt ? this.state = m.Text : (this.state = this.isTagStartChar(e) ? m.InClosingTagName : m.InSpecialComment, this.sectionStart = this.index));
		}
		stateInClosingTagName(e) {
			(e === E.Gt || ke(e)) && (this.cbs.onclosetag(this.sectionStart, this.index), this.sectionStart = -1, this.state = m.AfterClosingTagName, this.stateAfterClosingTagName(e));
		}
		stateAfterClosingTagName(e) {
			(e === E.Gt || this.fastForwardTo(E.Gt)) && (this.state = m.Text, this.sectionStart = this.index + 1);
		}
		stateBeforeAttributeName(e) {
			e === E.Gt ? (this.cbs.onopentagend(this.index), this.isSpecial ? (this.state = m.InSpecialTag, this.sequenceIndex = 0) : this.state = m.Text, this.sectionStart = this.index + 1) : e === E.Slash ? this.state = m.InSelfClosingTag : ke(e) || (this.state = m.InAttributeName, this.sectionStart = this.index);
		}
		stateInSelfClosingTag(e) {
			e === E.Gt ? (this.cbs.onselfclosingtag(this.index), this.state = m.Text, this.sectionStart = this.index + 1, this.isSpecial = !1) : ke(e) || (this.state = m.BeforeAttributeName, this.stateBeforeAttributeName(e));
		}
		stateInAttributeName(e) {
			(e === E.Eq || cn(e)) && (this.cbs.onattribname(this.sectionStart, this.index), this.sectionStart = this.index, this.state = m.AfterAttributeName, this.stateAfterAttributeName(e));
		}
		stateAfterAttributeName(e) {
			e === E.Eq ? this.state = m.BeforeAttributeValue : e === E.Slash || e === E.Gt ? (this.cbs.onattribend(we.NoValue, this.sectionStart), this.sectionStart = -1, this.state = m.BeforeAttributeName, this.stateBeforeAttributeName(e)) : ke(e) || (this.cbs.onattribend(we.NoValue, this.sectionStart), this.state = m.InAttributeName, this.sectionStart = this.index);
		}
		stateBeforeAttributeValue(e) {
			e === E.DoubleQuote ? (this.state = m.InAttributeValueDq, this.sectionStart = this.index + 1) : e === E.SingleQuote ? (this.state = m.InAttributeValueSq, this.sectionStart = this.index + 1) : ke(e) || (this.sectionStart = this.index, this.state = m.InAttributeValueNq, this.stateInAttributeValueNoQuotes(e));
		}
		handleInAttributeValue(e, t) {
			e === t || !this.decodeEntities && this.fastForwardTo(t) ? (this.cbs.onattribdata(this.sectionStart, this.index), this.sectionStart = -1, this.cbs.onattribend(t === E.DoubleQuote ? we.Double : we.Single, this.index + 1), this.state = m.BeforeAttributeName) : this.decodeEntities && e === E.Amp && this.startEntity();
		}
		stateInAttributeValueDoubleQuotes(e) {
			this.handleInAttributeValue(e, E.DoubleQuote);
		}
		stateInAttributeValueSingleQuotes(e) {
			this.handleInAttributeValue(e, E.SingleQuote);
		}
		stateInAttributeValueNoQuotes(e) {
			ke(e) || e === E.Gt ? (this.cbs.onattribdata(this.sectionStart, this.index), this.sectionStart = -1, this.cbs.onattribend(we.Unquoted, this.index), this.state = m.BeforeAttributeName, this.stateBeforeAttributeName(e)) : this.decodeEntities && e === E.Amp && this.startEntity();
		}
		stateBeforeDeclaration(e) {
			e === E.OpeningSquareBracket ? (this.state = m.CDATASequence, this.sequenceIndex = 0) : this.state = e === E.Dash ? m.BeforeComment : m.InDeclaration;
		}
		stateInDeclaration(e) {
			(e === E.Gt || this.fastForwardTo(E.Gt)) && (this.cbs.ondeclaration(this.sectionStart, this.index), this.state = m.Text, this.sectionStart = this.index + 1);
		}
		stateInProcessingInstruction(e) {
			(e === E.Gt || this.fastForwardTo(E.Gt)) && (this.cbs.onprocessinginstruction(this.sectionStart, this.index), this.state = m.Text, this.sectionStart = this.index + 1);
		}
		stateBeforeComment(e) {
			e === E.Dash ? (this.state = m.InCommentLike, this.currentSequence = Y.CommentEnd, this.sequenceIndex = 2, this.sectionStart = this.index + 1) : this.state = m.InDeclaration;
		}
		stateInSpecialComment(e) {
			(e === E.Gt || this.fastForwardTo(E.Gt)) && (this.cbs.oncomment(this.sectionStart, this.index, 0), this.state = m.Text, this.sectionStart = this.index + 1);
		}
		stateBeforeSpecialS(e) {
			const t = e | 32;
			t === Y.ScriptEnd[3] ? this.startSpecial(Y.ScriptEnd, 4) : t === Y.StyleEnd[3] ? this.startSpecial(Y.StyleEnd, 4) : (this.state = m.InTagName, this.stateInTagName(e));
		}
		stateBeforeSpecialT(e) {
			switch (e | 32) {
				case Y.TitleEnd[3]:
					this.startSpecial(Y.TitleEnd, 4);
					break;
				case Y.TextareaEnd[3]:
					this.startSpecial(Y.TextareaEnd, 4);
					break;
				case Y.XmpEnd[3]:
					this.startSpecial(Y.XmpEnd, 4);
					break;
				default: this.state = m.InTagName, this.stateInTagName(e);
			}
		}
		startEntity() {
			this.baseState = this.state, this.state = m.InEntity, this.entityStart = this.index, this.entityDecoder.startEntity(this.xmlMode ? Be.Strict : this.baseState === m.Text || this.baseState === m.InSpecialTag ? Be.Legacy : Be.Attribute);
		}
		stateInEntity() {
			const e = this.index - this.offset, t = this.entityDecoder.write(this.buffer, e);
			if (t >= 0) this.state = this.baseState, t === 0 && (this.index -= 1);
			else {
				if (e < this.buffer.length && this.buffer.charCodeAt(e) === E.Amp) {
					this.state = this.baseState, this.index -= 1;
					return;
				}
				this.index = this.offset + this.buffer.length - 1;
			}
		}
		cleanup() {
			this.running && this.sectionStart !== this.index && (this.state === m.Text || this.state === m.InSpecialTag && this.sequenceIndex === 0 ? (this.cbs.ontext(this.sectionStart, this.index), this.sectionStart = this.index) : (this.state === m.InAttributeValueDq || this.state === m.InAttributeValueSq || this.state === m.InAttributeValueNq) && (this.cbs.onattribdata(this.sectionStart, this.index), this.sectionStart = this.index));
		}
		shouldContinue() {
			return this.index < this.buffer.length + this.offset && this.running;
		}
		parse() {
			for (; this.shouldContinue();) {
				const e = this.buffer.charCodeAt(this.index - this.offset);
				switch (this.state) {
					case m.Text:
						this.stateText(e);
						break;
					case m.SpecialStartSequence:
						this.stateSpecialStartSequence(e);
						break;
					case m.InSpecialTag:
						this.stateInSpecialTag(e);
						break;
					case m.CDATASequence:
						this.stateCDATASequence(e);
						break;
					case m.InAttributeValueDq:
						this.stateInAttributeValueDoubleQuotes(e);
						break;
					case m.InAttributeName:
						this.stateInAttributeName(e);
						break;
					case m.InCommentLike:
						this.stateInCommentLike(e);
						break;
					case m.InSpecialComment:
						this.stateInSpecialComment(e);
						break;
					case m.BeforeAttributeName:
						this.stateBeforeAttributeName(e);
						break;
					case m.InTagName:
						this.stateInTagName(e);
						break;
					case m.InClosingTagName:
						this.stateInClosingTagName(e);
						break;
					case m.BeforeTagName:
						this.stateBeforeTagName(e);
						break;
					case m.AfterAttributeName:
						this.stateAfterAttributeName(e);
						break;
					case m.InAttributeValueSq:
						this.stateInAttributeValueSingleQuotes(e);
						break;
					case m.BeforeAttributeValue:
						this.stateBeforeAttributeValue(e);
						break;
					case m.BeforeClosingTagName:
						this.stateBeforeClosingTagName(e);
						break;
					case m.AfterClosingTagName:
						this.stateAfterClosingTagName(e);
						break;
					case m.BeforeSpecialS:
						this.stateBeforeSpecialS(e);
						break;
					case m.BeforeSpecialT:
						this.stateBeforeSpecialT(e);
						break;
					case m.InAttributeValueNq:
						this.stateInAttributeValueNoQuotes(e);
						break;
					case m.InSelfClosingTag:
						this.stateInSelfClosingTag(e);
						break;
					case m.InDeclaration:
						this.stateInDeclaration(e);
						break;
					case m.BeforeDeclaration:
						this.stateBeforeDeclaration(e);
						break;
					case m.BeforeComment:
						this.stateBeforeComment(e);
						break;
					case m.InProcessingInstruction:
						this.stateInProcessingInstruction(e);
						break;
					case m.InEntity: this.stateInEntity();
				}
				this.index++;
			}
			this.cleanup();
		}
		finish() {
			this.state === m.InEntity && (this.entityDecoder.end(), this.state = this.baseState), this.handleTrailingData(), this.cbs.onend();
		}
		handleTrailingData() {
			const e = this.buffer.length + this.offset;
			this.sectionStart >= e || (this.state === m.InCommentLike ? this.currentSequence === Y.CdataEnd ? this.cbs.oncdata(this.sectionStart, e, 0) : this.cbs.oncomment(this.sectionStart, e, 0) : this.state === m.InTagName || this.state === m.BeforeAttributeName || this.state === m.BeforeAttributeValue || this.state === m.AfterAttributeName || this.state === m.InAttributeName || this.state === m.InAttributeValueSq || this.state === m.InAttributeValueDq || this.state === m.InAttributeValueNq || this.state === m.InClosingTagName || this.cbs.ontext(this.sectionStart, e));
		}
		emitCodePoint(e, t) {
			this.baseState !== m.Text && this.baseState !== m.InSpecialTag ? (this.sectionStart < this.entityStart && this.cbs.onattribdata(this.sectionStart, this.entityStart), this.sectionStart = this.entityStart + t, this.index = this.sectionStart - 1, this.cbs.onattribentity(e)) : (this.sectionStart < this.entityStart && this.cbs.ontext(this.sectionStart, this.entityStart), this.sectionStart = this.entityStart + t, this.index = this.sectionStart - 1, this.cbs.ontextentity(e, this.sectionStart));
		}
	};
	const mt = /* @__PURE__ */ new Set([
		"input",
		"option",
		"optgroup",
		"select",
		"button",
		"datalist",
		"textarea"
	]), k = /* @__PURE__ */ new Set(["p"]), gi = /* @__PURE__ */ new Set(["thead", "tbody"]), fi = /* @__PURE__ */ new Set(["dd", "dt"]), pi = /* @__PURE__ */ new Set(["rt", "rp"]), Il = /* @__PURE__ */ new Map([
		["tr", /* @__PURE__ */ new Set([
			"tr",
			"th",
			"td"
		])],
		["th", /* @__PURE__ */ new Set(["th"])],
		["td", /* @__PURE__ */ new Set([
			"thead",
			"th",
			"td"
		])],
		["body", /* @__PURE__ */ new Set([
			"head",
			"link",
			"script"
		])],
		["li", /* @__PURE__ */ new Set(["li"])],
		["p", k],
		["h1", k],
		["h2", k],
		["h3", k],
		["h4", k],
		["h5", k],
		["h6", k],
		["select", mt],
		["input", mt],
		["output", mt],
		["button", mt],
		["datalist", mt],
		["textarea", mt],
		["option", /* @__PURE__ */ new Set(["option"])],
		["optgroup", /* @__PURE__ */ new Set(["optgroup", "option"])],
		["dd", fi],
		["dt", fi],
		["address", k],
		["article", k],
		["aside", k],
		["blockquote", k],
		["details", k],
		["div", k],
		["dl", k],
		["fieldset", k],
		["figcaption", k],
		["figure", k],
		["footer", k],
		["form", k],
		["header", k],
		["hr", k],
		["main", k],
		["nav", k],
		["ol", k],
		["pre", k],
		["section", k],
		["table", k],
		["ul", k],
		["rt", pi],
		["rp", pi],
		["tbody", gi],
		["tfoot", gi]
	]), vl = /* @__PURE__ */ new Set([
		"area",
		"base",
		"basefont",
		"br",
		"col",
		"command",
		"embed",
		"frame",
		"hr",
		"img",
		"input",
		"isindex",
		"keygen",
		"link",
		"meta",
		"param",
		"source",
		"track",
		"wbr"
	]), mi = /* @__PURE__ */ new Set(["math", "svg"]), Si = /* @__PURE__ */ new Set([
		"mi",
		"mo",
		"mn",
		"ms",
		"mtext",
		"annotation-xml",
		"foreignobject",
		"desc",
		"title"
	]), El = /\s|\//;
	var hn = class {
		constructor(e, t = {}) {
			var n, r, s, i, o, a;
			this.options = t, this.startIndex = 0, this.endIndex = 0, this.openTagStart = 0, this.tagname = "", this.attribname = "", this.attribvalue = "", this.attribs = null, this.stack = [], this.buffers = [], this.bufferOffset = 0, this.writeIndex = 0, this.ended = !1, this.cbs = e ?? {}, this.htmlMode = !this.options.xmlMode, this.lowerCaseTagNames = (n = t.lowerCaseTags) !== null && n !== void 0 ? n : this.htmlMode, this.lowerCaseAttributeNames = (r = t.lowerCaseAttributeNames) !== null && r !== void 0 ? r : this.htmlMode, this.recognizeSelfClosing = (s = t.recognizeSelfClosing) !== null && s !== void 0 ? s : !this.htmlMode, this.tokenizer = new ((i = t.Tokenizer) !== null && i !== void 0 ? i : di)(this.options, this), this.foreignContext = [!this.htmlMode], (a = (o = this.cbs).onparserinit) === null || a === void 0 || a.call(o, this);
		}
		ontext(e, t) {
			var n, r;
			const s = this.getSlice(e, t);
			this.endIndex = t - 1, (r = (n = this.cbs).ontext) === null || r === void 0 || r.call(n, s), this.startIndex = t;
		}
		ontextentity(e, t) {
			var n, r;
			this.endIndex = t - 1, (r = (n = this.cbs).ontext) === null || r === void 0 || r.call(n, Ai(e)), this.startIndex = t;
		}
		isVoidElement(e) {
			return this.htmlMode && vl.has(e);
		}
		onopentagname(e, t) {
			this.endIndex = t;
			let n = this.getSlice(e, t);
			this.lowerCaseTagNames && (n = n.toLowerCase()), this.emitOpenTag(n);
		}
		emitOpenTag(e) {
			var t, n, r, s;
			this.openTagStart = this.startIndex, this.tagname = e;
			const i = this.htmlMode && Il.get(e);
			if (i) for (; this.stack.length > 0 && i.has(this.stack[0]);) {
				const o = this.stack.shift();
				(n = (t = this.cbs).onclosetag) === null || n === void 0 || n.call(t, o, !0);
			}
			this.isVoidElement(e) || (this.stack.unshift(e), this.htmlMode && (mi.has(e) ? this.foreignContext.unshift(!0) : Si.has(e) && this.foreignContext.unshift(!1))), (s = (r = this.cbs).onopentagname) === null || s === void 0 || s.call(r, e), this.cbs.onopentag && (this.attribs = {});
		}
		endOpenTag(e) {
			var t, n;
			this.startIndex = this.openTagStart, this.attribs && ((n = (t = this.cbs).onopentag) === null || n === void 0 || n.call(t, this.tagname, this.attribs, e), this.attribs = null), this.cbs.onclosetag && this.isVoidElement(this.tagname) && this.cbs.onclosetag(this.tagname, !0), this.tagname = "";
		}
		onopentagend(e) {
			this.endIndex = e, this.endOpenTag(!1), this.startIndex = e + 1;
		}
		onclosetag(e, t) {
			var n, r, s, i, o, a, u, l;
			this.endIndex = t;
			let p = this.getSlice(e, t);
			if (this.lowerCaseTagNames && (p = p.toLowerCase()), this.htmlMode && (mi.has(p) || Si.has(p)) && this.foreignContext.shift(), this.isVoidElement(p)) this.htmlMode && p === "br" && ((i = (s = this.cbs).onopentagname) === null || i === void 0 || i.call(s, "br"), (a = (o = this.cbs).onopentag) === null || a === void 0 || a.call(o, "br", {}, !0), (l = (u = this.cbs).onclosetag) === null || l === void 0 || l.call(u, "br", !1));
			else {
				const f = this.stack.indexOf(p);
				if (f !== -1) for (let g = 0; g <= f; g++) {
					const y = this.stack.shift();
					(r = (n = this.cbs).onclosetag) === null || r === void 0 || r.call(n, y, g !== f);
				}
				else this.htmlMode && p === "p" && (this.emitOpenTag("p"), this.closeCurrentTag(!0));
			}
			this.startIndex = t + 1;
		}
		onselfclosingtag(e) {
			this.endIndex = e, this.recognizeSelfClosing || this.foreignContext[0] ? (this.closeCurrentTag(!1), this.startIndex = e + 1) : this.onopentagend(e);
		}
		closeCurrentTag(e) {
			var t, n;
			const r = this.tagname;
			this.endOpenTag(e), this.stack[0] === r && ((n = (t = this.cbs).onclosetag) === null || n === void 0 || n.call(t, r, !e), this.stack.shift());
		}
		onattribname(e, t) {
			this.startIndex = e;
			const n = this.getSlice(e, t);
			this.attribname = this.lowerCaseAttributeNames ? n.toLowerCase() : n;
		}
		onattribdata(e, t) {
			this.attribvalue += this.getSlice(e, t);
		}
		onattribentity(e) {
			this.attribvalue += Ai(e);
		}
		onattribend(e, t) {
			var n, r;
			this.endIndex = t, (r = (n = this.cbs).onattribute) === null || r === void 0 || r.call(n, this.attribname, this.attribvalue, e === we.Double ? "\"" : e === we.Single ? "'" : e === we.NoValue ? void 0 : null), this.attribs && !Object.prototype.hasOwnProperty.call(this.attribs, this.attribname) && (this.attribs[this.attribname] = this.attribvalue), this.attribvalue = "";
		}
		getInstructionName(e) {
			const t = e.search(El);
			let n = t < 0 ? e : e.substr(0, t);
			return this.lowerCaseTagNames && (n = n.toLowerCase()), n;
		}
		ondeclaration(e, t) {
			this.endIndex = t;
			const n = this.getSlice(e, t);
			if (this.cbs.onprocessinginstruction) {
				const r = this.getInstructionName(n);
				this.cbs.onprocessinginstruction(`!${r}`, `!${n}`);
			}
			this.startIndex = t + 1;
		}
		onprocessinginstruction(e, t) {
			this.endIndex = t;
			const n = this.getSlice(e, t);
			if (this.cbs.onprocessinginstruction) {
				const r = this.getInstructionName(n);
				this.cbs.onprocessinginstruction(`?${r}`, `?${n}`);
			}
			this.startIndex = t + 1;
		}
		oncomment(e, t, n) {
			var r, s, i, o;
			this.endIndex = t, (s = (r = this.cbs).oncomment) === null || s === void 0 || s.call(r, this.getSlice(e, t - n)), (o = (i = this.cbs).oncommentend) === null || o === void 0 || o.call(i), this.startIndex = t + 1;
		}
		oncdata(e, t, n) {
			var r, s, i, o, a, u, l, p, f, g;
			this.endIndex = t;
			const y = this.getSlice(e, t - n);
			!this.htmlMode || this.options.recognizeCDATA ? ((s = (r = this.cbs).oncdatastart) === null || s === void 0 || s.call(r), (o = (i = this.cbs).ontext) === null || o === void 0 || o.call(i, y), (u = (a = this.cbs).oncdataend) === null || u === void 0 || u.call(a)) : ((p = (l = this.cbs).oncomment) === null || p === void 0 || p.call(l, `[CDATA[${y}]]`), (g = (f = this.cbs).oncommentend) === null || g === void 0 || g.call(f)), this.startIndex = t + 1;
		}
		onend() {
			var e, t;
			if (this.cbs.onclosetag) {
				this.endIndex = this.startIndex;
				for (let n = 0; n < this.stack.length; n++) this.cbs.onclosetag(this.stack[n], !0);
			}
			(t = (e = this.cbs).onend) === null || t === void 0 || t.call(e);
		}
		reset() {
			var e, t, n, r;
			(t = (e = this.cbs).onreset) === null || t === void 0 || t.call(e), this.tokenizer.reset(), this.tagname = "", this.attribname = "", this.attribs = null, this.stack.length = 0, this.startIndex = 0, this.endIndex = 0, (r = (n = this.cbs).onparserinit) === null || r === void 0 || r.call(n, this), this.buffers.length = 0, this.foreignContext.length = 0, this.foreignContext.unshift(!this.htmlMode), this.bufferOffset = 0, this.writeIndex = 0, this.ended = !1;
		}
		parseComplete(e) {
			this.reset(), this.end(e);
		}
		getSlice(e, t) {
			for (; e - this.bufferOffset >= this.buffers[0].length;) this.shiftBuffer();
			let n = this.buffers[0].slice(e - this.bufferOffset, t - this.bufferOffset);
			for (; t - this.bufferOffset > this.buffers[0].length;) this.shiftBuffer(), n += this.buffers[0].slice(0, t - this.bufferOffset);
			return n;
		}
		shiftBuffer() {
			this.bufferOffset += this.buffers[0].length, this.writeIndex--, this.buffers.shift();
		}
		write(e) {
			var t, n;
			if (this.ended) {
				(n = (t = this.cbs).onerror) === null || n === void 0 || n.call(t, /* @__PURE__ */ new Error(".write() after done!"));
				return;
			}
			this.buffers.push(e), this.tokenizer.running && (this.tokenizer.write(e), this.writeIndex++);
		}
		end(e) {
			var t, n;
			if (this.ended) {
				(n = (t = this.cbs).onerror) === null || n === void 0 || n.call(t, /* @__PURE__ */ new Error(".end() after done!"));
				return;
			}
			e && this.write(e), this.ended = !0, this.tokenizer.end();
		}
		pause() {
			this.tokenizer.pause();
		}
		resume() {
			for (this.tokenizer.resume(); this.tokenizer.running && this.writeIndex < this.buffers.length;) this.tokenizer.write(this.buffers[this.writeIndex++]);
			this.ended && this.tokenizer.end();
		}
		parseChunk(e) {
			this.write(e);
		}
		done(e) {
			this.end(e);
		}
	}, xl = Me({
		CDATA: () => Ti,
		Comment: () => Ii,
		Directive: () => Ci,
		Doctype: () => Bi,
		ElementType: () => R,
		Root: () => yi,
		Script: () => vi,
		Style: () => Ei,
		Tag: () => xi,
		Text: () => wi,
		isTag: () => bi
	}), R;
	(function(e) {
		e.Root = "root", e.Text = "text", e.Directive = "directive", e.Comment = "comment", e.Script = "script", e.Style = "style", e.Tag = "tag", e.CDATA = "cdata", e.Doctype = "doctype";
	})(R || (R = {}));
	function bi(e) {
		return e.type === R.Tag || e.type === R.Script || e.type === R.Style;
	}
	const yi = R.Root, wi = R.Text, Ci = R.Directive, Ii = R.Comment, vi = R.Script, Ei = R.Style, xi = R.Tag, Ti = R.CDATA, Bi = R.Doctype;
	var Qi = class {
		constructor() {
			this.parent = null, this.prev = null, this.next = null, this.startIndex = null, this.endIndex = null;
		}
		get parentNode() {
			return this.parent;
		}
		set parentNode(e) {
			this.parent = e;
		}
		get previousSibling() {
			return this.prev;
		}
		set previousSibling(e) {
			this.prev = e;
		}
		get nextSibling() {
			return this.next;
		}
		set nextSibling(e) {
			this.next = e;
		}
		cloneNode(e = !1) {
			return ki(this, e);
		}
	}, Ir = class extends Qi {
		constructor(e) {
			super(), this.data = e;
		}
		get nodeValue() {
			return this.data;
		}
		set nodeValue(e) {
			this.data = e;
		}
	}, vr = class extends Ir {
		constructor() {
			super(...arguments), this.type = R.Text;
		}
		get nodeType() {
			return 3;
		}
	}, Ri = class extends Ir {
		constructor() {
			super(...arguments), this.type = R.Comment;
		}
		get nodeType() {
			return 8;
		}
	}, Ni = class extends Ir {
		constructor(e, t) {
			super(t), this.name = e, this.type = R.Directive;
		}
		get nodeType() {
			return 1;
		}
	}, Er = class extends Qi {
		constructor(e) {
			super(), this.children = e;
		}
		get firstChild() {
			var e;
			return (e = this.children[0]) !== null && e !== void 0 ? e : null;
		}
		get lastChild() {
			return this.children.length > 0 ? this.children[this.children.length - 1] : null;
		}
		get childNodes() {
			return this.children;
		}
		set childNodes(e) {
			this.children = e;
		}
	}, Di = class extends Er {
		constructor() {
			super(...arguments), this.type = R.CDATA;
		}
		get nodeType() {
			return 4;
		}
	}, xr = class extends Er {
		constructor() {
			super(...arguments), this.type = R.Root;
		}
		get nodeType() {
			return 9;
		}
	}, _i = class extends Er {
		constructor(e, t, n = [], r = e === "script" ? R.Script : e === "style" ? R.Style : R.Tag) {
			super(n), this.name = e, this.attribs = t, this.type = r;
		}
		get nodeType() {
			return 1;
		}
		get tagName() {
			return this.name;
		}
		set tagName(e) {
			this.name = e;
		}
		get attributes() {
			return Object.keys(this.attribs).map((e) => {
				var t, n;
				return {
					name: e,
					value: this.attribs[e],
					namespace: (t = this["x-attribsNamespace"]) === null || t === void 0 ? void 0 : t[e],
					prefix: (n = this["x-attribsPrefix"]) === null || n === void 0 ? void 0 : n[e]
				};
			});
		}
	};
	function ue(e) {
		return bi(e);
	}
	function dn(e) {
		return e.type === R.CDATA;
	}
	function ze(e) {
		return e.type === R.Text;
	}
	function Tr(e) {
		return e.type === R.Comment;
	}
	function Tl(e) {
		return e.type === R.Directive;
	}
	function Fi(e) {
		return e.type === R.Root;
	}
	function Ce(e) {
		return Object.prototype.hasOwnProperty.call(e, "children");
	}
	function ki(e, t = !1) {
		let n;
		if (ze(e)) n = new vr(e.data);
		else if (Tr(e)) n = new Ri(e.data);
		else if (ue(e)) {
			const r = t ? Br(e.children) : [], s = new _i(e.name, { ...e.attribs }, r);
			r.forEach((i) => i.parent = s), e.namespace != null && (s.namespace = e.namespace), e["x-attribsNamespace"] && (s["x-attribsNamespace"] = { ...e["x-attribsNamespace"] }), e["x-attribsPrefix"] && (s["x-attribsPrefix"] = { ...e["x-attribsPrefix"] }), n = s;
		} else if (dn(e)) {
			const r = t ? Br(e.children) : [], s = new Di(r);
			r.forEach((i) => i.parent = s), n = s;
		} else if (Fi(e)) {
			const r = t ? Br(e.children) : [], s = new xr(r);
			r.forEach((i) => i.parent = s), e["x-mode"] && (s["x-mode"] = e["x-mode"]), n = s;
		} else if (Tl(e)) {
			const r = new Ni(e.name, e.data);
			e["x-name"] != null && (r["x-name"] = e["x-name"], r["x-publicId"] = e["x-publicId"], r["x-systemId"] = e["x-systemId"]), n = r;
		} else throw new Error(`Not implemented yet: ${e.type}`);
		return n.startIndex = e.startIndex, n.endIndex = e.endIndex, e.sourceCodeLocation != null && (n.sourceCodeLocation = e.sourceCodeLocation), n;
	}
	function Br(e) {
		const t = e.map((n) => ki(n, !0));
		for (let n = 1; n < t.length; n++) t[n].prev = t[n - 1], t[n - 1].next = t[n];
		return t;
	}
	const Mi = {
		withStartIndices: !1,
		withEndIndices: !1,
		xmlMode: !1
	};
	var Nt = class {
		constructor(e, t, n) {
			this.dom = [], this.root = new xr(this.dom), this.done = !1, this.tagStack = [this.root], this.lastNode = null, this.parser = null, typeof t == "function" && (n = t, t = Mi), typeof e == "object" && (t = e, e = void 0), this.callback = e ?? null, this.options = t ?? Mi, this.elementCB = n ?? null;
		}
		onparserinit(e) {
			this.parser = e;
		}
		onreset() {
			this.dom = [], this.root = new xr(this.dom), this.done = !1, this.tagStack = [this.root], this.lastNode = null, this.parser = null;
		}
		onend() {
			this.done || (this.done = !0, this.parser = null, this.handleCallback(null));
		}
		onerror(e) {
			this.handleCallback(e);
		}
		onclosetag() {
			this.lastNode = null;
			const e = this.tagStack.pop();
			this.options.withEndIndices && (e.endIndex = this.parser.endIndex), this.elementCB && this.elementCB(e);
		}
		onopentag(e, t) {
			const r = new _i(e, t, void 0, this.options.xmlMode ? R.Tag : void 0);
			this.addNode(r), this.tagStack.push(r);
		}
		ontext(e) {
			const { lastNode: t } = this;
			if (t && t.type === R.Text) t.data += e, this.options.withEndIndices && (t.endIndex = this.parser.endIndex);
			else {
				const n = new vr(e);
				this.addNode(n), this.lastNode = n;
			}
		}
		oncomment(e) {
			if (this.lastNode && this.lastNode.type === R.Comment) {
				this.lastNode.data += e;
				return;
			}
			const t = new Ri(e);
			this.addNode(t), this.lastNode = t;
		}
		oncommentend() {
			this.lastNode = null;
		}
		oncdatastart() {
			const e = new vr(""), t = new Di([e]);
			this.addNode(t), e.parent = t, this.lastNode = e;
		}
		oncdataend() {
			this.lastNode = null;
		}
		onprocessinginstruction(e, t) {
			const n = new Ni(e, t);
			this.addNode(n);
		}
		handleCallback(e) {
			if (typeof this.callback == "function") this.callback(e, this.dom);
			else if (e) throw e;
		}
		addNode(e) {
			const t = this.tagStack[this.tagStack.length - 1], n = t.children[t.children.length - 1];
			this.options.withStartIndices && (e.startIndex = this.parser.startIndex), this.options.withEndIndices && (e.endIndex = this.parser.endIndex), t.children.push(e), n && (e.prev = n, n.next = e), e.parent = t, this.lastNode = null;
		}
	};
	const Gi = /["&'<>$\x80-\uFFFF]/g, Hi = /* @__PURE__ */ new Map([
		[34, "&quot;"],
		[38, "&amp;"],
		[39, "&apos;"],
		[60, "&lt;"],
		[62, "&gt;"]
	]), Bl = String.prototype.codePointAt != null ? (e, t) => e.codePointAt(t) : (e, t) => (e.charCodeAt(t) & 64512) === 55296 ? (e.charCodeAt(t) - 55296) * 1024 + e.charCodeAt(t + 1) - 56320 + 65536 : e.charCodeAt(t);
	function Li(e) {
		let t = "", n = 0, r;
		for (; (r = Gi.exec(e)) !== null;) {
			const s = r.index, i = e.charCodeAt(s), o = Hi.get(i);
			o !== void 0 ? (t += e.substring(n, s) + o, n = s + 1) : (t += `${e.substring(n, s)}&#x${Bl(e, s).toString(16)};`, n = Gi.lastIndex += +((i & 64512) === 55296));
		}
		return t + e.substr(n);
	}
	function Qr(e, t) {
		return function(r) {
			let s, i = 0, o = "";
			for (; s = e.exec(r);) i !== s.index && (o += r.substring(i, s.index)), o += t.get(s[0].charCodeAt(0)), i = s.index + 1;
			return o + r.substring(i);
		};
	}
	const Ql = Qr(/["&\u00A0]/g, /* @__PURE__ */ new Map([
		[34, "&quot;"],
		[38, "&amp;"],
		[160, "&nbsp;"]
	])), Rl = Qr(/[&<>\u00A0]/g, /* @__PURE__ */ new Map([
		[38, "&amp;"],
		[60, "&lt;"],
		[62, "&gt;"],
		[160, "&nbsp;"]
	])), Nl = new Map([
		"altGlyph",
		"altGlyphDef",
		"altGlyphItem",
		"animateColor",
		"animateMotion",
		"animateTransform",
		"clipPath",
		"feBlend",
		"feColorMatrix",
		"feComponentTransfer",
		"feComposite",
		"feConvolveMatrix",
		"feDiffuseLighting",
		"feDisplacementMap",
		"feDistantLight",
		"feDropShadow",
		"feFlood",
		"feFuncA",
		"feFuncB",
		"feFuncG",
		"feFuncR",
		"feGaussianBlur",
		"feImage",
		"feMerge",
		"feMergeNode",
		"feMorphology",
		"feOffset",
		"fePointLight",
		"feSpecularLighting",
		"feSpotLight",
		"feTile",
		"feTurbulence",
		"foreignObject",
		"glyphRef",
		"linearGradient",
		"radialGradient",
		"textPath"
	].map((e) => [e.toLowerCase(), e])), Dl = new Map([
		"definitionURL",
		"attributeName",
		"attributeType",
		"baseFrequency",
		"baseProfile",
		"calcMode",
		"clipPathUnits",
		"diffuseConstant",
		"edgeMode",
		"filterUnits",
		"glyphRef",
		"gradientTransform",
		"gradientUnits",
		"kernelMatrix",
		"kernelUnitLength",
		"keyPoints",
		"keySplines",
		"keyTimes",
		"lengthAdjust",
		"limitingConeAngle",
		"markerHeight",
		"markerUnits",
		"markerWidth",
		"maskContentUnits",
		"maskUnits",
		"numOctaves",
		"pathLength",
		"patternContentUnits",
		"patternTransform",
		"patternUnits",
		"pointsAtX",
		"pointsAtY",
		"pointsAtZ",
		"preserveAlpha",
		"preserveAspectRatio",
		"primitiveUnits",
		"refX",
		"refY",
		"repeatCount",
		"repeatDur",
		"requiredExtensions",
		"requiredFeatures",
		"specularConstant",
		"specularExponent",
		"spreadMethod",
		"startOffset",
		"stdDeviation",
		"stitchTiles",
		"surfaceScale",
		"systemLanguage",
		"tableValues",
		"targetX",
		"targetY",
		"textLength",
		"viewBox",
		"viewTarget",
		"xChannelSelector",
		"yChannelSelector",
		"zoomAndPan"
	].map((e) => [e.toLowerCase(), e])), _l = /* @__PURE__ */ new Set([
		"style",
		"script",
		"xmp",
		"iframe",
		"noembed",
		"noframes",
		"plaintext",
		"noscript"
	]);
	function Fl(e) {
		return e.replace(/"/g, "&quot;");
	}
	function kl(e, t) {
		var n;
		if (!e) return;
		const r = ((n = t.encodeEntities) !== null && n !== void 0 ? n : t.decodeEntities) === !1 ? Fl : t.xmlMode || t.encodeEntities !== "utf8" ? Li : Ql;
		return Object.keys(e).map((s) => {
			var i, o;
			const a = (i = e[s]) !== null && i !== void 0 ? i : "";
			return t.xmlMode === "foreign" && (s = (o = Dl.get(s)) !== null && o !== void 0 ? o : s), !t.emptyAttrs && !t.xmlMode && a === "" ? s : `${s}="${r(a)}"`;
		}).join(" ");
	}
	const Oi = /* @__PURE__ */ new Set([
		"area",
		"base",
		"basefont",
		"br",
		"col",
		"command",
		"embed",
		"frame",
		"hr",
		"img",
		"input",
		"isindex",
		"keygen",
		"link",
		"meta",
		"param",
		"source",
		"track",
		"wbr"
	]);
	function Rr(e, t = {}) {
		const n = "length" in e ? e : [e];
		let r = "";
		for (let s = 0; s < n.length; s++) r += Ml(n[s], t);
		return r;
	}
	function Ml(e, t) {
		switch (e.type) {
			case yi: return Rr(e.children, t);
			case Bi:
			case Ci: return Ol(e);
			case Ii: return Pl(e);
			case Ti: return Jl(e);
			case vi:
			case Ei:
			case xi: return Ll(e, t);
			case wi: return Kl(e, t);
		}
	}
	const Gl = /* @__PURE__ */ new Set([
		"mi",
		"mo",
		"mn",
		"ms",
		"mtext",
		"annotation-xml",
		"foreignObject",
		"desc",
		"title"
	]), Hl = /* @__PURE__ */ new Set(["svg", "math"]);
	function Ll(e, t) {
		var n;
		t.xmlMode === "foreign" && (e.name = (n = Nl.get(e.name)) !== null && n !== void 0 ? n : e.name, e.parent && Gl.has(e.parent.name) && (t = {
			...t,
			xmlMode: !1
		})), !t.xmlMode && Hl.has(e.name) && (t = {
			...t,
			xmlMode: "foreign"
		});
		let r = `<${e.name}`;
		const s = kl(e.attribs, t);
		return s && (r += ` ${s}`), e.children.length === 0 && (t.xmlMode ? t.selfClosingTags !== !1 : t.selfClosingTags && Oi.has(e.name)) ? (t.xmlMode || (r += " "), r += "/>") : (r += ">", e.children.length > 0 && (r += Rr(e.children, t)), (t.xmlMode || !Oi.has(e.name)) && (r += `</${e.name}>`)), r;
	}
	function Ol(e) {
		return `<${e.data}>`;
	}
	function Kl(e, t) {
		var n;
		let r = e.data || "";
		return ((n = t.encodeEntities) !== null && n !== void 0 ? n : t.decodeEntities) !== !1 && !(!t.xmlMode && e.parent && _l.has(e.parent.name)) && (r = t.xmlMode || t.encodeEntities !== "utf8" ? Li(r) : Rl(r)), r;
	}
	function Jl(e) {
		return `<![CDATA[${e.children[0].data}]]>`;
	}
	function Pl(e) {
		return `<!--${e.data}-->`;
	}
	function Ki(e, t) {
		return Rr(e, t);
	}
	function Wl(e, t) {
		return Ce(e) ? e.children.map((n) => Ki(n, t)).join("") : "";
	}
	function gn(e) {
		return Array.isArray(e) ? e.map(gn).join("") : ue(e) ? e.name === "br" ? `
` : gn(e.children) : dn(e) ? gn(e.children) : ze(e) ? e.data : "";
	}
	function fn(e) {
		return Array.isArray(e) ? e.map(fn).join("") : Ce(e) && !Tr(e) ? fn(e.children) : ze(e) ? e.data : "";
	}
	function Nr(e) {
		return Array.isArray(e) ? e.map(Nr).join("") : Ce(e) && (e.type === R.Tag || dn(e)) ? Nr(e.children) : ze(e) ? e.data : "";
	}
	function Ji(e) {
		return Ce(e) ? e.children : [];
	}
	function Pi(e) {
		return e.parent || null;
	}
	function Yl(e) {
		const t = Pi(e);
		if (t != null) return Ji(t);
		const n = [e];
		let { prev: r, next: s } = e;
		for (; r != null;) n.unshift(r), {prev: r} = r;
		for (; s != null;) n.push(s), {next: s} = s;
		return n;
	}
	function Ul(e, t) {
		var n;
		return (n = e.attribs) === null || n === void 0 ? void 0 : n[t];
	}
	function Vl(e, t) {
		return e.attribs != null && Object.prototype.hasOwnProperty.call(e.attribs, t) && e.attribs[t] != null;
	}
	function Zl(e) {
		return e.name;
	}
	function jl(e) {
		let { next: t } = e;
		for (; t !== null && !ue(t);) ({next: t} = t);
		return t;
	}
	function Xl(e) {
		let { prev: t } = e;
		for (; t !== null && !ue(t);) ({prev: t} = t);
		return t;
	}
	function Dt(e) {
		if (e.prev && (e.prev.next = e.next), e.next && (e.next.prev = e.prev), e.parent) {
			const t = e.parent.children, n = t.lastIndexOf(e);
			n >= 0 && t.splice(n, 1);
		}
		e.next = null, e.prev = null, e.parent = null;
	}
	function ql(e, t) {
		const n = t.prev = e.prev;
		n && (n.next = t);
		const r = t.next = e.next;
		r && (r.prev = t);
		const s = t.parent = e.parent;
		if (s) {
			const i = s.children;
			i[i.lastIndexOf(e)] = t, e.parent = null;
		}
	}
	function $l(e, t) {
		if (Dt(t), t.next = null, t.parent = e, e.children.push(t) > 1) {
			const n = e.children[e.children.length - 2];
			n.next = t, t.prev = n;
		} else t.prev = null;
	}
	function zl(e, t) {
		Dt(t);
		const { parent: n } = e, r = e.next;
		if (t.next = r, t.prev = e, e.next = t, t.parent = n, r) {
			if (r.prev = t, n) {
				const s = n.children;
				s.splice(s.lastIndexOf(r), 0, t);
			}
		} else n && n.children.push(t);
	}
	function eA(e, t) {
		if (Dt(t), t.parent = e, t.prev = null, e.children.unshift(t) !== 1) {
			const n = e.children[1];
			n.prev = t, t.next = n;
		} else t.next = null;
	}
	function tA(e, t) {
		Dt(t);
		const { parent: n } = e;
		if (n) {
			const r = n.children;
			r.splice(r.indexOf(e), 0, t);
		}
		e.prev && (e.prev.next = t), t.parent = n, t.prev = e.prev, t.next = e, e.prev = t;
	}
	function _t(e, t, n = !0, r = 1 / 0) {
		return Wi(e, Array.isArray(t) ? t : [t], n, r);
	}
	function Wi(e, t, n, r) {
		const s = [], i = [Array.isArray(t) ? t : [t]], o = [0];
		for (;;) {
			if (o[0] >= i[0].length) {
				if (o.length === 1) return s;
				i.shift(), o.shift();
				continue;
			}
			const a = i[0][o[0]++];
			if (e(a) && (s.push(a), --r <= 0)) return s;
			n && Ce(a) && a.children.length > 0 && (o.unshift(0), i.unshift(a.children));
		}
	}
	function nA(e, t) {
		return t.find(e);
	}
	function Dr(e, t, n = !0) {
		const r = Array.isArray(t) ? t : [t];
		for (let s = 0; s < r.length; s++) {
			const i = r[s];
			if (ue(i) && e(i)) return i;
			if (n && Ce(i) && i.children.length > 0) {
				const o = Dr(e, i.children, !0);
				if (o) return o;
			}
		}
		return null;
	}
	function Yi(e, t) {
		return (Array.isArray(t) ? t : [t]).some((n) => ue(n) && e(n) || Ce(n) && Yi(e, n.children));
	}
	function rA(e, t) {
		const n = [], r = [Array.isArray(t) ? t : [t]], s = [0];
		for (;;) {
			if (s[0] >= r[0].length) {
				if (r.length === 1) return n;
				r.shift(), s.shift();
				continue;
			}
			const i = r[0][s[0]++];
			ue(i) && e(i) && n.push(i), Ce(i) && i.children.length > 0 && (s.unshift(0), r.unshift(i.children));
		}
	}
	const pn = {
		tag_name(e) {
			return typeof e == "function" ? (t) => ue(t) && e(t.name) : e === "*" ? ue : (t) => ue(t) && t.name === e;
		},
		tag_type(e) {
			return typeof e == "function" ? (t) => e(t.type) : (t) => t.type === e;
		},
		tag_contains(e) {
			return typeof e == "function" ? (t) => ze(t) && e(t.data) : (t) => ze(t) && t.data === e;
		}
	};
	function _r(e, t) {
		return typeof t == "function" ? (n) => ue(n) && t(n.attribs[e]) : (n) => ue(n) && n.attribs[e] === t;
	}
	function sA(e, t) {
		return (n) => e(n) || t(n);
	}
	function Ui(e) {
		const t = Object.keys(e).map((n) => {
			const r = e[n];
			return Object.prototype.hasOwnProperty.call(pn, n) ? pn[n](r) : _r(n, r);
		});
		return t.length === 0 ? null : t.reduce(sA);
	}
	function iA(e, t) {
		const n = Ui(e);
		return n ? n(t) : !0;
	}
	function oA(e, t, n, r = 1 / 0) {
		const s = Ui(e);
		return s ? _t(s, t, n, r) : [];
	}
	function aA(e, t, n = !0) {
		return Array.isArray(t) || (t = [t]), Dr(_r("id", e), t, n);
	}
	function St(e, t, n = !0, r = 1 / 0) {
		return _t(pn.tag_name(e), t, n, r);
	}
	function uA(e, t, n = !0, r = 1 / 0) {
		return _t(_r("class", e), t, n, r);
	}
	function lA(e, t, n = !0, r = 1 / 0) {
		return _t(pn.tag_type(e), t, n, r);
	}
	function AA(e) {
		let t = e.length;
		for (; --t >= 0;) {
			const n = e[t];
			if (t > 0 && e.lastIndexOf(n, t - 1) >= 0) {
				e.splice(t, 1);
				continue;
			}
			for (let r = n.parent; r; r = r.parent) if (e.includes(r)) {
				e.splice(t, 1);
				break;
			}
		}
		return e;
	}
	var he;
	(function(e) {
		e[e.DISCONNECTED = 1] = "DISCONNECTED", e[e.PRECEDING = 2] = "PRECEDING", e[e.FOLLOWING = 4] = "FOLLOWING", e[e.CONTAINS = 8] = "CONTAINS", e[e.CONTAINED_BY = 16] = "CONTAINED_BY";
	})(he || (he = {}));
	function Vi(e, t) {
		const n = [], r = [];
		if (e === t) return 0;
		let s = Ce(e) ? e : e.parent;
		for (; s;) n.unshift(s), s = s.parent;
		for (s = Ce(t) ? t : t.parent; s;) r.unshift(s), s = s.parent;
		const i = Math.min(n.length, r.length);
		let o = 0;
		for (; o < i && n[o] === r[o];) o++;
		if (o === 0) return he.DISCONNECTED;
		const a = n[o - 1], u = a.children, l = n[o], p = r[o];
		return u.indexOf(l) > u.indexOf(p) ? a === t ? he.FOLLOWING | he.CONTAINED_BY : he.FOLLOWING : a === e ? he.PRECEDING | he.CONTAINS : he.PRECEDING;
	}
	function cA(e) {
		return e = e.filter((t, n, r) => !r.includes(t, n + 1)), e.sort((t, n) => {
			const r = Vi(t, n);
			return r & he.PRECEDING ? -1 : r & he.FOLLOWING ? 1 : 0;
		}), e;
	}
	function Fr(e) {
		const t = mn(pA, e);
		return t ? t.name === "feed" ? hA(t) : dA(t) : null;
	}
	function hA(e) {
		var t;
		const n = e.children, r = {
			type: "atom",
			items: St("entry", n).map((o) => {
				var a;
				const { children: u } = o, l = { media: Zi(u) };
				ie(l, "id", "id", u), ie(l, "title", "title", u);
				const p = (a = mn("link", u)) === null || a === void 0 ? void 0 : a.attribs.href;
				p && (l.link = p);
				const f = Le("summary", u) || Le("content", u);
				f && (l.description = f);
				const g = Le("updated", u);
				return g && (l.pubDate = new Date(g)), l;
			})
		};
		ie(r, "id", "id", n), ie(r, "title", "title", n);
		const s = (t = mn("link", n)) === null || t === void 0 ? void 0 : t.attribs.href;
		s && (r.link = s), ie(r, "description", "subtitle", n);
		const i = Le("updated", n);
		return i && (r.updated = new Date(i)), ie(r, "author", "email", n, !0), r;
	}
	function dA(e) {
		var t, n;
		const r = (n = (t = mn("channel", e.children)) === null || t === void 0 ? void 0 : t.children) !== null && n !== void 0 ? n : [], s = {
			type: e.name.substr(0, 3),
			id: "",
			items: St("item", e.children).map((o) => {
				const { children: a } = o, u = { media: Zi(a) };
				ie(u, "id", "guid", a), ie(u, "title", "title", a), ie(u, "link", "link", a), ie(u, "description", "description", a);
				const l = Le("pubDate", a) || Le("dc:date", a);
				return l && (u.pubDate = new Date(l)), u;
			})
		};
		ie(s, "title", "title", r), ie(s, "link", "link", r), ie(s, "description", "description", r);
		const i = Le("lastBuildDate", r);
		return i && (s.updated = new Date(i)), ie(s, "author", "managingEditor", r, !0), s;
	}
	const gA = [
		"url",
		"type",
		"lang"
	], fA = [
		"fileSize",
		"bitrate",
		"framerate",
		"samplingrate",
		"channels",
		"duration",
		"height",
		"width"
	];
	function Zi(e) {
		return St("media:content", e).map((t) => {
			const { attribs: n } = t, r = {
				medium: n.medium,
				isDefault: !!n.isDefault
			};
			for (const s of gA) n[s] && (r[s] = n[s]);
			for (const s of fA) n[s] && (r[s] = parseInt(n[s], 10));
			return n.expression && (r.expression = n.expression), r;
		});
	}
	function mn(e, t) {
		return St(e, t, !0, 1)[0];
	}
	function Le(e, t, n = !1) {
		return fn(St(e, t, n, 1)).trim();
	}
	function ie(e, t, n, r, s = !1) {
		const i = Le(n, r, s);
		i && (e[t] = i);
	}
	function pA(e) {
		return e === "rss" || e === "feed" || e === "rdf:RDF";
	}
	var mA = Me({
		DocumentPosition: () => he,
		append: () => zl,
		appendChild: () => $l,
		compareDocumentPosition: () => Vi,
		existsOne: () => Yi,
		filter: () => _t,
		find: () => Wi,
		findAll: () => rA,
		findOne: () => Dr,
		findOneChild: () => nA,
		getAttributeValue: () => Ul,
		getChildren: () => Ji,
		getElementById: () => aA,
		getElements: () => oA,
		getElementsByClassName: () => uA,
		getElementsByTagName: () => St,
		getElementsByTagType: () => lA,
		getFeed: () => Fr,
		getInnerHTML: () => Wl,
		getName: () => Zl,
		getOuterHTML: () => Ki,
		getParent: () => Pi,
		getSiblings: () => Yl,
		getText: () => gn,
		hasAttrib: () => Vl,
		hasChildren: () => Ce,
		innerText: () => Nr,
		isCDATA: () => dn,
		isComment: () => Tr,
		isDocument: () => Fi,
		isTag: () => ue,
		isText: () => ze,
		nextElementSibling: () => jl,
		prepend: () => tA,
		prependChild: () => eA,
		prevElementSibling: () => Xl,
		removeElement: () => Dt,
		removeSubsets: () => AA,
		replaceElement: () => ql,
		testElement: () => iA,
		textContent: () => fn,
		uniqueSort: () => cA
	}), SA = Me({
		DefaultHandler: () => Nt,
		DomHandler: () => Nt,
		DomUtils: () => mA,
		ElementType: () => xl,
		Parser: () => hn,
		QuoteType: () => we,
		Tokenizer: () => di,
		createDocumentStream: () => bA,
		createDomStream: () => yA,
		getFeed: () => Fr,
		parseDOM: () => Xi,
		parseDocument: () => ji,
		parseFeed: () => CA
	});
	function ji(e, t) {
		const n = new Nt(void 0, t);
		return new hn(n, t).end(e), n.root;
	}
	function Xi(e, t) {
		return ji(e, t).children;
	}
	function bA(e, t, n) {
		const r = new Nt((s) => e(s, r.root), t, n);
		return new hn(r, t);
	}
	function yA(e, t, n) {
		return new hn(new Nt(e, t, n), t);
	}
	const wA = { xmlMode: !0 };
	function CA(e, t = wA) {
		return Fr(Xi(e, t));
	}
	const IA = /* @__PURE__ */ new Set([
		"ARTICLE",
		"ASIDE",
		"BLOCKQUOTE",
		"BODY",
		"BR",
		"BUTTON",
		"CANVAS",
		"CAPTION",
		"COL",
		"COLGROUP",
		"DD",
		"DIV",
		"DL",
		"DT",
		"EMBED",
		"FIELDSET",
		"FIGCAPTION",
		"FIGURE",
		"FOOTER",
		"FORM",
		"H1",
		"H2",
		"H3",
		"H4",
		"H5",
		"H6",
		"LI",
		"UL",
		"OL",
		"P"
	]), kr = "http://www.w3.org/2000/svg", { assign: vA, create: EA, defineProperties: xA, entries: TA, getOwnPropertyDescriptors: yg, keys: BA, setPrototypeOf: re } = Object, et = String, oe = (e) => e.nodeType === 1 ? e[v] : e, Oe = ({ ownerDocument: e }) => e[ft].ignoreCase, Ie = (e, t) => {
		e[w] = t, t[X] = e;
	}, qi = (e, t, n) => {
		Ie(e, t), Ie(oe(t), n);
	}, $i = (e, t, n, r) => {
		Ie(e, t), Ie(oe(n), r);
	}, Sn = (e, t, n) => {
		Ie(e, t), Ie(t, n);
	}, Mr = ({ localName: e, ownerDocument: t }) => t[ft].ignoreCase ? e.toUpperCase() : e, zi = (e, t) => {
		e && (e[w] = t), t && (t[X] = e);
	}, eo = (e, t) => {
		const n = e.createDocumentFragment(), r = e.createElement("");
		r.innerHTML = t;
		const { firstChild: s, lastChild: i } = r;
		if (s) {
			$i(n, s, i, n[v]);
			let o = s;
			do
				o.parentNode = n;
			while (o !== i && (o = oe(o)[w]));
		}
		return n;
	}, Ke = /* @__PURE__ */ new WeakMap();
	let bn = !1;
	const bt = /* @__PURE__ */ new WeakMap(), tt = /* @__PURE__ */ new WeakMap(), yn = (e, t, n, r) => {
		bn && tt.has(e) && e.attributeChangedCallback && e.constructor.observedAttributes.includes(t) && e.attributeChangedCallback(t, n, r);
	}, to = (e, t) => (n) => {
		if (tt.has(n)) {
			const r = tt.get(n);
			r.connected !== t && n.isConnected === t && (r.connected = t, e in n && n[e]());
		}
	}, no = to("connectedCallback", !0), Gr = (e) => {
		if (bn) {
			no(e), Ke.has(e) && (e = Ke.get(e).shadowRoot);
			let { [w]: t, [v]: n } = e;
			for (; t !== n;) t.nodeType === 1 && no(t), t = t[w];
		}
	}, ro = to("disconnectedCallback", !1), QA = (e) => {
		if (bn) {
			ro(e), Ke.has(e) && (e = Ke.get(e).shadowRoot);
			let { [w]: t, [v]: n } = e;
			for (; t !== n;) t.nodeType === 1 && ro(t), t = t[w];
		}
	};
	var RA = class {
		constructor(e) {
			this.ownerDocument = e, this.registry = /* @__PURE__ */ new Map(), this.waiting = /* @__PURE__ */ new Map(), this.active = !1;
		}
		define(e, t, n = {}) {
			const { ownerDocument: r, registry: s, waiting: i } = this;
			if (s.has(e)) throw new Error("unable to redefine " + e);
			if (bt.has(t)) throw new Error("unable to redefine the same class: " + t);
			this.active = bn = !0;
			const { extends: o } = n;
			bt.set(t, {
				ownerDocument: r,
				options: { is: o ? e : "" },
				localName: o || e
			});
			const a = o ? (u) => u.localName === o && u.getAttribute("is") === e : (u) => u.localName === e;
			if (s.set(e, {
				Class: t,
				check: a
			}), i.has(e)) {
				for (const u of i.get(e)) u(t);
				i.delete(e);
			}
			r.querySelectorAll(o ? `${o}[is="${e}"]` : e).forEach(this.upgrade, this);
		}
		upgrade(e) {
			if (tt.has(e)) return;
			const { ownerDocument: t, registry: n } = this, r = e.getAttribute("is") || e.localName;
			if (n.has(r)) {
				const { Class: s, check: i } = n.get(r);
				if (i(e)) {
					const { attributes: o, isConnected: a } = e;
					for (const l of o) e.removeAttributeNode(l);
					const u = TA(e);
					for (const [l] of u) delete e[l];
					re(e, s.prototype), t[Rt] = {
						element: e,
						values: u
					}, new s(t, r), tt.set(e, { connected: a });
					for (const l of o) e.setAttributeNode(l);
					a && e.connectedCallback && e.connectedCallback();
				}
			}
		}
		whenDefined(e) {
			const { registry: t, waiting: n } = this;
			return new Promise((r) => {
				t.has(e) ? r(t.get(e).Class) : (n.has(e) || n.set(e, []), n.get(e).push(r));
			});
		}
		get(e) {
			const t = this.registry.get(e);
			return t && t.Class;
		}
		getName(e) {
			if (bt.has(e)) {
				const { localName: t } = bt.get(e);
				return t;
			}
			return null;
		}
	};
	const { Parser: NA } = SA;
	const nt = (e, t, n) => {
		const r = e[v];
		return t.parentNode = e, qi(r[X], t, r), n && t.nodeType === 1 && Gr(t), t;
	}, DA = (e, t, n, r, s) => {
		n[M] = r, n.ownerElement = e, Sn(t[X], n, t), n.name === "class" && (e.className = r), s && yn(e, n.name, null, r);
	}, io = (e, t, n) => {
		const { active: r, registry: s } = e[be];
		let i = e, o = null, a = !1;
		const u = new NA({
			onprocessinginstruction(l, p) {
				l.toLowerCase() === "!doctype" && (e.doctype = p.slice(l.length).trim());
			},
			onopentag(l, p) {
				let f = !0;
				if (t) {
					if (o) i = nt(i, e.createElementNS(kr, l), r), i.ownerSVGElement = o, f = !1;
					else if (l === "svg" || l === "SVG") o = e.createElementNS(kr, l), i = nt(i, o, r), f = !1;
					else if (r) {
						const y = l.includes("-") ? l : p.is || "";
						if (y && s.has(y)) {
							const { Class: B } = s.get(y);
							i = nt(i, new B(), r), delete p.is, f = !1;
						}
					}
				}
				f && (i = nt(i, e.createElement(l), !1));
				let g = i[v];
				for (const y of BA(p)) DA(i, g, e.createAttribute(y), p[y], r);
			},
			oncomment(l) {
				nt(i, e.createComment(l), r);
			},
			ontext(l) {
				a ? nt(i, e.createCDATASection(l), r) : nt(i, e.createTextNode(l), r);
			},
			oncdatastart() {
				a = !0;
			},
			oncdataend() {
				a = !1;
			},
			onclosetag() {
				t && i === o && (o = null), i = i.parentNode;
			}
		}, {
			lowerCaseAttributeNames: !1,
			decodeEntities: !0,
			xmlMode: !t
		});
		return u.write(n), u.end(), e;
	}, wn = /* @__PURE__ */ new Map(), V = (e, t) => {
		for (const n of [].concat(e)) wn.set(n, t), wn.set(n.toUpperCase(), t);
	}, oo = ({ [w]: e, [v]: t }, n) => {
		for (; e !== t;) {
			switch (e.nodeType) {
				case 2:
					ao(e, n);
					break;
				case 3:
				case 8:
				case 4:
					uo(e, n);
					break;
				case 1:
					Ao(e, n), e = oe(e);
					break;
				case 10: lo(e, n);
			}
			e = e[w];
		}
		const r = n.length - 1, s = n[r];
		typeof s == "number" && s < 0 ? n[r] += -1 : n.push(-1);
	}, ao = (e, t) => {
		t.push(2, e.name);
		const n = e[M].trim();
		n && t.push(n);
	}, uo = (e, t) => {
		const n = e[M];
		n.trim() && t.push(e.nodeType, n);
	}, _A = (e, t) => {
		t.push(e.nodeType), oo(e, t);
	}, lo = ({ name: e, publicId: t, systemId: n }, r) => {
		r.push(10, e), t && r.push(t), n && r.push(n);
	}, Ao = (e, t) => {
		t.push(1, e.localName), oo(e, t);
	}, co = (e, t, n, r, s, i, o) => ({
		type: e,
		target: t,
		addedNodes: r,
		removedNodes: s,
		attributeName: i,
		oldValue: o,
		previousSibling: n?.previousSibling || null,
		nextSibling: n?.nextSibling || null
	}), ho = (e, t, n, r, s, i) => {
		if (!r || r.includes(n)) {
			const { callback: o, records: a, scheduled: u } = e;
			a.push(co("attributes", t, null, [], [], n, s ? i : void 0)), u || (e.scheduled = !0, Promise.resolve().then(() => {
				e.scheduled = !1, o(a.splice(0), e);
			}));
		}
	}, Hr = (e, t, n) => {
		const { ownerDocument: r } = e, { active: s, observers: i } = r[He];
		if (s) {
			for (const o of i) for (const [a, { childList: u, subtree: l, attributes: p, attributeFilter: f, attributeOldValue: g }] of o.nodes) if (u) {
				if (l && (a === r || a.contains(e)) || !l && a.children.includes(e)) {
					ho(o, e, t, f, g, n);
					break;
				}
			} else if (p && a === e) {
				ho(o, e, t, f, g, n);
				break;
			}
		}
	}, Ft = (e, t) => {
		const { ownerDocument: n } = e, { active: r, observers: s } = n[He];
		if (r) {
			for (const i of s) for (const [o, { subtree: a, childList: u, characterData: l }] of i.nodes) if (u && (t && (o === t || a && o.contains(t)) || !t && (a && (o === n || o.contains(e)) || !a && o[l ? "childNodes" : "children"].includes(e)))) {
				const { callback: p, records: f, scheduled: g } = i;
				f.push(co("childList", o, e, t ? [] : [e], t ? [e] : [])), g || (i.scheduled = !0, Promise.resolve().then(() => {
					i.scheduled = !1, p(f.splice(0), i);
				}));
				break;
			}
		}
	};
	var FA = class {
		constructor(e) {
			const t = /* @__PURE__ */ new Set();
			this.observers = t, this.active = !1, this.class = class {
				constructor(r) {
					this.callback = r, this.nodes = /* @__PURE__ */ new Map(), this.records = [], this.scheduled = !1;
				}
				disconnect() {
					this.records.splice(0), this.nodes.clear(), t.delete(this), e[He].active = !!t.size;
				}
				observe(r, s = {
					subtree: !1,
					childList: !1,
					attributes: !1,
					attributeFilter: null,
					attributeOldValue: !1,
					characterData: !1
				}) {
					("attributeOldValue" in s || "attributeFilter" in s) && (s.attributes = !0), s.childList = !!s.childList, s.subtree = !!s.subtree, this.nodes.set(r, s), t.add(this), e[He].active = !0;
				}
				takeRecords() {
					return this.records.splice(0);
				}
			};
		}
	};
	const kA = /* @__PURE__ */ new Set([
		"allowfullscreen",
		"allowpaymentrequest",
		"async",
		"autofocus",
		"autoplay",
		"checked",
		"class",
		"contenteditable",
		"controls",
		"default",
		"defer",
		"disabled",
		"draggable",
		"formnovalidate",
		"hidden",
		"id",
		"ismap",
		"itemscope",
		"loop",
		"multiple",
		"muted",
		"nomodule",
		"novalidate",
		"open",
		"playsinline",
		"readonly",
		"required",
		"reversed",
		"selected",
		"style",
		"truespeed"
	]), Lr = (e, t) => {
		const { [M]: n, name: r } = t;
		t.ownerElement = e, Sn(e, t, e[w]), r === "class" && (e.className = n), Hr(e, r, null), yn(e, r, null, n);
	}, go = (e, t) => {
		const { [M]: n, name: r } = t;
		Ie(t[X], t[w]), t.ownerElement = t[X] = t[w] = null, r === "class" && (e[gt] = null), Hr(e, r, n), yn(e, r, n, null);
	}, _ = {
		get(e, t) {
			return e.hasAttribute(t);
		},
		set(e, t, n) {
			n ? e.setAttribute(t, "") : e.removeAttribute(t);
		}
	}, Je = {
		get(e, t) {
			return parseFloat(e.getAttribute(t) || 0);
		},
		set(e, t, n) {
			e.setAttribute(t, n);
		}
	}, S = {
		get(e, t) {
			return e.getAttribute(t) || "";
		},
		set(e, t, n) {
			e.setAttribute(t, n);
		}
	}, Cn = /* @__PURE__ */ new WeakMap();
	function MA(e, t) {
		return typeof t == "function" ? t.call(e.target, e) : t.handleEvent(e), e._stopImmediatePropagationFlag;
	}
	function GA({ currentTarget: e, target: t }) {
		const n = Cn.get(e);
		if (n && n.has(this.type)) {
			const r = n.get(this.type);
			e === t ? this.eventPhase = this.AT_TARGET : this.eventPhase = this.BUBBLING_PHASE, this.currentTarget = e, this.target = t;
			for (const [s, i] of r) if (i && i.once && r.delete(s), MA(this, s)) break;
			return delete this.currentTarget, delete this.target, this.cancelBubble;
		}
	}
	var Or = class {
		constructor() {
			Cn.set(this, /* @__PURE__ */ new Map());
		}
		_getParent() {
			return null;
		}
		addEventListener(e, t, n) {
			const r = Cn.get(this);
			r.has(e) || r.set(e, /* @__PURE__ */ new Map()), r.get(e).set(t, n);
		}
		removeEventListener(e, t) {
			const n = Cn.get(this);
			if (n.has(e)) {
				const r = n.get(e);
				r.delete(t) && !r.size && n.delete(e);
			}
		}
		dispatchEvent(e) {
			let t = this;
			for (e.eventPhase = e.CAPTURING_PHASE; t;) t.dispatchEvent && e._path.push({
				currentTarget: t,
				target: this
			}), t = e.bubbles && t._getParent && t._getParent();
			return e._path.some(GA, e), e._path = [], e.eventPhase = e.NONE, !e.defaultPrevented;
		}
	}, Qe = class extends Array {
		item(e) {
			return e < this.length ? this[e] : null;
		}
	};
	const fo = ({ parentNode: e }) => {
		let t = 0;
		for (; e;) t++, e = e.parentNode;
		return t;
	};
	var rt = class extends Or {
		static get ELEMENT_NODE() {
			return 1;
		}
		static get ATTRIBUTE_NODE() {
			return 2;
		}
		static get TEXT_NODE() {
			return 3;
		}
		static get CDATA_SECTION_NODE() {
			return 4;
		}
		static get COMMENT_NODE() {
			return 8;
		}
		static get DOCUMENT_NODE() {
			return 9;
		}
		static get DOCUMENT_FRAGMENT_NODE() {
			return 11;
		}
		static get DOCUMENT_TYPE_NODE() {
			return 10;
		}
		constructor(e, t, n) {
			super(), this.ownerDocument = e, this.localName = t, this.nodeType = n, this.parentNode = null, this[w] = null, this[X] = null;
		}
		get ELEMENT_NODE() {
			return 1;
		}
		get ATTRIBUTE_NODE() {
			return 2;
		}
		get TEXT_NODE() {
			return 3;
		}
		get CDATA_SECTION_NODE() {
			return 4;
		}
		get COMMENT_NODE() {
			return 8;
		}
		get DOCUMENT_NODE() {
			return 9;
		}
		get DOCUMENT_FRAGMENT_NODE() {
			return 11;
		}
		get DOCUMENT_TYPE_NODE() {
			return 10;
		}
		get baseURI() {
			const e = this.nodeType === 9 ? this : this.ownerDocument;
			if (e) {
				const t = e.querySelector("base");
				if (t) return t.getAttribute("href");
				const { location: n } = e.defaultView;
				if (n) return n.href;
			}
			return null;
		}
		get isConnected() {
			return !1;
		}
		get nodeName() {
			return this.localName;
		}
		get parentElement() {
			return null;
		}
		get previousSibling() {
			return null;
		}
		get previousElementSibling() {
			return null;
		}
		get nextSibling() {
			return null;
		}
		get nextElementSibling() {
			return null;
		}
		get childNodes() {
			return new Qe();
		}
		get firstChild() {
			return null;
		}
		get lastChild() {
			return null;
		}
		get nodeValue() {
			return null;
		}
		set nodeValue(e) {}
		get textContent() {
			return null;
		}
		set textContent(e) {}
		normalize() {}
		cloneNode() {
			return null;
		}
		contains() {
			return !1;
		}
		insertBefore(e, t) {
			return e;
		}
		appendChild(e) {
			return e;
		}
		replaceChild(e, t) {
			return t;
		}
		removeChild(e) {
			return e;
		}
		toString() {
			return "";
		}
		hasChildNodes() {
			return !!this.lastChild;
		}
		isSameNode(e) {
			return this === e;
		}
		compareDocumentPosition(e) {
			let t = 0;
			if (this !== e) {
				let n = fo(this), r = fo(e);
				if (n < r) t += 4, this.contains(e) && (t += 16);
				else if (r < n) t += 2, e.contains(this) && (t += 8);
				else if (n && r) {
					const { childNodes: s } = this.parentNode;
					s.indexOf(this) < s.indexOf(e) ? t += 4 : t += 2;
				}
				(!n || !r) && (t += 32, t += 1);
			}
			return t;
		}
		isEqualNode(e) {
			if (this === e) return !0;
			if (this.nodeType === e.nodeType) {
				switch (this.nodeType) {
					case 9:
					case 11: {
						const t = this.childNodes, n = e.childNodes;
						return t.length === n.length && t.every((r, s) => r.isEqualNode(n[s]));
					}
				}
				return this.toString() === e.toString();
			}
			return !1;
		}
		_getParent() {
			return this.parentNode;
		}
		getRootNode() {
			let e = this;
			for (; e.parentNode;) e = e.parentNode;
			return e;
		}
	};
	const { replace: HA } = "", LA = /[<>&\xA0]/g, OA = {
		"\xA0": "&#160;",
		"&": "&amp;",
		"<": "&lt;",
		">": "&gt;"
	}, KA = (e) => OA[e], Kr = (e) => HA.call(e, LA, KA), JA = /"/g;
	var kt = class Qu extends rt {
		constructor(t, n, r = "") {
			super(t, n, 2), this.ownerElement = null, this.name = et(n), this[M] = et(r), this[un] = !1;
		}
		get value() {
			return this[M];
		}
		set value(t) {
			const { [M]: n, name: r, ownerElement: s } = this;
			this[M] = et(t), this[un] = !0, s && (Hr(s, r, n), yn(s, r, n, this[M]));
		}
		cloneNode() {
			const { ownerDocument: t, name: n, [M]: r } = this;
			return new Qu(t, n, r);
		}
		toString() {
			const { name: t, [M]: n } = this;
			return kA.has(t) && !n ? Oe(this) ? t : `${t}=""` : `${t}="${(Oe(this) ? n : Kr(n)).replace(JA, "&quot;")}"`;
		}
		toJSON() {
			const t = [];
			return ao(this, t), t;
		}
	};
	const po = ({ ownerDocument: e, parentNode: t }) => {
		for (; t;) {
			if (t === e) return !0;
			t = t.parentNode || t.host;
		}
		return !1;
	}, mo = ({ parentNode: e }) => {
		if (e) switch (e.nodeType) {
			case 9:
			case 11: return null;
		}
		return e;
	}, Mt = ({ [X]: e }) => {
		switch (e ? e.nodeType : 0) {
			case -1: return e[se];
			case 3:
			case 8:
			case 4: return e;
		}
		return null;
	}, st = (e) => {
		const t = oe(e)[w];
		return t && (t.nodeType === -1 ? null : t);
	}, Jr = (e) => {
		let t = st(e);
		for (; t && t.nodeType !== 1;) t = st(t);
		return t;
	}, So = (e) => {
		let t = Mt(e);
		for (; t && t.nodeType !== 1;) t = Mt(t);
		return t;
	}, Pr = (e, t) => {
		const n = e.createDocumentFragment();
		return n.append(...t), n;
	}, bo = (e, t) => {
		const { ownerDocument: n, parentNode: r } = e;
		r && r.insertBefore(Pr(n, t), e);
	}, yo = (e, t) => {
		const { ownerDocument: n, parentNode: r } = e;
		r && r.insertBefore(Pr(n, t), oe(e)[w]);
	}, Wr = (e, t) => {
		const { ownerDocument: n, parentNode: r } = e;
		r && (t.includes(e) && Wr(e, [e = e.cloneNode()]), r.insertBefore(Pr(n, t), e), e.remove());
	}, wo = (e, t, n) => {
		const { parentNode: r, nodeType: s } = t;
		(e || n) && (zi(e, n), t[X] = null, oe(t)[w] = null), r && (t.parentNode = null, Ft(t, r), s === 1 && QA(t));
	};
	var Gt = class extends rt {
		constructor(e, t, n, r) {
			super(e, t, n), this[M] = et(r);
		}
		get isConnected() {
			return po(this);
		}
		get parentElement() {
			return mo(this);
		}
		get previousSibling() {
			return Mt(this);
		}
		get nextSibling() {
			return st(this);
		}
		get previousElementSibling() {
			return So(this);
		}
		get nextElementSibling() {
			return Jr(this);
		}
		before(...e) {
			bo(this, e);
		}
		after(...e) {
			yo(this, e);
		}
		replaceWith(...e) {
			Wr(this, e);
		}
		remove() {
			wo(this[X], this, this[w]);
		}
		get data() {
			return this[M];
		}
		set data(e) {
			this[M] = et(e), Ft(this, this.parentNode);
		}
		get nodeValue() {
			return this.data;
		}
		set nodeValue(e) {
			this.data = e;
		}
		get textContent() {
			return this.data;
		}
		set textContent(e) {
			this.data = e;
		}
		get length() {
			return this.data.length;
		}
		substringData(e, t) {
			return this.data.substr(e, t);
		}
		appendData(e) {
			this.data += e;
		}
		insertData(e, t) {
			const { data: n } = this;
			this.data = n.slice(0, e) + t + n.slice(e);
		}
		deleteData(e, t) {
			const { data: n } = this;
			this.data = n.slice(0, e) + n.slice(e + t);
		}
		replaceData(e, t, n) {
			const { data: r } = this;
			this.data = r.slice(0, e) + n + r.slice(e + t);
		}
		toJSON() {
			const e = [];
			return uo(this, e), e;
		}
	}, Yr = class Ru extends Gt {
		constructor(t, n = "") {
			super(t, "#cdatasection", 4, n);
		}
		cloneNode() {
			const { ownerDocument: t, [M]: n } = this;
			return new Ru(t, n);
		}
		toString() {
			return `<![CDATA[${this[M]}]]>`;
		}
	}, Ur = class Nu extends Gt {
		constructor(t, n = "") {
			super(t, "#comment", 8, n);
		}
		cloneNode() {
			const { ownerDocument: t, [M]: n } = this;
			return new Nu(t, n);
		}
		toString() {
			return `<!--${this[M]}-->`;
		}
	};
	function it() {
		return !0;
	}
	function J() {
		return !1;
	}
	var x;
	(function(e) {
		e.Attribute = "attribute", e.Pseudo = "pseudo", e.PseudoElement = "pseudo-element", e.Tag = "tag", e.Universal = "universal", e.Adjacent = "adjacent", e.Child = "child", e.Descendant = "descendant", e.Parent = "parent", e.Sibling = "sibling", e.ColumnCombinator = "column-combinator";
	})(x || (x = {}));
	var U;
	(function(e) {
		e.Any = "any", e.Element = "element", e.End = "end", e.Equals = "equals", e.Exists = "exists", e.Hyphen = "hyphen", e.Not = "not", e.Start = "start";
	})(U || (U = {}));
	const Co = /^[^#\\]?(?:\\(?:[\da-f]{1,6}\s?|.)|[\w\u00B0-\uFFFF-])+/, PA = /\\([\da-f]{1,6}\s?|(\s)|.)/gi;
	var I;
	(function(e) {
		e[e.LeftParenthesis = 40] = "LeftParenthesis", e[e.RightParenthesis = 41] = "RightParenthesis", e[e.LeftSquareBracket = 91] = "LeftSquareBracket", e[e.RightSquareBracket = 93] = "RightSquareBracket", e[e.Comma = 44] = "Comma", e[e.Period = 46] = "Period", e[e.Colon = 58] = "Colon", e[e.SingleQuote = 39] = "SingleQuote", e[e.DoubleQuote = 34] = "DoubleQuote", e[e.Plus = 43] = "Plus", e[e.Tilde = 126] = "Tilde", e[e.QuestionMark = 63] = "QuestionMark", e[e.ExclamationMark = 33] = "ExclamationMark", e[e.Slash = 47] = "Slash", e[e.Equal = 61] = "Equal", e[e.Dollar = 36] = "Dollar", e[e.Pipe = 124] = "Pipe", e[e.Circumflex = 94] = "Circumflex", e[e.Asterisk = 42] = "Asterisk", e[e.GreaterThan = 62] = "GreaterThan", e[e.LessThan = 60] = "LessThan", e[e.Hash = 35] = "Hash", e[e.LowerI = 105] = "LowerI", e[e.LowerS = 115] = "LowerS", e[e.BackSlash = 92] = "BackSlash", e[e.Space = 32] = "Space", e[e.Tab = 9] = "Tab", e[e.NewLine = 10] = "NewLine", e[e.FormFeed = 12] = "FormFeed", e[e.CarriageReturn = 13] = "CarriageReturn";
	})(I || (I = {}));
	const WA = /* @__PURE__ */ new Map([
		[I.Tilde, U.Element],
		[I.Circumflex, U.Start],
		[I.Dollar, U.End],
		[I.Asterisk, U.Any],
		[I.ExclamationMark, U.Not],
		[I.Pipe, U.Hyphen]
	]), YA = /* @__PURE__ */ new Set([
		"has",
		"not",
		"matches",
		"is",
		"where",
		"host",
		"host-context"
	]), UA = /* @__PURE__ */ new Set([
		"before",
		"after",
		"first-line",
		"first-letter"
	]);
	function Io(e) {
		switch (e.type) {
			case x.Adjacent:
			case x.Child:
			case x.Descendant:
			case x.Parent:
			case x.Sibling:
			case x.ColumnCombinator: return !0;
			case x.Attribute:
			case x.Pseudo:
			case x.PseudoElement:
			case x.Tag:
			case x.Universal: return !1;
		}
	}
	const VA = /* @__PURE__ */ new Set(["contains", "icontains"]);
	function ZA(e, t, n) {
		const r = Number.parseInt(t, 16) - 65536;
		return Number.isNaN(r) || n ? t : r < 0 ? String.fromCharCode(r + 65536) : String.fromCharCode(r >> 10 | 55296, r & 1023 | 56320);
	}
	function Ht(e) {
		return e.replace(PA, ZA);
	}
	function Vr(e) {
		return e === I.SingleQuote || e === I.DoubleQuote;
	}
	function vo(e) {
		return e === I.Space || e === I.Tab || e === I.NewLine || e === I.FormFeed || e === I.CarriageReturn;
	}
	function Zr(e) {
		const t = [], n = Eo(t, `${e}`, 0);
		if (n < e.length) throw new Error(`Unmatched selector: ${e.slice(n)}`);
		return t;
	}
	function Eo(e, t, n) {
		let r = [];
		function s(f) {
			const g = t.slice(n + f).match(Co);
			if (!g) throw new Error(`Expected name, found ${t.slice(n)}`);
			const [y] = g;
			return n += f + y.length, Ht(y);
		}
		function i(f) {
			for (n += f; n < t.length && vo(t.charCodeAt(n));) n++;
		}
		function o() {
			n += 1;
			const f = n;
			for (let g = 1; n < t.length; n++) switch (t.charCodeAt(n)) {
				case I.BackSlash:
					n += 1;
					break;
				case I.LeftParenthesis:
					g += 1;
					break;
				case I.RightParenthesis: if (g -= 1, g === 0) return Ht(t.slice(f, n++));
			}
			throw new Error("Parenthesis not matched");
		}
		function a() {
			if (r.length > 0 && Io(r[r.length - 1])) throw new Error("Did not expect successive traversals.");
		}
		function u(f) {
			if (r.length > 0 && r[r.length - 1].type === x.Descendant) {
				r[r.length - 1].type = f;
				return;
			}
			a(), r.push({ type: f });
		}
		function l(f, g) {
			r.push({
				type: x.Attribute,
				name: f,
				action: g,
				value: s(1),
				namespace: null,
				ignoreCase: "quirks"
			});
		}
		function p() {
			if (r.length > 0 && r[r.length - 1].type === x.Descendant && r.pop(), r.length === 0) throw new Error("Empty sub-selector");
			e.push(r);
		}
		if (i(0), t.length === n) return n;
		e: for (; n < t.length;) {
			const f = t.charCodeAt(n);
			switch (f) {
				case I.Space:
				case I.Tab:
				case I.NewLine:
				case I.FormFeed:
				case I.CarriageReturn:
					(r.length === 0 || r[0].type !== x.Descendant) && (a(), r.push({ type: x.Descendant })), i(1);
					break;
				case I.GreaterThan:
					u(x.Child), i(1);
					break;
				case I.LessThan:
					u(x.Parent), i(1);
					break;
				case I.Tilde:
					u(x.Sibling), i(1);
					break;
				case I.Plus:
					u(x.Adjacent), i(1);
					break;
				case I.Period:
					l("class", U.Element);
					break;
				case I.Hash:
					l("id", U.Equals);
					break;
				case I.LeftSquareBracket: {
					i(1);
					let g, y = null;
					t.charCodeAt(n) === I.Pipe ? g = s(1) : t.startsWith("*|", n) ? (y = "*", g = s(2)) : (g = s(0), t.charCodeAt(n) === I.Pipe && t.charCodeAt(n + 1) !== I.Equal && (y = g, g = s(1))), i(0);
					let B = U.Exists;
					const N = WA.get(t.charCodeAt(n));
					if (N) {
						if (B = N, t.charCodeAt(n + 1) !== I.Equal) throw new Error("Expected `=`");
						i(2);
					} else t.charCodeAt(n) === I.Equal && (B = U.Equals, i(1));
					let z = "", W = null;
					if (B !== "exists") {
						if (Vr(t.charCodeAt(n))) {
							const G = t.charCodeAt(n);
							n += 1;
							const P = n;
							for (; n < t.length && t.charCodeAt(n) !== G;) n += t.charCodeAt(n) === I.BackSlash ? 2 : 1;
							if (t.charCodeAt(n) !== G) throw new Error("Attribute value didn't end");
							z = Ht(t.slice(P, n)), n += 1;
						} else {
							const G = n;
							for (; n < t.length && !vo(t.charCodeAt(n)) && t.charCodeAt(n) !== I.RightSquareBracket;) n += t.charCodeAt(n) === I.BackSlash ? 2 : 1;
							z = Ht(t.slice(G, n));
						}
						switch (i(0), t.charCodeAt(n) | 32) {
							case I.LowerI:
								W = !0, i(1);
								break;
							case I.LowerS: W = !1, i(1);
						}
					}
					if (t.charCodeAt(n) !== I.RightSquareBracket) throw new Error("Attribute selector didn't terminate");
					n += 1;
					const F = {
						type: x.Attribute,
						name: g,
						action: B,
						value: z,
						namespace: y,
						ignoreCase: W
					};
					r.push(F);
					break;
				}
				case I.Colon: {
					if (t.charCodeAt(n + 1) === I.Colon) {
						r.push({
							type: x.PseudoElement,
							name: s(2).toLowerCase(),
							data: t.charCodeAt(n) === I.LeftParenthesis ? o() : null
						});
						break;
					}
					const g = s(1).toLowerCase();
					if (UA.has(g)) {
						r.push({
							type: x.PseudoElement,
							name: g,
							data: null
						});
						break;
					}
					let y = null;
					if (t.charCodeAt(n) === I.LeftParenthesis) if (YA.has(g)) {
						if (Vr(t.charCodeAt(n + 1))) throw new Error(`Pseudo-selector ${g} cannot be quoted`);
						if (y = [], n = Eo(y, t, n + 1), t.charCodeAt(n) !== I.RightParenthesis) throw new Error(`Missing closing parenthesis in :${g} (${t})`);
						n += 1;
					} else {
						if (y = o(), VA.has(g)) {
							const B = y.charCodeAt(0);
							B === y.charCodeAt(y.length - 1) && Vr(B) && (y = y.slice(1, -1));
						}
						y = Ht(y);
					}
					r.push({
						type: x.Pseudo,
						name: g,
						data: y
					});
					break;
				}
				case I.Comma:
					p(), r = [], i(1);
					break;
				default: {
					if (t.startsWith("/*", n)) {
						const B = t.indexOf("*/", n + 2);
						if (B === -1) throw new Error("Comment was not terminated");
						n = B + 2, r.length === 0 && i(0);
						break;
					}
					let g = null, y;
					if (f === I.Asterisk) n += 1, y = "*";
					else if (f === I.Pipe) {
						if (y = "", t.charCodeAt(n + 1) === I.Pipe) {
							u(x.ColumnCombinator), i(2);
							break;
						}
					} else if (Co.test(t.slice(n))) y = s(0);
					else break e;
					t.charCodeAt(n) === I.Pipe && t.charCodeAt(n + 1) !== I.Pipe && (g = y, t.charCodeAt(n + 1) === I.Asterisk ? (y = "*", n += 2) : y = s(1)), r.push(y === "*" ? {
						type: x.Universal,
						namespace: g
					} : {
						type: x.Tag,
						name: y,
						namespace: g
					});
				}
			}
		}
		return p(), n;
	}
	var $;
	(function(e) {
		e.Root = "root", e.Text = "text", e.Directive = "directive", e.Comment = "comment", e.Script = "script", e.Style = "style", e.Tag = "tag", e.CDATA = "cdata", e.Doctype = "doctype";
	})($ || ($ = {}));
	function jA(e) {
		return e.type === $.Tag || e.type === $.Script || e.type === $.Style;
	}
	const XA = $.Root, qA = $.Text, $A = $.Directive, zA = $.Comment, ec = $.Script, tc = $.Style, nc = $.Tag, rc = $.CDATA;
	$.Doctype;
	function de(e) {
		return jA(e);
	}
	function xo(e) {
		return e.type === $.CDATA;
	}
	function Lt(e) {
		return e.type === $.Text;
	}
	function sc(e) {
		return e.type === $.Comment;
	}
	function Re(e) {
		return Object.hasOwn(e, "children");
	}
	function Ot(e, t, n = !0, r = Number.POSITIVE_INFINITY) {
		return To(e, Array.isArray(t) ? t : [t], n, r);
	}
	function To(e, t, n, r) {
		const s = [], i = [Array.isArray(t) ? t : [t]], o = [0];
		for (;;) {
			if (o[0] >= i[0].length) {
				if (o.length === 1) return s;
				i.shift(), o.shift();
				continue;
			}
			const a = i[0][o[0]++];
			if (e(a) && (s.push(a), --r <= 0)) return s;
			n && Re(a) && a.children.length > 0 && (o.unshift(0), i.unshift(a.children));
		}
	}
	function jr(e, t, n = !0) {
		const r = Array.isArray(t) ? t : [t];
		for (const s of r) {
			if (de(s) && e(s)) return s;
			if (n && Re(s) && s.children.length > 0) {
				const i = jr(e, s.children, !0);
				if (i) return i;
			}
		}
		return null;
	}
	function Bo(e, t) {
		return (Array.isArray(t) ? t : [t]).some((n) => de(n) && e(n) || Re(n) && Bo(e, n.children));
	}
	function ic(e, t) {
		const n = [], r = [Array.isArray(t) ? t : [t]], s = [0];
		for (;;) {
			if (s[0] >= r[0].length) {
				if (r.length === 1) return n;
				r.shift(), s.shift();
				continue;
			}
			const i = r[0][s[0]++];
			de(i) && e(i) && n.push(i), Re(i) && i.children.length > 0 && (s.unshift(0), r.unshift(i.children));
		}
	}
	const In = {
		tag_name(e) {
			return typeof e == "function" ? (t) => de(t) && e(t.name) : e === "*" ? de : (t) => de(t) && t.name === e;
		},
		tag_type(e) {
			return typeof e == "function" ? (t) => e(t.type) : (t) => t.type === e;
		},
		tag_contains(e) {
			return typeof e == "function" ? (t) => Lt(t) && e(t.data) : (t) => Lt(t) && t.data === e;
		}
	};
	function Xr(e, t) {
		return typeof t == "function" ? (n) => de(n) && t(n.attribs[e]) : (n) => de(n) && n.attribs[e] === t;
	}
	function oc(e, t) {
		return (n) => e(n) || t(n);
	}
	function Qo(e) {
		const t = Object.keys(e).map((n) => {
			const r = e[n];
			return Object.hasOwn(In, n) ? In[n](r) : Xr(n, r);
		});
		return t.length === 0 ? null : t.reduce(oc);
	}
	function ac(e, t) {
		const n = Qo(e);
		return n ? n(t) : !0;
	}
	function uc(e, t, n, r = Number.POSITIVE_INFINITY) {
		const s = Qo(e);
		return s ? Ot(s, t, n, r) : [];
	}
	function lc(e, t, n = !0) {
		return Array.isArray(t) || (t = [t]), jr(Xr("id", e), t, n);
	}
	function yt(e, t, n = !0, r = Number.POSITIVE_INFINITY) {
		return Ot(In.tag_name(e), t, n, r);
	}
	function Ac(e, t, n = !0, r = Number.POSITIVE_INFINITY) {
		return Ot(Xr("class", e), t, n, r);
	}
	function cc(e, t, n = !0, r = Number.POSITIVE_INFINITY) {
		return Ot(In.tag_type(e), t, n, r);
	}
	const hc = /* @__PURE__ */ new Map([
		[34, "&quot;"],
		[38, "&amp;"],
		[39, "&apos;"],
		[60, "&lt;"],
		[62, "&gt;"]
	]), dc = typeof String.prototype.codePointAt == "function" ? (e, t) => e.codePointAt(t) : (e, t) => (e.charCodeAt(t) & 64512) === 55296 ? (e.charCodeAt(t) - 55296) * 1024 + e.charCodeAt(t + 1) - 56320 + 65536 : e.charCodeAt(t);
	function Ro(e) {
		let t, n = 0;
		const { length: r } = e;
		for (let s = 0; s < r; s++) {
			const i = e.charCodeAt(s);
			if (i < 128 && ((1342177476 >>> i & 1) === 0 || i >= 64 || i < 32)) continue;
			if (t === void 0 ? t = e.substring(0, s) : n !== s && (t += e.substring(n, s)), i < 64) {
				t += hc.get(i), n = s + 1;
				continue;
			}
			const o = dc(e, s);
			t += `&#x${o.toString(16)};`, o !== i && s++, n = s + 1;
		}
		return t === void 0 ? e : (n < r && (t += e.substr(n)), t);
	}
	function No(e, t) {
		return function(r) {
			let s, i = 0, o = "";
			for (; s = e.exec(r);) i !== s.index && (o += r.substring(i, s.index)), o += t.get(s[0].charCodeAt(0)), i = s.index + 1;
			return o + r.substring(i);
		};
	}
	const gc = No(/["&\u00A0]/g, /* @__PURE__ */ new Map([
		[34, "&quot;"],
		[38, "&amp;"],
		[160, "&nbsp;"]
	])), fc = No(/[&<>\u00A0]/g, /* @__PURE__ */ new Map([
		[38, "&amp;"],
		[60, "&lt;"],
		[62, "&gt;"],
		[160, "&nbsp;"]
	])), pc = new Map("altGlyph altGlyphDef altGlyphItem animateColor animateMotion animateTransform clipPath feBlend feColorMatrix feComponentTransfer feComposite feConvolveMatrix feDiffuseLighting feDisplacementMap feDistantLight feDropShadow feFlood feFuncA feFuncB feFuncG feFuncR feGaussianBlur feImage feMerge feMergeNode feMorphology feOffset fePointLight feSpecularLighting feSpotLight feTile feTurbulence foreignObject glyphRef linearGradient radialGradient textPath".split(" ").map((e) => [e.toLowerCase(), e])), mc = new Map("definitionURL attributeName attributeType baseFrequency baseProfile calcMode clipPathUnits diffuseConstant edgeMode filterUnits glyphRef gradientTransform gradientUnits kernelMatrix kernelUnitLength keyPoints keySplines keyTimes lengthAdjust limitingConeAngle markerHeight markerUnits markerWidth maskContentUnits maskUnits numOctaves pathLength patternContentUnits patternTransform patternUnits pointsAtX pointsAtY pointsAtZ preserveAlpha preserveAspectRatio primitiveUnits refX refY repeatCount repeatDur requiredExtensions requiredFeatures specularConstant specularExponent spreadMethod startOffset stdDeviation stitchTiles surfaceScale systemLanguage tableValues targetX targetY textLength viewBox viewTarget xChannelSelector yChannelSelector zoomAndPan".split(" ").map((e) => [e.toLowerCase(), e])), Sc = new Set("style script xmp iframe noembed noframes plaintext noscript".split(" ")), bc = new Set("area base basefont br col command embed frame hr img input isindex keygen link meta param source track wbr".split(" ")), yc = /* @__PURE__ */ new Set(["svg", "math"]), wc = new Set("mi mo mn ms mtext annotation-xml foreignObject desc title".split(" "));
	function Cc(e, t = {}) {
		const n = "length" in e ? e : [e], r = t.xmlMode ?? !1;
		let s = "";
		for (let i = 0; i < n.length; i++) s += _o(n[i], t, r);
		return s;
	}
	function Do(e, t, n) {
		let r = "";
		for (let s = 0; s < e.length; s++) r += _o(e[s], t, n);
		return r;
	}
	function _o(e, t, n) {
		switch (e.type) {
			case XA: return Do(e.children, t, n);
			case $A: return `<${e.data}>`;
			case zA: return `<!--${e.data}-->`;
			case rc: return `<![CDATA[${e.children[0].data}]]>`;
			case ec:
			case tc:
			case nc: return Ic(e, t, n);
			case qA: {
				const r = e, s = r.data || "";
				return (t.encodeEntities ?? t.decodeEntities) !== !1 && !(!n && r.parent && Sc.has(r.parent.name)) ? n || t.encodeEntities !== "utf8" ? Ro(s) : fc(s) : s;
			}
		}
	}
	function Ic(e, t, n) {
		n === "foreign" && (e.name = pc.get(e.name) ?? e.name, e.parent && wc.has(e.parent.name) && (n = !1)), !n && yc.has(e.name) && (n = "foreign");
		const { name: r, children: s } = e, i = !n && bc.has(r);
		let o = `<${r}${Ec(e.attribs, t, n)}`;
		return s.length === 0 && (n ? t.selfClosingTags !== !1 : t.selfClosingTags && i) ? o += n ? "/>" : " />" : (o += ">", s.length > 0 && (o += Do(s, t, n)), i || (o += `</${r}>`)), o;
	}
	function vc(e) {
		return e.replaceAll("\"", "&quot;");
	}
	function Ec(e, t, n) {
		if (!e) return "";
		const r = (t.encodeEntities ?? t.decodeEntities) === !1 ? vc : n || t.encodeEntities !== "utf8" ? Ro : gc, s = n === "foreign", i = !!(t.emptyAttrs ?? n);
		let o = "";
		for (const a in e) {
			if (!Object.hasOwn(e, a)) continue;
			const u = e[a], l = s ? mc.get(a) ?? a : a;
			o += !i && (u == null || u === "") ? ` ${l}` : ` ${l}="${r(u == null ? "" : String(u))}"`;
		}
		return o;
	}
	function Fo(e, t) {
		return Cc(e, t);
	}
	function xc(e, t) {
		return Re(e) ? e.children.map((n) => Fo(n, t)).join("") : "";
	}
	function vn(e) {
		return Array.isArray(e) ? e.map(vn).join("") : de(e) ? e.name === "br" ? `
` : vn(e.children) : xo(e) ? vn(e.children) : Lt(e) ? e.data : "";
	}
	function En(e) {
		return Array.isArray(e) ? e.map(En).join("") : Re(e) && !sc(e) ? En(e.children) : Lt(e) ? e.data : "";
	}
	function qr(e) {
		return Array.isArray(e) ? e.map(qr).join("") : Re(e) && (e.type === $.Tag || xo(e)) ? qr(e.children) : Lt(e) ? e.data : "";
	}
	function Tc(e) {
		const t = xn(Dc, e);
		return t ? t.name === "feed" ? Bc(t) : Qc(t) : null;
	}
	function Bc(e) {
		const t = e.children, n = {
			type: "atom",
			items: yt("entry", t).map((i) => {
				const { children: o } = i, a = { media: ko(o) };
				ae(a, "id", "id", o), ae(a, "title", "title", o);
				const u = xn("link", o)?.attribs.href;
				u && (a.link = u);
				const l = Pe("summary", o) || Pe("content", o);
				l && (a.description = l);
				const p = Pe("updated", o);
				return p && (a.pubDate = new Date(p)), a;
			})
		};
		ae(n, "id", "id", t), ae(n, "title", "title", t);
		const r = xn("link", t)?.attribs.href;
		r && (n.link = r), ae(n, "description", "subtitle", t);
		const s = Pe("updated", t);
		return s && (n.updated = new Date(s)), ae(n, "author", "email", t, !0), n;
	}
	function Qc(e) {
		const t = xn("channel", e.children)?.children ?? [], n = {
			type: e.name.substr(0, 3),
			id: "",
			items: yt("item", e.children).map((s) => {
				const { children: i } = s, o = { media: ko(i) };
				ae(o, "id", "guid", i), ae(o, "title", "title", i), ae(o, "link", "link", i), ae(o, "description", "description", i);
				const a = Pe("pubDate", i) || Pe("dc:date", i);
				return a && (o.pubDate = new Date(a)), o;
			})
		};
		ae(n, "title", "title", t), ae(n, "link", "link", t), ae(n, "description", "description", t);
		const r = Pe("lastBuildDate", t);
		return r && (n.updated = new Date(r)), ae(n, "author", "managingEditor", t, !0), n;
	}
	const Rc = [
		"url",
		"type",
		"lang"
	], Nc = [
		"fileSize",
		"bitrate",
		"framerate",
		"samplingrate",
		"channels",
		"duration",
		"height",
		"width"
	];
	function ko(e) {
		return yt("media:content", e).map((t) => {
			const { attribs: n } = t, r = {
				medium: n.medium,
				isDefault: !!n.isDefault
			};
			for (const s of Rc) n[s] && (r[s] = n[s]);
			for (const s of Nc) n[s] && (r[s] = Number.parseInt(n[s], 10));
			return n.expression && (r.expression = n.expression), r;
		});
	}
	function xn(e, t) {
		return yt(e, t, !0, 1)[0];
	}
	function Pe(e, t, n = !1) {
		return En(yt(e, t, n, 1)).trim();
	}
	function ae(e, t, n, r, s = !1) {
		const i = Pe(n, r, s);
		i && (e[t] = i);
	}
	function Dc(e) {
		return e === "rss" || e === "feed" || e === "rdf:RDF";
	}
	function _c(e) {
		let t = e.length;
		for (; --t >= 0;) {
			const n = e[t];
			if (t > 0 && e.lastIndexOf(n, t - 1) >= 0) {
				e.splice(t, 1);
				continue;
			}
			for (let r = n.parent; r; r = r.parent) if (e.includes(r)) {
				e.splice(t, 1);
				break;
			}
		}
		return e;
	}
	var ge;
	(function(e) {
		e[e.DISCONNECTED = 1] = "DISCONNECTED", e[e.PRECEDING = 2] = "PRECEDING", e[e.FOLLOWING = 4] = "FOLLOWING", e[e.CONTAINS = 8] = "CONTAINS", e[e.CONTAINED_BY = 16] = "CONTAINED_BY";
	})(ge || (ge = {}));
	function Mo(e, t) {
		const n = [], r = [];
		if (e === t) return 0;
		let s = Re(e) ? e : e.parent;
		for (; s;) n.unshift(s), s = s.parent;
		for (s = Re(t) ? t : t.parent; s;) r.unshift(s), s = s.parent;
		const i = Math.min(n.length, r.length);
		let o = 0;
		for (; o < i && n[o] === r[o];) o++;
		if (o === 0) return ge.DISCONNECTED;
		const a = n[o - 1], u = a.children, l = n[o], p = r[o];
		return u.indexOf(l) > u.indexOf(p) ? a === t ? ge.FOLLOWING | ge.CONTAINED_BY : ge.FOLLOWING : a === e ? ge.PRECEDING | ge.CONTAINS : ge.PRECEDING;
	}
	function Fc(e) {
		return e = e.filter((t, n, r) => !r.includes(t, n + 1)), e.sort((t, n) => {
			const r = Mo(t, n);
			return r & ge.PRECEDING ? -1 : r & ge.FOLLOWING ? 1 : 0;
		}), e;
	}
	function Kt(e) {
		if (e.prev && (e.prev.next = e.next), e.next && (e.next.prev = e.prev), e.parent) {
			const t = e.parent.children, n = t.lastIndexOf(e);
			n !== -1 && t.splice(n, 1);
		}
		e.next = null, e.prev = null, e.parent = null;
	}
	function kc(e, t) {
		if (t.prev = e.prev, t.prev && (t.prev.next = t), t.next = e.next, t.next && (t.next.prev = t), t.parent = e.parent, t.parent) {
			const { children: n } = t.parent, r = n.lastIndexOf(e);
			if (r === -1) return;
			n[r] = t, e.parent = null;
		}
	}
	function Mc(e, t) {
		if (Kt(t), t.next = null, t.parent = e, e.children.push(t) > 1) {
			const n = e.children[e.children.length - 2];
			n.next = t, t.prev = n;
		} else t.prev = null;
	}
	function Gc(e, t) {
		Kt(t);
		const { parent: n } = e, r = e.next;
		if (t.next = r, t.prev = e, e.next = t, t.parent = n, r) {
			if (r.prev = t, n) {
				const s = n.children;
				s.splice(s.lastIndexOf(r), 0, t);
			}
		} else n && n.children.push(t);
	}
	function Hc(e, t) {
		if (Kt(t), t.parent = e, t.prev = null, e.children.unshift(t) === 1) t.next = null;
		else {
			const n = e.children[1];
			n.prev = t, t.next = n;
		}
	}
	function Lc(e, t) {
		Kt(t);
		const { parent: n } = e;
		if (n) {
			const r = n.children;
			r.splice(r.indexOf(e), 0, t);
		}
		e.prev && (e.prev.next = t), t.parent = n, t.prev = e.prev, t.next = e, e.prev = t;
	}
	function Go(e) {
		return Re(e) ? e.children : [];
	}
	function Ho(e) {
		return e.parent || null;
	}
	function Oc(e) {
		const t = Ho(e);
		if (t != null) return Go(t);
		const n = [e];
		let { prev: r, next: s } = e;
		for (; r != null;) n.unshift(r), {prev: r} = r;
		for (; s != null;) n.push(s), {next: s} = s;
		return n;
	}
	function Kc(e, t) {
		const { attribs: n } = e;
		return n?.[t];
	}
	function Jc(e, t) {
		const { attribs: n } = e;
		return n != null && Object.hasOwn(n, t) && n[t] != null;
	}
	function Pc(e) {
		return e.name;
	}
	function Wc(e) {
		let { next: t } = e;
		for (; t !== null && !de(t);) ({next: t} = t);
		return t;
	}
	function Yc(e) {
		let { prev: t } = e;
		for (; t !== null && !de(t);) ({prev: t} = t);
		return t;
	}
	var Uc = Me({
		DocumentPosition: () => ge,
		append: () => Gc,
		appendChild: () => Mc,
		compareDocumentPosition: () => Mo,
		existsOne: () => Bo,
		filter: () => Ot,
		find: () => To,
		findAll: () => ic,
		findOne: () => jr,
		getAttributeValue: () => Kc,
		getChildren: () => Go,
		getElementById: () => lc,
		getElements: () => uc,
		getElementsByClassName: () => Ac,
		getElementsByTagName: () => yt,
		getElementsByTagType: () => cc,
		getFeed: () => Tc,
		getInnerHTML: () => xc,
		getName: () => Pc,
		getOuterHTML: () => Fo,
		getParent: () => Ho,
		getSiblings: () => Oc,
		getText: () => vn,
		hasAttrib: () => Jc,
		innerText: () => qr,
		nextElementSibling: () => Wc,
		prepend: () => Lc,
		prependChild: () => Hc,
		prevElementSibling: () => Yc,
		removeElement: () => Kt,
		removeSubsets: () => _c,
		replaceElement: () => kc,
		testElement: () => ac,
		textContent: () => En,
		uniqueSort: () => Fc
	});
	const Vc = /[-[\]{}()*+?.,\\^$|#\s]/g, Zc = /\s/;
	function Lo(e) {
		return e.replace(Vc, "\\$&");
	}
	const jc = /* @__PURE__ */ new Set([
		"accept",
		"accept-charset",
		"align",
		"alink",
		"axis",
		"bgcolor",
		"charset",
		"checked",
		"clear",
		"codetype",
		"color",
		"compact",
		"declare",
		"defer",
		"dir",
		"direction",
		"disabled",
		"enctype",
		"face",
		"frame",
		"hreflang",
		"http-equiv",
		"lang",
		"language",
		"link",
		"media",
		"method",
		"multiple",
		"nohref",
		"noresize",
		"noshade",
		"nowrap",
		"readonly",
		"rel",
		"rev",
		"rules",
		"scope",
		"scrolling",
		"selected",
		"shape",
		"target",
		"text",
		"type",
		"valign",
		"valuetype",
		"vlink"
	]);
	function ot(e, t) {
		return typeof e.ignoreCase == "boolean" ? e.ignoreCase : e.ignoreCase === "quirks" ? !!t.quirksMode : !t.xmlMode && jc.has(e.name);
	}
	const Xc = {
		equals(e, t, n) {
			const { adapter: r } = n, { name: s } = t;
			let { value: i } = t;
			return ot(t, n) ? (i = i.toLowerCase(), (o) => {
				const a = r.getAttributeValue(o, s);
				return a != null && a.length === i.length && a.toLowerCase() === i && e(o);
			}) : (o) => r.getAttributeValue(o, s) === i && e(o);
		},
		hyphen(e, t, n) {
			const { adapter: r } = n, { name: s } = t;
			let { value: i } = t;
			const { length: o } = i;
			return ot(t, n) ? (i = i.toLowerCase(), function(u) {
				const l = r.getAttributeValue(u, s);
				return l != null && (l.length === o || l.charAt(o) === "-") && l.substr(0, o).toLowerCase() === i && e(u);
			}) : function(u) {
				const l = r.getAttributeValue(u, s);
				return l != null && (l.length === o || l.charAt(o) === "-") && l.substr(0, o) === i && e(u);
			};
		},
		element(e, t, n) {
			const { adapter: r } = n, { name: s, value: i } = t;
			if (Zc.test(i)) return J;
			const o = new RegExp(`(?:^|\\s)${Lo(i)}(?:$|\\s)`, ot(t, n) ? "i" : "");
			return function(u) {
				const l = r.getAttributeValue(u, s);
				return l != null && l.length >= i.length && o.test(l) && e(u);
			};
		},
		exists(e, { name: t }, { adapter: n }) {
			return (r) => n.hasAttrib(r, t) && e(r);
		},
		start(e, t, n) {
			const { adapter: r } = n, { name: s } = t;
			let { value: i } = t;
			const { length: o } = i;
			return o === 0 ? J : ot(t, n) ? (i = i.toLowerCase(), (a) => {
				const u = r.getAttributeValue(a, s);
				return u != null && u.length >= o && u.substr(0, o).toLowerCase() === i && e(a);
			}) : (a) => !!r.getAttributeValue(a, s)?.startsWith(i) && e(a);
		},
		end(e, t, n) {
			const { adapter: r } = n, { name: s } = t;
			let { value: i } = t;
			const o = -i.length;
			return o === 0 ? J : ot(t, n) ? (i = i.toLowerCase(), (a) => r.getAttributeValue(a, s)?.substr(o).toLowerCase() === i && e(a)) : (a) => !!r.getAttributeValue(a, s)?.endsWith(i) && e(a);
		},
		any(e, t, n) {
			const { adapter: r } = n, { name: s, value: i } = t;
			if (i === "") return J;
			if (ot(t, n)) {
				const o = new RegExp(Lo(i), "i");
				return function(u) {
					const l = r.getAttributeValue(u, s);
					return l != null && l.length >= i.length && o.test(l) && e(u);
				};
			}
			return (o) => !!r.getAttributeValue(o, s)?.includes(i) && e(o);
		},
		not(e, t, n) {
			const { adapter: r } = n, { name: s } = t;
			let { value: i } = t;
			return i === "" ? (o) => !!r.getAttributeValue(o, s) && e(o) : ot(t, n) ? (i = i.toLowerCase(), (o) => {
				const a = r.getAttributeValue(o, s);
				return (a == null || a.length !== i.length || a.toLowerCase() !== i) && e(o);
			}) : (o) => r.getAttributeValue(o, s) !== i && e(o);
		}
	};
	function Tn(e, t, n) {
		const { adapter: r, xmlMode: s = !1 } = n, i = [t], o = [0];
		for (;;) {
			if (o[0] >= i[0].length) {
				if (i.length === 1) return null;
				i.shift(), o.shift();
				continue;
			}
			const a = i[0][o[0]++];
			if (r.isTag(a)) {
				if (e(a)) return a;
				if (s || r.getName(a) !== "template") {
					const u = r.getChildren(a);
					u.length > 0 && (i.unshift(u), o.unshift(0));
				}
			}
		}
	}
	function Oo(e, t) {
		const n = t.getSiblings(e);
		if (n.length <= 1) return [];
		const r = n.indexOf(e);
		return r === -1 || r === n.length - 1 ? [] : n.slice(r + 1).filter(t.isTag);
	}
	function We(e, t) {
		const n = t.getParent(e);
		return n != null && t.isTag(n) ? n : null;
	}
	const Ko = "input:is([type=text i],[type=search i],[type=url i],[type=tel i],[type=email i],[type=password i],[type=date i],[type=month i],[type=week i],[type=time i],[type=datetime-local i],[type=number i])", $c = {
		"any-link": ":is(a, area, link)[href]",
		link: ":any-link:not(:visited)",
		disabled: `:is(
        :is(button, input, select, textarea, optgroup, option)[disabled],
        optgroup[disabled] > option,
        fieldset[disabled]:not(fieldset[disabled] legend:first-of-type *)
    )`,
		enabled: ":is(button, input, select, textarea, optgroup, option, fieldset):not(:disabled)",
		checked: ":is(:is(input[type=radio], input[type=checkbox])[checked], :selected)",
		required: ":is(input, select, textarea)[required]",
		optional: ":is(input, select, textarea):not([required])",
		"read-only": `[readonly]:is(textarea, ${Ko})`,
		"read-write": `:not([readonly]):is(textarea, ${Ko})`,
		selected: "option:is([selected], select:not([multiple]):not(:has(> option[selected])) > :first-of-type)",
		checkbox: "[type=checkbox]",
		file: "[type=file]",
		password: "[type=password]",
		radio: "[type=radio]",
		reset: "[type=reset]",
		image: "[type=image]",
		submit: "[type=submit]",
		parent: ":not(:empty)",
		header: ":is(h1, h2, h3, h4, h5, h6)",
		button: ":is(button, input[type=button])",
		input: ":is(input, textarea, select, button)",
		text: "input:is(:not([type!='']), [type=text])"
	};
	function zc(e) {
		const t = e[0], n = e[1] - 1;
		if (n < 0 && t <= 0) return J;
		if (t === -1) return (i) => i <= n;
		if (t === 0) return (i) => i === n;
		if (t === 1) return n < 0 ? it : (i) => i >= n;
		const r = Math.abs(t), s = (n % r + r) % r;
		return t > 1 ? (i) => i >= n && i % r === s : (i) => i <= n && i % r === s;
	}
	const eh = /* @__PURE__ */ new Set([
		9,
		10,
		12,
		13,
		32
	]), Jo = 48, th = 57;
	function nh(e) {
		switch (e = e.trim().toLowerCase(), e) {
			case "even": return [2, 0];
			case "odd": return [2, 1];
		}
		let t = 0, n = 0, r = i(), s = o();
		if (t < e.length && e.charAt(t) === "n" && (t++, n = r * (s ?? 1), a(), t < e.length ? (r = i(), a(), s = o()) : r = s = 0), s === null || t < e.length) throw new Error(`n-th rule couldn't be parsed ('${e}')`);
		return [n, r * s];
		function i() {
			switch (e.charAt(t)) {
				case "-": return t++, -1;
				case "+": t++;
			}
			return 1;
		}
		function o() {
			const u = t;
			let l = 0;
			for (; t < e.length && e.charCodeAt(t) >= Jo && e.charCodeAt(t) <= th;) l = l * 10 + (e.charCodeAt(t) - Jo), t++;
			return t === u ? null : l;
		}
		function a() {
			for (; t < e.length && eh.has(e.charCodeAt(t));) t++;
		}
	}
	function rh(e) {
		return zc(nh(e));
	}
	function Bn(e, { adapter: t, cacheResults: n }, r) {
		if (n === !1 || typeof WeakMap > "u") return (o) => e(o) && r(o);
		const s = /* @__PURE__ */ new WeakMap();
		function i(o) {
			const a = r(o);
			return s.set(o, a), a;
		}
		return function(a) {
			if (!e(a)) return !1;
			if (s.has(a)) return s.get(a) ?? !1;
			let u = a;
			do {
				const l = We(u, t);
				if (l === null) return i(a);
				u = l;
			} while (!s.has(u));
			return s.get(u) ? i(a) : !1;
		};
	}
	function Qn(e) {
		const { context: t, rootFunc: n, ...r } = e;
		return r;
	}
	function sh(e, t) {
		if (t[0] !== "*" && t[0] !== e[0]) return !1;
		let n = 1;
		for (let r = 1; r < t.length; r++) if (t[r] !== "*") {
			for (; n < e.length && e[n] !== t[r];) if (e[n++].length <= 1) return !1;
			if (n >= e.length) return !1;
			n++;
		}
		return !0;
	}
	const ih = /^(.+?)\s+of\s+(.+)$/is;
	function Rn(e, t) {
		return function(r, s, i, o, a) {
			const { adapter: u, equals: l } = i, p = t ? null : s.match(ih), f = rh(p ? p[1].trim() : s);
			if (f === J) return J;
			const g = p && a ? a(Zr(p[2].trim()), Qn(i), o) : void 0;
			if (g === J) return J;
			if (f === it && !g) return (B) => We(B, u) !== null && r(B);
			const y = g ? (B, N) => g(N) : t ? (B, N) => u.getName(N) === u.getName(B) : it;
			return e ? function(N) {
				if (g && !g(N)) return !1;
				const z = u.getSiblings(N);
				let W = 0;
				for (let F = z.length - 1; F >= 0; F--) {
					const G = z[F];
					if (l(N, G)) break;
					u.isTag(G) && y(N, G) && W++;
				}
				return f(W) && r(N);
			} : function(N) {
				if (g && !g(N)) return !1;
				const z = u.getSiblings(N);
				let W = 0;
				for (const F of z) {
					if (l(N, F)) break;
					u.isTag(F) && y(N, F) && W++;
				}
				return f(W) && r(N);
			};
		};
	}
	const $r = {
		contains(e, t, n) {
			const { getText: r } = n.adapter;
			return Bn(e, n, (s) => r(s).includes(t));
		},
		icontains(e, t, n) {
			const r = t.toLowerCase(), { getText: s } = n.adapter;
			return Bn(e, n, (i) => s(i).toLowerCase().includes(r));
		},
		"nth-child": Rn(!1, !1),
		"nth-last-child": Rn(!0, !1),
		"nth-of-type": Rn(!1, !0),
		"nth-last-of-type": Rn(!0, !0),
		root(e, t, { adapter: n }) {
			return (r) => We(r, n) === null && e(r);
		},
		scope(e, t, n, r) {
			const { equals: s } = n;
			return !r || r.length === 0 ? $r.root(e, t, n) : r.length === 1 ? (i) => s(r[0], i) && e(i) : (i) => r.includes(i) && e(i);
		},
		lang(e, t, { adapter: n }) {
			const r = t.split(",").map((s) => s.trim()).filter((s) => s.length > 0).map((s) => s.replace(/^['"]|['"]$/g, "").toLowerCase().split("-"));
			return function(i) {
				let o = i;
				for (; o != null;) {
					const a = n.getAttributeValue(o, "xml:lang") ?? n.getAttributeValue(o, "lang");
					if (a != null) {
						if (!a) return r.some((p) => p[0] === "") && e(i);
						const l = a.toLowerCase().split("-");
						return r.some((p) => sh(l, p)) && e(i);
					}
					const u = n.getParent(o);
					o = u != null && n.isTag(u) ? u : null;
				}
				return r.some((a) => a[0] === "") && e(i);
			};
		},
		hover: zr("isHovered"),
		visited: zr("isVisited"),
		active: zr("isActive")
	};
	function zr(e) {
		return function(n, r, { adapter: s }) {
			const i = s[e];
			return typeof i != "function" ? J : function(a) {
				return i(a) && n(a);
			};
		};
	}
	const oh = /^[ \t\r\n]*$/, Po = {
		empty(e, { adapter: t }) {
			const n = t.getChildren(e);
			return n.every((r) => !t.isTag(r)) && n.every((r) => oh.test(t.getText(r)));
		},
		"first-child"(e, { adapter: t, equals: n }) {
			if (t.prevElementSibling) return t.prevElementSibling(e) == null;
			const r = t.getSiblings(e).find((s) => t.isTag(s));
			return r != null && n(e, r);
		},
		"last-child"(e, { adapter: t, equals: n }) {
			const r = t.getSiblings(e);
			for (let s = r.length - 1; s >= 0; s--) {
				if (n(e, r[s])) return !0;
				if (t.isTag(r[s])) break;
			}
			return !1;
		},
		"first-of-type"(e, { adapter: t, equals: n }) {
			const r = t.getSiblings(e), s = t.getName(e);
			for (const i of r) {
				if (n(e, i)) return !0;
				if (t.isTag(i) && t.getName(i) === s) break;
			}
			return !1;
		},
		"last-of-type"(e, { adapter: t, equals: n }) {
			const r = t.getSiblings(e), s = t.getName(e);
			for (let i = r.length - 1; i >= 0; i--) {
				const o = r[i];
				if (n(e, o)) return !0;
				if (t.isTag(o) && t.getName(o) === s) break;
			}
			return !1;
		},
		"only-of-type"(e, { adapter: t, equals: n }) {
			const r = t.getName(e);
			return t.getSiblings(e).every((s) => n(e, s) || !t.isTag(s) || t.getName(s) !== r);
		},
		"only-child"(e, { adapter: t, equals: n }) {
			return t.getSiblings(e).every((r) => n(e, r) || !t.isTag(r));
		}
	};
	function Wo(e, t, n, r) {
		if (n === null) {
			if (e.length > r) throw new Error(`Pseudo-class :${t} requires an argument`);
		} else if (e.length === r) throw new Error(`Pseudo-class :${t} doesn't have any arguments`);
	}
	function Nn(e) {
		return e.type === "_flexibleDescendant" || Io(e);
	}
	function ah(e) {
		const t = e.map(es);
		for (let n = 1; n < e.length; n++) {
			const r = t[n];
			if (!(r < 0)) for (let s = n; s > 0 && r < t[s - 1]; s--) {
				const i = e[s];
				e[s] = e[s - 1], e[s - 1] = i, t[s] = t[s - 1], t[s - 1] = r;
			}
		}
	}
	function uh(e) {
		switch (e.action) {
			case U.Exists: return 10;
			case U.Equals: return e.name === "id" ? 9 : 8;
			case U.Not: return 7;
			case U.Start: return 6;
			case U.End: return 6;
			case U.Any: return 5;
			case U.Hyphen: return 4;
			case U.Element: return 3;
		}
	}
	function es(e) {
		switch (e.type) {
			case x.Universal: return 50;
			case x.Tag: return 30;
			case x.Attribute: return Math.floor(uh(e) / (e.ignoreCase ? 2 : 1));
			case x.Pseudo: return e.data ? e.name === "has" || e.name === "contains" || e.name === "icontains" ? 0 : Array.isArray(e.data) ? Math.max(0, Math.min(...e.data.map((t) => Math.min(...t.map(es))))) : 2 : 3;
			default: return -1;
		}
	}
	function ts(e) {
		return e.type === x.Pseudo && (e.name === "scope" || Array.isArray(e.data) && e.data.some((t) => t.some(ts)));
	}
	const Yo = {};
	function lh(e) {
		return e.some((t) => t.length > 0 && (Nn(t[0]) || t.some(ts)));
	}
	const ns = (e, t, n, r, s) => {
		const i = s(t, Qn(n), r);
		return i === it ? e : i === J ? J : (o) => i(o) && e(o);
	}, rs = {
		is: ns,
		matches: ns,
		where: ns,
		not(e, t, n, r, s) {
			const i = s(t, Qn(n), r);
			return i === J ? e : i === it ? J : (o) => !i(o) && e(o);
		},
		has(e, t, n, r, s) {
			const { adapter: i } = n, o = Qn(n);
			o.relativeSelector = !0;
			const a = t.some((f) => f.some(Nn)) ? [Yo] : void 0, u = lh(t), l = s(t, o, a);
			if (l === J) return J;
			if (a && l !== it) return u ? (f) => {
				if (!e(f)) return !1;
				a[0] = f;
				const g = i.getChildren(f);
				return Tn(l, l.shouldTestNextSiblings ? [...g, ...Oo(f, i)] : g, n) !== null;
			} : Bn(e, n, (f) => (a[0] = f, Tn(l, i.getChildren(f), n) !== null));
			const p = (f) => Tn(l, i.getChildren(f), n) !== null;
			return u ? (f) => e(f) && p(f) : Bn(e, n, p);
		}
	};
	function Ah(e, t, n, r, s) {
		const { name: i, data: o } = t;
		if (Array.isArray(o)) {
			if (!(i in rs)) throw new Error(`Unknown pseudo-class :${i}(${o})`);
			return rs[i](e, o, n, r, s);
		}
		const a = n.pseudos?.[i], u = typeof a == "string" ? a : $c[i];
		if (typeof u == "string") {
			if (o != null) throw new Error(`Pseudo ${i} doesn't have any arguments`);
			const l = Zr(u);
			return rs.is(e, l, n, r, s);
		}
		if (typeof a == "function") return Wo(a, i, o, 1), (l) => a(l, o) && e(l);
		if (i in $r) return $r[i](e, o, n, r, s);
		if (i in Po) {
			const l = Po[i];
			return Wo(l, i, o, 2), (p) => l(p, n, o) && e(p);
		}
		throw new Error(`Unknown pseudo-class :${i}`);
	}
	function ch(e, t, n, r, s, i) {
		const { adapter: o, equals: a, cacheResults: u } = n;
		switch (t.type) {
			case x.PseudoElement: throw new Error("Pseudo-elements are not supported by css-select");
			case x.ColumnCombinator: throw new Error("Column combinators are not yet supported by css-select");
			case x.Attribute:
				if (t.namespace != null) throw new Error("Namespaced attributes are not yet supported by css-select");
				return (!n.xmlMode || n.lowerCaseAttributeNames) && (t.name = t.name.toLowerCase()), Xc[t.action](e, t, n);
			case x.Pseudo: return Ah(e, t, n, r, s);
			case x.Tag: {
				if (t.namespace != null) throw new Error("Namespaced tag names are not yet supported by css-select");
				let { name: l } = t;
				return (!n.xmlMode || n.lowerCaseTags) && (l = l.toLowerCase()), function(f) {
					return o.getName(f) === l && e(f);
				};
			}
			case x.Descendant: {
				if (!i || u === !1 || typeof WeakMap > "u") return function(f) {
					let g = f;
					for (; g = We(g, o);) if (e(g)) return !0;
					return !1;
				};
				const l = /* @__PURE__ */ new WeakMap();
				return function(f) {
					let g = f, y;
					for (; g = We(g, o);) {
						const B = l.get(g);
						if (B === void 0) {
							if (y ??= { matches: !1 }, y.matches = e(g), l.set(g, y), y.matches) return !0;
						} else return y && (y.matches = B.matches), B.matches;
					}
					return !1;
				};
			}
			case "_flexibleDescendant": return function(p) {
				let f = p;
				do {
					if (e(f)) return !0;
					f = We(f, o);
				} while (f);
				return !1;
			};
			case x.Parent: return function(p) {
				return o.getChildren(p).some((f) => o.isTag(f) && e(f));
			};
			case x.Child: return function(p) {
				const f = We(p, o);
				return f !== null && e(f);
			};
			case x.Sibling: return function(p) {
				const f = o.getSiblings(p);
				for (const g of f) {
					if (a(p, g)) break;
					if (o.isTag(g) && e(g)) return !0;
				}
				return !1;
			};
			case x.Adjacent: return o.prevElementSibling ? function(p) {
				const f = o.prevElementSibling(p);
				return f != null && e(f);
			} : function(p) {
				const f = o.getSiblings(p);
				let g;
				for (const y of f) {
					if (a(p, y)) break;
					o.isTag(y) && (g = y);
				}
				return !!g && e(g);
			};
			case x.Universal:
				if (t.namespace != null && t.namespace !== "*") throw new Error("Namespaced universal selectors are not yet supported by css-select");
				return e;
		}
	}
	const hh = { type: x.Descendant }, dh = { type: "_flexibleDescendant" }, gh = {
		type: x.Pseudo,
		name: "scope",
		data: null
	};
	function fh(e, { adapter: t }, n) {
		const r = !!n?.every((s) => s === Yo || t.isTag(s) && We(s, t) !== null);
		for (const s of e) {
			if (!(s.length > 0 && Nn(s[0]) && s[0].type !== x.Descendant)) if (r && !s.some(ts)) s.unshift(hh);
			else continue;
			s.unshift(gh);
		}
	}
	function Uo(e, t, n) {
		for (const l of e) ah(l);
		const { context: r = n, rootFunc: s = it } = t, i = Array.isArray(r), o = r && (Array.isArray(r) ? r : [r]);
		if (t.relativeSelector !== !1) fh(e, t, o);
		else if (e.some((l) => l.length > 0 && Nn(l[0]))) throw new Error("Relative selectors are not allowed when the `relativeSelector` option is disabled");
		let a = !1, u = J;
		e: for (const l of e) {
			if (l.length >= 2) {
				const [g, y] = l;
				g.type !== x.Pseudo || g.name !== "scope" || (i && y.type === x.Descendant ? l[1] = dh : (y.type === x.Adjacent || y.type === x.Sibling) && (a = !0));
			}
			let p = s, f = !1;
			for (const g of l) if (p = ch(p, g, t, o, Uo, f), es(g) === 0 && (f = !0), p === J) continue e;
			if (p === s) return s;
			u = u === J ? p : ph(u, p);
		}
		return u.shouldTestNextSiblings = a, u;
	}
	function ph(e, t) {
		return (n) => e(n) || t(n);
	}
	const Vo = (e, t) => e === t, Zo = {
		adapter: {
			...Uc,
			isTag: de
		},
		equals: Vo
	};
	function ss(e) {
		const t = e ?? Zo;
		return t.adapter ??= Zo.adapter, t.equals ??= t.adapter?.equals ?? Vo, t;
	}
	function jo(e, t, n) {
		const r = ss(t), s = Xo(e, r, n);
		return s === J ? J : (i) => r.adapter.isTag(i) && s(i);
	}
	function Xo(e, t, n) {
		return Uo(typeof e == "string" ? Zr(e) : e, ss(t), n);
	}
	function bh(e, t, n) {
		return (typeof t == "function" ? t : jo(t, n))(e);
	}
	const { isArray: yh } = Array, Dn = ({ nodeType: e }) => e === 1, $o = (e, t) => t.some((n) => Dn(n) && (e(n) || $o(e, wt(n)))), wh = (e, t) => t === "class" ? e.classList.value : e.getAttribute(t), wt = ({ childNodes: e }) => e, Ch = (e) => {
		const { localName: t } = e;
		return Oe(e) ? t.toLowerCase() : t;
	}, Ih = ({ parentNode: e }) => e, vh = (e) => {
		const { parentNode: t } = e;
		return t ? wt(t) : e;
	}, is = (e) => yh(e) ? e.map(is).join("") : Dn(e) ? is(wt(e)) : e.nodeType === 3 ? e.data : "", Eh = (e, t) => e.hasAttribute(t), xh = (e) => {
		let { length: t } = e;
		for (; t--;) {
			const n = e[t];
			if (t && -1 < e.lastIndexOf(n, t - 1)) {
				e.splice(t, 1);
				continue;
			}
			for (let { parentNode: r } = n; r; r = r.parentNode) if (e.includes(r)) {
				e.splice(t, 1);
				break;
			}
		}
		return e;
	}, zo = (e, t) => {
		const n = [];
		for (const r of t) Dn(r) && (e(r) && n.push(r), n.push(...zo(e, wt(r))));
		return n;
	}, ea = (e, t) => {
		for (let n of t) if (e(n) || (n = ea(e, wt(n)))) return n;
		return null;
	}, ta = {
		isTag: Dn,
		existsOne: $o,
		getAttributeValue: wh,
		getChildren: wt,
		getName: Ch,
		getParent: Ih,
		getSiblings: vh,
		getText: is,
		hasAttrib: Eh,
		removeSubsets: xh,
		findAll: zo,
		findOne: ea
	}, os = (e, t) => jo(t, {
		context: t.includes(":scope") ? e : void 0,
		xmlMode: !Oe(e),
		adapter: ta
	}), Th = (e, t) => bh(e, t, {
		strict: !0,
		context: t.includes(":scope") ? e : void 0,
		xmlMode: !Oe(e),
		adapter: ta
	});
	var Jt = class Du extends Gt {
		constructor(t, n = "") {
			super(t, "#text", 3, n);
		}
		get wholeText() {
			const t = [];
			let { previousSibling: n, nextSibling: r } = this;
			for (; n && n.nodeType === 3;) {
				t.unshift(n[M]);
				n = n.previousSibling;
			}
			for (t.push(this[M]); r && r.nodeType === 3;) {
				t.push(r[M]);
				r = r.nextSibling;
			}
			return t.join("");
		}
		cloneNode() {
			const { ownerDocument: t, [M]: n } = this;
			return new Du(t, n);
		}
		toString() {
			return Kr(this[M]);
		}
	};
	const Bh = (e) => e instanceof rt, as = (e, t, n) => {
		const { ownerDocument: r } = e;
		for (const s of n) e.insertBefore(Bh(s) ? s : new Jt(r, s), t);
	};
	var na = class extends rt {
		constructor(e, t, n) {
			super(e, t, n), this[ee] = null, this[w] = this[v] = {
				[w]: null,
				[X]: this,
				[se]: this,
				nodeType: -1,
				ownerDocument: this.ownerDocument,
				parentNode: null
			};
		}
		get childNodes() {
			const e = new Qe();
			let { firstChild: t } = this;
			for (; t;) e.push(t), t = st(t);
			return e;
		}
		get children() {
			const e = new Qe();
			let { firstElementChild: t } = this;
			for (; t;) e.push(t), t = Jr(t);
			return e;
		}
		get firstChild() {
			let { [w]: e, [v]: t } = this;
			for (; e.nodeType === 2;) e = e[w];
			return e === t ? null : e;
		}
		get firstElementChild() {
			let { firstChild: e } = this;
			for (; e;) {
				if (e.nodeType === 1) return e;
				e = st(e);
			}
			return null;
		}
		get lastChild() {
			const e = this[v][X];
			switch (e.nodeType) {
				case -1: return e[se];
				case 2: return null;
			}
			return e === this ? null : e;
		}
		get lastElementChild() {
			let { lastChild: e } = this;
			for (; e;) {
				if (e.nodeType === 1) return e;
				e = Mt(e);
			}
			return null;
		}
		get childElementCount() {
			return this.children.length;
		}
		prepend(...e) {
			as(this, this.firstChild, e);
		}
		append(...e) {
			as(this, this[v], e);
		}
		replaceChildren(...e) {
			let { [w]: t, [v]: n } = this;
			for (; t !== n && t.nodeType === 2;) t = t[w];
			for (; t !== n;) {
				const r = oe(t)[w];
				t.remove(), t = r;
			}
			e.length && as(this, n, e);
		}
		getElementsByClassName(e) {
			const t = new Qe();
			let { [w]: n, [v]: r } = this;
			for (; n !== r;) n.nodeType === 1 && n.hasAttribute("class") && n.classList.has(e) && t.push(n), n = n[w];
			return t;
		}
		getElementsByTagName(e) {
			const t = new Qe();
			let { [w]: n, [v]: r } = this;
			for (; n !== r;) n.nodeType === 1 && (n.localName === e || Mr(n) === e) && t.push(n), n = n[w];
			return t;
		}
		querySelector(e) {
			const t = os(this, e);
			let { [w]: n, [v]: r } = this;
			for (; n !== r;) {
				if (n.nodeType === 1 && t(n)) return n;
				n = n.nodeType === 1 && n.localName === "template" ? n[v] : n[w];
			}
			return null;
		}
		querySelectorAll(e) {
			const t = os(this, e), n = new Qe();
			let { [w]: r, [v]: s } = this;
			for (; r !== s;) r.nodeType === 1 && t(r) && n.push(r), r = r.nodeType === 1 && r.localName === "template" ? r[v] : r[w];
			return n;
		}
		appendChild(e) {
			return this.insertBefore(e, this[v]);
		}
		contains(e) {
			let t = e;
			for (; t && t !== this;) t = t.parentNode;
			return t === this;
		}
		insertBefore(e, t = null) {
			if (e === t) return e;
			if (e === this) throw new Error("unable to append a node to itself");
			const n = t || this[v];
			switch (e.nodeType) {
				case 1:
					e.remove(), e.parentNode = this, qi(n[X], e, n), Ft(e, null), Gr(e);
					break;
				case 11: {
					let { [ee]: r, firstChild: s, lastChild: i } = e;
					if (s) {
						$i(n[X], s, i, n), Ie(e, e[v]), r && r.replaceChildren();
						do
							s.parentNode = this, Ft(s, null), s.nodeType === 1 && Gr(s);
						while (s !== i && (s = st(s)));
					}
					break;
				}
				case 3:
				case 8:
				case 4: e.remove();
				default: e.parentNode = this, Sn(n[X], e, n), Ft(e, null);
			}
			return e;
		}
		normalize() {
			let { [w]: e, [v]: t } = this;
			for (; e !== t;) {
				const { [w]: n, [X]: r, nodeType: s } = e;
				s === 3 && (e[M] ? r && r.nodeType === 3 && (r.textContent += e.textContent, e.remove()) : e.remove()), e = n;
			}
		}
		removeChild(e) {
			if (e.parentNode !== this) throw new Error("node is not a child");
			return e.remove(), e;
		}
		replaceChild(e, t) {
			const n = oe(t)[w];
			return t.remove(), this.insertBefore(e, n), t;
		}
	}, us = class extends na {
		getElementById(e) {
			let { [w]: t, [v]: n } = this;
			for (; t !== n;) {
				if (t.nodeType === 1 && t.id === e) return t;
				t = t[w];
			}
			return null;
		}
		cloneNode(e) {
			const { ownerDocument: t, constructor: n } = this, r = new n(t);
			if (e) {
				const { [v]: s } = r;
				for (const i of this.childNodes) r.insertBefore(i.cloneNode(e), s);
			}
			return r;
		}
		toString() {
			const { childNodes: e, localName: t } = this;
			return `<${t}>${e.join("")}</${t}>`;
		}
		toJSON() {
			const e = [];
			return _A(this, e), e;
		}
	}, ls = class extends us {
		constructor(e) {
			super(e, "#document-fragment", 11);
		}
	}, _n = class _u extends rt {
		constructor(t, n, r = "", s = "") {
			super(t, "#document-type", 10), this.name = n, this.publicId = r, this.systemId = s;
		}
		cloneNode() {
			const { ownerDocument: t, name: n, publicId: r, systemId: s } = this;
			return new _u(t, n, r, s);
		}
		toString() {
			const { name: t, publicId: n, systemId: r } = this, s = 0 < n.length, i = [t];
			return s && i.push("PUBLIC", `"${n}"`), r.length && (s || i.push("SYSTEM"), i.push(`"${r}"`)), `<!DOCTYPE ${i.join(" ")}>`;
		}
		toJSON() {
			const t = [];
			return lo(this, t), t;
		}
	};
	const ra = (e) => e.childNodes.join(""), sa = (e, t) => {
		const { ownerDocument: n } = e, { constructor: r } = n, s = new r();
		s[be] = n[be];
		const { childNodes: i } = io(s, Oe(e), t);
		e.replaceChildren(...i.map(ia, n));
	};
	function ia(e) {
		switch (e.ownerDocument = this, e.nodeType) {
			case 1:
			case 11: e.childNodes.forEach(ia, this);
		}
		return e;
	}
	var Fn = (e) => e.replace(/(([A-Z0-9])([A-Z0-9][a-z]))|(([a-z0-9]+)([A-Z]))/g, "$2$5-$3$6").toLowerCase();
	const kn = /* @__PURE__ */ new WeakMap(), As = (e) => `data-${Fn(e)}`, Qh = (e) => e.slice(5).replace(/-([a-z])/g, (t, n) => n.toUpperCase()), Rh = {
		get(e, t) {
			if (t in e) return kn.get(e).getAttribute(As(t));
		},
		set(e, t, n) {
			return e[t] = n, kn.get(e).setAttribute(As(t), n), !0;
		},
		deleteProperty(e, t) {
			return t in e && kn.get(e).removeAttribute(As(t)), delete e[t];
		}
	};
	var oa = class {
		constructor(e) {
			for (const { name: t, value: n } of e.attributes) /^data-/.test(t) && (this[Qh(t)] = n);
			return kn.set(this, e), new Proxy(this, Rh);
		}
	};
	re(oa.prototype, null);
	const { add: Nh } = Set.prototype, aa = (e, t) => {
		for (const n of t) n && Nh.call(e, n);
	}, Pt = ({ [li]: e, value: t }) => {
		const n = e.getAttributeNode("class");
		n ? n.value = t : Lr(e, new kt(e.ownerDocument, "class", t));
	};
	var Dh = class extends Set {
		constructor(e) {
			super(), this[li] = e;
			const t = e.getAttributeNode("class");
			t && aa(this, t.value.split(/\s+/));
		}
		get length() {
			return this.size;
		}
		get value() {
			return [...this].join(" ");
		}
		add(...e) {
			aa(this, e), Pt(this);
		}
		contains(e) {
			return this.has(e);
		}
		remove(...e) {
			for (const t of e) this.delete(t);
			Pt(this);
		}
		toggle(e, t) {
			if (this.has(e)) {
				if (t) return !0;
				this.delete(e), Pt(this);
			} else if (t || arguments.length === 1) return super.add(e), Pt(this), !0;
			return !1;
		}
		replace(e, t) {
			return this.has(e) ? (this.delete(e), super.add(t), Pt(this), !0) : !1;
		}
		supports() {
			return !0;
		}
	};
	const Mn = /* @__PURE__ */ new WeakMap(), cs = (e) => [...e.keys()].filter((t) => t !== ee), Gn = (e) => {
		const t = Mn.get(e).getAttributeNode("style");
		if ((!t || t[un] || e.get(ee) !== t) && (e.clear(), t)) {
			e.set(ee, t);
			for (const n of t[M].split(/\s*;\s*/)) {
				let [r, ...s] = n.split(":");
				if (s.length > 0) {
					r = r.trim();
					const i = s.join(":").trim();
					r && i && e.set(r, i);
				}
			}
		}
		return t;
	}, Hn = {
		get(e, t) {
			return t in _h ? e[t] : (Gn(e), t === "length" ? cs(e).length : /^\d+$/.test(t) ? cs(e)[t] : e.get(Fn(t)) ?? "");
		},
		set(e, t, n) {
			if (t === "cssText") e[t] = n;
			else {
				let r = Gn(e);
				if (n == null ? e.delete(Fn(t)) : e.set(Fn(t), n), !r) {
					const s = Mn.get(e);
					r = s.ownerDocument.createAttribute("style"), s.setAttributeNode(r), e.set(ee, r);
				}
				r[un] = !1, r[M] = e.toString();
			}
			return !0;
		}
	};
	var ua = class extends Map {
		constructor(e) {
			return super(), Mn.set(this, e), new Proxy(this, Hn);
		}
		get cssText() {
			return this.toString();
		}
		set cssText(e) {
			Mn.get(this).setAttribute("style", e);
		}
		getPropertyValue(e) {
			const t = this[ee];
			return Hn.get(t, e);
		}
		setProperty(e, t) {
			const n = this[ee];
			Hn.set(n, e, t);
		}
		removeProperty(e) {
			const t = this[ee];
			Hn.set(t, e, null);
		}
		[Symbol.iterator]() {
			const e = this[ee];
			Gn(e);
			const t = cs(e), { length: n } = t;
			let r = 0;
			return { next() {
				const s = r === n;
				return {
					done: s,
					value: s ? null : t[r++]
				};
			} };
		}
		get [ee]() {
			return this;
		}
		toString() {
			const e = this[ee];
			Gn(e);
			const t = [];
			return e.forEach(Fh, t), t.join(";");
		}
	};
	const { prototype: _h } = ua;
	function Fh(e, t) {
		t !== ee && this.push(`${t}:${e}`);
	}
	const la = 3, Aa = 2, ca = 1, ha = 0;
	function kh(e) {
		return e.currentTarget;
	}
	var at = class {
		static get BUBBLING_PHASE() {
			return la;
		}
		static get AT_TARGET() {
			return Aa;
		}
		static get CAPTURING_PHASE() {
			return ca;
		}
		static get NONE() {
			return ha;
		}
		constructor(e, t = {}) {
			this.type = e, this.bubbles = !!t.bubbles, this.cancelBubble = !1, this._stopImmediatePropagationFlag = !1, this.cancelable = !!t.cancelable, this.eventPhase = this.NONE, this.timeStamp = Date.now(), this.defaultPrevented = !1, this.originalTarget = null, this.returnValue = null, this.srcElement = null, this.target = null, this._path = [];
		}
		get BUBBLING_PHASE() {
			return la;
		}
		get AT_TARGET() {
			return Aa;
		}
		get CAPTURING_PHASE() {
			return ca;
		}
		get NONE() {
			return ha;
		}
		preventDefault() {
			this.defaultPrevented = !0;
		}
		composedPath() {
			return this._path.map(kh);
		}
		stopPropagation() {
			this.cancelBubble = !0;
		}
		stopImmediatePropagation() {
			this.stopPropagation(), this._stopImmediatePropagationFlag = !0;
		}
	}, da = class extends Array {
		constructor(e) {
			super(), this.ownerElement = e;
		}
		getNamedItem(e) {
			return this.ownerElement.getAttributeNode(e);
		}
		setNamedItem(e) {
			this.ownerElement.setAttributeNode(e), this.unshift(e);
		}
		removeNamedItem(e) {
			const t = this.getNamedItem(e);
			this.ownerElement.removeAttribute(e), this.splice(this.indexOf(t), 1);
		}
		item(e) {
			return e < this.length ? this[e] : null;
		}
		getNamedItemNS(e, t) {
			return this.getNamedItem(t);
		}
		setNamedItemNS(e, t) {
			return this.setNamedItem(t);
		}
		removeNamedItemNS(e, t) {
			return this.removeNamedItem(t);
		}
	}, hs = class extends us {
		constructor(e) {
			super(e.ownerDocument, "#shadow-root", 11), this.host = e;
		}
		get innerHTML() {
			return ra(this);
		}
		set innerHTML(e) {
			sa(this, e);
		}
	};
	const Mh = { get(e, t) {
		return t in e ? e[t] : e.find(({ name: n }) => n === t);
	} }, ga = (e, t, n) => {
		if ("ownerSVGElement" in t) {
			const r = e.createElementNS(kr, n);
			return r.ownerSVGElement = t.ownerSVGElement, r;
		}
		return e.createElement(n);
	}, Gh = ({ localName: e, ownerDocument: t }) => t[ft].voidElements.test(e);
	var Wt = class extends na {
		constructor(e, t) {
			super(e, t, 1), this[gt] = null, this[Sr] = null, this[yr] = null;
		}
		get isConnected() {
			return po(this);
		}
		get parentElement() {
			return mo(this);
		}
		get previousSibling() {
			return Mt(this);
		}
		get nextSibling() {
			return st(this);
		}
		get namespaceURI() {
			return "http://www.w3.org/1999/xhtml";
		}
		get previousElementSibling() {
			return So(this);
		}
		get nextElementSibling() {
			return Jr(this);
		}
		before(...e) {
			bo(this, e);
		}
		after(...e) {
			yo(this, e);
		}
		replaceWith(...e) {
			Wr(this, e);
		}
		remove() {
			wo(this[X], this, this[v][w]);
		}
		get id() {
			return S.get(this, "id");
		}
		set id(e) {
			S.set(this, "id", e);
		}
		get className() {
			return this.classList.value;
		}
		set className(e) {
			const { classList: t } = this;
			t.clear(), t.add(...et(e).split(/\s+/));
		}
		get nodeName() {
			return Mr(this);
		}
		get tagName() {
			return Mr(this);
		}
		get classList() {
			return this[gt] || (this[gt] = new Dh(this));
		}
		get dataset() {
			return this[Sr] || (this[Sr] = new oa(this));
		}
		getBoundingClientRect() {
			return {
				x: 0,
				y: 0,
				bottom: 0,
				height: 0,
				left: 0,
				right: 0,
				top: 0,
				width: 0
			};
		}
		get nonce() {
			return S.get(this, "nonce");
		}
		set nonce(e) {
			S.set(this, "nonce", e);
		}
		get style() {
			return this[yr] || (this[yr] = new ua(this));
		}
		get tabIndex() {
			return Je.get(this, "tabindex") || -1;
		}
		set tabIndex(e) {
			Je.set(this, "tabindex", e);
		}
		get slot() {
			return S.get(this, "slot");
		}
		set slot(e) {
			S.set(this, "slot", e);
		}
		get innerText() {
			const e = [];
			let { [w]: t, [v]: n } = this;
			for (; t !== n;) t.nodeType === 3 ? e.push(t.textContent.replace(/\s+/g, " ")) : e.length && t[w] != n && IA.has(t.tagName) && e.push(`
`), t = t[w];
			return e.join("");
		}
		get textContent() {
			const e = [];
			let { [w]: t, [v]: n } = this;
			for (; t !== n;) {
				const r = t.nodeType;
				(r === 3 || r === 4) && e.push(t.textContent), t = t[w];
			}
			return e.join("");
		}
		set textContent(e) {
			this.replaceChildren(), e != null && e !== "" && this.appendChild(new Jt(this.ownerDocument, e));
		}
		get innerHTML() {
			return ra(this);
		}
		set innerHTML(e) {
			sa(this, e);
		}
		get outerHTML() {
			return this.toString();
		}
		set outerHTML(e) {
			const t = this.ownerDocument.createElement("");
			t.innerHTML = e, this.replaceWith(...t.childNodes);
		}
		get attributes() {
			const e = new da(this);
			let t = this[w];
			for (; t.nodeType === 2;) e.push(t), t = t[w];
			return new Proxy(e, Mh);
		}
		focus() {
			this.dispatchEvent(new at("focus"));
		}
		getAttribute(e) {
			if (e === "class") return this.className;
			const t = this.getAttributeNode(e);
			return t && (Oe(this) ? t.value : Kr(t.value));
		}
		getAttributeNode(e) {
			let t = this[w];
			for (; t.nodeType === 2;) {
				if (t.name === e) return t;
				t = t[w];
			}
			return null;
		}
		getAttributeNames() {
			const e = new Qe();
			let t = this[w];
			for (; t.nodeType === 2;) e.push(t.name), t = t[w];
			return e;
		}
		hasAttribute(e) {
			return !!this.getAttributeNode(e);
		}
		hasAttributes() {
			return this[w].nodeType === 2;
		}
		removeAttribute(e) {
			e === "class" && this[gt] && this[gt].clear();
			let t = this[w];
			for (; t.nodeType === 2;) {
				if (t.name === e) {
					go(this, t);
					return;
				}
				t = t[w];
			}
		}
		removeAttributeNode(e) {
			let t = this[w];
			for (; t.nodeType === 2;) {
				if (t === e) {
					go(this, t);
					return;
				}
				t = t[w];
			}
		}
		setAttribute(e, t) {
			if (e === "class") this.className = t;
			else {
				const n = this.getAttributeNode(e);
				n ? n.value = t : Lr(this, new kt(this.ownerDocument, e, t));
			}
		}
		setAttributeNode(e) {
			const { name: t } = e, n = this.getAttributeNode(t);
			if (n !== e) {
				n && this.removeAttributeNode(n);
				const { ownerElement: r } = e;
				r && r.removeAttributeNode(e), Lr(this, e);
			}
			return n;
		}
		toggleAttribute(e, t) {
			return this.hasAttribute(e) ? t ? !0 : (this.removeAttribute(e), !1) : t || arguments.length === 1 ? (this.setAttribute(e, ""), !0) : !1;
		}
		get shadowRoot() {
			if (Ke.has(this)) {
				const { mode: e, shadowRoot: t } = Ke.get(this);
				if (e === "open") return t;
			}
			return null;
		}
		attachShadow(e) {
			if (Ke.has(this)) throw new Error("operation not supported");
			const t = new hs(this);
			return Ke.set(this, {
				mode: e.mode,
				shadowRoot: t
			}), t;
		}
		matches(e) {
			return Th(this, e);
		}
		closest(e) {
			let t = this;
			const n = os(t, e);
			for (; t && !n(t);) t = t.parentElement;
			return t;
		}
		insertAdjacentElement(e, t) {
			const { parentElement: n } = this;
			switch (e) {
				case "beforebegin":
					if (n) {
						n.insertBefore(t, this);
						break;
					}
					return null;
				case "afterbegin":
					this.insertBefore(t, this.firstChild);
					break;
				case "beforeend":
					this.insertBefore(t, null);
					break;
				case "afterend":
					if (n) {
						n.insertBefore(t, this.nextSibling);
						break;
					}
					return null;
			}
			return t;
		}
		insertAdjacentHTML(e, t) {
			this.insertAdjacentElement(e, eo(this.ownerDocument, t));
		}
		insertAdjacentText(e, t) {
			const n = this.ownerDocument.createTextNode(t);
			this.insertAdjacentElement(e, n);
		}
		cloneNode(e = !1) {
			const { ownerDocument: t, localName: n } = this, r = (l) => {
				l.parentNode = i, Ie(o, l), o = l;
			}, s = ga(t, this, n);
			let i = s, o = s, { [w]: a, [v]: u } = this;
			for (; a !== u && (e || a.nodeType === 2);) {
				switch (a.nodeType) {
					case -1:
						Ie(o, i[v]), o = i[v], i = i.parentNode;
						break;
					case 1: {
						const l = ga(t, a, a.localName);
						r(l), i = l;
						break;
					}
					case 2: {
						const l = a.cloneNode(e);
						l.ownerElement = i, r(l);
						break;
					}
					case 3:
					case 8:
					case 4: r(a.cloneNode(e));
				}
				a = a[w];
			}
			return Ie(o, s[v]), s;
		}
		toString() {
			const e = [], { [v]: t } = this;
			let n = { [w]: this }, r = !1;
			do
				switch (n = n[w], n.nodeType) {
					case 2: {
						const s = " " + n;
						switch (s) {
							case " id":
							case " class":
							case " style": break;
							default: e.push(s);
						}
						break;
					}
					case -1: {
						const s = n[se];
						r ? ("ownerSVGElement" in s ? e.push(" />") : Gh(s) ? e.push(Oe(s) ? ">" : " />") : e.push(`></${s.localName}>`), r = !1) : e.push(`</${s.localName}>`);
						break;
					}
					case 1:
						r && e.push(">"), n.toString !== this.toString ? (e.push(n.toString()), n = n[v], r = !1) : (e.push(`<${n.localName}`), r = !0);
						break;
					case 3:
					case 8:
					case 4: e.push((r ? ">" : "") + n), r = !1;
				}
			while (n !== t);
			return e.join("");
		}
		toJSON() {
			const e = [];
			return Ao(this, e), e;
		}
		getAttributeNS(e, t) {
			return this.getAttribute(t);
		}
		getElementsByTagNameNS(e, t) {
			return this.getElementsByTagName(t);
		}
		hasAttributeNS(e, t) {
			return this.hasAttribute(t);
		}
		removeAttributeNS(e, t) {
			this.removeAttribute(t);
		}
		setAttributeNS(e, t, n) {
			this.setAttribute(t, n);
		}
		setAttributeNodeNS(e) {
			return this.setAttributeNode(e);
		}
	};
	const ds = /* @__PURE__ */ new WeakMap(), Hh = {
		get(e, t) {
			return e[t];
		},
		set(e, t, n) {
			return e[t] = n, !0;
		}
	};
	var Yt = class extends Wt {
		constructor(e, t, n = null) {
			super(e, t), this.ownerSVGElement = n;
		}
		get className() {
			return ds.has(this) || ds.set(this, new Proxy({
				baseVal: "",
				animVal: ""
			}, Hh)), ds.get(this);
		}
		set className(e) {
			const { classList: t } = this;
			t.clear(), t.add(...et(e).split(/\s+/));
		}
		get namespaceURI() {
			return "http://www.w3.org/2000/svg";
		}
		getAttribute(e) {
			return e === "class" ? [...this.classList].join(" ") : super.getAttribute(e);
		}
		setAttribute(e, t) {
			if (e === "class") this.className = t;
			else if (e === "style") {
				const { className: n } = this;
				n.baseVal = n.animVal = t;
			}
			super.setAttribute(e, t);
		}
	};
	const le = () => {
		throw new TypeError("Illegal constructor");
	};
	function gs() {
		le();
	}
	re(gs, kt), gs.prototype = kt.prototype;
	function fs() {
		le();
	}
	re(fs, Yr), fs.prototype = Yr.prototype;
	function ps() {
		le();
	}
	re(ps, Gt), ps.prototype = Gt.prototype;
	function ms() {
		le();
	}
	re(ms, Ur), ms.prototype = Ur.prototype;
	function Ss() {
		le();
	}
	re(Ss, ls), Ss.prototype = ls.prototype;
	function bs() {
		le();
	}
	re(bs, _n), bs.prototype = _n.prototype;
	function ys() {
		le();
	}
	re(ys, Wt), ys.prototype = Wt.prototype;
	function ws() {
		le();
	}
	re(ws, rt), ws.prototype = rt.prototype;
	function Cs() {
		le();
	}
	re(Cs, hs), Cs.prototype = hs.prototype;
	function Is() {
		le();
	}
	re(Is, Jt), Is.prototype = Jt.prototype;
	function vs() {
		le();
	}
	re(vs, Yt), vs.prototype = Yt.prototype;
	const Lh = {
		Attr: gs,
		CDATASection: fs,
		CharacterData: ps,
		Comment: ms,
		DocumentFragment: Ss,
		DocumentType: bs,
		Element: ys,
		Node: ws,
		ShadowRoot: Cs,
		Text: Is,
		SVGElement: vs
	}, Ut = /* @__PURE__ */ new WeakMap(), d = {
		get(e, t) {
			return Ut.has(e) && Ut.get(e)[t] || null;
		},
		set(e, t, n) {
			Ut.has(e) || Ut.set(e, {});
			const r = Ut.get(e), s = t.slice(2);
			r[t] && e.removeEventListener(s, r[t], !1), (r[t] = n) && e.addEventListener(s, n, !1);
		}
	};
	var C = class extends Wt {
		static get observedAttributes() {
			return [];
		}
		constructor(e = null, t = "") {
			super(e, t);
			const n = !e;
			let r;
			if (n) {
				const { constructor: s } = this;
				if (!bt.has(s)) throw new Error("unable to initialize this Custom Element");
				({ownerDocument: e, localName: t, options: r} = bt.get(s));
			}
			if (e[Rt]) {
				const { element: s, values: i } = e[Rt];
				e[Rt] = null;
				for (const [o, a] of i) s[o] = a;
				return s;
			}
			n && (this.ownerDocument = this[v].ownerDocument = e, this.localName = t, tt.set(this, { connected: !1 }), r.is && this.setAttribute("is", r.is));
		}
		blur() {
			this.dispatchEvent(new at("blur"));
		}
		click() {
			const e = new at("click", {
				bubbles: !0,
				cancelable: !0
			});
			e.button = 0, this.dispatchEvent(e);
		}
		get accessKeyLabel() {
			const { accessKey: e } = this;
			return e && `Alt+Shift+${e}`;
		}
		get isContentEditable() {
			return this.hasAttribute("contenteditable");
		}
		get contentEditable() {
			return _.get(this, "contenteditable");
		}
		set contentEditable(e) {
			_.set(this, "contenteditable", e);
		}
		get draggable() {
			return _.get(this, "draggable");
		}
		set draggable(e) {
			_.set(this, "draggable", e);
		}
		get hidden() {
			return _.get(this, "hidden");
		}
		set hidden(e) {
			_.set(this, "hidden", e);
		}
		get spellcheck() {
			return _.get(this, "spellcheck");
		}
		set spellcheck(e) {
			_.set(this, "spellcheck", e);
		}
		get accessKey() {
			return S.get(this, "accesskey");
		}
		set accessKey(e) {
			S.set(this, "accesskey", e);
		}
		get dir() {
			return S.get(this, "dir");
		}
		set dir(e) {
			S.set(this, "dir", e);
		}
		get lang() {
			return S.get(this, "lang");
		}
		set lang(e) {
			S.set(this, "lang", e);
		}
		get title() {
			return S.get(this, "title");
		}
		set title(e) {
			S.set(this, "title", e);
		}
		get onabort() {
			return d.get(this, "onabort");
		}
		set onabort(e) {
			d.set(this, "onabort", e);
		}
		get onblur() {
			return d.get(this, "onblur");
		}
		set onblur(e) {
			d.set(this, "onblur", e);
		}
		get oncancel() {
			return d.get(this, "oncancel");
		}
		set oncancel(e) {
			d.set(this, "oncancel", e);
		}
		get oncanplay() {
			return d.get(this, "oncanplay");
		}
		set oncanplay(e) {
			d.set(this, "oncanplay", e);
		}
		get oncanplaythrough() {
			return d.get(this, "oncanplaythrough");
		}
		set oncanplaythrough(e) {
			d.set(this, "oncanplaythrough", e);
		}
		get onchange() {
			return d.get(this, "onchange");
		}
		set onchange(e) {
			d.set(this, "onchange", e);
		}
		get onclick() {
			return d.get(this, "onclick");
		}
		set onclick(e) {
			d.set(this, "onclick", e);
		}
		get onclose() {
			return d.get(this, "onclose");
		}
		set onclose(e) {
			d.set(this, "onclose", e);
		}
		get oncontextmenu() {
			return d.get(this, "oncontextmenu");
		}
		set oncontextmenu(e) {
			d.set(this, "oncontextmenu", e);
		}
		get oncuechange() {
			return d.get(this, "oncuechange");
		}
		set oncuechange(e) {
			d.set(this, "oncuechange", e);
		}
		get ondblclick() {
			return d.get(this, "ondblclick");
		}
		set ondblclick(e) {
			d.set(this, "ondblclick", e);
		}
		get ondrag() {
			return d.get(this, "ondrag");
		}
		set ondrag(e) {
			d.set(this, "ondrag", e);
		}
		get ondragend() {
			return d.get(this, "ondragend");
		}
		set ondragend(e) {
			d.set(this, "ondragend", e);
		}
		get ondragenter() {
			return d.get(this, "ondragenter");
		}
		set ondragenter(e) {
			d.set(this, "ondragenter", e);
		}
		get ondragleave() {
			return d.get(this, "ondragleave");
		}
		set ondragleave(e) {
			d.set(this, "ondragleave", e);
		}
		get ondragover() {
			return d.get(this, "ondragover");
		}
		set ondragover(e) {
			d.set(this, "ondragover", e);
		}
		get ondragstart() {
			return d.get(this, "ondragstart");
		}
		set ondragstart(e) {
			d.set(this, "ondragstart", e);
		}
		get ondrop() {
			return d.get(this, "ondrop");
		}
		set ondrop(e) {
			d.set(this, "ondrop", e);
		}
		get ondurationchange() {
			return d.get(this, "ondurationchange");
		}
		set ondurationchange(e) {
			d.set(this, "ondurationchange", e);
		}
		get onemptied() {
			return d.get(this, "onemptied");
		}
		set onemptied(e) {
			d.set(this, "onemptied", e);
		}
		get onended() {
			return d.get(this, "onended");
		}
		set onended(e) {
			d.set(this, "onended", e);
		}
		get onerror() {
			return d.get(this, "onerror");
		}
		set onerror(e) {
			d.set(this, "onerror", e);
		}
		get onfocus() {
			return d.get(this, "onfocus");
		}
		set onfocus(e) {
			d.set(this, "onfocus", e);
		}
		get oninput() {
			return d.get(this, "oninput");
		}
		set oninput(e) {
			d.set(this, "oninput", e);
		}
		get oninvalid() {
			return d.get(this, "oninvalid");
		}
		set oninvalid(e) {
			d.set(this, "oninvalid", e);
		}
		get onkeydown() {
			return d.get(this, "onkeydown");
		}
		set onkeydown(e) {
			d.set(this, "onkeydown", e);
		}
		get onkeypress() {
			return d.get(this, "onkeypress");
		}
		set onkeypress(e) {
			d.set(this, "onkeypress", e);
		}
		get onkeyup() {
			return d.get(this, "onkeyup");
		}
		set onkeyup(e) {
			d.set(this, "onkeyup", e);
		}
		get onload() {
			return d.get(this, "onload");
		}
		set onload(e) {
			d.set(this, "onload", e);
		}
		get onloadeddata() {
			return d.get(this, "onloadeddata");
		}
		set onloadeddata(e) {
			d.set(this, "onloadeddata", e);
		}
		get onloadedmetadata() {
			return d.get(this, "onloadedmetadata");
		}
		set onloadedmetadata(e) {
			d.set(this, "onloadedmetadata", e);
		}
		get onloadstart() {
			return d.get(this, "onloadstart");
		}
		set onloadstart(e) {
			d.set(this, "onloadstart", e);
		}
		get onmousedown() {
			return d.get(this, "onmousedown");
		}
		set onmousedown(e) {
			d.set(this, "onmousedown", e);
		}
		get onmouseenter() {
			return d.get(this, "onmouseenter");
		}
		set onmouseenter(e) {
			d.set(this, "onmouseenter", e);
		}
		get onmouseleave() {
			return d.get(this, "onmouseleave");
		}
		set onmouseleave(e) {
			d.set(this, "onmouseleave", e);
		}
		get onmousemove() {
			return d.get(this, "onmousemove");
		}
		set onmousemove(e) {
			d.set(this, "onmousemove", e);
		}
		get onmouseout() {
			return d.get(this, "onmouseout");
		}
		set onmouseout(e) {
			d.set(this, "onmouseout", e);
		}
		get onmouseover() {
			return d.get(this, "onmouseover");
		}
		set onmouseover(e) {
			d.set(this, "onmouseover", e);
		}
		get onmouseup() {
			return d.get(this, "onmouseup");
		}
		set onmouseup(e) {
			d.set(this, "onmouseup", e);
		}
		get onmousewheel() {
			return d.get(this, "onmousewheel");
		}
		set onmousewheel(e) {
			d.set(this, "onmousewheel", e);
		}
		get onpause() {
			return d.get(this, "onpause");
		}
		set onpause(e) {
			d.set(this, "onpause", e);
		}
		get onplay() {
			return d.get(this, "onplay");
		}
		set onplay(e) {
			d.set(this, "onplay", e);
		}
		get onplaying() {
			return d.get(this, "onplaying");
		}
		set onplaying(e) {
			d.set(this, "onplaying", e);
		}
		get onprogress() {
			return d.get(this, "onprogress");
		}
		set onprogress(e) {
			d.set(this, "onprogress", e);
		}
		get onratechange() {
			return d.get(this, "onratechange");
		}
		set onratechange(e) {
			d.set(this, "onratechange", e);
		}
		get onreset() {
			return d.get(this, "onreset");
		}
		set onreset(e) {
			d.set(this, "onreset", e);
		}
		get onresize() {
			return d.get(this, "onresize");
		}
		set onresize(e) {
			d.set(this, "onresize", e);
		}
		get onscroll() {
			return d.get(this, "onscroll");
		}
		set onscroll(e) {
			d.set(this, "onscroll", e);
		}
		get onseeked() {
			return d.get(this, "onseeked");
		}
		set onseeked(e) {
			d.set(this, "onseeked", e);
		}
		get onseeking() {
			return d.get(this, "onseeking");
		}
		set onseeking(e) {
			d.set(this, "onseeking", e);
		}
		get onselect() {
			return d.get(this, "onselect");
		}
		set onselect(e) {
			d.set(this, "onselect", e);
		}
		get onshow() {
			return d.get(this, "onshow");
		}
		set onshow(e) {
			d.set(this, "onshow", e);
		}
		get onstalled() {
			return d.get(this, "onstalled");
		}
		set onstalled(e) {
			d.set(this, "onstalled", e);
		}
		get onsubmit() {
			return d.get(this, "onsubmit");
		}
		set onsubmit(e) {
			d.set(this, "onsubmit", e);
		}
		get onsuspend() {
			return d.get(this, "onsuspend");
		}
		set onsuspend(e) {
			d.set(this, "onsuspend", e);
		}
		get ontimeupdate() {
			return d.get(this, "ontimeupdate");
		}
		set ontimeupdate(e) {
			d.set(this, "ontimeupdate", e);
		}
		get ontoggle() {
			return d.get(this, "ontoggle");
		}
		set ontoggle(e) {
			d.set(this, "ontoggle", e);
		}
		get onvolumechange() {
			return d.get(this, "onvolumechange");
		}
		set onvolumechange(e) {
			d.set(this, "onvolumechange", e);
		}
		get onwaiting() {
			return d.get(this, "onwaiting");
		}
		set onwaiting(e) {
			d.set(this, "onwaiting", e);
		}
		get onauxclick() {
			return d.get(this, "onauxclick");
		}
		set onauxclick(e) {
			d.set(this, "onauxclick", e);
		}
		get ongotpointercapture() {
			return d.get(this, "ongotpointercapture");
		}
		set ongotpointercapture(e) {
			d.set(this, "ongotpointercapture", e);
		}
		get onlostpointercapture() {
			return d.get(this, "onlostpointercapture");
		}
		set onlostpointercapture(e) {
			d.set(this, "onlostpointercapture", e);
		}
		get onpointercancel() {
			return d.get(this, "onpointercancel");
		}
		set onpointercancel(e) {
			d.set(this, "onpointercancel", e);
		}
		get onpointerdown() {
			return d.get(this, "onpointerdown");
		}
		set onpointerdown(e) {
			d.set(this, "onpointerdown", e);
		}
		get onpointerenter() {
			return d.get(this, "onpointerenter");
		}
		set onpointerenter(e) {
			d.set(this, "onpointerenter", e);
		}
		get onpointerleave() {
			return d.get(this, "onpointerleave");
		}
		set onpointerleave(e) {
			d.set(this, "onpointerleave", e);
		}
		get onpointermove() {
			return d.get(this, "onpointermove");
		}
		set onpointermove(e) {
			d.set(this, "onpointermove", e);
		}
		get onpointerout() {
			return d.get(this, "onpointerout");
		}
		set onpointerout(e) {
			d.set(this, "onpointerout", e);
		}
		get onpointerover() {
			return d.get(this, "onpointerover");
		}
		set onpointerover(e) {
			d.set(this, "onpointerover", e);
		}
		get onpointerup() {
			return d.get(this, "onpointerup");
		}
		set onpointerup(e) {
			d.set(this, "onpointerup", e);
		}
	};
	const fa = "template";
	var pa = class extends C {
		constructor(e) {
			super(e, fa);
			const t = this.ownerDocument.createDocumentFragment();
			(this[ln] = t)[ee] = this;
		}
		get content() {
			if (this.hasChildNodes() && !this[ln].hasChildNodes()) for (const e of this.childNodes) this[ln].appendChild(e.cloneNode(!0));
			return this[ln];
		}
	};
	V(fa, pa);
	var Oh = class extends C {
		constructor(e, t = "html") {
			super(e, t);
		}
	};
	const { toString: Kh } = C.prototype;
	var Ln = class extends C {
		get innerHTML() {
			return this.textContent;
		}
		set innerHTML(e) {
			this.textContent = e;
		}
		toString() {
			return Kh.call(this.cloneNode()).replace("><", () => `>${this.textContent}<`);
		}
	};
	const ma = "script";
	var Sa = class extends Ln {
		constructor(e, t = ma) {
			super(e, t);
		}
		get type() {
			return S.get(this, "type");
		}
		set type(e) {
			S.set(this, "type", e);
		}
		get src() {
			return S.get(this, "src");
		}
		set src(e) {
			S.set(this, "src", e);
		}
		get defer() {
			return _.get(this, "defer");
		}
		set defer(e) {
			_.set(this, "defer", e);
		}
		get crossOrigin() {
			return S.get(this, "crossorigin");
		}
		set crossOrigin(e) {
			S.set(this, "crossorigin", e);
		}
		get nomodule() {
			return _.get(this, "nomodule");
		}
		set nomodule(e) {
			_.set(this, "nomodule", e);
		}
		get referrerPolicy() {
			return S.get(this, "referrerpolicy");
		}
		set referrerPolicy(e) {
			S.set(this, "referrerpolicy", e);
		}
		get nonce() {
			return S.get(this, "nonce");
		}
		set nonce(e) {
			S.set(this, "nonce", e);
		}
		get async() {
			return _.get(this, "async");
		}
		set async(e) {
			_.set(this, "async", e);
		}
		get text() {
			return this.textContent;
		}
		set text(e) {
			this.textContent = e;
		}
	};
	V(ma, Sa);
	var Jh = class extends C {
		constructor(e, t = "frame") {
			super(e, t);
		}
	};
	const ba = "iframe";
	var ya = class extends C {
		constructor(e, t = ba) {
			super(e, t);
		}
		get src() {
			return S.get(this, "src");
		}
		set src(e) {
			S.set(this, "src", e);
		}
		get srcdoc() {
			return S.get(this, "srcdoc");
		}
		set srcdoc(e) {
			S.set(this, "srcdoc", e);
		}
		get name() {
			return S.get(this, "name");
		}
		set name(e) {
			S.set(this, "name", e);
		}
		get allow() {
			return S.get(this, "allow");
		}
		set allow(e) {
			S.set(this, "allow", e);
		}
		get allowFullscreen() {
			return _.get(this, "allowfullscreen");
		}
		set allowFullscreen(e) {
			_.set(this, "allowfullscreen", e);
		}
		get referrerPolicy() {
			return S.get(this, "referrerpolicy");
		}
		set referrerPolicy(e) {
			S.set(this, "referrerpolicy", e);
		}
		get loading() {
			return S.get(this, "loading");
		}
		set loading(e) {
			S.set(this, "loading", e);
		}
	};
	V(ba, ya);
	var Ph = class extends C {
		constructor(e, t = "object") {
			super(e, t);
		}
	}, Wh = class extends C {
		constructor(e, t = "head") {
			super(e, t);
		}
	}, Yh = class extends C {
		constructor(e, t = "body") {
			super(e, t);
		}
	}, wa = L(((e) => {
		var t = {};
		t.StyleSheet = function() {
			this.parentStyleSheet = null;
		}, e.StyleSheet = t.StyleSheet;
	})), Ae = L(((e) => {
		var t = {};
		t.CSSRule = function() {
			this.parentRule = null, this.parentStyleSheet = null;
		}, t.CSSRule.UNKNOWN_RULE = 0, t.CSSRule.STYLE_RULE = 1, t.CSSRule.CHARSET_RULE = 2, t.CSSRule.IMPORT_RULE = 3, t.CSSRule.MEDIA_RULE = 4, t.CSSRule.FONT_FACE_RULE = 5, t.CSSRule.PAGE_RULE = 6, t.CSSRule.KEYFRAMES_RULE = 7, t.CSSRule.KEYFRAME_RULE = 8, t.CSSRule.MARGIN_RULE = 9, t.CSSRule.NAMESPACE_RULE = 10, t.CSSRule.COUNTER_STYLE_RULE = 11, t.CSSRule.SUPPORTS_RULE = 12, t.CSSRule.DOCUMENT_RULE = 13, t.CSSRule.FONT_FEATURE_VALUES_RULE = 14, t.CSSRule.VIEWPORT_RULE = 15, t.CSSRule.REGION_STYLE_RULE = 16, t.CSSRule.prototype = { constructor: t.CSSRule }, e.CSSRule = t.CSSRule;
	})), On = L(((e) => {
		var t = {
			CSSStyleDeclaration: It().CSSStyleDeclaration,
			CSSRule: Ae().CSSRule
		};
		t.CSSStyleRule = function() {
			t.CSSRule.call(this), this.selectorText = "", this.style = new t.CSSStyleDeclaration(), this.style.parentRule = this;
		}, t.CSSStyleRule.prototype = new t.CSSRule(), t.CSSStyleRule.prototype.constructor = t.CSSStyleRule, t.CSSStyleRule.prototype.type = 1, Object.defineProperty(t.CSSStyleRule.prototype, "cssText", {
			get: function() {
				var n;
				return this.selectorText ? n = this.selectorText + " {" + this.style.cssText + "}" : n = "", n;
			},
			set: function(n) {
				var r = t.CSSStyleRule.parse(n);
				this.style = r.style, this.selectorText = r.selectorText;
			}
		}), t.CSSStyleRule.parse = function(n) {
			for (var r = 0, s = "selector", i, o = r, a = "", u = {
				selector: !0,
				value: !0
			}, l = new t.CSSStyleRule(), p, f = "", g; g = n.charAt(r); r++) switch (g) {
				case " ":
				case "	":
				case "\r":
				case `
`:
				case "\f":
					if (u[s]) switch (n.charAt(r - 1)) {
						case " ":
						case "	":
						case "\r":
						case `
`:
						case "\f": break;
						default: a += " ";
					}
					break;
				case "\"":
					if (o = r + 1, i = n.indexOf("\"", o) + 1, !i) throw "\" is missing";
					a += n.slice(r, i), r = i - 1;
					break;
				case "'":
					if (o = r + 1, i = n.indexOf("'", o) + 1, !i) throw "' is missing";
					a += n.slice(r, i), r = i - 1;
					break;
				case "/":
					if (n.charAt(r + 1) === "*") {
						if (r += 2, i = n.indexOf("*/", r), i === -1) throw new SyntaxError("Missing */");
						r = i + 1;
					} else a += g;
					break;
				case "{":
					s === "selector" && (l.selectorText = a.trim(), a = "", s = "name");
					break;
				case ":":
					s === "name" ? (p = a.trim(), a = "", s = "value") : a += g;
					break;
				case "!":
					s === "value" && n.indexOf("!important", r) === r ? (f = "important", r += 9) : a += g;
					break;
				case ";":
					s === "value" ? (l.style.setProperty(p, a.trim(), f), f = "", a = "", s = "name") : a += g;
					break;
				case "}":
					if (s === "value") l.style.setProperty(p, a.trim(), f), f = "", a = "";
					else {
						if (s === "name") break;
						a += g;
					}
					s = "selector";
					break;
				default: a += g;
			}
			return l;
		}, e.CSSStyleRule = t.CSSStyleRule;
	})), Kn = L(((e) => {
		var t = {
			StyleSheet: wa().StyleSheet,
			CSSStyleRule: On().CSSStyleRule
		};
		t.CSSStyleSheet = function() {
			t.StyleSheet.call(this), this.cssRules = [];
		}, t.CSSStyleSheet.prototype = new t.StyleSheet(), t.CSSStyleSheet.prototype.constructor = t.CSSStyleSheet, t.CSSStyleSheet.prototype.insertRule = function(n, r) {
			if (r < 0 || r > this.cssRules.length) throw new RangeError("INDEX_SIZE_ERR");
			var s = t.parse(n).cssRules[0];
			return s.parentStyleSheet = this, this.cssRules.splice(r, 0, s), r;
		}, t.CSSStyleSheet.prototype.deleteRule = function(n) {
			if (n < 0 || n >= this.cssRules.length) throw new RangeError("INDEX_SIZE_ERR");
			this.cssRules.splice(n, 1);
		}, t.CSSStyleSheet.prototype.toString = function() {
			for (var n = "", r = this.cssRules, s = 0; s < r.length; s++) n += r[s].cssText + `
`;
			return n;
		}, e.CSSStyleSheet = t.CSSStyleSheet, t.parse = Rs().parse;
	})), Es = L(((e) => {
		var t = {};
		t.MediaList = function() {
			this.length = 0;
		}, t.MediaList.prototype = {
			constructor: t.MediaList,
			get mediaText() {
				return Array.prototype.join.call(this, ", ");
			},
			set mediaText(n) {
				for (var r = n.split(","), s = this.length = r.length, i = 0; i < s; i++) this[i] = r[i].trim();
			},
			appendMedium: function(n) {
				Array.prototype.indexOf.call(this, n) === -1 && (this[this.length] = n, this.length++);
			},
			deleteMedium: function(n) {
				var r = Array.prototype.indexOf.call(this, n);
				r !== -1 && Array.prototype.splice.call(this, r, 1);
			}
		}, e.MediaList = t.MediaList;
	})), Ca = L(((e) => {
		var t = {
			CSSRule: Ae().CSSRule,
			CSSStyleSheet: Kn().CSSStyleSheet,
			MediaList: Es().MediaList
		};
		t.CSSImportRule = function() {
			t.CSSRule.call(this), this.href = "", this.media = new t.MediaList(), this.styleSheet = new t.CSSStyleSheet();
		}, t.CSSImportRule.prototype = new t.CSSRule(), t.CSSImportRule.prototype.constructor = t.CSSImportRule, t.CSSImportRule.prototype.type = 3, Object.defineProperty(t.CSSImportRule.prototype, "cssText", {
			get: function() {
				var n = this.media.mediaText;
				return "@import url(" + this.href + ")" + (n ? " " + n : "") + ";";
			},
			set: function(n) {
				for (var r = 0, s = "", i = "", o, a; a = n.charAt(r); r++) switch (a) {
					case " ":
					case "	":
					case "\r":
					case `
`:
					case "\f":
						s === "after-import" ? s = "url" : i += a;
						break;
					case "@":
						!s && n.indexOf("@import", r) === r && (s = "after-import", r += 6, i = "");
						break;
					case "u":
						if (s === "url" && n.indexOf("url(", r) === r) {
							if (o = n.indexOf(")", r + 1), o === -1) throw r + ": \")\" not found";
							r += 4;
							var u = n.slice(r, o);
							u[0] === u[u.length - 1] && (u[0] === "\"" || u[0] === "'") && (u = u.slice(1, -1)), this.href = u, r = o, s = "media";
						}
						break;
					case "\"":
						if (s === "url") {
							if (o = n.indexOf("\"", r + 1), !o) throw r + `: '"' not found`;
							this.href = n.slice(r + 1, o), r = o, s = "media";
						}
						break;
					case "'":
						if (s === "url") {
							if (o = n.indexOf("'", r + 1), !o) throw r + `: "'" not found`;
							this.href = n.slice(r + 1, o), r = o, s = "media";
						}
						break;
					case ";":
						s === "media" && i && (this.media.mediaText = i.trim());
						break;
					default: s === "media" && (i += a);
				}
			}
		}), e.CSSImportRule = t.CSSImportRule;
	})), Ct = L(((e) => {
		var t = { CSSRule: Ae().CSSRule };
		t.CSSGroupingRule = function() {
			t.CSSRule.call(this), this.cssRules = [];
		}, t.CSSGroupingRule.prototype = new t.CSSRule(), t.CSSGroupingRule.prototype.constructor = t.CSSGroupingRule, t.CSSGroupingRule.prototype.insertRule = function(r, s) {
			if (s < 0 || s > this.cssRules.length) throw new RangeError("INDEX_SIZE_ERR");
			var i = t.parse(r).cssRules[0];
			return i.parentRule = this, this.cssRules.splice(s, 0, i), s;
		}, t.CSSGroupingRule.prototype.deleteRule = function(r) {
			if (r < 0 || r >= this.cssRules.length) throw new RangeError("INDEX_SIZE_ERR");
			this.cssRules.splice(r, 1)[0].parentRule = null;
		}, e.CSSGroupingRule = t.CSSGroupingRule;
	})), Vt = L(((e) => {
		var t = {
			CSSRule: Ae().CSSRule,
			CSSGroupingRule: Ct().CSSGroupingRule
		};
		t.CSSConditionRule = function() {
			t.CSSGroupingRule.call(this), this.cssRules = [];
		}, t.CSSConditionRule.prototype = new t.CSSGroupingRule(), t.CSSConditionRule.prototype.constructor = t.CSSConditionRule, t.CSSConditionRule.prototype.conditionText = "", t.CSSConditionRule.prototype.cssText = "", e.CSSConditionRule = t.CSSConditionRule;
	})), xs = L(((e) => {
		var t = {
			CSSRule: Ae().CSSRule,
			CSSGroupingRule: Ct().CSSGroupingRule,
			CSSConditionRule: Vt().CSSConditionRule,
			MediaList: Es().MediaList
		};
		t.CSSMediaRule = function() {
			t.CSSConditionRule.call(this), this.media = new t.MediaList();
		}, t.CSSMediaRule.prototype = new t.CSSConditionRule(), t.CSSMediaRule.prototype.constructor = t.CSSMediaRule, t.CSSMediaRule.prototype.type = 4, Object.defineProperties(t.CSSMediaRule.prototype, {
			conditionText: {
				get: function() {
					return this.media.mediaText;
				},
				set: function(n) {
					this.media.mediaText = n;
				},
				configurable: !0,
				enumerable: !0
			},
			cssText: {
				get: function() {
					for (var n = [], r = 0, s = this.cssRules.length; r < s; r++) n.push(this.cssRules[r].cssText);
					return "@media " + this.media.mediaText + " {" + n.join("") + "}";
				},
				configurable: !0,
				enumerable: !0
			}
		}), e.CSSMediaRule = t.CSSMediaRule;
	})), Ts = L(((e) => {
		var t = {
			CSSRule: Ae().CSSRule,
			CSSGroupingRule: Ct().CSSGroupingRule,
			CSSConditionRule: Vt().CSSConditionRule
		};
		t.CSSSupportsRule = function() {
			t.CSSConditionRule.call(this);
		}, t.CSSSupportsRule.prototype = new t.CSSConditionRule(), t.CSSSupportsRule.prototype.constructor = t.CSSSupportsRule, t.CSSSupportsRule.prototype.type = 12, Object.defineProperty(t.CSSSupportsRule.prototype, "cssText", { get: function() {
			for (var n = [], r = 0, s = this.cssRules.length; r < s; r++) n.push(this.cssRules[r].cssText);
			return "@supports " + this.conditionText + " {" + n.join("") + "}";
		} }), e.CSSSupportsRule = t.CSSSupportsRule;
	})), Ia = L(((e) => {
		var t = {
			CSSStyleDeclaration: It().CSSStyleDeclaration,
			CSSRule: Ae().CSSRule
		};
		t.CSSFontFaceRule = function() {
			t.CSSRule.call(this), this.style = new t.CSSStyleDeclaration(), this.style.parentRule = this;
		}, t.CSSFontFaceRule.prototype = new t.CSSRule(), t.CSSFontFaceRule.prototype.constructor = t.CSSFontFaceRule, t.CSSFontFaceRule.prototype.type = 5, Object.defineProperty(t.CSSFontFaceRule.prototype, "cssText", { get: function() {
			return "@font-face {" + this.style.cssText + "}";
		} }), e.CSSFontFaceRule = t.CSSFontFaceRule;
	})), va = L(((e) => {
		var t = { CSSRule: Ae().CSSRule };
		t.CSSHostRule = function() {
			t.CSSRule.call(this), this.cssRules = [];
		}, t.CSSHostRule.prototype = new t.CSSRule(), t.CSSHostRule.prototype.constructor = t.CSSHostRule, t.CSSHostRule.prototype.type = 1001, Object.defineProperty(t.CSSHostRule.prototype, "cssText", { get: function() {
			for (var n = [], r = 0, s = this.cssRules.length; r < s; r++) n.push(this.cssRules[r].cssText);
			return "@host {" + n.join("") + "}";
		} }), e.CSSHostRule = t.CSSHostRule;
	})), Bs = L(((e) => {
		var t = {
			CSSRule: Ae().CSSRule,
			CSSStyleDeclaration: It().CSSStyleDeclaration
		};
		t.CSSKeyframeRule = function() {
			t.CSSRule.call(this), this.keyText = "", this.style = new t.CSSStyleDeclaration(), this.style.parentRule = this;
		}, t.CSSKeyframeRule.prototype = new t.CSSRule(), t.CSSKeyframeRule.prototype.constructor = t.CSSKeyframeRule, t.CSSKeyframeRule.prototype.type = 8, Object.defineProperty(t.CSSKeyframeRule.prototype, "cssText", { get: function() {
			return this.keyText + " {" + this.style.cssText + "} ";
		} }), e.CSSKeyframeRule = t.CSSKeyframeRule;
	})), Qs = L(((e) => {
		var t = { CSSRule: Ae().CSSRule };
		t.CSSKeyframesRule = function() {
			t.CSSRule.call(this), this.name = "", this.cssRules = [];
		}, t.CSSKeyframesRule.prototype = new t.CSSRule(), t.CSSKeyframesRule.prototype.constructor = t.CSSKeyframesRule, t.CSSKeyframesRule.prototype.type = 7, Object.defineProperty(t.CSSKeyframesRule.prototype, "cssText", { get: function() {
			for (var n = [], r = 0, s = this.cssRules.length; r < s; r++) n.push("  " + this.cssRules[r].cssText);
			return "@" + (this._vendorPrefix || "") + "keyframes " + this.name + ` { 
` + n.join(`
`) + `
}`;
		} }), e.CSSKeyframesRule = t.CSSKeyframesRule;
	})), Ea = L(((e) => {
		var t = {};
		t.CSSValue = function() {}, t.CSSValue.prototype = {
			constructor: t.CSSValue,
			set cssText(n) {
				var r = this._getConstructorName();
				throw new Error("DOMException: property \"cssText\" of \"" + r + "\" is readonly and can not be replaced with \"" + n + "\"!");
			},
			get cssText() {
				var n = this._getConstructorName();
				throw new Error("getter \"cssText\" of \"" + n + "\" is not implemented!");
			},
			_getConstructorName: function() {
				return this.constructor.toString().match(/function\s([^\(]+)/)[1];
			}
		}, e.CSSValue = t.CSSValue;
	})), xa = L(((e) => {
		var t = { CSSValue: Ea().CSSValue };
		t.CSSValueExpression = function(r, s) {
			this._token = r, this._idx = s;
		}, t.CSSValueExpression.prototype = new t.CSSValue(), t.CSSValueExpression.prototype.constructor = t.CSSValueExpression, t.CSSValueExpression.prototype.parse = function() {
			for (var n = this._token, r = this._idx, s = "", i = "", o = "", a, u = [];; ++r) {
				if (s = n.charAt(r), s === "") {
					o = "css expression error: unfinished expression!";
					break;
				}
				switch (s) {
					case "(":
						u.push(s), i += s;
						break;
					case ")":
						u.pop(s), i += s;
						break;
					case "/":
						(a = this._parseJSComment(n, r)) ? a.error ? o = "css expression error: unfinished comment in expression!" : r = a.idx : (a = this._parseJSRexExp(n, r)) ? (r = a.idx, i += a.text) : i += s;
						break;
					case "'":
					case "\"":
						a = this._parseJSString(n, r, s), a ? (r = a.idx, i += a.text) : i += s;
						break;
					default: i += s;
				}
				if (o || u.length === 0) break;
			}
			var l;
			return o ? l = { error: o } : l = {
				idx: r,
				expression: i
			}, l;
		}, t.CSSValueExpression.prototype._parseJSComment = function(n, r) {
			var s = n.charAt(r + 1), i;
			if (s === "/" || s === "*") {
				var o = r, a, u;
				return s === "/" ? u = `
` : s === "*" && (u = "*/"), a = n.indexOf(u, o + 1 + 1), a !== -1 ? (a = a + u.length - 1, i = n.substring(r, a + 1), {
					idx: a,
					text: i
				}) : { error: "css expression error: unfinished comment in expression!" };
			} else return !1;
		}, t.CSSValueExpression.prototype._parseJSString = function(n, r, s) {
			var i = this._findMatchedIdx(n, r, s), o;
			return i === -1 ? !1 : (o = n.substring(r, i + s.length), {
				idx: i,
				text: o
			});
		}, t.CSSValueExpression.prototype._parseJSRexExp = function(n, r) {
			var s = n.substring(0, r).replace(/\s+$/, "");
			return [
				/^$/,
				/\($/,
				/\[$/,
				/\!$/,
				/\+$/,
				/\-$/,
				/\*$/,
				/\/\s+/,
				/\%$/,
				/\=$/,
				/\>$/,
				/<$/,
				/\&$/,
				/\|$/,
				/\^$/,
				/\~$/,
				/\?$/,
				/\,$/,
				/delete$/,
				/in$/,
				/instanceof$/,
				/new$/,
				/typeof$/,
				/void$/
			].some(function(i) {
				return i.test(s);
			}) ? this._parseJSString(n, r, "/") : !1;
		}, t.CSSValueExpression.prototype._findMatchedIdx = function(n, r, s) {
			for (var i = r, o, a = -1;;) if (o = n.indexOf(s, i + 1), o === -1) {
				o = a;
				break;
			} else {
				var u = n.substring(r + 1, o).match(/\\+$/);
				if (!u || u[0] % 2 === 0) break;
				i = o;
			}
			return n.indexOf(`
`, r + 1) < o && (o = a), o;
		}, e.CSSValueExpression = t.CSSValueExpression;
	})), Ta = L(((e) => {
		var t = {};
		t.MatcherList = function() {
			this.length = 0;
		}, t.MatcherList.prototype = {
			constructor: t.MatcherList,
			get matcherText() {
				return Array.prototype.join.call(this, ", ");
			},
			set matcherText(n) {
				for (var r = n.split(","), s = this.length = r.length, i = 0; i < s; i++) this[i] = r[i].trim();
			},
			appendMatcher: function(n) {
				Array.prototype.indexOf.call(this, n) === -1 && (this[this.length] = n, this.length++);
			},
			deleteMatcher: function(n) {
				var r = Array.prototype.indexOf.call(this, n);
				r !== -1 && Array.prototype.splice.call(this, r, 1);
			}
		}, e.MatcherList = t.MatcherList;
	})), Ba = L(((e) => {
		var t = {
			CSSRule: Ae().CSSRule,
			MatcherList: Ta().MatcherList
		};
		t.CSSDocumentRule = function() {
			t.CSSRule.call(this), this.matcher = new t.MatcherList(), this.cssRules = [];
		}, t.CSSDocumentRule.prototype = new t.CSSRule(), t.CSSDocumentRule.prototype.constructor = t.CSSDocumentRule, t.CSSDocumentRule.prototype.type = 10, Object.defineProperty(t.CSSDocumentRule.prototype, "cssText", { get: function() {
			for (var n = [], r = 0, s = this.cssRules.length; r < s; r++) n.push(this.cssRules[r].cssText);
			return "@-moz-document " + this.matcher.matcherText + " {" + n.join("") + "}";
		} }), e.CSSDocumentRule = t.CSSDocumentRule;
	})), Rs = L(((e) => {
		var t = {};
		t.parse = function(r) {
			for (var s = 0, i = "before-selector", o, a = "", u = 0, l = {
				selector: !0,
				value: !0,
				"value-parenthesis": !0,
				atRule: !0,
				"importRule-begin": !0,
				importRule: !0,
				atBlock: !0,
				conditionBlock: !0,
				"documentRule-begin": !0
			}, p = new t.CSSStyleSheet(), f = p, g, y = [], B = !1, N, z, W = "", F, G, P, H, Ne, De, fe, Ye, Wn = /@(-(?:\w+-)+)?keyframes/g, lt = function(Yn) {
				var jt = r.substring(0, s).split(`
`), Un = jt.length, ht = jt.pop().length + 1, pe = /* @__PURE__ */ new Error(Yn + " (line " + Un + ", char " + ht + ")");
				throw pe.line = Un, pe.char = ht, pe.styleSheet = p, pe;
			}, te; te = r.charAt(s); s++) switch (te) {
				case " ":
				case "	":
				case "\r":
				case `
`:
				case "\f":
					l[i] && (a += te);
					break;
				case "\"":
					o = s + 1;
					do
						o = r.indexOf("\"", o) + 1, o || lt("Unmatched \"");
					while (r[o - 2] === "\\");
					switch (a += r.slice(s, o), s = o - 1, i) {
						case "before-value":
							i = "value";
							break;
						case "importRule-begin": i = "importRule";
					}
					break;
				case "'":
					o = s + 1;
					do
						o = r.indexOf("'", o) + 1, o || lt("Unmatched '");
					while (r[o - 2] === "\\");
					switch (a += r.slice(s, o), s = o - 1, i) {
						case "before-value":
							i = "value";
							break;
						case "importRule-begin": i = "importRule";
					}
					break;
				case "/":
					r.charAt(s + 1) === "*" ? (s += 2, o = r.indexOf("*/", s), o === -1 ? lt("Missing */") : s = o + 1) : a += te, i === "importRule-begin" && (a += " ", i = "importRule");
					break;
				case "@":
					if (r.indexOf("@-moz-document", s) === s) {
						i = "documentRule-begin", fe = new t.CSSDocumentRule(), fe.__starts = s, s += 13, a = "";
						break;
					} else if (r.indexOf("@media", s) === s) {
						i = "atBlock", G = new t.CSSMediaRule(), G.__starts = s, s += 5, a = "";
						break;
					} else if (r.indexOf("@supports", s) === s) {
						i = "conditionBlock", P = new t.CSSSupportsRule(), P.__starts = s, s += 8, a = "";
						break;
					} else if (r.indexOf("@host", s) === s) {
						i = "hostRule-begin", s += 4, Ye = new t.CSSHostRule(), Ye.__starts = s, a = "";
						break;
					} else if (r.indexOf("@import", s) === s) {
						i = "importRule-begin", s += 6, a += "@import";
						break;
					} else if (r.indexOf("@font-face", s) === s) {
						i = "fontFaceRule-begin", s += 9, Ne = new t.CSSFontFaceRule(), Ne.__starts = s, a = "";
						break;
					} else {
						Wn.lastIndex = s;
						var At = Wn.exec(r);
						if (At && At.index === s) {
							i = "keyframesRule-begin", De = new t.CSSKeyframesRule(), De.__starts = s, De._vendorPrefix = At[1], s += At[0].length - 1, a = "";
							break;
						} else i === "selector" && (i = "atRule");
					}
					a += te;
					break;
				case "{":
					i === "selector" || i === "atRule" ? (F.selectorText = a.trim(), F.style.__starts = s, a = "", i = "before-name") : i === "atBlock" ? (G.media.mediaText = a.trim(), g && y.push(g), f = g = G, G.parentStyleSheet = p, a = "", i = "before-selector") : i === "conditionBlock" ? (P.conditionText = a.trim(), g && y.push(g), f = g = P, P.parentStyleSheet = p, a = "", i = "before-selector") : i === "hostRule-begin" ? (g && y.push(g), f = g = Ye, Ye.parentStyleSheet = p, a = "", i = "before-selector") : i === "fontFaceRule-begin" ? (g && (Ne.parentRule = g), Ne.parentStyleSheet = p, F = Ne, a = "", i = "before-name") : i === "keyframesRule-begin" ? (De.name = a.trim(), g && (y.push(g), De.parentRule = g), De.parentStyleSheet = p, f = g = De, a = "", i = "keyframeRule-begin") : i === "keyframeRule-begin" ? (F = new t.CSSKeyframeRule(), F.keyText = a.trim(), F.__starts = s, a = "", i = "before-name") : i === "documentRule-begin" && (fe.matcher.matcherText = a.trim(), g && (y.push(g), fe.parentRule = g), f = g = fe, fe.parentStyleSheet = p, a = "", i = "before-selector");
					break;
				case ":":
					i === "name" ? (z = a.trim(), a = "", i = "before-value") : a += te;
					break;
				case "(":
					if (i === "value") if (a.trim() === "expression") {
						var ct = new t.CSSValueExpression(r, s).parse();
						ct.error ? lt(ct.error) : (a += ct.expression, s = ct.idx);
					} else i = "value-parenthesis", u = 1, a += te;
					else i === "value-parenthesis" && u++, a += te;
					break;
				case ")":
					i === "value-parenthesis" && (u--, u === 0 && (i = "value")), a += te;
					break;
				case "!":
					i === "value" && r.indexOf("!important", s) === s ? (W = "important", s += 9) : a += te;
					break;
				case ";":
					switch (i) {
						case "value":
							F.style.setProperty(z, a.trim(), W), W = "", a = "", i = "before-name";
							break;
						case "atRule":
							a = "", i = "before-selector";
							break;
						case "importRule":
							H = new t.CSSImportRule(), H.parentStyleSheet = H.styleSheet.parentStyleSheet = p, H.cssText = a + te, p.cssRules.push(H), a = "", i = "before-selector";
							break;
						default: a += te;
					}
					break;
				case "}":
					switch (i) {
						case "value": F.style.setProperty(z, a.trim(), W), W = "";
						case "before-name":
						case "name":
							F.__ends = s + 1, g && (F.parentRule = g), F.parentStyleSheet = p, f.cssRules.push(F), a = "", f.constructor === t.CSSKeyframesRule ? i = "keyframeRule-begin" : i = "before-selector";
							break;
						case "keyframeRule-begin":
						case "before-selector":
						case "selector":
							for (g || lt("Unexpected }"), B = y.length > 0; y.length > 0;) {
								if (g = y.pop(), g.constructor.name === "CSSMediaRule" || g.constructor.name === "CSSSupportsRule") {
									N = f, f = g, f.cssRules.push(N);
									break;
								}
								y.length === 0 && (B = !1);
							}
							B || (f.__ends = s + 1, p.cssRules.push(f), f = p, g = null), a = "", i = "before-selector";
					}
					break;
				default:
					switch (i) {
						case "before-selector":
							i = "selector", F = new t.CSSStyleRule(), F.__starts = s;
							break;
						case "before-name":
							i = "name";
							break;
						case "before-value":
							i = "value";
							break;
						case "importRule-begin": i = "importRule";
					}
					a += te;
			}
			return p;
		}, e.parse = t.parse, t.CSSStyleSheet = Kn().CSSStyleSheet, t.CSSStyleRule = On().CSSStyleRule, t.CSSImportRule = Ca().CSSImportRule, t.CSSGroupingRule = Ct().CSSGroupingRule, t.CSSMediaRule = xs().CSSMediaRule, t.CSSConditionRule = Vt().CSSConditionRule, t.CSSSupportsRule = Ts().CSSSupportsRule, t.CSSFontFaceRule = Ia().CSSFontFaceRule, t.CSSHostRule = va().CSSHostRule, t.CSSStyleDeclaration = It().CSSStyleDeclaration, t.CSSKeyframeRule = Bs().CSSKeyframeRule, t.CSSKeyframesRule = Qs().CSSKeyframesRule, t.CSSValueExpression = xa().CSSValueExpression, t.CSSDocumentRule = Ba().CSSDocumentRule;
	})), It = L(((e) => {
		var t = {};
		t.CSSStyleDeclaration = function() {
			this.length = 0, this.parentRule = null, this._importants = {};
		}, t.CSSStyleDeclaration.prototype = {
			constructor: t.CSSStyleDeclaration,
			getPropertyValue: function(n) {
				return this[n] || "";
			},
			setProperty: function(n, r, s) {
				this[n] ? Array.prototype.indexOf.call(this, n) < 0 && (this[this.length] = n, this.length++) : (this[this.length] = n, this.length++), this[n] = r + "", this._importants[n] = s;
			},
			removeProperty: function(n) {
				if (!(n in this)) return "";
				var r = Array.prototype.indexOf.call(this, n);
				if (r < 0) return "";
				var s = this[n];
				return this[n] = "", Array.prototype.splice.call(this, r, 1), s;
			},
			getPropertyCSSValue: function() {},
			getPropertyPriority: function(n) {
				return this._importants[n] || "";
			},
			getPropertyShorthand: function() {},
			isPropertyImplicit: function() {},
			get cssText() {
				for (var n = [], r = 0, s = this.length; r < s; ++r) {
					var i = this[r], o = this.getPropertyValue(i), a = this.getPropertyPriority(i);
					a && (a = " !" + a), n[r] = i + ": " + o + a + ";";
				}
				return n.join(" ");
			},
			set cssText(n) {
				var r, s;
				for (r = this.length; r--;) s = this[r], this[s] = "";
				Array.prototype.splice.call(this, 0, this.length), this._importants = {};
				var i = t.parse("#bogus{" + n + "}").cssRules[0].style, o = i.length;
				for (r = 0; r < o; ++r) s = i[r], this.setProperty(i[r], i.getPropertyValue(s), i.getPropertyPriority(s));
			}
		}, e.CSSStyleDeclaration = t.CSSStyleDeclaration, t.parse = Rs().parse;
	})), Uh = L(((e) => {
		var t = {
			CSSStyleSheet: Kn().CSSStyleSheet,
			CSSRule: Ae().CSSRule,
			CSSStyleRule: On().CSSStyleRule,
			CSSGroupingRule: Ct().CSSGroupingRule,
			CSSConditionRule: Vt().CSSConditionRule,
			CSSMediaRule: xs().CSSMediaRule,
			CSSSupportsRule: Ts().CSSSupportsRule,
			CSSStyleDeclaration: It().CSSStyleDeclaration,
			CSSKeyframeRule: Bs().CSSKeyframeRule,
			CSSKeyframesRule: Qs().CSSKeyframesRule
		};
		t.clone = function n(r) {
			var s = new t.CSSStyleSheet(), i = r.cssRules;
			if (!i) return s;
			for (var o = 0, a = i.length; o < a; o++) {
				var u = i[o], l = s.cssRules[o] = new u.constructor(), p = u.style;
				if (p) {
					for (var f = l.style = new t.CSSStyleDeclaration(), g = 0, y = p.length; g < y; g++) {
						var B = f[g] = p[g];
						f[B] = p[B], f._importants[B] = p.getPropertyPriority(B);
					}
					f.length = p.length;
				}
				u.hasOwnProperty("keyText") && (l.keyText = u.keyText), u.hasOwnProperty("selectorText") && (l.selectorText = u.selectorText), u.hasOwnProperty("mediaText") && (l.mediaText = u.mediaText), u.hasOwnProperty("conditionText") && (l.conditionText = u.conditionText), u.hasOwnProperty("cssRules") && (l.cssRules = n(u).cssRules);
			}
			return s;
		}, e.clone = t.clone;
	})), Zh = L(((e) => {
		e.CSSStyleDeclaration = It().CSSStyleDeclaration, e.CSSRule = Ae().CSSRule, e.CSSGroupingRule = Ct().CSSGroupingRule, e.CSSConditionRule = Vt().CSSConditionRule, e.CSSStyleRule = On().CSSStyleRule, e.MediaList = Es().MediaList, e.CSSMediaRule = xs().CSSMediaRule, e.CSSSupportsRule = Ts().CSSSupportsRule, e.CSSImportRule = Ca().CSSImportRule, e.CSSFontFaceRule = Ia().CSSFontFaceRule, e.CSSHostRule = va().CSSHostRule, e.StyleSheet = wa().StyleSheet, e.CSSStyleSheet = Kn().CSSStyleSheet, e.CSSKeyframesRule = Qs().CSSKeyframesRule, e.CSSKeyframeRule = Bs().CSSKeyframeRule, e.MatcherList = Ta().MatcherList, e.CSSDocumentRule = Ba().CSSDocumentRule, e.CSSValue = Ea().CSSValue, e.CSSValueExpression = xa().CSSValueExpression, e.parse = Rs().parse, e.clone = Uh().clone;
	}))();
	const Qa = "style";
	var Ra = class extends Ln {
		constructor(e, t = Qa) {
			super(e, t), this[pt] = null;
		}
		get sheet() {
			const e = this[pt];
			return e !== null ? e : this[pt] = (0, Zh.parse)(this.textContent);
		}
		get innerHTML() {
			return super.innerHTML || "";
		}
		set innerHTML(e) {
			super.textContent = e, this[pt] = null;
		}
		get innerText() {
			return super.innerText || "";
		}
		set innerText(e) {
			super.textContent = e, this[pt] = null;
		}
		get textContent() {
			return super.textContent || "";
		}
		set textContent(e) {
			super.textContent = e, this[pt] = null;
		}
	};
	V(Qa, Ra);
	var Na = class extends C {
		constructor(e, t = "time") {
			super(e, t);
		}
		get dateTime() {
			return S.get(this, "datetime");
		}
		set dateTime(e) {
			S.set(this, "datetime", e);
		}
	};
	V("time", Na);
	var jh = class extends C {
		constructor(e, t = "fieldset") {
			super(e, t);
		}
	}, Xh = class extends C {
		constructor(e, t = "embed") {
			super(e, t);
		}
	}, qh = class extends C {
		constructor(e, t = "hr") {
			super(e, t);
		}
	}, $h = class extends C {
		constructor(e, t = "progress") {
			super(e, t);
		}
	}, zh = class extends C {
		constructor(e, t = "p") {
			super(e, t);
		}
	}, ed = class extends C {
		constructor(e, t = "table") {
			super(e, t);
		}
	}, td = class extends C {
		constructor(e, t = "frameset") {
			super(e, t);
		}
	}, nd = class extends C {
		constructor(e, t = "li") {
			super(e, t);
		}
	}, rd = class extends C {
		constructor(e, t = "base") {
			super(e, t);
		}
	}, sd = class extends C {
		constructor(e, t = "datalist") {
			super(e, t);
		}
	};
	const Da = "input";
	var _a = class extends C {
		constructor(e, t = Da) {
			super(e, t);
		}
		get autofocus() {
			return _.get(this, "autofocus") || -1;
		}
		set autofocus(e) {
			_.set(this, "autofocus", e);
		}
		get disabled() {
			return _.get(this, "disabled");
		}
		set disabled(e) {
			_.set(this, "disabled", e);
		}
		get name() {
			return this.getAttribute("name");
		}
		set name(e) {
			this.setAttribute("name", e);
		}
		get placeholder() {
			return this.getAttribute("placeholder");
		}
		set placeholder(e) {
			this.setAttribute("placeholder", e);
		}
		get type() {
			return this.getAttribute("type");
		}
		set type(e) {
			this.setAttribute("type", e);
		}
		get value() {
			return S.get(this, "value");
		}
		set value(e) {
			S.set(this, "value", e);
		}
	};
	V(Da, _a);
	var id = class extends C {
		constructor(e, t = "param") {
			super(e, t);
		}
	}, od = class extends C {
		constructor(e, t = "media") {
			super(e, t);
		}
	}, ad = class extends C {
		constructor(e, t = "audio") {
			super(e, t);
		}
	};
	const Fa = "h1";
	var ka = class extends C {
		constructor(e, t = Fa) {
			super(e, t);
		}
	};
	V([
		Fa,
		"h2",
		"h3",
		"h4",
		"h5",
		"h6"
	], ka);
	var ud = class extends C {
		constructor(e, t = "dir") {
			super(e, t);
		}
	}, ld = class extends C {
		constructor(e, t = "quote") {
			super(e, t);
		}
	}, Ad = Me({ default: () => Ma }), Ma, cd = Se((() => {
		throw Ma = {}, /* @__PURE__ */ new Error("Could not resolve \"canvas\" imported by \"linkedom\". Is it installed?");
	})), hd = L(((e, t) => {
		var n = class {
			constructor(r, s) {
				this.width = r, this.height = s;
			}
			getContext() {
				return null;
			}
			toDataURL() {
				return "";
			}
		};
		t.exports = { createCanvas: (r, s) => new n(r, s) };
	}));
	const { createCanvas: fd } = Ou(L(((e, t) => {
		try {
			t.exports = (cd(), Ku(Ad));
		} catch {
			t.exports = hd();
		}
	}))(), 1).default, Ga = "canvas";
	var Ha = class extends C {
		constructor(e, t = Ga) {
			super(e, t), this[ye] = fd(300, 150);
		}
		get width() {
			return this[ye].width;
		}
		set width(e) {
			Je.set(this, "width", e), this[ye].width = e;
		}
		get height() {
			return this[ye].height;
		}
		set height(e) {
			Je.set(this, "height", e), this[ye].height = e;
		}
		getContext(e) {
			return this[ye].getContext(e);
		}
		toDataURL(...e) {
			return this[ye].toDataURL(...e);
		}
	};
	V(Ga, Ha);
	var pd = class extends C {
		constructor(e, t = "legend") {
			super(e, t);
		}
	};
	const La = "option";
	var Oa = class extends C {
		constructor(e, t = La) {
			super(e, t);
		}
		get value() {
			return S.get(this, "value");
		}
		set value(e) {
			S.set(this, "value", e);
		}
		get selected() {
			return _.get(this, "selected");
		}
		set selected(e) {
			const t = this.parentElement?.querySelector("option[selected]");
			t && t !== this && (t.selected = !1), _.set(this, "selected", e);
		}
	};
	V(La, Oa);
	var md = class extends C {
		constructor(e, t = "span") {
			super(e, t);
		}
	}, Sd = class extends C {
		constructor(e, t = "meter") {
			super(e, t);
		}
	}, bd = class extends C {
		constructor(e, t = "video") {
			super(e, t);
		}
	}, yd = class extends C {
		constructor(e, t = "td") {
			super(e, t);
		}
	};
	const Ka = "title";
	var Ja = class extends Ln {
		constructor(e, t = Ka) {
			super(e, t);
		}
	};
	V(Ka, Ja);
	var wd = class extends C {
		constructor(e, t = "output") {
			super(e, t);
		}
	}, Cd = class extends C {
		constructor(e, t = "tr") {
			super(e, t);
		}
	}, Id = class extends C {
		constructor(e, t = "data") {
			super(e, t);
		}
	}, vd = class extends C {
		constructor(e, t = "menu") {
			super(e, t);
		}
	};
	const Pa = "select";
	var Wa = class extends C {
		constructor(e, t = Pa) {
			super(e, t);
		}
		get options() {
			let e = new Qe(), { firstElementChild: t } = this;
			for (; t;) t.tagName === "OPTGROUP" ? e.push(...t.children) : e.push(t), t = t.nextElementSibling;
			return e;
		}
		get disabled() {
			return _.get(this, "disabled");
		}
		set disabled(e) {
			_.set(this, "disabled", e);
		}
		get name() {
			return this.getAttribute("name");
		}
		set name(e) {
			this.setAttribute("name", e);
		}
		get value() {
			return this.querySelector("option[selected]")?.value;
		}
	};
	V(Pa, Wa);
	var Ed = class extends C {
		constructor(e, t = "br") {
			super(e, t);
		}
	};
	const Ya = "button";
	var Ua = class extends C {
		constructor(e, t = Ya) {
			super(e, t);
		}
		get disabled() {
			return _.get(this, "disabled");
		}
		set disabled(e) {
			_.set(this, "disabled", e);
		}
		get name() {
			return this.getAttribute("name");
		}
		set name(e) {
			this.setAttribute("name", e);
		}
		get type() {
			return this.getAttribute("type");
		}
		set type(e) {
			this.setAttribute("type", e);
		}
	};
	V(Ya, Ua);
	var xd = class extends C {
		constructor(e, t = "map") {
			super(e, t);
		}
	}, Td = class extends C {
		constructor(e, t = "optgroup") {
			super(e, t);
		}
	}, Bd = class extends C {
		constructor(e, t = "dl") {
			super(e, t);
		}
	};
	const Va = "textarea";
	var Za = class extends Ln {
		constructor(e, t = Va) {
			super(e, t);
		}
		get disabled() {
			return _.get(this, "disabled");
		}
		set disabled(e) {
			_.set(this, "disabled", e);
		}
		get name() {
			return this.getAttribute("name");
		}
		set name(e) {
			this.setAttribute("name", e);
		}
		get placeholder() {
			return this.getAttribute("placeholder");
		}
		set placeholder(e) {
			this.setAttribute("placeholder", e);
		}
		get type() {
			return this.getAttribute("type");
		}
		set type(e) {
			this.setAttribute("type", e);
		}
		get value() {
			return this.textContent;
		}
		set value(e) {
			this.textContent = e;
		}
	};
	V(Va, Za);
	var Qd = class extends C {
		constructor(e, t = "font") {
			super(e, t);
		}
	}, Rd = class extends C {
		constructor(e, t = "div") {
			super(e, t);
		}
	};
	const ja = "link";
	var Xa = class extends C {
		constructor(e, t = ja) {
			super(e, t);
		}
		get disabled() {
			return _.get(this, "disabled");
		}
		set disabled(e) {
			_.set(this, "disabled", e);
		}
		get href() {
			return S.get(this, "href").trim();
		}
		set href(e) {
			S.set(this, "href", e);
		}
		get hreflang() {
			return S.get(this, "hreflang");
		}
		set hreflang(e) {
			S.set(this, "hreflang", e);
		}
		get media() {
			return S.get(this, "media");
		}
		set media(e) {
			S.set(this, "media", e);
		}
		get rel() {
			return S.get(this, "rel");
		}
		set rel(e) {
			S.set(this, "rel", e);
		}
		get type() {
			return S.get(this, "type");
		}
		set type(e) {
			S.set(this, "type", e);
		}
	};
	V(ja, Xa);
	const qa = "slot";
	var $a = class extends C {
		constructor(e, t = qa) {
			super(e, t);
		}
		get name() {
			return this.getAttribute("name");
		}
		set name(e) {
			this.setAttribute("name", e);
		}
		assign() {}
		assignedNodes(e) {
			const t = !!this.name, n = this.getRootNode().host?.childNodes ?? [];
			let r;
			if (t ? r = [...n].filter((s) => s.slot === this.name) : r = [...n].filter((s) => !s.slot), e?.flatten) {
				const s = [];
				for (let i of r) i.localName === "slot" ? s.push(...i.assignedNodes({ flatten: !0 })) : s.push(i);
				r = s;
			}
			return r.length ? r : [...this.childNodes];
		}
		assignedElements(e) {
			const t = this.assignedNodes(e).filter((n) => n.nodeType === 1);
			return t.length ? t : [...this.children];
		}
	};
	V(qa, $a);
	var Nd = class extends C {
		constructor(e, t = "form") {
			super(e, t);
		}
	};
	const za = "img";
	var Ns = class extends C {
		constructor(e, t = za) {
			super(e, t);
		}
		get alt() {
			return S.get(this, "alt");
		}
		set alt(e) {
			S.set(this, "alt", e);
		}
		get sizes() {
			return S.get(this, "sizes");
		}
		set sizes(e) {
			S.set(this, "sizes", e);
		}
		get src() {
			return S.get(this, "src");
		}
		set src(e) {
			S.set(this, "src", e);
		}
		get srcset() {
			return S.get(this, "srcset");
		}
		set srcset(e) {
			S.set(this, "srcset", e);
		}
		get title() {
			return S.get(this, "title");
		}
		set title(e) {
			S.set(this, "title", e);
		}
		get width() {
			return Je.get(this, "width");
		}
		set width(e) {
			Je.set(this, "width", e);
		}
		get height() {
			return Je.get(this, "height");
		}
		set height(e) {
			Je.set(this, "height", e);
		}
	};
	V(za, Ns);
	var Dd = class extends C {
		constructor(e, t = "pre") {
			super(e, t);
		}
	}, _d = class extends C {
		constructor(e, t = "ul") {
			super(e, t);
		}
	};
	const eu = "meta";
	var tu = class extends C {
		constructor(e, t = eu) {
			super(e, t);
		}
		get name() {
			return S.get(this, "name");
		}
		set name(e) {
			S.set(this, "name", e);
		}
		get httpEquiv() {
			return S.get(this, "http-equiv");
		}
		set httpEquiv(e) {
			S.set(this, "http-equiv", e);
		}
		get content() {
			return S.get(this, "content");
		}
		set content(e) {
			S.set(this, "content", e);
		}
		get charset() {
			return S.get(this, "charset");
		}
		set charset(e) {
			S.set(this, "charset", e);
		}
		get media() {
			return S.get(this, "media");
		}
		set media(e) {
			S.set(this, "media", e);
		}
	};
	V(eu, tu);
	var Fd = class extends C {
		constructor(e, t = "picture") {
			super(e, t);
		}
	}, kd = class extends C {
		constructor(e, t = "area") {
			super(e, t);
		}
	}, Md = class extends C {
		constructor(e, t = "ol") {
			super(e, t);
		}
	}, Gd = class extends C {
		constructor(e, t = "caption") {
			super(e, t);
		}
	};
	const nu = "a";
	var ru = class extends C {
		constructor(e, t = nu) {
			super(e, t);
		}
		get href() {
			return encodeURI(decodeURI(S.get(this, "href"))).trim();
		}
		set href(e) {
			S.set(this, "href", decodeURI(e));
		}
		get download() {
			return encodeURI(decodeURI(S.get(this, "download")));
		}
		set download(e) {
			S.set(this, "download", decodeURI(e));
		}
		get target() {
			return S.get(this, "target");
		}
		set target(e) {
			S.set(this, "target", e);
		}
		get type() {
			return S.get(this, "type");
		}
		set type(e) {
			S.set(this, "type", e);
		}
		get rel() {
			return S.get(this, "rel");
		}
		set rel(e) {
			S.set(this, "rel", e);
		}
	};
	V(nu, ru);
	var Hd = class extends C {
		constructor(e, t = "label") {
			super(e, t);
		}
	}, Ld = class extends C {
		constructor(e, t = "unknown") {
			super(e, t);
		}
	}, Od = class extends C {
		constructor(e, t = "mod") {
			super(e, t);
		}
	}, Kd = class extends C {
		constructor(e, t = "details") {
			super(e, t);
		}
	};
	const su = "source";
	var iu = class extends C {
		constructor(e, t = su) {
			super(e, t);
		}
		get src() {
			return S.get(this, "src");
		}
		set src(e) {
			S.set(this, "src", e);
		}
		get srcset() {
			return S.get(this, "srcset");
		}
		set srcset(e) {
			S.set(this, "srcset", e);
		}
		get sizes() {
			return S.get(this, "sizes");
		}
		set sizes(e) {
			S.set(this, "sizes", e);
		}
		get type() {
			return S.get(this, "type");
		}
		set type(e) {
			S.set(this, "type", e);
		}
	};
	V(su, iu);
	var Jd = class extends C {
		constructor(e, t = "track") {
			super(e, t);
		}
	}, Pd = class extends C {
		constructor(e, t = "marquee") {
			super(e, t);
		}
	};
	const Wd = {
		HTMLElement: C,
		HTMLTemplateElement: pa,
		HTMLHtmlElement: Oh,
		HTMLScriptElement: Sa,
		HTMLFrameElement: Jh,
		HTMLIFrameElement: ya,
		HTMLObjectElement: Ph,
		HTMLHeadElement: Wh,
		HTMLBodyElement: Yh,
		HTMLStyleElement: Ra,
		HTMLTimeElement: Na,
		HTMLFieldSetElement: jh,
		HTMLEmbedElement: Xh,
		HTMLHRElement: qh,
		HTMLProgressElement: $h,
		HTMLParagraphElement: zh,
		HTMLTableElement: ed,
		HTMLFrameSetElement: td,
		HTMLLIElement: nd,
		HTMLBaseElement: rd,
		HTMLDataListElement: sd,
		HTMLInputElement: _a,
		HTMLParamElement: id,
		HTMLMediaElement: od,
		HTMLAudioElement: ad,
		HTMLHeadingElement: ka,
		HTMLDirectoryElement: ud,
		HTMLQuoteElement: ld,
		HTMLCanvasElement: Ha,
		HTMLLegendElement: pd,
		HTMLOptionElement: Oa,
		HTMLSpanElement: md,
		HTMLMeterElement: Sd,
		HTMLVideoElement: bd,
		HTMLTableCellElement: yd,
		HTMLTitleElement: Ja,
		HTMLOutputElement: wd,
		HTMLTableRowElement: Cd,
		HTMLDataElement: Id,
		HTMLMenuElement: vd,
		HTMLSelectElement: Wa,
		HTMLBRElement: Ed,
		HTMLButtonElement: Ua,
		HTMLMapElement: xd,
		HTMLOptGroupElement: Td,
		HTMLDListElement: Bd,
		HTMLTextAreaElement: Za,
		HTMLFontElement: Qd,
		HTMLDivElement: Rd,
		HTMLLinkElement: Xa,
		HTMLSlotElement: $a,
		HTMLFormElement: Nd,
		HTMLImageElement: Ns,
		HTMLPreElement: Dd,
		HTMLUListElement: _d,
		HTMLMetaElement: tu,
		HTMLPictureElement: Fd,
		HTMLAreaElement: kd,
		HTMLOListElement: Md,
		HTMLTableCaptionElement: Gd,
		HTMLAnchorElement: ru,
		HTMLLabelElement: Hd,
		HTMLUnknownElement: Ld,
		HTMLModElement: Od,
		HTMLDetailsElement: Kd,
		HTMLSourceElement: iu,
		HTMLTrackElement: Jd,
		HTMLMarqueeElement: Pd
	}, Jn = { test: () => !0 }, Yd = {
		"text/html": {
			docType: "<!DOCTYPE html>",
			ignoreCase: !0,
			voidElements: /^(?:area|base|br|col|embed|hr|img|input|keygen|link|menuitem|meta|param|source|track|wbr)$/i
		},
		"image/svg+xml": {
			docType: "<?xml version=\"1.0\" encoding=\"utf-8\"?>",
			ignoreCase: !1,
			voidElements: Jn
		},
		"text/xml": {
			docType: "<?xml version=\"1.0\" encoding=\"utf-8\"?>",
			ignoreCase: !1,
			voidElements: Jn
		},
		"application/xml": {
			docType: "<?xml version=\"1.0\" encoding=\"utf-8\"?>",
			ignoreCase: !1,
			voidElements: Jn
		},
		"application/xhtml+xml": {
			docType: "<?xml version=\"1.0\" encoding=\"utf-8\"?>",
			ignoreCase: !1,
			voidElements: Jn
		}
	};
	var ou = class extends at {
		constructor(e, t = {}) {
			super(e, t), this.detail = t.detail;
		}
	}, Ud = class extends at {
		constructor(e, t = {}) {
			super(e, t), this.inputType = t.inputType, this.data = t.data, this.dataTransfer = t.dataTransfer, this.isComposing = t.isComposing || !1, this.ranges = t.ranges;
		}
	};
	const Vd = (e) => class extends Ns {
		constructor(n, r) {
			switch (super(e), arguments.length) {
				case 1:
					this.height = n, this.width = n;
					break;
				case 2: this.height = r, this.width = n;
			}
		}
	}, au = ({ [se]: e, [v]: t }, n = null) => {
		zi(e[X], t[w]);
		do {
			const r = oe(e), s = r === t ? r : r[w];
			n ? n.insertBefore(e, n[v]) : e.remove(), e = s;
		} while (e !== t);
	};
	var Zd = class Fu {
		constructor() {
			this[se] = null, this[v] = null, this.commonAncestorContainer = null;
		}
		insertNode(t) {
			this[v].parentNode.insertBefore(t, this[se]);
		}
		selectNode(t) {
			this[se] = t, this[v] = oe(t);
		}
		selectNodeContents(t) {
			this.selectNode(t), this.commonAncestorContainer = t;
		}
		surroundContents(t) {
			t.replaceChildren(this.extractContents());
		}
		setStartBefore(t) {
			this[se] = t;
		}
		setStartAfter(t) {
			this[se] = t.nextSibling;
		}
		setEndBefore(t) {
			this[v] = oe(t.previousSibling);
		}
		setEndAfter(t) {
			this[v] = oe(t);
		}
		cloneContents() {
			let { [se]: t, [v]: n } = this;
			const r = t.ownerDocument.createDocumentFragment();
			for (; t !== n;) r.insertBefore(t.cloneNode(!0), r[v]), t = oe(t), t !== n && (t = t[w]);
			return r;
		}
		deleteContents() {
			au(this);
		}
		extractContents() {
			const t = this[se].ownerDocument.createDocumentFragment();
			return au(this, t), t;
		}
		createContextualFragment(t) {
			const { commonAncestorContainer: n } = this, r = "ownerSVGElement" in n, s = r ? n.ownerDocument : n;
			let i = eo(s, t);
			if (r) {
				const o = [...i.childNodes];
				i = s.createDocumentFragment(), Object.setPrototypeOf(i, Yt.prototype), i.ownerSVGElement = s;
				for (const a of o) Object.setPrototypeOf(a, Yt.prototype), a.ownerSVGElement = s, i.appendChild(a);
			} else this.selectNode(i);
			return i;
		}
		cloneRange() {
			const t = new Fu();
			return t[se] = this[se], t[v] = this[v], t;
		}
	};
	const jd = ({ nodeType: e }, t) => {
		switch (e) {
			case 1: return t & 1;
			case 3: return t & 4;
			case 8: return t & 128;
			case 4: return t & 8;
		}
		return 0;
	};
	var Xd = class {
		constructor(e, t = -1) {
			this.root = e, this.currentNode = e, this.whatToShow = t;
			let { [w]: n, [v]: r } = e;
			if (e.nodeType === 9) {
				const { documentElement: i } = e;
				n = i, r = i[v];
			}
			const s = [];
			for (; n && n !== r;) jd(n, t) && s.push(n), n = n[w];
			this[ee] = {
				i: 0,
				nodes: s
			};
		}
		nextNode() {
			const e = this[ee];
			return this.currentNode = e.i < e.nodes.length ? e.nodes[e.i++] : null, this.currentNode;
		}
	};
	const uu = (e, t, n) => {
		let { [w]: r, [v]: s } = t;
		return e.call({
			ownerDocument: t,
			[w]: r,
			[v]: s
		}, n);
	}, lu = vA({}, Lh, Wd, {
		CustomEvent: ou,
		Event: at,
		EventTarget: Or,
		InputEvent: Ud,
		NamedNodeMap: da,
		NodeList: Qe
	}), Pn = /* @__PURE__ */ new WeakMap();
	var ut = class extends us {
		constructor(e) {
			super(null, "#document", 9), this[be] = {
				active: !1,
				registry: null
			}, this[He] = {
				active: !1,
				class: null
			}, this[ft] = Yd[e], this[$e] = null, this[br] = null, this[An] = null, this[ye] = null, this[Rt] = null;
		}
		get defaultView() {
			return Pn.has(this) || Pn.set(this, new Proxy(globalThis, {
				set: (e, t, n) => {
					switch (t) {
						case "addEventListener":
						case "removeEventListener":
						case "dispatchEvent":
							this[Qt][t] = n;
							break;
						default: e[t] = n;
					}
					return !0;
				},
				get: (e, t) => {
					switch (t) {
						case "addEventListener":
						case "removeEventListener":
						case "dispatchEvent":
							if (!this[Qt]) {
								const n = this[Qt] = new Or();
								n.dispatchEvent = n.dispatchEvent.bind(n), n.addEventListener = n.addEventListener.bind(n), n.removeEventListener = n.removeEventListener.bind(n);
							}
							return this[Qt][t];
						case "document": return this;
						case "navigator": return { userAgent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.150 Safari/537.36" };
						case "window": return Pn.get(this);
						case "customElements": return this[be].registry || (this[be] = new RA(this)), this[be];
						case "performance": return e.performance;
						case "DOMParser": return this[br];
						case "Image": return this[ye] || (this[ye] = Vd(this)), this[ye];
						case "MutationObserver": return this[He].class || (this[He] = new FA(this)), this[He].class;
					}
					return this[An] && this[An][t] || lu[t] || e[t];
				}
			})), Pn.get(this);
		}
		get doctype() {
			const e = this[$e];
			if (e) return e;
			const { firstChild: t } = this;
			return t && t.nodeType === 10 ? this[$e] = t : null;
		}
		set doctype(e) {
			if (/^([a-z:]+)(\s+system|\s+public(\s+"([^"]+)")?)?(\s+"([^"]+)")?/i.test(e)) {
				const { $1: t, $4: n, $6: r } = RegExp;
				this[$e] = new _n(this, t, n, r), Sn(this, this[$e], this[w]);
			}
		}
		get documentElement() {
			return this.firstElementChild;
		}
		get isConnected() {
			return !0;
		}
		_getParent() {
			return this[Qt];
		}
		createAttribute(e) {
			return new kt(this, e);
		}
		createCDATASection(e) {
			return new Yr(this, e);
		}
		createComment(e) {
			return new Ur(this, e);
		}
		createDocumentFragment() {
			return new ls(this);
		}
		createDocumentType(e, t, n) {
			return new _n(this, e, t, n);
		}
		createElement(e) {
			return new Wt(this, e);
		}
		createRange() {
			const e = new Zd();
			return e.commonAncestorContainer = this, e;
		}
		createTextNode(e) {
			return new Jt(this, e);
		}
		createTreeWalker(e, t = -1) {
			return new Xd(e, t);
		}
		createNodeIterator(e, t = -1) {
			return this.createTreeWalker(e, t);
		}
		createEvent(e) {
			const t = EA(e === "Event" ? new at("") : new ou(""));
			return t.initEvent = t.initCustomEvent = (n, r = !1, s = !1, i) => {
				t.bubbles = !!r, xA(t, {
					type: { value: n },
					canBubble: { value: r },
					cancelable: { value: s },
					detail: { value: i }
				});
			}, t;
		}
		cloneNode(e = !1) {
			const { constructor: t, [be]: n, [$e]: r } = this, s = new t();
			if (s[be] = n, e) {
				const i = s[v], { childNodes: o } = this;
				for (let { length: a } = o, u = 0; u < a; u++) s.insertBefore(o[u].cloneNode(!0), i);
				r && (s[$e] = o[0]);
			}
			return s;
		}
		importNode(e) {
			const t = 1 < arguments.length && !!arguments[1], n = e.cloneNode(t), { [be]: r } = this, { active: s } = r, i = (o) => {
				const { ownerDocument: a, nodeType: u } = o;
				o.ownerDocument = this, s && a !== this && u === 1 && r.upgrade(o);
			};
			if (i(n), t) switch (n.nodeType) {
				case 1:
				case 11: {
					let { [w]: o, [v]: a } = n;
					for (; o !== a;) o.nodeType === 1 && i(o), o = o[w];
					break;
				}
			}
			return n;
		}
		toString() {
			return this.childNodes.join("");
		}
		querySelector(e) {
			return uu(super.querySelector, this, e);
		}
		querySelectorAll(e) {
			return uu(super.querySelectorAll, this, e);
		}
		getElementsByTagNameNS(e, t) {
			return this.getElementsByTagName(t);
		}
		createAttributeNS(e, t) {
			return this.createAttribute(t);
		}
		createElementNS(e, t, n) {
			return e === "http://www.w3.org/2000/svg" ? new Yt(this, t, null) : this.createElement(t, n);
		}
	};
	re(lu.Document = function() {
		le();
	}, ut).prototype = ut.prototype;
	const qd = (e, t, n, r) => {
		if (!t && wn.has(n)) return new (wn.get(n))(e, n);
		const { [be]: { active: s, registry: i } } = e;
		if (s) {
			const o = t ? r.is : n;
			if (i.has(o)) {
				const { Class: a } = i.get(o), u = new a(e, n);
				return tt.set(u, { connected: !1 }), u;
			}
		}
		return new C(e, n);
	};
	var $d = class extends ut {
		constructor() {
			super("text/html");
		}
		get all() {
			const e = new Qe();
			let { [w]: t, [v]: n } = this;
			for (; t !== n;) t.nodeType === 1 && e.push(t), t = t[w];
			return e;
		}
		get head() {
			const { documentElement: e } = this;
			let { firstElementChild: t } = e;
			return (!t || t.tagName !== "HEAD") && (t = this.createElement("head"), e.prepend(t)), t;
		}
		get body() {
			const { head: e } = this;
			let { nextElementSibling: t } = e;
			return (!t || t.tagName !== "BODY") && (t = this.createElement("body"), e.after(t)), t;
		}
		get title() {
			const { head: e } = this;
			return e.getElementsByTagName("title").at(0)?.textContent || "";
		}
		set title(e) {
			const { head: t } = this;
			let n = t.getElementsByTagName("title").at(0);
			n ? n.textContent = e : t.insertBefore(this.createElement("title"), t.firstChild).textContent = e;
		}
		createElement(e, t) {
			const n = !!(t && t.is), r = qd(this, n, e, t);
			return n && r.setAttribute("is", t.is), r;
		}
	}, zd = class extends ut {
		constructor() {
			super("image/svg+xml");
		}
		toString() {
			return this[ft].docType + super.toString();
		}
	}, eg = class extends ut {
		constructor() {
			super("text/xml");
		}
		toString() {
			return this[ft].docType + super.toString();
		}
	}, tg = class ku {
		parseFromString(t, n, r = null) {
			let s = !1, i;
			return n === "text/html" ? (s = !0, i = new $d()) : n === "image/svg+xml" ? i = new zd() : i = new eg(), i[br] = ku, r && (i[An] = r), s && t === "..." && (t = "<!doctype html><html><head></head><body></body></html>"), t ? io(i, s, t) : i;
		}
	};
	const { parse: Eg } = JSON, ng = (e, t = null) => new tg().parseFromString(e, "text/html", t).defaultView;
	function rg() {
		le();
	}
	re(rg, ut).prototype = ut.prototype;
	const sg = "http://127.0.0.1:19876", Au = /* @__PURE__ */ new Map(), ig = `
var cheerio = { load: function(html) {
  var docId = __cheerio_load(html);
  var $ = function(selector, context) {
    if (selector && selector._elementIds) return selector;
    if (context && context._elementIds) {
      var ids = [];
      context._elementIds.forEach(function(id) { ids = ids.concat(JSON.parse(__cheerio_find(docId, id, selector))); });
      return wrap(docId, ids);
    }
    return wrap(docId, selector);
  };
  $.html = function(el) { return __cheerio_html(docId, el && el._elementIds && el._elementIds.length ? el._elementIds[0] : ''); };
  return $;
} };
function wrap(docId, selectorOrIds) {
  var ids = Array.isArray(selectorOrIds) ? selectorOrIds : (typeof selectorOrIds === 'string' ? JSON.parse(__cheerio_select(docId, selectorOrIds)) : []);
  var wrapper = {
    _docId: docId, _elementIds: ids, length: ids.length,
    each: function(cb) { ids.forEach(function(id, i) { cb.call(wrap(docId, [id]), i, wrap(docId, [id])); }); return wrapper; },
    find: function(sel) { var found = []; ids.forEach(function(id) { found = found.concat(JSON.parse(__cheerio_find(docId, id, sel))); }); return wrap(docId, found); },
    text: function() { return ids.length ? __cheerio_text(docId, ids.join(',')) : ''; },
    html: function() { return ids.length ? __cheerio_inner_html(ids[0]) : ''; },
    attr: function(name) { if (!ids.length) return undefined; var value = __cheerio_attr(docId, ids[0], name); return value === '__UNDEFINED__' ? undefined : value; },
    first: function() { return wrap(docId, ids.length ? [ids[0]] : []); },
    last: function() { return wrap(docId, ids.length ? [ids[ids.length - 1]] : []); },
    next: function() { return wrap(docId, ids.map(function(id) { return __cheerio_next(docId, id); }).filter(function(id) { return id !== '__NONE__'; })); },
    prev: function() { return wrap(docId, ids.map(function(id) { return __cheerio_prev(docId, id); }).filter(function(id) { return id !== '__NONE__'; })); },
    eq: function(i) { return wrap(docId, i >= 0 && i < ids.length ? [ids[i]] : []); },
    get: function(i) { return typeof i === 'number' ? (i >= 0 && i < ids.length ? wrap(docId, [ids[i]]) : undefined) : ids.map(function(id) { return wrap(docId, [id]); }); },
    map: function(cb) { var values = ids.map(function(id, i) { return cb.call(wrap(docId, [id]), i, wrap(docId, [id])); }).filter(function(value) { return value !== undefined && value !== null; }); return { length: values.length, get: function(i) { return typeof i === 'number' ? values[i] : values; }, toArray: function() { return values; } }; },
    filter: function(value) { if (typeof value !== 'function') return wrapper; return wrap(docId, ids.filter(function(id, i) { return value.call(wrap(docId, [id]), i, wrap(docId, [id])); })); }
  };
  return wrapper;
}`, og = `
globalThis.global = globalThis; globalThis.window = globalThis; globalThis.self = globalThis;
if (!globalThis.console) globalThis.console = {};
if (!globalThis.console.log) globalThis.console.log = function() {};
if (!globalThis.console.warn) globalThis.console.warn = function() {};
if (!globalThis.console.error) globalThis.console.error = function() {};
if (!globalThis.URLSearchParams) { globalThis.URLSearchParams = function(value) { this._pairs = []; var self = this; if (typeof value === 'string') value.replace(/^\\?/, '').split('&').forEach(function(part) { if (!part) return; var p = part.split('='); self.append(decodeURIComponent(p[0]), decodeURIComponent(p.slice(1).join('='))); }); }; URLSearchParams.prototype.append = function(k, v) { this._pairs.push([String(k), String(v)]); }; URLSearchParams.prototype.get = function(k) { var p = this._pairs.find(function(pair) { return pair[0] === String(k); }); return p ? p[1] : null; }; URLSearchParams.prototype.toString = function() { return this._pairs.map(function(p) { return encodeURIComponent(p[0]) + '=' + encodeURIComponent(p[1]); }).join('&'); }; }
if (!globalThis.atob) globalThis.atob = function(value) { var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='; var output = ''; var buffer = 0; var bits = 0; String(value).replace(/=+$/, '').split('').forEach(function(char) { var index = chars.indexOf(char); if (index < 0) return; buffer = (buffer << 6) | index; bits += 6; if (bits >= 8) { bits -= 8; output += String.fromCharCode((buffer >> bits) & 255); } }); return output; };
if (!globalThis.btoa) globalThis.btoa = function(value) { var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='; var output = ''; var index = 0; while (index < String(value).length) { var a = String(value).charCodeAt(index++); var b = String(value).charCodeAt(index++); var c = String(value).charCodeAt(index++); var n = (a << 16) | ((b || 0) << 8) | (c || 0); output += chars[(n >> 18) & 63] + chars[(n >> 12) & 63] + (index - 2 > String(value).length ? '=' : chars[(n >> 6) & 63]) + (index - 1 > String(value).length ? '=' : chars[n & 63]); } return output; };
`, cu = /* @__PURE__ */ new Map(), ve = /* @__PURE__ */ new Map();
	let Zt = 0;
	function ag() {
		return `dom_${Zt++}`;
	}
	function ug(e) {
		const t = (n, r) => {
			const s = e.newFunction(n, (...i) => e.newString(r(...i.map((o) => e.getString(o)))));
			e.setProp(e.global, n, s), s.dispose();
		};
		t("__cheerio_load", (n) => {
			const r = ag();
			return cu.set(r, ng(n).document), r;
		}), t("__cheerio_select", (n, r) => JSON.stringify([...cu.get(n)?.querySelectorAll(r) || []].map((s) => {
			const i = `${n}:${Zt++}`;
			return ve.set(i, s), i;
		}))), t("__cheerio_find", (n, r, s) => JSON.stringify([...ve.get(r)?.querySelectorAll(s) || []].map((i) => {
			const o = `${n}:${Zt++}`;
			return ve.set(o, i), o;
		}))), t("__cheerio_text", (n, r) => r.split(",").map((s) => ve.get(s)?.textContent || "").join(" ")), t("__cheerio_html", (n, r) => r && ve.get(r)?.outerHTML || ""), t("__cheerio_inner_html", (n) => ve.get(n)?.innerHTML || ""), t("__cheerio_attr", (n, r) => ve.get(n)?.getAttribute(r) ?? "__UNDEFINED__"), t("__cheerio_next", (n, r) => {
			const s = ve.get(r)?.nextElementSibling;
			if (!s) return "__NONE__";
			const i = `${n}:${Zt++}`;
			return ve.set(i, s), i;
		}), t("__cheerio_prev", (n, r) => {
			const s = ve.get(r)?.previousElementSibling;
			if (!s) return "__NONE__";
			const i = `${n}:${Zt++}`;
			return ve.set(i, s), i;
		});
	}
	async function lg(e) {
		let t = Au.get(e.scraperId);
		if (!t) {
			t = await hl(), Au.set(e.scraperId, t), ug(t);
			const u = t.newFunction("__native_fetch", (f, g, y, B) => {
				const N = t.getString(f), z = t.getString(g), W = JSON.parse(t.getString(y)), F = t.getString(B), G = t.newPromise();
				return G.settled.then(() => t.runtime.executePendingJobs()), (async () => {
					let H = null;
					try {
						H = await fetch(N, {
							method: z,
							headers: W,
							body: F || void 0,
							signal: AbortSignal.timeout(8e3)
						});
					} catch {
						try {
							H = await fetch(`${sg}/proxy?url=${encodeURIComponent(N)}&h=${encodeURIComponent(JSON.stringify(W))}`, {
								method: z,
								body: F || void 0,
								signal: AbortSignal.timeout(4e3)
							});
						} catch {
							H = null;
						}
					}
					const Ne = H ? {
						ok: H.ok,
						status: H.status,
						statusText: H.statusText,
						url: H.url || N,
						headers: {},
						body: await H.text()
					} : {
						ok: !1,
						status: 0,
						statusText: "NetworkError",
						url: N,
						headers: {},
						body: ""
					};
					G.resolve(t.newString(JSON.stringify(Ne))), t.runtime.executePendingJobs();
				})().catch(() => {
					G.resolve(t.newString(JSON.stringify({
						ok: !1,
						status: 0,
						statusText: "NetworkError",
						url: N,
						headers: {},
						body: ""
					}))), t.runtime.executePendingJobs();
				}), G.handle;
			});
			t.setProp(t.global, "__native_fetch", u), u.dispose();
			const l = `${og}${ig}globalThis.fetch = function(url, options) { options = options || {}; return Promise.resolve(__native_fetch(String(url), String(options.method || 'GET'), JSON.stringify(options.headers || {}), options.body == null ? '' : String(options.body))).then(function(raw) { var data = JSON.parse(raw); return { ok: data.ok, status: data.status, statusText: data.statusText || '', url: data.url, headers: data.headers || {}, text: function() { return Promise.resolve(data.body); }, json: function() { try { return Promise.resolve(JSON.parse(data.body)); } catch (_) { return Promise.resolve(null); } }, clone: function() { return this; } }; }); }; globalThis.require = function(name) { if (String(name).indexOf('cheerio') !== -1) return cheerio; throw new Error('module not available: ' + name); };`, p = await t.evalCodeAsync(l, "plugin-setup.js");
			if ("error" in p) {
				const f = p.error;
				if (!f) throw new Error("plugin setup failed");
				const g = t.dump(f);
				throw f.dispose(), new Error(String(g));
			}
			p.value.dispose();
		}
		const n = `(async function() { try { globalThis.SCRAPER_ID = ${JSON.stringify(e.scraperId)}; globalThis.SCRAPER_SETTINGS = JSON.parse(${JSON.stringify(e.settingsJson || "{}")}); var module = { exports: {} }; var exports = module.exports; (function() { ${e.code}
 })(); var getStreams = module.exports.getStreams || globalThis.getStreams; if (!getStreams) return '[]'; return JSON.stringify(await getStreams(${JSON.stringify(e.tmdbId)}, ${JSON.stringify(e.mediaType)}, ${e.season == null ? "null" : e.season}, ${e.episode == null ? "null" : e.episode}) || []); } catch (_) { return '[]'; } })()`, r = await t.evalCodeAsync(n, `${e.scraperId}.js`);
		if ("error" in r) {
			const u = r.error;
			if (!u) throw new Error("plugin execution failed");
			const l = t.dump(u);
			throw u.dispose(), new Error(String(l));
		}
		const s = t.unwrapResult(r), i = await t.resolvePromise(s);
		if (s.dispose(), "error" in i) {
			const u = i.error;
			if (!u) throw new Error("plugin promise failed");
			const l = t.dump(u);
			throw u.dispose(), new Error(String(l));
		}
		const o = t.unwrapResult(i), a = t.getString(o);
		return o.dispose(), a;
	}
	self.onmessage = (e) => {
		lg(e.data).then((t) => self.postMessage({
			id: e.data.id,
			result: t
		})).catch((t) => self.postMessage({
			id: e.data.id,
			error: t instanceof Error ? t.message : String(t)
		}));
	};
})();
