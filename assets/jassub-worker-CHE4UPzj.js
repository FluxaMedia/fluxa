(function() {
	if (self.name.startsWith("em-pthread")) {
		const R = self.name.split("-").slice(2).join("-"), v = globalThis.fetch;
		globalThis.fetch = (Hr) => v(R), self.name = "em-pthread";
	}
	async function Ke(R = {}) {
		var v = R, Hr = !0, pr = !!globalThis.WorkerGlobalScope, gr = (...r) => console.log(...r), V = (...r) => console.error(...r);
		function Je() {
			m && zr();
		}
		var m = pr && globalThis.name == "em-pthread";
		pr && (Xe = self.location.href);
		var Xe;
		const Nr = !!WebAssembly.Memory.prototype.toResizableBuffer;
		if (z = () => {
			if (!(Nr && self.HEAPU8RAW)) {
				var r = Nr ? F.toResizableBuffer() : F.buffer;
				D = new Int8Array(r), Y = new Int16Array(r), P = new Uint8Array(r), K = new Uint16Array(r), U = new Int32Array(r), y = new Uint32Array(r), Mr = new Float32Array(r), H = new Float64Array(r), M = new BigInt64Array(r), Pr = new BigUint64Array(r), self.HEAPU8RAW = new Uint8Array(r), self.WASMMEMORY = F;
			}
		}, R.__out && (gr = R.__out), R.__err && (V = R.__err), !self.name.startsWith("em-pthread")) {
			const r = globalThis.Worker;
			globalThis.Worker = class extends r {
				constructor(e, t = {}) {
					super(e, {
						...t,
						name: "em-pthread-" + R.__url
					});
				}
			};
		}
		const qe = hr;
		hr = function() {
			qe(), jr = new Proxy({}, { get() {
				return (r) => Xr(T.toValue(r));
			} });
		};
		function Lr(r) {
			throw r;
		}
		class er {}
		class xe extends er {}
		function u() {
			F.buffer != D.buffer && z();
		}
		var zr;
		if (m) {
			let r = function(e) {
				try {
					var t = e.data, n = t.cmd;
					if (n == 1) {
						let a = [];
						self.onmessage = (i) => a.push(i), zr = () => {
							postMessage({ cmd: 3 });
							for (let i of a) r(i);
							self.onmessage = r;
						};
						for (const i of t.handlers) (!v[i] || v[i].proxy) && (v[i] = (...o) => {
							postMessage({
								cmd: 9,
								handler: i,
								args: o
							});
						}, i == "print" && (gr = v[i]), i == "printErr" && (V = v[i]));
						F = t.wasmMemory, z(), v.wasm = t.wasmModule, hr();
					} else if (n == 2) {
						st(t.pthread_ptr), Br(t.pthread_ptr, 0, 0, 1, 0, 0), h.threadInitTLS(), Or(t.pthread_ptr), mr || (Fe(), mr = !0);
						try {
							ot(t.start_routine, t.arg);
						} catch (a) {
							if (a != "unwind") throw a;
						}
					} else n == 4 ? mr && cr() : n && (V(`worker: received unknown command ${n}`), V(t));
				} catch (a) {
					throw Zr && Me(), a;
				}
			};
			var mr = !1;
			self.onunhandledrejection = (e) => {
				throw e.reason || e;
			}, self.onmessage = r;
		}
		var Zr = !1;
		function rt() {
			return F.buffer;
		}
		function z() {
			if (!D?.buffer?.growable) {
				var r = rt();
				D = new Int8Array(r), Y = new Int16Array(r), P = new Uint8Array(r), K = new Uint16Array(r), U = new Int32Array(r), y = new Uint32Array(r), Mr = new Float32Array(r), H = new Float64Array(r), M = new BigInt64Array(r), Pr = new BigUint64Array(r);
			}
		}
		function et() {
			m || (F = new WebAssembly.Memory({
				initial: 512,
				maximum: 32768,
				shared: !0
			}), z());
		}
		et();
		var D, Gr = (r) => {
			r.terminate(), r.onmessage = (e) => {};
		}, Qr = (r) => {
			var e = h.pthreads[r];
			h.returnWorkerToPool(e);
		}, tt = (r) => {
			for (; r.length > 0;) r.shift()(v);
		}, nt = [], Yr = (r) => {
			var e = h.getNewWorker();
			if (!e) return 6;
			h.pthreads[r.pthread_ptr] = e, e.pthread_ptr = r.pthread_ptr;
			var t = {
				cmd: 2,
				start_routine: r.startRoutine,
				arg: r.arg,
				pthread_ptr: r.pthread_ptr
			};
			return e.postMessage(t, r.transferList), 0;
		}, tr = () => He(), Z = (r) => Be(r), at = (r) => De(r), H, M, W = (r, e, t, ...n) => {
			var a = 8 * n.length * 2, i = tr(), o = at(a), s = o >> 3;
			for (var l of n) typeof l == "bigint" ? ((u(), M)[s++] = 1n, (u(), M)[s++] = l) : ((u(), M)[s++] = 0n, (u(), H)[s++] = l);
			var f = Oe(r, e, a, o, t);
			return Z(i), f;
		};
		function yr(r) {
			if (m) return W(0, 0, 1, r);
			throw `exit(${r})`;
		}
		var it = yr, Kr = !Atomics.waitAsync || globalThis.navigator?.userAgent && Number((navigator.userAgent.match(/Chrom(e|ium)\/([0-9]+)\./) || [])[2]) < 91, U, h = {
			unusedWorkers: [],
			tlsInitFunctions: [],
			pthreads: {},
			init() {
				m || h.initMainThread();
			},
			initMainThread() {
				for (var r = !navigator.userAgent.toLowerCase().includes("firefox") && self.crossOriginIsolated ? Math.min(Math.max(0, navigator.hardwareConcurrency - 2), 8) : 0; r--;) h.allocateUnusedWorker();
			},
			terminateAllThreads: () => {
				for (var r of Object.values(h.pthreads)) Gr(r);
				for (var r of h.unusedWorkers) Gr(r);
				h.unusedWorkers = [], h.pthreads = {};
			},
			terminateRuntime: () => {
				h.terminateAllThreads();
				var r = vr();
				Se(0, 0, 0, 1), Kr || Atomics.notify((u(), U), r >> 2);
			},
			returnWorkerToPool: (r) => {
				var e = r.pthread_ptr;
				delete h.pthreads[e], h.unusedWorkers.push(r), r.pthread_ptr = 0, Re(e);
			},
			threadInitTLS() {
				h.tlsInitFunctions.forEach((r) => r());
			},
			loadWasmModuleToWorker: (r) => new Promise((e) => {
				r.onmessage = (a) => {
					var i = a.data, o = i.cmd;
					if (i.targetThread && i.targetThread != vr()) {
						h.pthreads[i.targetThread]?.postMessage(i);
						return;
					}
					if (i === "setimmediate" || i === "_si") {
						r.postMessage(i);
						return;
					}
					switch (o) {
						case 4:
							cr();
							break;
						case 5:
							Yr(i);
							break;
						case 6:
							Ur(() => Qr(i.thread));
							break;
						case 3:
							e(r);
							break;
						case 9:
							v[i.handler](...i.args);
							break;
						default: o && V(`worker sent an unknown command ${o}`);
					}
				}, r.onerror = (a) => {
					throw V(`worker sent an error! ${a.filename}:${a.lineno}: ${a.message}`), a;
				};
				var t = [];
				for (var n of []) v.propertyIsEnumerable(n) && t.push(n);
				r.postMessage({
					cmd: 1,
					handlers: t,
					wasmMemory: F,
					wasmModule: Ze
				});
			}),
			async loadWasmModuleToAllWorkers() {
				if (!m) return Promise.all(h.unusedWorkers.map(h.loadWasmModuleToWorker));
			},
			allocateUnusedWorker() {
				var r = new Worker(self.location.href, {
					type: "module",
					name: "em-pthread"
				});
				return h.unusedWorkers.push(r), r;
			},
			getNewWorker() {
				if (h.unusedWorkers.length == 0) {
					var r = h.allocateUnusedWorker();
					h.loadWasmModuleToWorker(r);
				}
				return h.unusedWorkers.pop();
			}
		}, y;
		function st(r) {
			var e = (u(), y)[r + 48 >> 2], t = e - (u(), y)[r + 52 >> 2];
			je(e, t), Z(e);
		}
		var Jr = [], G = (r) => {
			var e = Jr[r];
			return e || (Jr[r] = e = Ne.get(r)), e;
		}, ot = (r, e) => {
			var t = G(r)(e);
			function n(a) {
				Ve(a);
			}
			n(t);
		}, br = (r) => {
			for (var e = 0, t = 0; t < r.length; ++t) {
				var n = r.charCodeAt(t);
				n <= 127 ? e++ : n <= 2047 ? e += 2 : n >= 55296 && n <= 57343 ? (e += 4, ++t) : e += 3;
			}
			return e;
		}, lt = (r, e, t, n) => {
			if (!(n > 0)) return 0;
			for (var a = t, i = t + n - 1, o = 0; o < r.length; ++o) {
				var s = r.codePointAt(o);
				if (s <= 127) {
					if (t >= i) break;
					e[t++] = s;
				} else if (s <= 2047) {
					if (t + 1 >= i) break;
					e[t++] = 192 | s >> 6, e[t++] = 128 | s & 63;
				} else if (s <= 65535) {
					if (t + 2 >= i) break;
					e[t++] = 224 | s >> 12, e[t++] = 128 | s >> 6 & 63, e[t++] = 128 | s & 63;
				} else {
					if (t + 3 >= i) break;
					e[t++] = 240 | s >> 18, e[t++] = 128 | s >> 12 & 63, e[t++] = 128 | s >> 6 & 63, e[t++] = 128 | s & 63, o++;
				}
			}
			return e[t] = 0, t - a;
		}, P, $r = (r, e, t) => lt(r, (u(), P), e, t), Xr = (r) => {
			var e = br(r) + 1, t = _r(e);
			return t && $r(r, t, e), t;
		}, F, qr = new TextDecoder(), Tr = (r, e, t, n) => {
			var a = e + t;
			if (n) return a;
			for (; r[e] && !(e >= a);) ++e;
			return e;
		}, N = (r, e, t) => {
			if (!r) return "";
			var n = Tr((u(), P), r, e, t);
			return qr.decode((u(), P).slice(r, n));
		}, ut = (r, e, t, n) => Lr(`Assertion failed: ${N(r)}, at: ` + [
			e ? N(e) : "unknown filename",
			t,
			n ? N(n) : "unknown function"
		]);
		function xr(r, e, t, n) {
			return m ? W(1, 0, 1, r, e, t, n) : re(r, e, t, n);
		}
		var ft = () => !!globalThis.SharedArrayBuffer, re = (r, e, t, n) => {
			if (!ft()) return 6;
			var a = [], i = 0;
			if (m && (a.length === 0 || i)) return xr(r, e, t, n);
			if (i) return i;
			var o = {
				startRoutine: t,
				pthread_ptr: r,
				arg: n,
				transferList: a
			};
			return m ? (o.cmd = 5, postMessage(o, a), 0) : Yr(o);
		}, wr = {
			varargs: void 0,
			getStr(r) {
				return N(r);
			}
		};
		function ee(r, e, t) {
			return m ? W(2, 0, 1, r, e, t) : (wr.varargs = t, 0);
		}
		function te(r, e, t) {
			if (m) return W(3, 0, 1, r, e, t);
		}
		function ne(r, e, t) {
			return m ? W(4, 0, 1, r, e, t) : (wr.varargs = t, 0);
		}
		function ae(r, e, t, n) {
			if (m) return W(5, 0, 1, r, e, t, n);
			wr.varargs = n;
		}
		var ct = () => Lr(""), C = (r) => {
			for (var e = "";;) {
				var t = (u(), P)[r++];
				if (!t) return e;
				e += String.fromCharCode(t);
			}
		}, L = {}, j = {}, nr = {};
		class Q extends Error {
			constructor(e) {
				super(e), this.name = "BindingError";
			}
		}
		var d = (r) => {
			throw new Q(r);
		};
		function vt(r, e, t = {}) {
			var n = e.name;
			if (r || d(`type "${n}" must have a positive integer typeid pointer`), j.hasOwnProperty(r)) {
				if (t.ignoreDuplicateRegistrations) return;
				d(`Cannot register type '${n}' twice`);
			}
			if (j[r] = e, delete nr[r], L.hasOwnProperty(r)) {
				var a = L[r];
				delete L[r], a.forEach((i) => i());
			}
		}
		function S(r, e, t = {}) {
			return vt(r, e, t);
		}
		var Y, K, Pr, ie = (r, e, t) => {
			switch (e) {
				case 1: return t ? (n) => (u(), D)[n] : (n) => (u(), P)[n];
				case 2: return t ? (n) => (u(), Y)[n >> 1] : (n) => (u(), K)[n >> 1];
				case 4: return t ? (n) => (u(), U)[n >> 2] : (n) => (u(), y)[n >> 2];
				case 8: return t ? (n) => (u(), M)[n >> 3] : (n) => (u(), Pr)[n >> 3];
				default: throw new TypeError(`invalid integer width (${e}): ${r}`);
			}
		}, _t = (r, e, t, n, a) => {
			e = C(e);
			const i = n === 0n;
			let o = (s) => s;
			if (i) {
				const s = t * 8;
				o = (l) => BigInt.asUintN(s, l), a = o(a);
			}
			S(r, {
				name: e,
				fromWireType: o,
				toWireType: (s, l) => (typeof l == "number" && (l = BigInt(l)), l),
				readValueFromPointer: ie(e, t, !i),
				destructorFunction: null
			});
		}, dt = (r, e, t, n) => {
			e = C(e), S(r, {
				name: e,
				fromWireType: function(a) {
					return !!a;
				},
				toWireType: function(a, i) {
					return i ? t : n;
				},
				readValueFromPointer: function(a) {
					return this.fromWireType((u(), P)[a]);
				},
				destructorFunction: null
			});
		}, ht = (r) => ({
			count: r.count,
			deleteScheduled: r.deleteScheduled,
			preservePointerOnDelete: r.preservePointerOnDelete,
			ptr: r.ptr,
			ptrType: r.ptrType,
			smartPtr: r.smartPtr,
			smartPtrType: r.smartPtrType
		}), Cr = (r) => {
			function e(t) {
				return t.$$.ptrType.registeredClass.name;
			}
			d(e(r) + " instance already deleted");
		}, Wr = !1, se = (r) => {}, pt = (r) => {
			r.smartPtr ? r.smartPtrType.rawDestructor(r.smartPtr) : r.ptrType.registeredClass.rawDestructor(r.ptr);
		}, oe = (r) => {
			r.count.value -= 1, r.count.value === 0 && pt(r);
		}, J = (r) => globalThis.FinalizationRegistry ? (Wr = new FinalizationRegistry((e) => {
			oe(e.$$);
		}), J = (e) => {
			var t = e.$$;
			if (t.smartPtr) {
				var n = { $$: t };
				Wr.register(e, n, e);
			}
			return e;
		}, se = (e) => Wr.unregister(e), J(r)) : (J = (e) => e, r), ar = [], gt = () => {
			for (; ar.length;) {
				var r = ar.pop();
				r.$$.deleteScheduled = !1, r.delete();
			}
		}, le, mt = () => {
			let r = ir.prototype;
			Object.assign(r, {
				isAliasOf(t) {
					if (!(this instanceof ir) || !(t instanceof ir)) return !1;
					var n = this.$$.ptrType.registeredClass, a = this.$$.ptr;
					t.$$ = t.$$;
					for (var i = t.$$.ptrType.registeredClass, o = t.$$.ptr; n.baseClass;) a = n.upcast(a), n = n.baseClass;
					for (; i.baseClass;) o = i.upcast(o), i = i.baseClass;
					return n === i && a === o;
				},
				clone() {
					if (this.$$.ptr || Cr(this), this.$$.preservePointerOnDelete) return this.$$.count.value += 1, this;
					var t = J(Object.create(Object.getPrototypeOf(this), { $$: { value: ht(this.$$) } }));
					return t.$$.count.value += 1, t.$$.deleteScheduled = !1, t;
				},
				delete() {
					this.$$.ptr || Cr(this), this.$$.deleteScheduled && !this.$$.preservePointerOnDelete && d("Object already scheduled for deletion"), se(this), oe(this.$$), this.$$.preservePointerOnDelete || (this.$$.smartPtr = void 0, this.$$.ptr = void 0);
				},
				isDeleted() {
					return !this.$$.ptr;
				},
				deleteLater() {
					return this.$$.ptr || Cr(this), this.$$.deleteScheduled && !this.$$.preservePointerOnDelete && d("Object already scheduled for deletion"), ar.push(this), ar.length === 1 && le && le(gt), this.$$.deleteScheduled = !0, this;
				}
			});
			const e = Symbol.dispose;
			e && (r[e] = r.delete);
		};
		function ir() {}
		var Ar = (r, e) => Object.defineProperty(e, "name", { value: r }), ue = {}, fe = (r, e, t) => {
			if (r[e].overloadTable === void 0) {
				var n = r[e];
				r[e] = function(...a) {
					return r[e].overloadTable.hasOwnProperty(a.length) || d(`Function '${t}' called with an invalid number of arguments (${a.length}) - expects one of (${r[e].overloadTable})!`), r[e].overloadTable[a.length].apply(this, a);
				}, r[e].overloadTable = [], r[e].overloadTable[n.argCount] = n;
			}
		}, yt = (r, e, t) => {
			v.hasOwnProperty(r) ? ((t === void 0 || v[r].overloadTable !== void 0 && v[r].overloadTable[t] !== void 0) && d(`Cannot register public name '${r}' twice`), fe(v, r, r), v[r].overloadTable.hasOwnProperty(t) && d(`Cannot register multiple overloads of a function with the same number of arguments (${t})!`), v[r].overloadTable[t] = e) : (v[r] = e, v[r].argCount = t);
		}, bt = 48, $t = 57, Tt = (r) => {
			r = r.replace(/[^a-zA-Z0-9_]/g, "$");
			var e = r.charCodeAt(0);
			return e >= bt && e <= $t ? `_${r}` : r;
		};
		function wt(r, e, t, n, a, i, o, s) {
			this.name = r, this.constructor = e, this.instancePrototype = t, this.rawDestructor = n, this.baseClass = a, this.getActualType = i, this.upcast = o, this.downcast = s, this.pureVirtualFunctions = [];
		}
		var sr = (r, e, t) => {
			for (; e !== t;) e.upcast || d(`Expected null or instance of ${t.name}, got an instance of ${e.name}`), r = e.upcast(r), e = e.baseClass;
			return r;
		}, kr = (r) => {
			if (r === null) return "null";
			var e = typeof r;
			return e === "object" || e === "array" || e === "function" ? r.toString() : "" + r;
		};
		function Pt(r, e) {
			if (e === null) return this.isReference && d(`null is not a valid ${this.name}`), 0;
			e.$$ || d(`Cannot pass "${kr(e)}" as a ${this.name}`), e.$$.ptr || d(`Cannot pass deleted object as a pointer of type ${this.name}`);
			var t = e.$$.ptrType.registeredClass;
			return sr(e.$$.ptr, t, this.registeredClass);
		}
		function Ct(r, e) {
			var t;
			if (e === null) return this.isReference && d(`null is not a valid ${this.name}`), this.isSmartPointer ? (t = this.rawConstructor(), r !== null && r.push(this.rawDestructor, t), t) : 0;
			(!e || !e.$$) && d(`Cannot pass "${kr(e)}" as a ${this.name}`), e.$$.ptr || d(`Cannot pass deleted object as a pointer of type ${this.name}`), !this.isConst && e.$$.ptrType.isConst && d(`Cannot convert argument of type ${e.$$.smartPtrType ? e.$$.smartPtrType.name : e.$$.ptrType.name} to parameter type ${this.name}`);
			var n = e.$$.ptrType.registeredClass;
			if (t = sr(e.$$.ptr, n, this.registeredClass), this.isSmartPointer) switch (e.$$.smartPtr === void 0 && d("Passing raw pointer to smart pointer is illegal"), this.sharingPolicy) {
				case 0:
					e.$$.smartPtrType === this ? t = e.$$.smartPtr : d(`Cannot convert argument of type ${e.$$.smartPtrType ? e.$$.smartPtrType.name : e.$$.ptrType.name} to parameter type ${this.name}`);
					break;
				case 1:
					t = e.$$.smartPtr;
					break;
				case 2:
					if (e.$$.smartPtrType === this) t = e.$$.smartPtr;
					else {
						var a = e.clone();
						t = this.rawShare(t, T.toHandle(() => a.delete())), r !== null && r.push(this.rawDestructor, t);
					}
					break;
				default: d("Unsupported sharing policy");
			}
			return t;
		}
		function Wt(r, e) {
			if (e === null) return this.isReference && d(`null is not a valid ${this.name}`), 0;
			e.$$ || d(`Cannot pass "${kr(e)}" as a ${this.name}`), e.$$.ptr || d(`Cannot pass deleted object as a pointer of type ${this.name}`), e.$$.ptrType.isConst && d(`Cannot convert argument of type ${e.$$.ptrType.name} to parameter type ${this.name}`);
			var t = e.$$.ptrType.registeredClass;
			return sr(e.$$.ptr, t, this.registeredClass);
		}
		function or(r) {
			return this.fromWireType((u(), y)[r >> 2]);
		}
		var ce = (r, e, t) => {
			if (e === t) return r;
			if (t.baseClass === void 0) return null;
			var n = ce(r, e, t.baseClass);
			return n === null ? null : t.downcast(n);
		}, At = {}, kt = (r, e) => {
			for (e === void 0 && d("ptr should not be undefined"); r.baseClass;) e = r.upcast(e), r = r.baseClass;
			return e;
		}, Ft = (r, e) => (e = kt(r, e), At[e]);
		class St extends Error {
			constructor(e) {
				super(e), this.name = "InternalError";
			}
		}
		var lr = (r) => {
			throw new St(r);
		}, ur = (r, e) => ((!e.ptrType || !e.ptr) && lr("makeClassHandle requires ptr and ptrType"), !!e.smartPtrType != !!e.smartPtr && lr("Both smartPtrType and smartPtr must be specified"), e.count = { value: 1 }, J(Object.create(r, { $$: {
			value: e,
			writable: !0
		} })));
		function Mt(r) {
			var e = this.getPointee(r);
			if (!e) return this.destructor(r), null;
			var t = Ft(this.registeredClass, e);
			if (t !== void 0) {
				if (t.$$.count.value === 0) return t.$$.ptr = e, t.$$.smartPtr = r, t.clone();
				var n = t.clone();
				return this.destructor(r), n;
			}
			function a() {
				return this.isSmartPointer ? ur(this.registeredClass.instancePrototype, {
					ptrType: this.pointeeType,
					ptr: e,
					smartPtrType: this,
					smartPtr: r
				}) : ur(this.registeredClass.instancePrototype, {
					ptrType: this,
					ptr: r
				});
			}
			var i = ue[this.registeredClass.getActualType(e)];
			if (!i) return a.call(this);
			var o;
			this.isConst ? o = i.constPointerType : o = i.pointerType;
			var s = ce(e, this.registeredClass, o.registeredClass);
			return s === null ? a.call(this) : this.isSmartPointer ? ur(o.registeredClass.instancePrototype, {
				ptrType: o,
				ptr: s,
				smartPtrType: this,
				smartPtr: r
			}) : ur(o.registeredClass.instancePrototype, {
				ptrType: o,
				ptr: s
			});
		}
		var Ut = () => {
			Object.assign(fr.prototype, {
				getPointee(r) {
					return this.rawGetPointee && (r = this.rawGetPointee(r)), r;
				},
				destructor(r) {
					this.rawDestructor?.(r);
				},
				readValueFromPointer: or,
				fromWireType: Mt
			});
		};
		function fr(r, e, t, n, a, i, o, s, l, f, c) {
			this.name = r, this.registeredClass = e, this.isReference = t, this.isConst = n, this.isSmartPointer = a, this.pointeeType = i, this.sharingPolicy = o, this.rawGetPointee = s, this.rawConstructor = l, this.rawShare = f, this.rawDestructor = c, !a && e.baseClass === void 0 ? n ? (this.toWireType = Pt, this.destructorFunction = null) : (this.toWireType = Wt, this.destructorFunction = null) : this.toWireType = Ct;
		}
		var Ot = (r, e, t) => {
			v.hasOwnProperty(r) || lr("Replacing nonexistent public symbol"), v[r].overloadTable !== void 0 && t !== void 0 ? v[r].overloadTable[t] = e : (v[r] = e, v[r].argCount = t);
		}, E = (r, e, t = !1) => {
			r = C(r);
			function n() {
				return G(e);
			}
			var a = n();
			return typeof a != "function" && d(`unknown function pointer with signature ${r}: ${e}`), a;
		};
		class Rt extends Error {}
		var ve = (r) => {
			var e = ke(r), t = C(e);
			return O(e), t;
		}, X = (r, e) => {
			var t = [], n = {};
			function a(i) {
				if (!n[i] && !j[i]) {
					if (nr[i]) {
						nr[i].forEach(a);
						return;
					}
					t.push(i), n[i] = !0;
				}
			}
			throw e.forEach(a), new Rt(`${r}: ` + t.map(ve).join([", "]));
		}, B = (r, e, t) => {
			r.forEach((s) => nr[s] = e);
			function n(s) {
				var l = t(s);
				l.length !== r.length && lr("Mismatched type converter count");
				for (var f = 0; f < r.length; ++f) S(r[f], l[f]);
			}
			var a = new Array(e.length), i = [], o = 0;
			for (let [s, l] of e.entries()) j.hasOwnProperty(l) ? a[s] = j[l] : (i.push(l), L.hasOwnProperty(l) || (L[l] = []), L[l].push(() => {
				a[s] = j[l], ++o, o === i.length && n(a);
			}));
			i.length === 0 && n(a);
		}, Vt = (r, e, t, n, a, i, o, s, l, f, c, _, p) => {
			c = C(c), i = E(a, i), s &&= E(o, s), f &&= E(l, f), p = E(_, p);
			var g = Tt(c);
			yt(g, function() {
				X(`Cannot construct ${c} due to unbound types`, [n]);
			}), B([
				r,
				e,
				t
			], n ? [n] : [], (b) => {
				b = b[0];
				var $, w;
				n ? ($ = b.registeredClass, w = $.instancePrototype) : w = ir.prototype;
				var k = Ar(c, function(...Dr) {
					if (Object.getPrototypeOf(this) !== rr) throw new Q(`Use 'new' to construct ${c}`);
					if (A.constructor_body === void 0) throw new Q(`${c} has no accessible constructor`);
					var Ye = A.constructor_body[Dr.length];
					if (Ye === void 0) throw new Q(`Tried to invoke ctor of ${c} with invalid number of parameters (${Dr.length}) - expected (${Object.keys(A.constructor_body).toString()}) parameters instead!`);
					return Ye.apply(this, Dr);
				}), rr = Object.create(w, { constructor: { value: k } });
				k.prototype = rr;
				var A = new wt(c, k, rr, p, $, i, s, f);
				A.baseClass && (A.baseClass.__derivedClasses ??= [], A.baseClass.__derivedClasses.push(A));
				var ea = new fr(c, A, !0, !1, !1), Ge = new fr(c + "*", A, !1, !1, !1), Qe = new fr(c + " const*", A, !1, !0, !1);
				return ue[r] = {
					pointerType: Ge,
					constPointerType: Qe
				}, Ot(g, k), [
					ea,
					Ge,
					Qe
				];
			});
		}, _e = (r, e) => {
			for (var t = [], n = 0; n < r; n++) t.push((u(), y)[e + n * 4 >> 2]);
			return t;
		}, Fr = (r) => {
			for (; r.length;) {
				var e = r.pop();
				r.pop()(e);
			}
		};
		function Et(r) {
			for (var e = 1; e < r.length; ++e) if (r[e] !== null && r[e].destructorFunction === void 0) return !0;
			return !1;
		}
		var It = {
			ftfnnn: function(e, t, n, a, i, o, s, l, f, c) {
				return function(_, p, g) {
					return o(n(a, l(null, _), f(null, p), c(null, g)));
				};
			},
			tffn: function(e, t, n, a, i, o, s) {
				return function() {
					n(a, s(null, this));
				};
			},
			tffnn: function(e, t, n, a, i, o, s, l) {
				return function(f) {
					n(a, s(null, this), l(null, f));
				};
			},
			tffnnnnn: function(e, t, n, a, i, o, s, l, f, c, _) {
				return function(p, g, b, $) {
					n(a, s(null, this), l(null, p), f(null, g), c(null, b), _(null, $));
				};
			},
			tffnnn: function(e, t, n, a, i, o, s, l, f) {
				return function(c, _) {
					n(a, s(null, this), l(null, c), f(null, _));
				};
			},
			ttfnn: function(e, t, n, a, i, o, s, l) {
				return function(f) {
					return o(n(a, s(null, this), l(null, f)));
				};
			},
			tffnt: function(e, t, n, a, i, o, s, l, f) {
				return function(c) {
					var _ = s(null, this), p = l(null, c);
					n(a, _, p), f(p);
				};
			},
			tffntnn: function(e, t, n, a, i, o, s, l, f, c, _) {
				return function(p, g, b) {
					var $ = s(null, this), w = l(null, p);
					n(a, $, w, f(null, g), c(null, b)), _(w);
				};
			},
			tffnnnn: function(e, t, n, a, i, o, s, l, f, c) {
				return function(_, p, g) {
					n(a, s(null, this), l(null, _), f(null, p), c(null, g));
				};
			},
			ttfn: function(e, t, n, a, i, o, s) {
				return function() {
					return o(n(a, s(null, this)));
				};
			},
			ttfnnn: function(e, t, n, a, i, o, s, l, f) {
				return function(c, _) {
					return o(n(a, s(null, this), l(null, c), f(null, _)));
				};
			}
		};
		function jt(r, e, t, n) {
			const a = [
				e ? "t" : "f",
				t ? "t" : "f",
				n ? "t" : "f"
			];
			for (let i = e ? 1 : 2; i < r.length; ++i) {
				const o = r[i];
				let s = "";
				o.destructorFunction === void 0 ? s = "u" : o.destructorFunction === null ? s = "n" : s = "t", a.push(s);
			}
			return a.join("");
		}
		function de(r, e, t, n, a, i) {
			var o = e.length;
			o < 2 && d("argTypes array size mismatch! Must at least get return value and receiver (this) types!");
			for (var s = e[1] !== null && t !== null, l = Et(e), f = !e[0].isVoid, c = e[0], _ = e[1], p = [
				r,
				d,
				n,
				a,
				Fr,
				c.fromWireType.bind(c),
				_?.toWireType.bind(_)
			], g = 2; g < o; ++g) {
				var b = e[g];
				p.push(b.toWireType.bind(b));
			}
			if (!l) for (var g = s ? 1 : 2; g < e.length; ++g) e[g].destructorFunction !== null && p.push(e[g].destructorFunction);
			return Ar(r, It[jt(e, s, f, i)](...p));
		}
		var Bt = (r, e, t, n, a, i) => {
			var o = _e(e, t);
			a = E(n, a), B([], [r], (s) => {
				s = s[0];
				var l = `constructor ${s.name}`;
				if (s.registeredClass.constructor_body === void 0 && (s.registeredClass.constructor_body = []), s.registeredClass.constructor_body[e - 1] !== void 0) throw new Q(`Cannot register multiple constructors with identical number of parameters (${e - 1}) for class '${s.name}'! Overload resolution is currently only performed using the parameter count, not actual type info!`);
				return s.registeredClass.constructor_body[e - 1] = () => {
					X(`Cannot construct ${s.name} due to unbound types`, o);
				}, B([], o, (f) => (f.splice(1, 0, null), s.registeredClass.constructor_body[e - 1] = de(l, f, null, a, i), [])), [];
			});
		}, Dt = (r) => {
			r = r.trim();
			const e = r.indexOf("(");
			return e === -1 ? r : r.slice(0, e);
		}, Ht = (r, e, t, n, a, i, o, s, l, f) => {
			var c = _e(t, n);
			e = C(e), e = Dt(e), i = E(a, i, l), B([], [r], (_) => {
				_ = _[0];
				var p = `${_.name}.${e}`;
				e.startsWith("@@") && (e = Symbol[e.substring(2)]), s && _.registeredClass.pureVirtualFunctions.push(e);
				function g() {
					X(`Cannot call ${p} due to unbound types`, c);
				}
				var b = _.registeredClass.instancePrototype, $ = b[e];
				return $ === void 0 || $.overloadTable === void 0 && $.className !== _.name && $.argCount === t - 2 ? (g.argCount = t - 2, g.className = _.name, b[e] = g) : (fe(b, e, p), b[e].overloadTable[t - 2] = g), B([], c, (w) => {
					var k = de(p, w, _, i, o, l);
					return b[e].overloadTable === void 0 ? (k.argCount = t - 2, b[e] = k) : b[e].overloadTable[t - 2] = k, [];
				}), [];
			});
		}, he = (r, e, t) => (r instanceof Object || d(`${t} with invalid "this": ${r}`), r instanceof e.registeredClass.constructor || d(`${t} incompatible with "this" of type ${r.constructor.name}`), r.$$.ptr || d(`cannot call emscripten binding method ${t} on deleted object`), sr(r.$$.ptr, r.$$.ptrType.registeredClass, e.registeredClass)), Nt = (r, e, t, n, a, i, o, s, l, f) => {
			e = C(e), a = E(n, a), B([], [r], (c) => {
				c = c[0];
				var _ = `${c.name}.${e}`, p = {
					get() {
						X(`Cannot access ${_} due to unbound types`, [t, o]);
					},
					enumerable: !0,
					configurable: !0
				};
				return l ? p.set = () => X(`Cannot access ${_} due to unbound types`, [t, o]) : p.set = (g) => d(_ + " is a read-only property"), Object.defineProperty(c.registeredClass.instancePrototype, e, p), B([], l ? [t, o] : [t], (g) => {
					var b = g[0], $ = {
						get() {
							var k = he(this, c, _ + " getter");
							return b.fromWireType(a(i, k));
						},
						enumerable: !0
					};
					if (l) {
						l = E(s, l);
						var w = g[1];
						$.set = function(k) {
							var rr = he(this, c, _ + " setter"), A = [];
							l(f, rr, w.toWireType(A, k)), Fr(A);
						};
					}
					return Object.defineProperty(c.registeredClass.instancePrototype, e, $), [];
				}), [];
			});
		}, pe = [], I = [
			0,
			1,
			,
			1,
			null,
			1,
			!0,
			1,
			!1,
			1
		], Sr = (r) => {
			r > 9 && --I[r + 1] === 0 && (I[r], I[r] = void 0, pe.push(r));
		}, T = {
			toValue: (r) => (r || d(`Cannot use deleted val. handle = ${r}`), I[r]),
			toHandle: (r) => {
				switch (r) {
					case void 0: return 2;
					case null: return 4;
					case !0: return 6;
					case !1: return 8;
					default: {
						const e = pe.pop() || I.length;
						return I[e] = r, I[e + 1] = 1, e;
					}
				}
			}
		}, Lt = {
			name: "emscripten::val",
			fromWireType: (r) => {
				var e = T.toValue(r);
				return Sr(r), e;
			},
			toWireType: (r, e) => T.toHandle(e),
			readValueFromPointer: or,
			destructorFunction: null
		}, zt = (r) => S(r, Lt), Mr, Zt = (r, e) => {
			switch (e) {
				case 4: return function(t) {
					return this.fromWireType((u(), Mr)[t >> 2]);
				};
				case 8: return function(t) {
					return this.fromWireType((u(), H)[t >> 3]);
				};
				default: throw new TypeError(`invalid float width (${e}): ${r}`);
			}
		}, Gt = (r, e, t) => {
			e = C(e), S(r, {
				name: e,
				fromWireType: (n) => n,
				toWireType: (n, a) => a,
				readValueFromPointer: Zt(e, t),
				destructorFunction: null
			});
		}, Qt = (r, e, t, n, a) => {
			e = C(e);
			const i = n === 0;
			let o = (l) => l;
			if (i) {
				var s = 32 - 8 * t;
				o = (l) => l << s >>> s, a = o(a);
			}
			S(r, {
				name: e,
				fromWireType: o,
				toWireType: (l, f) => f,
				readValueFromPointer: ie(e, t, n !== 0),
				destructorFunction: null
			});
		}, Yt = (r, e, t) => {
			var n = [
				Int8Array,
				Uint8Array,
				Int16Array,
				Uint16Array,
				Int32Array,
				Uint32Array,
				Float32Array,
				Float64Array,
				BigInt64Array,
				BigUint64Array
			][e];
			function a(i) {
				var o = (u(), y)[i >> 2], s = (u(), y)[i + 4 >> 2];
				return new n((u(), D).buffer, s, o);
			}
			t = C(t), S(r, {
				name: t,
				fromWireType: a,
				readValueFromPointer: a
			}, { ignoreDuplicateRegistrations: !0 });
		}, Kt = (r, e) => {
			e = C(e);
			var t = !0;
			S(r, {
				name: e,
				fromWireType(n) {
					var a = (u(), y)[n >> 2], i = n + 4, o;
					if (t) o = N(i, a, !0);
					else {
						o = "";
						for (var s = 0; s < a; ++s) o += String.fromCharCode((u(), P)[i + s]);
					}
					return O(n), o;
				},
				toWireType(n, a) {
					a instanceof ArrayBuffer && (a = new Uint8Array(a));
					var i, o = typeof a == "string";
					o || ArrayBuffer.isView(a) && a.BYTES_PER_ELEMENT == 1 || d("Cannot pass non-string to std::string"), t && o ? i = br(a) : i = a.length;
					var s = _r(4 + i + 1), l = s + 4;
					if ((u(), y)[s >> 2] = i, o) if (t) $r(a, l, i + 1);
					else for (var f = 0; f < i; ++f) {
						var c = a.charCodeAt(f);
						c > 255 && (O(s), d("String has UTF-16 code units that do not fit in 8 bits")), (u(), P)[l + f] = c;
					}
					else (u(), P).set(a, l);
					return n !== null && n.push(O, s), s;
				},
				readValueFromPointer: or,
				destructorFunction(n) {
					O(n);
				}
			});
		}, Jt = new TextDecoder("utf-16le"), Xt = (r, e, t) => {
			var n = r >> 1, a = Tr((u(), K), n, e / 2, t);
			return Jt.decode((u(), K).slice(n, a));
		}, qt = (r, e, t = 2147483647) => {
			if (t < 2) return 0;
			t -= 2;
			for (var n = e, a = t < r.length * 2 ? t / 2 : r.length, i = 0; i < a; ++i) {
				var o = r.charCodeAt(i);
				(u(), Y)[e >> 1] = o, e += 2;
			}
			return (u(), Y)[e >> 1] = 0, e - n;
		}, xt = (r) => r.length * 2, rn = (r, e, t) => {
			for (var n = "", a = r >> 2, i = 0; !(i >= e / 4); i++) {
				var o = (u(), y)[a + i];
				if (!o && !t) break;
				n += String.fromCodePoint(o);
			}
			return n;
		}, en = (r, e, t = 2147483647) => {
			if (t < 4) return 0;
			for (var n = e, a = n + t - 4, i = 0; i < r.length; ++i) {
				var o = r.codePointAt(i);
				if (o > 65535 && i++, (u(), U)[e >> 2] = o, e += 4, e + 4 > a) break;
			}
			return (u(), U)[e >> 2] = 0, e - n;
		}, tn = (r) => {
			for (var e = 0, t = 0; t < r.length; ++t) r.codePointAt(t) > 65535 && t++, e += 4;
			return e;
		}, nn = (r, e, t) => {
			t = C(t);
			var n, a, i;
			e === 2 ? (n = Xt, a = qt, i = xt) : (n = rn, a = en, i = tn), S(r, {
				name: t,
				fromWireType: (o) => {
					var s = (u(), y)[o >> 2], l = n(o + 4, s * e, !0);
					return O(o), l;
				},
				toWireType: (o, s) => {
					typeof s != "string" && d(`Cannot pass non-string to C++ string type ${t}`);
					var l = i(s), f = _r(4 + l + e);
					return (u(), y)[f >> 2] = l / e, a(s, f + 4, l + e), o !== null && o.push(O, f), f;
				},
				readValueFromPointer: or,
				destructorFunction(o) {
					O(o);
				}
			});
		}, an = (r, e) => {
			e = C(e), S(r, {
				isVoid: !0,
				name: e,
				fromWireType: () => {},
				toWireType: (t, n) => {}
			});
		}, sn = (r) => {
			Br(r, !pr, 1, !Hr, 262144, !1), h.threadInitTLS();
		}, Ur = (r) => {
			r();
		}, Or = (r) => {
			if (!Kr) {
				Atomics.waitAsync((u(), U), r >> 2, r).value.then(cr);
				var e = r + 112;
				Atomics.store((u(), U), e >> 2, 1);
			}
		}, cr = () => {
			var r = vr();
			r && Ur(() => {
				Or(r), Ie();
			});
		}, on = (r, e) => {
			if (r == e) setTimeout(cr);
			else if (m) postMessage({
				targetThread: r,
				cmd: 4
			});
			else {
				var t = h.pthreads[r];
				if (!t) return;
				t.postMessage({ cmd: 4 });
			}
		}, Rr = [], ln = (r, e, t, n, a, i, o) => {
			Rr.length = 0;
			for (var s = a >> 3, l = a + n >> 3; s < l;) {
				var f;
				(u(), M)[s++] ? f = (u(), M)[s++] : f = (u(), H)[s++], Rr.push(f);
			}
			var c = e ? jr[e] : Yn[r];
			h.currentProxiedOperationCallerThread = t;
			var _ = c(...Rr);
			if (h.currentProxiedOperationCallerThread = 0, i) {
				_.then((p) => Ue(i, o, p));
				return;
			}
			return _;
		}, un = () => {}, fn = (r) => {
			m ? postMessage({
				cmd: 6,
				thread: r
			}) : Qr(r);
		}, cn = (r) => {}, vn = () => {
			throw new xe();
		}, Vr = [], _n = (r) => {
			var e = Vr.length;
			return Vr.push(r), e;
		}, dn = (r, e) => {
			var t = j[r];
			return t === void 0 && d(`${e} has unknown type ${ve(r)}`), t;
		}, hn = (r, e) => {
			for (var t = new Array(r), n = 0; n < r; ++n) t[n] = dn((u(), y)[e + n * 4 >> 2], `parameter ${n}`);
			return t;
		}, pn = (r, e, t) => {
			var n = [], a = r(n, t);
			return n.length && ((u(), y)[e >> 2] = T.toHandle(n)), a;
		}, gn = {}, ge = (r) => {
			var e = gn[r];
			return e === void 0 ? C(r) : e;
		}, mn = (r, e, t) => {
			var n = 8, [a, ...i] = hn(r, e), o = a.toWireType.bind(a), s = i.map((c) => c.readValueFromPointer.bind(c));
			r--;
			var l = new Array(r), f = (c, _, p, g) => {
				for (var b = 0, $ = 0; $ < r; ++$) l[$] = s[$](g + b), b += n;
				var w;
				switch (t) {
					case 0:
						w = T.toValue(c).apply(null, l);
						break;
					case 2:
						w = Reflect.construct(T.toValue(c), l);
						break;
					case 3:
						w = l[0];
						break;
					case 1: w = T.toValue(c)[ge(_)](...l);
				}
				return pn(o, p, w);
			};
			return _n(Ar(`methodCaller<(${i.map((c) => c.name)}) => ${a.name}>`, f));
		}, yn = (r, e) => (r = T.toValue(r), e = T.toValue(e), T.toHandle(r[e])), bn = (r) => {
			r > 9 && (I[r + 1] += 1);
		}, $n = (r, e, t, n, a) => Vr[r](e, t, n, a), Tn = () => T.toHandle([]), wn = (r) => T.toHandle(ge(r)), Pn = () => T.toHandle({}), Cn = (r) => {
			Fr(T.toValue(r)), Sr(r);
		}, Wn = (r, e, t) => {
			r = T.toValue(r), e = T.toValue(e), t = T.toValue(t), r[e] = t;
		}, q = {}, me = () => performance.timeOrigin + performance.now();
		function ye(r, e) {
			return m ? W(6, 0, 1, r, e) : (q[r] && (clearTimeout(q[r].id), delete q[r]), e && (q[r] = {
				id: setTimeout(() => {
					delete q[r], Ur(() => Ee(r, me()));
				}, e),
				timeout_ms: e
			}), 0);
		}
		var An = () => Date.now(), kn = 9007199254740992, Fn = -9007199254740992, Sn = (r) => r < Fn || r > kn ? NaN : Number(r), Er = [], Mn = (r, e) => {
			Er.length = 0;
			for (var t; t = (u(), P)[r++];) {
				var n = t != 105;
				n &= t != 112, e += n && e % 8 ? 4 : 0, Er.push(t == 112 ? (u(), y)[e >> 2] : t == 106 ? (u(), M)[e >> 3] : t == 105 ? (u(), U)[e >> 2] : (u(), H)[e >> 3]), e += n ? 8 : 4;
			}
			return Er;
		}, Un = (r, e, t) => {
			var n = Mn(e, t);
			return jr[r](...n);
		}, On = (r, e, t) => Un(r, e, t), Rn = () => {}, Vn = (r) => V(N(r)), En = () => {
			throw "unwind";
		}, be = () => 2147483648, In = () => be(), jn = () => navigator.hardwareConcurrency, Bn = (r, e) => Math.ceil(r / e) * e, Dn = (r) => {
			var e = (r - F.buffer.byteLength + 65535) / 65536 | 0;
			try {
				return F.grow(e), z(), 1;
			} catch {}
		}, Hn = (r) => {
			var e = (u(), P).length;
			if (r >>>= 0, r <= e) return !1;
			var t = be();
			if (r > t) return !1;
			for (var n = 1; n <= 4; n *= 2) {
				var a = e * (1 + .2 / n);
				if (a = Math.min(a, r + 100663296), Dn(Math.min(t, Bn(Math.max(r, a), 65536)))) return !0;
			}
			return !1;
		}, Ir = {}, Nn = () => "./this.program", x = () => {
			if (!x.strings) {
				var r = {
					USER: "web_user",
					LOGNAME: "web_user",
					PATH: "/",
					PWD: "/",
					HOME: "/home/web_user",
					LANG: (globalThis.navigator?.language ?? "C").replace("-", "_") + ".UTF-8",
					_: Nn()
				};
				for (var e in Ir) Ir[e] === void 0 ? delete r[e] : r[e] = Ir[e];
				var t = [];
				for (var e in r) t.push(`${e}=${r[e]}`);
				x.strings = t;
			}
			return x.strings;
		};
		function $e(r, e) {
			if (m) return W(7, 0, 1, r, e);
			var t = 0, n = 0;
			for (var a of x()) {
				var i = e + t;
				(u(), y)[r + n >> 2] = i, t += $r(a, i, 1 / 0) + 1, n += 4;
			}
			return 0;
		}
		function Te(r, e) {
			if (m) return W(8, 0, 1, r, e);
			var t = x();
			(u(), y)[r >> 2] = t.length;
			var n = 0;
			for (var a of t) n += br(a) + 1;
			return (u(), y)[e >> 2] = n, 0;
		}
		function we(r) {
			return m ? W(9, 0, 1, r) : 52;
		}
		function Pe(r, e, t, n) {
			return m ? W(10, 0, 1, r, e, t, n) : 52;
		}
		function Ce(r, e, t, n) {
			return m ? W(11, 0, 1, r, e, t, n) : (e = Sn(e), 70);
		}
		var Ln = [
			null,
			[],
			[]
		], zn = (r, e = 0, t, n) => {
			var a = Tr(r, e, t, n);
			return qr.decode(r.buffer ? r.buffer instanceof ArrayBuffer ? r.subarray(e, a) : r.slice(e, a) : new Uint8Array(r.slice(e, a)));
		}, Zn = (r, e) => {
			var t = Ln[r];
			e === 0 || e === 10 ? ((r === 1 ? gr : V)(zn(t)), t.length = 0) : t.push(e);
		};
		function We(r, e, t, n) {
			if (m) return W(12, 0, 1, r, e, t, n);
			for (var a = 0, i = 0; i < t; i++) {
				var o = (u(), y)[e >> 2], s = (u(), y)[e + 4 >> 2];
				e += 8;
				for (var l = 0; l < s; l++) Zn(r, (u(), P)[o + l]);
				a += s;
			}
			return (u(), y)[n >> 2] = a, 0;
		}
		var Gn = () => (r) => (r.set(crypto.getRandomValues(new Uint8Array(r.byteLength))), 0), Ae = (r) => (Ae = Gn())(r), Qn = (r, e) => Ae((u(), P).subarray(r, r + e));
		h.init(), mt(), Ut();
		var Yn = [
			yr,
			xr,
			ee,
			te,
			ne,
			ae,
			ye,
			$e,
			Te,
			we,
			Pe,
			Ce,
			We
		], jr = { 614285: (r) => Xr(T.toValue(r)) }, vr, O, _r, ke, Fe, Br, Se, Me, Ue, Oe, Re, Ve, Ee, Ie, dr, je, Be, De, He, Ne;
		function Kn(r) {
			v.__ZdlPvm = r.ia, vr = r.ja, v.__Znwm = r.ka, O = r.la, _r = v._malloc = r.ma, v._calloc = r.na, ke = r.oa, Fe = r.pa, v._emscripten_builtin_free = r.qa, r.ra, Br = r.ta, Se = r.ua, Me = r.va, v.___libc_free = r.wa, v._emscripten_builtin_malloc = r.xa, v.___libc_malloc = r.ya, Ue = r.za, Oe = r.Aa, Re = r.Ba, Ve = r.Ca, Ee = r.Da, Ie = r.Ea, v.__ZdaPv = r.Fa, v.__ZdaPvm = r.Ga, v.__ZdlPv = r.Ha, v.__Znam = r.Ia, v.__ZnamSt11align_val_t = r.Ja, v.__ZnwmSt11align_val_t = r.Ka, v.___libc_calloc = r.La, v.___libc_realloc = r.Ma, v._emscripten_builtin_calloc = r.Na, v._emscripten_builtin_realloc = r.Oa, v._malloc_size = r.Pa, v._malloc_usable_size = r.Qa, v._reallocf = r.Ra, dr = r.Sa, je = r.Ta, Be = r.Ua, De = r.Va, He = r.Wa, Ne = r.sa;
		}
		var Le;
		function Jn() {
			Le = {
				b: ut,
				Q: re,
				y: ee,
				P: te,
				Y: ne,
				z: ae,
				L: ct,
				B: _t,
				ba: dt,
				ga: Vt,
				fa: Bt,
				d: Ht,
				da: Nt,
				$: zt,
				A: Gt,
				l: Qt,
				e: Yt,
				aa: Kt,
				t: nn,
				ca: an,
				T: sn,
				M: on,
				x: ln,
				H: un,
				v: fn,
				S: Or,
				_: cn,
				F: vn,
				h: mn,
				c: Sr,
				ea: yn,
				i: bn,
				g: $n,
				p: Tn,
				j: wn,
				s: Pn,
				f: Cn,
				k: Wn,
				I: ye,
				n: On,
				w: Rn,
				m: An,
				u: Vn,
				Z: En,
				N: In,
				o: me,
				O: jn,
				J: Hn,
				U: $e,
				V: Te,
				q: it,
				r: we,
				X: Pe,
				R: Ce,
				W: We,
				E: Xn,
				C: xn,
				D: qn,
				a: F,
				G: yr,
				K: Qn
			};
		}
		function Xn(r, e, t) {
			var n = tr();
			try {
				return G(r)(e, t);
			} catch (a) {
				if (Z(n), !(a instanceof er)) throw a;
				dr(1, 0);
			}
		}
		function qn(r, e, t, n, a) {
			var i = tr();
			try {
				return G(r)(e, t, n, a);
			} catch (o) {
				if (Z(i), !(o instanceof er)) throw o;
				dr(1, 0);
			}
		}
		function xn(r, e, t, n) {
			var a = tr();
			try {
				return G(r)(e, t, n);
			} catch (i) {
				if (Z(a), !(i instanceof er)) throw i;
				dr(1, 0);
			}
		}
		function ra(r) {
			Zr = !0, h.tlsInitFunctions.push(r.ra), !m && r.ha();
		}
		var ze, Ze;
		function hr() {
			Jn();
			var r = { a: Le };
			ze = WebAssembly.instantiateStreaming(fetch(new URL("/fluxa/assets/jassub-worker-VfCk7loQ.wasm", "" + self.location.href)), r).then((e) => {
				var t = (e.instance || e).exports;
				return Ze = e.module || v.wasm, Kn(t), tt(nt), ra(t), h.loadWasmModuleToAllWorkers().then(Je);
			});
		}
		return m || hr(), await ze, v;
	}
	globalThis.name == "em-pthread" && Ke();
})();
