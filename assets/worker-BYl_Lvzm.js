(function() {
	var ya = Object.defineProperty, ti = (i, r, s) => () => {
		if (s) throw s[0];
		try {
			return i && (r = i(i = 0)), r;
		} catch (h) {
			throw s = [h], h;
		}
	}, ii = (i, r) => {
		let s = {};
		for (var h in i) ya(s, h, {
			get: i[h],
			enumerable: !0
		});
		return r || ya(s, Symbol.toStringTag, { value: "Module" }), s;
	};
	const me = {
		RAW: "RAW",
		PROXY: "PROXY",
		THROW: "THROW",
		HANDLER: "HANDLER"
	}, $ = {
		GET: "GET",
		SET: "SET",
		APPLY: "APPLY",
		CONSTRUCT: "CONSTRUCT",
		RELEASE: "RELEASE"
	}, $e = Symbol("Abslink.proxy"), _a = Symbol("Abslink.releaseProxy"), pa = Symbol("Abslink.finalizer"), ye = Symbol("Abslink.thrown"), ba = (i) => typeof i == "object" && i !== null || typeof i == "function", Aa = /* @__PURE__ */ new Map([["proxy", {
		canHandle: (i) => ba(i) && $e in i,
		serialize(i, r) {
			const s = i[$e];
			return wa(i, r, s), [s, []];
		},
		deserialize(i, r) {
			return si(r, i);
		}
	}], ["throw", {
		canHandle: (i) => ba(i) && ye in i,
		serialize({ value: i }) {
			let r;
			return i instanceof Error ? r = {
				isError: !0,
				value: {
					message: i.message,
					name: i.name,
					stack: i.stack
				}
			} : r = {
				isError: !1,
				value: i
			}, [r, []];
		},
		deserialize(i) {
			throw i.isError ? Object.assign(new Error(i.value.message), i.value) : i.value;
		}
	}]]);
	function oi(i, r) {
		let s = r;
		const h = i.slice(0, -1);
		for (const f of h) Object.prototype.hasOwnProperty.call(s, f) && (s = s[f]);
		const m = i[i.length - 1], v = m ? s[m] : s;
		return {
			parent: s,
			RawValue: v,
			lastSegment: m
		};
	}
	function wa(i, r, s) {
		return r.on("message", function h(m) {
			if (!m) return;
			const { id: v, type: f, path: p, markerID: C } = {
				path: [],
				...m
			};
			if (C !== s) return;
			const P = (m.argumentList ?? []).map((N) => Z(N, r));
			let b;
			try {
				const { parent: N, RawValue: k, lastSegment: G } = oi(p, i);
				switch (f) {
					case $.GET:
						b = k;
						break;
					case $.SET:
						N[G] = Z(m.value, r), b = !0;
						break;
					case $.APPLY:
						b = k.apply(N, P);
						break;
					case $.CONSTRUCT:
						b = new k(...P);
						break;
					case $.RELEASE:
						b = void 0;
						break;
					default: return;
				}
			} catch (N) {
				b = {
					value: N,
					[ye]: 0
				};
			}
			Promise.resolve(b).catch((N) => ({
				value: N,
				[ye]: 0
			})).then((N) => {
				f === $.CONSTRUCT && (N = hi(N));
				const [k, G] = Ae(N, r);
				r.postMessage({
					...k,
					id: v,
					markerID: s
				}, G), f === $.RELEASE && (r.off("message", h), i[pa]?.(), r.close?.());
			}).catch((N) => {
				const [k, G] = Ae({
					value: /* @__PURE__ */ new TypeError("Unserializable return value"),
					[ye]: 0
				}, r);
				r.postMessage({
					...k,
					id: v,
					markerID: s
				}, G);
			});
		}), i;
	}
	function si(i, r) {
		const s = /* @__PURE__ */ new Map();
		return i.on("message", (h) => {
			if (!h?.id) return;
			const m = s.get(h.id);
			if (m) try {
				m(h);
			} finally {
				s.delete(h.id);
			}
		}), He({
			endpoint: i,
			pendingListeners: s,
			rootMarkerID: r
		});
	}
	function _e(i) {
		if (i) throw new Error("Proxy has been released and is not useable");
	}
	async function Pa(i) {
		await re(i, { type: $.RELEASE }), i.endpoint.close?.();
	}
	const pe = /* @__PURE__ */ new WeakMap(), be = "FinalizationRegistry" in globalThis && new FinalizationRegistry((i) => {
		const r = (pe.get(i) ?? 0) - 1;
		pe.set(i, r), r === 0 && Pa(i).finally(() => {
			i.pendingListeners.clear();
		});
	});
	function li(i, r) {
		const s = (pe.get(r) ?? 0) + 1;
		pe.set(r, s), be && be.register(i, r, i);
	}
	function ui(i) {
		be && be.unregister(i);
	}
	function He(i, r = []) {
		let s = !1;
		const h = /* @__PURE__ */ new Map(), m = new Proxy(function() {}, {
			get(v, f) {
				if (_e(s), f === _a) return async () => {
					for (const P of h.values()) P[_a]();
					h.clear(), ui(m), Pa(i).finally(() => {
						i.pendingListeners.clear();
					}), s = !0;
				};
				if (f === "then") {
					if (r.length === 0) return { then: () => m };
					const P = re(i, {
						type: $.GET,
						path: r.map((b) => b.toString())
					}).then((b) => Z(b, i.endpoint));
					return P.then.bind(P);
				}
				const p = h.get(f);
				if (p) return p;
				const C = He(i, [...r, f]);
				return h.set(f, C), C;
			},
			set(v, f, p) {
				_e(s);
				const [C, P] = Ae(p, i.endpoint);
				return re(i, {
					type: $.SET,
					path: [...r, f].map((b) => b.toString()),
					value: C
				}, P).then((b) => Z(b, i.endpoint));
			},
			apply(v, f, p) {
				if (_e(s), r[r.length - 1] === "bind") return He(i, r.slice(0, -1));
				const [C, P] = Ca(p, i);
				return re(i, {
					type: $.APPLY,
					path: r.map((b) => b.toString()),
					argumentList: C
				}, P).then((b) => Z(b, i.endpoint));
			},
			construct(v, f) {
				_e(s);
				const [p, C] = Ca(f, i);
				return re(i, {
					type: $.CONSTRUCT,
					path: r.map((P) => P.toString()),
					argumentList: p
				}, C).then((P) => Z(P, i.endpoint));
			}
		});
		return li(m, i), m;
	}
	const ci = /* @__PURE__ */ new WeakMap();
	function Ca(i, r) {
		const s = [], h = [];
		for (const m of i) {
			const [v, f] = Ae(m, r.endpoint);
			s.push(v), h.push(...f);
		}
		return [s, h];
	}
	function hi(i) {
		return Object.assign(i, { [$e]: Ma() });
	}
	function Ae(i, r) {
		for (const [s, h] of Aa) if (h.canHandle(i)) {
			const [m, v] = h.serialize(i, r);
			return [{
				type: me.HANDLER,
				name: s,
				value: m
			}, v];
		}
		return [{
			type: me.RAW,
			value: i
		}, ci.get(i) ?? []];
	}
	function Z(i, r) {
		switch (i.type) {
			case me.HANDLER: return Aa.get(i.name).deserialize(i.value, r);
			case me.RAW: return i.value;
		}
	}
	function re(i, r, s) {
		return new Promise((h) => {
			const m = Ma();
			i.pendingListeners.set(m, h), i.endpoint.postMessage({
				id: m,
				...r,
				markerID: i.rootMarkerID
			}, s);
		});
	}
	const Ta = [], Ra = "0123456789abcdef";
	for (let i = 0; i < 256; i++) Ta[i] = Ra[i >> 4 & 15] + Ra[i & 15];
	let we = 0, Pe = "";
	function Ma() {
		let i = 0;
		if (!Pe || we + 16 > 512) for (Pe = "", we = 0; i < 256; ++i) Pe += Ta[Math.random() * 256 | 0];
		return Pe.substring(we, ++we + 16);
	}
	function di(i, r) {
		const s = /* @__PURE__ */ new WeakMap();
		return i.start?.(), r.start?.(), {
			on(h, m) {
				const v = (f) => m(f.data);
				"addEventListener" in i ? i.addEventListener(h, v) : "addListener" in i ? i.addListener(h, v) : i.on(h, v), s.set(m, v);
			},
			off(h, m) {
				const v = s.get(m);
				"removeEventListener" in i ? i.removeEventListener(h, v) : "removeListener" in i ? i.removeListener(h, v) : i.off(h, v), s.delete(m);
			},
			postMessage(h, m) {
				r.postMessage(h, m);
			},
			close() {
				i !== globalThis && i.close?.(), r !== globalThis && r.close?.();
			}
		};
	}
	function fi(i, r = globalThis, s = r) {
		return wa(i, di(r, s));
	}
	function ne(i) {
		"@babel/helpers - typeof";
		return ne = typeof Symbol == "function" && typeof Symbol.iterator == "symbol" ? function(r) {
			return typeof r;
		} : function(r) {
			return r && typeof Symbol == "function" && r.constructor === Symbol && r !== Symbol.prototype ? "symbol" : typeof r;
		}, ne(i);
	}
	function gi(i, r) {
		if (ne(i) != "object" || !i) return i;
		var s = i[Symbol.toPrimitive];
		if (s !== void 0) {
			var h = s.call(i, r || "default");
			if (ne(h) != "object") return h;
			throw new TypeError("@@toPrimitive must return a primitive value.");
		}
		return (r === "string" ? String : Number)(i);
	}
	function Si(i) {
		var r = gi(i, "string");
		return ne(r) == "symbol" ? r : r + "";
	}
	function S(i, r, s) {
		return (r = Si(r)) in i ? Object.defineProperty(i, r, {
			value: s,
			enumerable: !0,
			configurable: !0,
			writable: !0
		}) : i[r] = s, i;
	}
	function vi(i, r) {
		if (r.has(i)) throw new TypeError("Cannot initialize the same private elements twice on an object");
	}
	function Na(i, r, s) {
		vi(i, r), r.set(i, s);
	}
	function ka(i, r, s) {
		if (typeof i == "function" ? i === r : i.has(r)) return arguments.length < 3 ? r : s;
		throw new TypeError("Private element is not present on this object");
	}
	function Ba(i, r, s) {
		return i.set(ka(i, r), s), s;
	}
	function We(i, r) {
		return i.get(ka(i, r));
	}
	var mi = ii({ default: () => Ea }), Ea, yi = ti((() => {
		Ea = JSON.parse("[[\"ABeeZee\",[4,\"4i\"]],[\"ADLaM Display\",[4]],[\"AR One Sans\",[\"4a7\"]],[\"Abel\",[4]],[\"Abhaya Libre\",[4,5,6,7,8]],[\"Aboreto\",[4]],[\"Abril Fatface\",[4]],[\"Abyssinica SIL\",[4]],[\"Aclonica\",[4]],[\"Acme\",[4]],[\"Actor\",[4]],[\"Adamina\",[4]],[\"Advent Pro\",[\"1a9\",\"1a9i\"]],[\"Afacad\",[\"4a7\",\"4a7i\"]],[\"Afacad Flux\",[\"1a10\"]],[\"Agbalumo\",[4]],[\"Agdasima\",[4,7]],[\"Agu Display\",[4]],[\"Aguafina Script\",[4]],[\"Akatab\",[4,5,6,7,8,9]],[\"Akaya Kanadaka\",[4]],[\"Akaya Telivigala\",[4]],[\"Akronim\",[4]],[\"Akshar\",[\"3a7\"]],[\"Akt\",[\"1a9\"]],[\"Aladin\",[4]],[\"Alan Sans\",[\"3a9\"]],[\"Alata\",[4]],[\"Alatsi\",[4]],[\"Albert Sans\",[\"1a9\",\"1a9i\"]],[\"Aldrich\",[4]],[\"Alef\",[4,7]],[\"Alegreya\",[\"4a9\",\"4a9i\"]],[\"Alegreya SC\",[4,5,7,8,9,\"4i\",\"5i\",\"7i\",\"8i\",\"9i\"]],[\"Alegreya Sans\",[1,3,4,5,7,8,9,\"1i\",\"3i\",\"4i\",\"5i\",\"7i\",\"8i\",\"9i\"]],[\"Alegreya Sans SC\",[1,3,4,5,7,8,9,\"1i\",\"3i\",\"4i\",\"5i\",\"7i\",\"8i\",\"9i\"]],[\"Aleo\",[\"1a9\",\"1a9i\"]],[\"Alex Brush\",[4]],[\"Alexandria\",[\"1a9\"]],[\"Alfa Slab One\",[4]],[\"Alice\",[4]],[\"Alien Block\",[4]],[\"Alike\",[4]],[\"Alike Angular\",[4]],[\"Alkalami\",[4]],[\"Alkatra\",[\"4a7\"]],[\"Allan\",[4,7]],[\"Allerta\",[4]],[\"Allerta Stencil\",[4]],[\"Allison\",[4]],[\"Allkin\",[4]],[\"Allura\",[4]],[\"Almarai\",[3,4,7,8]],[\"Almendra\",[4,7,\"4i\",\"7i\"]],[\"Almendra Display\",[4]],[\"Almendra SC\",[4]],[\"Alumni Sans\",[\"1a9\",\"1a9i\"]],[\"Alumni Sans Collegiate One\",[4,\"4i\"]],[\"Alumni Sans Inline One\",[4,\"4i\"]],[\"Alumni Sans Pinstripe\",[4,\"4i\"]],[\"Alumni Sans SC\",[\"1a9\",\"1a9i\"]],[\"Alyamama\",[\"3a9\"]],[\"Amarante\",[4]],[\"Amaranth\",[4,7,\"4i\",\"7i\"]],[\"Amarna\",[\"1a7\",\"1a7i\"]],[\"Amatic SC\",[4,7]],[\"Amethysta\",[4]],[\"Amiko\",[4,6,7]],[\"Amiri\",[4,7,\"4i\",\"7i\"]],[\"Amiri Quran\",[4]],[\"Amita\",[4,7]],[\"Anaheim\",[\"4a8\"]],[\"Ancizar Sans\",[\"1a10\",\"1a10i\"]],[\"Ancizar Serif\",[\"3a9\",\"3a9i\"]],[\"Andada Pro\",[\"4a8\",\"4a8i\"]],[\"Andika\",[4,7,\"4i\",\"7i\"]],[\"Anek Bangla\",[\"1a8\"]],[\"Anek Devanagari\",[\"1a8\"]],[\"Anek Gujarati\",[\"1a8\"]],[\"Anek Gurmukhi\",[\"1a8\"]],[\"Anek Kannada\",[\"1a8\"]],[\"Anek Latin\",[\"1a8\"]],[\"Anek Malayalam\",[\"1a8\"]],[\"Anek Odia\",[\"1a8\"]],[\"Anek Tamil\",[\"1a8\"]],[\"Anek Telugu\",[\"1a8\"]],[\"Angkor\",[4]],[\"Annapurna SIL\",[4,7]],[\"Annie Use Your Telescope\",[4]],[\"Anonymous Pro\",[4,7,\"4i\",\"7i\"]],[\"Anta\",[4]],[\"Antic\",[4]],[\"Antic Didone\",[4]],[\"Antic Slab\",[4]],[\"Anton\",[4]],[\"Anton SC\",[4]],[\"Antonio\",[\"1a7\"]],[\"Anuphan\",[\"1a7\"]],[\"Anybody\",[\"1a9\",\"1a9i\"]],[\"Aoboshi One\",[4]],[\"Arapey\",[4,\"4i\"]],[\"Arbutus\",[4]],[\"Arbutus Slab\",[4]],[\"Architects Daughter\",[4]],[\"Archivo\",[\"1a9\",\"1a9i\"]],[\"Archivo Black\",[4]],[\"Archivo Narrow\",[\"4a7\",\"4a7i\"]],[\"Are You Serious\",[4]],[\"Aref Ruqaa\",[4,7]],[\"Aref Ruqaa Ink\",[4,7]],[\"Arima\",[\"1a7\"]],[\"Arimo\",[\"4a7\",\"4a7i\"]],[\"Arizonia\",[4]],[\"Armata\",[4]],[\"Arsenal\",[4,7,\"4i\",\"7i\"]],[\"Arsenal SC\",[4,7,\"4i\",\"7i\"]],[\"Artifika\",[4]],[\"Arvo\",[4,7,\"4i\",\"7i\"]],[\"Arya\",[4,7]],[\"Asap\",[\"1a9\",\"1a9i\"]],[\"Asap Condensed\",[2,3,4,5,6,7,8,9,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Asap Sharp\",[\"1a9\",\"1a9i\"]],[\"Asar\",[4]],[\"Asimovian\",[4]],[\"Asset\",[4]],[\"Assistant\",[\"2a8\"]],[\"Asta Sans\",[\"3a8\"]],[\"Astloch\",[4,7]],[\"Asul\",[4,7]],[\"Athiti\",[2,3,4,5,6,7]],[\"Atkinson Hyperlegible\",[4,7,\"4i\",\"7i\"]],[\"Atkinson Hyperlegible Mono\",[\"2a8\",\"2a8i\"]],[\"Atkinson Hyperlegible Next\",[\"2a8\",\"2a8i\"]],[\"Atma\",[3,4,5,6,7]],[\"Atomic Age\",[4]],[\"Aubrey\",[4]],[\"Audiowide\",[4]],[\"Autour One\",[4]],[\"Average\",[4]],[\"Average Sans\",[4]],[\"Averia Gruesa Libre\",[4]],[\"Averia Libre\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"Averia Sans Libre\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"Averia Serif Libre\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"Azeret Mono\",[\"1a9\",\"1a9i\"]],[\"B612\",[4,7,\"4i\",\"7i\"]],[\"B612 Mono\",[4,7,\"4i\",\"7i\"]],[\"BBH Bartle\",[4]],[\"BBH Bogle\",[4]],[\"BBH Hegarty\",[4]],[\"BIZ UDGothic\",[4,7]],[\"BIZ UDMincho\",[4,7]],[\"BIZ UDPGothic\",[4,7]],[\"BIZ UDPMincho\",[4,7]],[\"BJCree\",[4,5,6,7]],[\"Babylonica\",[4]],[\"Bacasime Antique\",[4]],[\"Bad Script\",[4]],[\"Badeen Display\",[4]],[\"Bagel Fat One\",[4]],[\"Bahiana\",[4]],[\"Bahianita\",[4]],[\"Bai Jamjuree\",[2,3,4,5,6,7,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Bakbak One\",[4]],[\"Ballet\",[4]],[\"Baloo 2\",[\"4a8\"]],[\"Baloo Bhai 2\",[\"4a8\"]],[\"Baloo Bhaijaan 2\",[\"4a8\"]],[\"Baloo Bhaina 2\",[\"4a8\"]],[\"Baloo Chettan 2\",[\"4a8\"]],[\"Baloo Da 2\",[\"4a8\"]],[\"Baloo Paaji 2\",[\"4a8\"]],[\"Baloo Tamma 2\",[\"4a8\"]],[\"Baloo Tammudu 2\",[\"4a8\"]],[\"Baloo Thambi 2\",[\"4a8\"]],[\"Balsamiq Sans\",[4,7,\"4i\",\"7i\"]],[\"Balthazar\",[4]],[\"Bangers\",[4]],[\"Barlow\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Barlow Condensed\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Barlow Semi Condensed\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Barriecito\",[4]],[\"Barrio\",[4]],[\"Basic\",[4]],[\"Baskervville\",[\"4a7\",\"4a7i\"]],[\"Baskervville SC\",[\"4a7\"]],[\"Battambang\",[1,3,4,7,9]],[\"Baumans\",[4]],[\"Bayon\",[4]],[\"Be Vietnam Pro\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Beau Rivage\",[4]],[\"Bebas Neue\",[4]],[\"Beiruti\",[\"2a9\"]],[\"Belanosima\",[4,6,7]],[\"Belgrano\",[4]],[\"Bellefair\",[4]],[\"Belleza\",[4]],[\"Bellota\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"Bellota Text\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"BenchNine\",[3,4,7]],[\"Benne\",[4]],[\"Bentham\",[4]],[\"Berkshire Swash\",[4]],[\"Besley\",[\"4a9\",\"4a9i\"]],[\"Betania Patmos\",[4]],[\"Betania Patmos GDL\",[4]],[\"Betania Patmos In\",[4]],[\"Betania Patmos In GDL\",[4]],[\"Beth Ellen\",[4]],[\"Bevan\",[4,\"4i\"]],[\"BhuTuka Expanded One\",[4]],[\"Big Shoulders\",[\"1a9\"]],[\"Big Shoulders Inline\",[\"1a9\"]],[\"Big Shoulders Stencil\",[\"1a9\"]],[\"Bigelow Rules\",[4]],[\"Bigshot One\",[4]],[\"Bilbo\",[4]],[\"Bilbo Swash Caps\",[4]],[\"BioRhyme\",[\"2a8\"]],[\"BioRhyme Expanded\",[2,3,4,7,8]],[\"Birthstone\",[4]],[\"Birthstone Bounce\",[4,5]],[\"Biryani\",[2,3,4,6,7,8,9]],[\"Bitcount\",[\"1a9\"]],[\"Bitcount Grid Double\",[\"1a9\"]],[\"Bitcount Grid Double Ink\",[\"1a9\"]],[\"Bitcount Grid Single\",[\"1a9\"]],[\"Bitcount Grid Single Ink\",[\"1a9\"]],[\"Bitcount Ink\",[\"1a9\"]],[\"Bitcount Prop Double\",[\"1a9\"]],[\"Bitcount Prop Double Ink\",[\"1a9\"]],[\"Bitcount Prop Single\",[\"1a9\"]],[\"Bitcount Prop Single Ink\",[\"1a9\"]],[\"Bitcount Single\",[\"1a9\"]],[\"Bitcount Single Ink\",[\"1a9\"]],[\"Bitter\",[\"1a9\",\"1a9i\"]],[\"Black And White Picture\",[4]],[\"Black Han Sans\",[4]],[\"Black Ops One\",[4]],[\"Blaka\",[4]],[\"Blaka Hollow\",[4]],[\"Blaka Ink\",[4]],[\"Blinker\",[1,2,3,4,6,7,8,9]],[\"Bodoni Moda\",[\"4a9\",\"4a9i\"]],[\"Bodoni Moda SC\",[\"4a9\",\"4a9i\"]],[\"Bokor\",[4]],[\"Boldonse\",[4]],[\"Bona Nova\",[4,7,\"4i\"]],[\"Bona Nova SC\",[4,7,\"4i\"]],[\"Bonbon\",[4]],[\"Bonheur Royale\",[4]],[\"Boogaloo\",[4]],[\"Borel\",[4]],[\"Bowlby One\",[4]],[\"Bowlby One SC\",[4]],[\"Bpmf Huninn\",[4]],[\"Bpmf Iansui\",[4]],[\"Bpmf Zihi Kai Std\",[4]],[\"Braah One\",[4]],[\"Brawler\",[4,7]],[\"Bree Serif\",[4]],[\"Bricolage Grotesque\",[\"2a8\"]],[\"Bruno Ace\",[4]],[\"Bruno Ace SC\",[4]],[\"Brygada 1918\",[\"4a7\",\"4a7i\"]],[\"Bubblegum Sans\",[4]],[\"Bubbler One\",[4]],[\"Buda\",[3]],[\"Buenard\",[\"4a7\"]],[\"Bungee\",[4]],[\"Bungee Hairline\",[4]],[\"Bungee Inline\",[4]],[\"Bungee Outline\",[4]],[\"Bungee Shade\",[4]],[\"Bungee Spice\",[4]],[\"Bungee Tint\",[4]],[\"Butcherman\",[4]],[\"Butterfly Kids\",[4]],[\"Bytesized\",[4]],[\"Caacupe One\",[4]],[\"Cabin\",[\"4a7\",\"4a7i\"]],[\"Cabin Condensed\",[4,5,6,7]],[\"Cabin Sketch\",[4,7]],[\"Cactus Classical Serif\",[4]],[\"Caesar Dressing\",[4]],[\"Cagliostro\",[4]],[\"Cairo\",[\"2a10\"]],[\"Cairo Play\",[\"2a10\"]],[\"Cal Sans\",[4]],[\"Caladea\",[4,7,\"4i\",\"7i\"]],[\"Calistoga\",[4]],[\"Calligraffitti\",[4]],[\"Cambay\",[4,7,\"4i\",\"7i\"]],[\"Cambo\",[4]],[\"Candal\",[4]],[\"Cantarell\",[4,7,\"4i\",\"7i\"]],[\"Cantata One\",[4]],[\"Cantora One\",[4]],[\"Caprasimo\",[4]],[\"Capriola\",[4]],[\"Caramel\",[4]],[\"Carattere\",[4]],[\"Cardo\",[4,7,\"4i\"]],[\"Carlito\",[4,7,\"4i\",\"7i\"]],[\"Carme\",[4]],[\"Carrois Gothic\",[4]],[\"Carrois Gothic SC\",[4]],[\"Carter One\",[4]],[\"Cascadia Code\",[\"2a7\",\"2a7i\"]],[\"Cascadia Mono\",[\"2a7\",\"2a7i\"]],[\"Castoro\",[4,\"4i\"]],[\"Castoro Titling\",[4]],[\"Catamaran\",[\"1a9\"]],[\"Caudex\",[4,7,\"4i\",\"7i\"]],[\"Cause\",[\"1a9\"]],[\"Caveat\",[\"4a7\"]],[\"Caveat Brush\",[4]],[\"Cedarville Cursive\",[4]],[\"Ceviche One\",[4]],[\"Chakra Petch\",[3,4,5,6,7,\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Changa\",[\"2a8\"]],[\"Changa One\",[4,\"4i\"]],[\"Chango\",[4]],[\"Charis SIL\",[4,7,\"4i\",\"7i\"]],[\"Charm\",[4,7]],[\"Charmonman\",[4,7]],[\"Chathura\",[1,3,4,7,8]],[\"Chau Philomene One\",[4,\"4i\"]],[\"Chela One\",[4]],[\"Chelsea Market\",[4]],[\"Chenla\",[4]],[\"Cherish\",[4]],[\"Cherry Bomb One\",[4]],[\"Cherry Cream Soda\",[4]],[\"Cherry Swash\",[4,7]],[\"Chewy\",[4]],[\"Chicle\",[4]],[\"Chilanka\",[4]],[\"Chiron GoRound TC\",[\"2a9\"]],[\"Chiron Hei HK\",[\"2a9\",\"2a9i\"]],[\"Chiron Sung HK\",[\"2a9\",\"2a9i\"]],[\"Chivo\",[\"1a9\",\"1a9i\"]],[\"Chivo Mono\",[\"1a9\",\"1a9i\"]],[\"Chocolate Classical Sans\",[4]],[\"Chokokutai\",[4]],[\"Chonburi\",[4]],[\"Cinzel\",[\"4a9\"]],[\"Cinzel Decorative\",[4,7,9]],[\"Clicker Script\",[4]],[\"Climate Crisis\",[4]],[\"Coda\",[4,8]],[\"Codystar\",[3,4]],[\"Coiny\",[4]],[\"Combo\",[4]],[\"Comfortaa\",[\"3a7\"]],[\"Comforter\",[4]],[\"Comforter Brush\",[4]],[\"Comic Neue\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"Comic Relief\",[4,7]],[\"Coming Soon\",[4]],[\"Comme\",[\"1a9\"]],[\"Commissioner\",[\"1a9\"]],[\"Concert One\",[4]],[\"Condiment\",[4]],[\"Content\",[4,7]],[\"Contrail One\",[4]],[\"Convergence\",[4]],[\"Cookie\",[4]],[\"Copse\",[4]],[\"Coral Pixels\",[4]],[\"Corben\",[4,7]],[\"Corinthia\",[4,7]],[\"Cormorant\",[\"3a7\",\"3a7i\"]],[\"Cormorant Garamond\",[\"3a7\",\"3a7i\"]],[\"Cormorant Infant\",[\"3a7\",\"3a7i\"]],[\"Cormorant SC\",[3,4,5,6,7]],[\"Cormorant Unicase\",[3,4,5,6,7]],[\"Cormorant Upright\",[3,4,5,6,7]],[\"Cossette Texte\",[4,7]],[\"Cossette Titre\",[4,7]],[\"Courgette\",[4]],[\"Courier Prime\",[4,7,\"4i\",\"7i\"]],[\"Cousine\",[4,7,\"4i\",\"7i\"]],[\"Coustard\",[4,9]],[\"Covered By Your Grace\",[4]],[\"Crafty Girls\",[4]],[\"Creepster\",[4]],[\"Crete Round\",[4,\"4i\"]],[\"Crimson Pro\",[\"2a9\",\"2a9i\"]],[\"Crimson Text\",[4,6,7,\"4i\",\"6i\",\"7i\"]],[\"Croissant One\",[4]],[\"Crushed\",[4]],[\"Cuprum\",[\"4a7\",\"4a7i\"]],[\"Cute Font\",[4]],[\"Cutive\",[4]],[\"Cutive Mono\",[4]],[\"DM Mono\",[3,4,5,\"3i\",\"4i\",\"5i\"]],[\"DM Sans\",[\"1a10\",\"1a10i\"]],[\"DM Serif Display\",[4,\"4i\"]],[\"DM Serif Text\",[4,\"4i\"]],[\"Dai Banna SIL\",[3,4,5,6,7,\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Damion\",[4]],[\"Dancing Script\",[\"4a7\"]],[\"Danfo\",[4]],[\"Dangrek\",[4]],[\"Darker Grotesque\",[\"3a9\"]],[\"Darumadrop One\",[4]],[\"Datatype\",[\"1a9\"]],[\"David Libre\",[4,5,7]],[\"Dawning of a New Day\",[4]],[\"Days One\",[4]],[\"Dekko\",[4]],[\"Dela Gothic One\",[4]],[\"Delicious Handrawn\",[4]],[\"Delius\",[4]],[\"Delius Swash Caps\",[4]],[\"Delius Unicase\",[4,7]],[\"Della Respira\",[4]],[\"Denk One\",[4]],[\"Devonshire\",[4]],[\"Dhurjati\",[4]],[\"Didact Gothic\",[4]],[\"Diphylleia\",[4]],[\"Diplomata\",[4]],[\"Diplomata SC\",[4]],[\"Do Hyeon\",[4]],[\"Dokdo\",[4]],[\"Domine\",[\"4a7\"]],[\"Donegal One\",[4]],[\"Dongle\",[3,4,7]],[\"Doppio One\",[4]],[\"Dorsa\",[4]],[\"Dosis\",[\"2a8\"]],[\"DotGothic16\",[4]],[\"Doto\",[\"1a9\"]],[\"Dr Sugiyama\",[4]],[\"Duru Sans\",[4]],[\"DynaPuff\",[\"4a7\"]],[\"Dynalight\",[4]],[\"EB Garamond\",[\"4a8\",\"4a8i\"]],[\"Eagle Lake\",[4]],[\"East Sea Dokdo\",[4]],[\"Eater\",[4]],[\"Economica\",[4,7,\"4i\",\"7i\"]],[\"Eczar\",[\"4a8\"]],[\"Edu AU VIC WA NT Arrows\",[\"4a7\"]],[\"Edu AU VIC WA NT Dots\",[\"4a7\"]],[\"Edu AU VIC WA NT Guides\",[\"4a7\"]],[\"Edu AU VIC WA NT Hand\",[\"4a7\"]],[\"Edu AU VIC WA NT Pre\",[\"4a7\"]],[\"Edu NSW ACT Cursive\",[\"4a7\"]],[\"Edu NSW ACT Foundation\",[\"4a7\"]],[\"Edu NSW ACT Hand Pre\",[\"4a7\"]],[\"Edu QLD Beginner\",[\"4a7\"]],[\"Edu QLD Hand\",[\"4a7\"]],[\"Edu SA Beginner\",[\"4a7\"]],[\"Edu SA Hand\",[\"4a7\"]],[\"Edu TAS Beginner\",[\"4a7\"]],[\"Edu VIC WA NT Beginner\",[\"4a7\"]],[\"Edu VIC WA NT Hand\",[\"4a7\"]],[\"Edu VIC WA NT Hand Pre\",[\"4a7\"]],[\"El Messiri\",[\"4a7\"]],[\"Electrolize\",[4]],[\"Elms Sans\",[\"1a9\",\"1a9i\"]],[\"Elsie\",[4,9]],[\"Elsie Swash Caps\",[4,9]],[\"Emblema One\",[4]],[\"Emilys Candy\",[4]],[\"Encode Sans\",[\"1a9\"]],[\"Encode Sans Condensed\",[1,2,3,4,5,6,7,8,9]],[\"Encode Sans Expanded\",[1,2,3,4,5,6,7,8,9]],[\"Encode Sans SC\",[\"1a9\"]],[\"Encode Sans Semi Condensed\",[1,2,3,4,5,6,7,8,9]],[\"Encode Sans Semi Expanded\",[1,2,3,4,5,6,7,8,9]],[\"Engagement\",[4]],[\"Englebert\",[4]],[\"Enriqueta\",[4,5,6,7]],[\"Ephesis\",[4]],[\"Epilogue\",[\"1a9\",\"1a9i\"]],[\"Epunda Sans\",[\"3a9\",\"3a9i\"]],[\"Epunda Slab\",[\"3a9\",\"3a9i\"]],[\"Erica One\",[4]],[\"Esteban\",[4]],[\"Estedad\",[\"1a9\"]],[\"Estonia\",[4]],[\"Euphoria Script\",[4]],[\"Ewert\",[4]],[\"Exile\",[4]],[\"Exo\",[\"1a9\",\"1a9i\"]],[\"Exo 2\",[\"1a9\",\"1a9i\"]],[\"Expletus Sans\",[\"4a7\",\"4a7i\"]],[\"Explora\",[4]],[\"Faculty Glyphic\",[4]],[\"Fahkwang\",[2,3,4,5,6,7,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Familjen Grotesk\",[\"4a7\",\"4a7i\"]],[\"Fanwood Text\",[4,\"4i\"]],[\"Farro\",[3,4,5,7]],[\"Farsan\",[4]],[\"Fascinate\",[4]],[\"Fascinate Inline\",[4]],[\"Faster One\",[4]],[\"Fasthand\",[4]],[\"Fauna One\",[4]],[\"Faustina\",[\"3a8\",\"3a8i\"]],[\"Federant\",[4]],[\"Federo\",[4]],[\"Felipa\",[4]],[\"Fenix\",[4]],[\"Festive\",[4]],[\"Figtree\",[\"3a9\",\"3a9i\"]],[\"Finger Paint\",[4]],[\"Finlandica Headline\",[\"1a9\",\"1a9i\"]],[\"Finlandica Text\",[\"1a9\",\"1a9i\"]],[\"Fira Code\",[\"3a7\"]],[\"Fira Mono\",[4,5,7]],[\"Fira Sans\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Fira Sans Condensed\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Fira Sans Extra Condensed\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Fjalla One\",[4]],[\"Fjord One\",[4]],[\"Flamenco\",[3,4]],[\"Flavors\",[4]],[\"Fleur De Leah\",[4]],[\"Flow Block\",[4]],[\"Flow Circular\",[4]],[\"Flow Rounded\",[4]],[\"Foldit\",[\"1a9\"]],[\"Fondamento\",[4,\"4i\"]],[\"Fontdiner Swanky\",[4]],[\"Forum\",[4]],[\"Fragment Mono\",[4,\"4i\"]],[\"Francois One\",[4]],[\"Frank Ruhl Libre\",[\"3a9\"]],[\"Fraunces\",[\"1a9\",\"1a9i\"]],[\"Freckle Face\",[4]],[\"Fredericka the Great\",[4]],[\"Fredoka\",[\"3a7\"]],[\"Freehand\",[4]],[\"Freeman\",[4]],[\"Fresca\",[4]],[\"Frijole\",[4]],[\"Fruktur\",[4,\"4i\"]],[\"Fugaz One\",[4]],[\"Fuggles\",[4]],[\"Funnel Display\",[\"3a8\"]],[\"Funnel Sans\",[\"3a8\",\"3a8i\"]],[\"Fustat\",[\"2a8\"]],[\"Fuzzy Bubbles\",[4,7]],[\"GFS Didot\",[4]],[\"GFS Neohellenic\",[4,7,\"4i\",\"7i\"]],[\"Ga Maamli\",[4]],[\"Gabarito\",[\"4a9\"]],[\"Gabriela\",[4]],[\"Gaegu\",[3,4,7]],[\"Gafata\",[4]],[\"Gajraj One\",[4]],[\"Galada\",[4]],[\"Galdeano\",[4]],[\"Galindo\",[4]],[\"Gamja Flower\",[4]],[\"Gantari\",[\"1a9\",\"1a9i\"]],[\"Gasoek One\",[4]],[\"Gayathri\",[1,4,7]],[\"Geist\",[\"1a9\",\"1a9i\"]],[\"Geist Mono\",[\"1a9\",\"1a9i\"]],[\"Geist Pixel\",[4]],[\"Gelasio\",[\"4a7\",\"4a7i\"]],[\"Gemunu Libre\",[\"2a8\"]],[\"Genos\",[\"1a9\",\"1a9i\"]],[\"Gentium Book Plus\",[4,7,\"4i\",\"7i\"]],[\"Gentium Plus\",[4,7,\"4i\",\"7i\"]],[\"Geo\",[4,\"4i\"]],[\"Geologica\",[\"1a9\"]],[\"Geom\",[\"3a9\",\"3a9i\"]],[\"Geomini\",[\"2a8\"]],[\"Georama\",[\"1a9\",\"1a9i\"]],[\"Geostar\",[4]],[\"Geostar Fill\",[4]],[\"Germania One\",[4]],[\"Gideon Roman\",[4]],[\"Gidole\",[4]],[\"Gidugu\",[4]],[\"Gilda Display\",[4]],[\"Girassol\",[4]],[\"Give You Glory\",[4]],[\"Glass Antiqua\",[4]],[\"Glegoo\",[4,7]],[\"Gloock\",[4]],[\"Gloria Hallelujah\",[4]],[\"Glory\",[\"1a8\",\"1a8i\"]],[\"Gluten\",[\"1a9\"]],[\"Goblin One\",[4]],[\"Gochi Hand\",[4]],[\"Goldman\",[4,7]],[\"Golos Text\",[\"4a9\"]],[\"Google Sans\",[\"4a7\",\"4a7i\"]],[\"Google Sans Code\",[\"3a8\",\"3a8i\"]],[\"Google Sans Flex\",[\"1a10\"]],[\"Gorditas\",[4,7]],[\"Gothic A1\",[1,2,3,4,5,6,7,8,9]],[\"Gotu\",[4]],[\"Goudy Bookletter 1911\",[4]],[\"Gowun Batang\",[4,7]],[\"Gowun Dodum\",[4]],[\"Graduate\",[4]],[\"Grand Hotel\",[4]],[\"Grandiflora One\",[4]],[\"Grandstander\",[\"1a9\",\"1a9i\"]],[\"Grape Nuts\",[4]],[\"Gravitas One\",[4]],[\"Great Vibes\",[4]],[\"Grechen Fuemen\",[4]],[\"Grenze\",[\"1a9\",\"1a9i\"]],[\"Grenze Gotisch\",[\"1a9\"]],[\"Grey Qo\",[4]],[\"Griffy\",[4]],[\"Gruppo\",[4]],[\"Gudea\",[4,7,\"4i\"]],[\"Gugi\",[4]],[\"Gulzar\",[4]],[\"Gupter\",[4,5,7]],[\"Gurajada\",[4]],[\"Gveret Levin\",[4]],[\"Gwendolyn\",[4,7]],[\"Habibi\",[4]],[\"Hachi Maru Pop\",[4]],[\"Hahmlet\",[\"1a9\"]],[\"Halant\",[3,4,5,6,7]],[\"Hammersmith One\",[4]],[\"Hanalei\",[4]],[\"Hanalei Fill\",[4]],[\"Handjet\",[\"1a9\"]],[\"Handlee\",[4]],[\"Hanken Grotesk\",[\"1a9\",\"1a9i\"]],[\"Hanuman\",[\"1a9\"]],[\"Happy Monkey\",[4]],[\"Harmattan\",[4,5,6,7]],[\"Headland One\",[4]],[\"Hedvig Letters Sans\",[4]],[\"Hedvig Letters Serif\",[4]],[\"Heebo\",[\"1a9\"]],[\"Henny Penny\",[4]],[\"Hepta Slab\",[\"1a9\"]],[\"Herr Von Muellerhoff\",[4]],[\"Hi Melody\",[4]],[\"Hibur Mono\",[4]],[\"Hina Mincho\",[4]],[\"Hind\",[3,4,5,6,7]],[\"Hind Guntur\",[3,4,5,6,7]],[\"Hind Madurai\",[3,4,5,6,7]],[\"Hind Mysuru\",[3,4,5,6,7]],[\"Hind Siliguri\",[3,4,5,6,7]],[\"Hind Vadodara\",[3,4,5,6,7]],[\"Holtwood One SC\",[4]],[\"Homemade Apple\",[4]],[\"Homenaje\",[4]],[\"Honk\",[4]],[\"Host Grotesk\",[\"3a8\",\"3a8i\"]],[\"Hubballi\",[4]],[\"Hubot Sans\",[\"2a9\",\"2a9i\"]],[\"Huninn\",[4]],[\"Hurricane\",[4]],[\"IBM Plex Mono\",[1,2,3,4,5,6,7,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"IBM Plex Sans\",[\"1a7\",\"1a7i\"]],[\"IBM Plex Sans Arabic\",[1,2,3,4,5,6,7]],[\"IBM Plex Sans Condensed\",[1,2,3,4,5,6,7,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"IBM Plex Sans Devanagari\",[1,2,3,4,5,6,7]],[\"IBM Plex Sans Hebrew\",[1,2,3,4,5,6,7]],[\"IBM Plex Sans JP\",[1,2,3,4,5,6,7]],[\"IBM Plex Sans KR\",[1,2,3,4,5,6,7]],[\"IBM Plex Sans Thai\",[1,2,3,4,5,6,7]],[\"IBM Plex Sans Thai Looped\",[1,2,3,4,5,6,7]],[\"IBM Plex Serif\",[1,2,3,4,5,6,7,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"IM Fell DW Pica\",[4,\"4i\"]],[\"IM Fell DW Pica SC\",[4]],[\"IM Fell Double Pica\",[4,\"4i\"]],[\"IM Fell Double Pica SC\",[4]],[\"IM Fell English\",[4,\"4i\"]],[\"IM Fell English SC\",[4]],[\"IM Fell French Canon\",[4,\"4i\"]],[\"IM Fell French Canon SC\",[4]],[\"IM Fell Great Primer\",[4,\"4i\"]],[\"IM Fell Great Primer SC\",[4]],[\"Iansui\",[4]],[\"Ibarra Real Nova\",[\"4a7\",\"4a7i\"]],[\"Iceberg\",[4]],[\"Iceland\",[4]],[\"Idiqlat\",[2,3,4]],[\"Imbue\",[\"1a9\"]],[\"Imperial Script\",[4]],[\"Imprima\",[4]],[\"Inclusive Sans\",[\"3a7\",\"3a7i\"]],[\"Inconsolata\",[\"2a9\"]],[\"Inder\",[4]],[\"Indie Flower\",[4]],[\"Ingrid Darling\",[4]],[\"Inika\",[4,7]],[\"Inknut Antiqua\",[3,4,5,6,7,8,9]],[\"Inria Sans\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"Inria Serif\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"Inspiration\",[4]],[\"Instrument Sans\",[\"4a7\",\"4a7i\"]],[\"Instrument Serif\",[4,\"4i\"]],[\"Intel One Mono\",[\"3a7\",\"3a7i\"]],[\"Inter\",[\"1a9\",\"1a9i\"]],[\"Inter Tight\",[\"1a9\",\"1a9i\"]],[\"Iosevka Charon\",[3,4,5,7,\"3i\",\"4i\",\"5i\",\"7i\"]],[\"Iosevka Charon Mono\",[3,4,5,7,\"3i\",\"4i\",\"5i\",\"7i\"]],[\"Irish Grover\",[4]],[\"Island Moments\",[4]],[\"Istok Web\",[4,7,\"4i\",\"7i\"]],[\"Italiana\",[4]],[\"Italianno\",[4]],[\"Itim\",[4]],[\"Jacquard 12\",[4]],[\"Jacquard 12 Charted\",[4]],[\"Jacquard 24\",[4]],[\"Jacquard 24 Charted\",[4]],[\"Jacquarda Bastarda 9\",[4]],[\"Jacquarda Bastarda 9 Charted\",[4]],[\"Jacques Francois\",[4]],[\"Jacques Francois Shadow\",[4]],[\"Jaini\",[4]],[\"Jaini Purva\",[4]],[\"Jaldi\",[4,7]],[\"Jaro\",[4]],[\"Jersey 10\",[4]],[\"Jersey 10 Charted\",[4]],[\"Jersey 15\",[4]],[\"Jersey 15 Charted\",[4]],[\"Jersey 20\",[4]],[\"Jersey 20 Charted\",[4]],[\"Jersey 25\",[4]],[\"Jersey 25 Charted\",[4]],[\"JetBrains Mono\",[\"1a8\",\"1a8i\"]],[\"Jim Nightshade\",[4]],[\"Joan\",[4]],[\"Jockey One\",[4]],[\"Jolly Lodger\",[4]],[\"Jomhuria\",[4]],[\"Jomolhari\",[4]],[\"Josefin Sans\",[\"1a7\",\"1a7i\"]],[\"Josefin Slab\",[\"1a7\",\"1a7i\"]],[\"Jost\",[\"1a9\",\"1a9i\"]],[\"Joti One\",[4]],[\"Jua\",[4]],[\"Judson\",[4,7,\"4i\"]],[\"Julee\",[4]],[\"Julius Sans One\",[4]],[\"Junge\",[4]],[\"Jura\",[\"3a7\"]],[\"Just Another Hand\",[4]],[\"Just Me Again Down Here\",[4]],[\"K2D\",[1,2,3,4,5,6,7,8,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\"]],[\"Kablammo\",[4]],[\"Kadwa\",[4,7]],[\"Kaisei Decol\",[4,5,7]],[\"Kaisei HarunoUmi\",[4,5,7]],[\"Kaisei Opti\",[4,5,7]],[\"Kaisei Tokumin\",[4,5,7,8]],[\"Kalam\",[3,4,7]],[\"Kalnia\",[\"1a7\"]],[\"Kalnia Glaze\",[\"1a7\"]],[\"Kameron\",[\"4a7\"]],[\"Kanchenjunga\",[4,5,6,7]],[\"Kanit\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Kantumruy Pro\",[\"1a7\",\"1a7i\"]],[\"Kapakana\",[\"3a4\"]],[\"Karantina\",[3,4,7]],[\"Karla\",[\"2a8\",\"2a8i\"]],[\"Karla Tamil Inclined\",[4,7]],[\"Karla Tamil Upright\",[4,7]],[\"Karma\",[3,4,5,6,7]],[\"Katibeh\",[4]],[\"Kaushan Script\",[4]],[\"Kavivanar\",[4]],[\"Kavoon\",[4]],[\"Kay Pho Du\",[4,5,6,7]],[\"Kdam Thmor Pro\",[4]],[\"Keania One\",[4]],[\"Kedebideri\",[4,5,6,7,8,9]],[\"Kelly Slab\",[4]],[\"Kenia\",[4]],[\"Khand\",[3,4,5,6,7]],[\"Khmer\",[4]],[\"Khula\",[3,4,6,7,8]],[\"Kings\",[4]],[\"Kirang Haerang\",[4]],[\"Kite One\",[4]],[\"Kiwi Maru\",[3,4,5]],[\"Klee One\",[4,6]],[\"Knewave\",[4]],[\"KoHo\",[2,3,4,5,6,7,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Kodchasan\",[2,3,4,5,6,7,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Kode Mono\",[\"4a7\"]],[\"Koh Santepheap\",[1,3,4,7,9]],[\"Kolker Brush\",[4]],[\"Konkhmer Sleokchher\",[4]],[\"Kosugi\",[4]],[\"Kosugi Maru\",[4]],[\"Kotta One\",[4]],[\"Koulen\",[4]],[\"Kranky\",[4]],[\"Kreon\",[\"3a7\"]],[\"Kristi\",[4]],[\"Krona One\",[4]],[\"Krub\",[2,3,4,5,6,7,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Kufam\",[\"4a9\",\"4a9i\"]],[\"Kulim Park\",[2,3,4,6,7,\"2i\",\"3i\",\"4i\",\"6i\",\"7i\"]],[\"Kumar One\",[4]],[\"Kumar One Outline\",[4]],[\"Kumbh Sans\",[\"1a9\"]],[\"Kurale\",[4]],[\"LINE Seed JP\",[1,4,7,8]],[\"LXGW Marker Gothic\",[4]],[\"LXGW WenKai Mono TC\",[3,4,7]],[\"LXGW WenKai TC\",[3,4,7]],[\"La Belle Aurore\",[4]],[\"Labrada\",[\"1a9\",\"1a9i\"]],[\"Lacquer\",[4]],[\"Laila\",[3,4,5,6,7]],[\"Lakki Reddy\",[4]],[\"Lalezar\",[4]],[\"Lancelot\",[4]],[\"Langar\",[4]],[\"Lateef\",[2,3,4,5,6,7,8]],[\"Lato\",[1,3,4,7,9,\"1i\",\"3i\",\"4i\",\"7i\",\"9i\"]],[\"Lavishly Yours\",[4]],[\"League Gothic\",[4]],[\"League Script\",[4]],[\"League Spartan\",[\"1a9\"]],[\"Leckerli One\",[4]],[\"Ledger\",[4]],[\"Lekton\",[4,7,\"4i\"]],[\"Lemon\",[4]],[\"Lemonada\",[\"3a7\"]],[\"Lexend\",[\"1a9\"]],[\"Lexend Deca\",[\"1a9\"]],[\"Lexend Exa\",[\"1a9\"]],[\"Lexend Giga\",[\"1a9\"]],[\"Lexend Mega\",[\"1a9\"]],[\"Lexend Peta\",[\"1a9\"]],[\"Lexend Tera\",[\"1a9\"]],[\"Lexend Zetta\",[\"1a9\"]],[\"Libertinus Keyboard\",[4]],[\"Libertinus Math\",[4]],[\"Libertinus Mono\",[4]],[\"Libertinus Sans\",[4,7,\"4i\"]],[\"Libertinus Serif\",[4,6,7,\"4i\",\"6i\",\"7i\"]],[\"Libertinus Serif Display\",[4]],[\"Libre Barcode 128\",[4]],[\"Libre Barcode 128 Text\",[4]],[\"Libre Barcode 39\",[4]],[\"Libre Barcode 39 Extended\",[4]],[\"Libre Barcode 39 Extended Text\",[4]],[\"Libre Barcode 39 Text\",[4]],[\"Libre Barcode EAN13 Text\",[4]],[\"Libre Baskerville\",[\"4a7\",\"4a7i\"]],[\"Libre Bodoni\",[\"4a7\",\"4a7i\"]],[\"Libre Caslon Display\",[4]],[\"Libre Caslon Text\",[4,7,\"4i\"]],[\"Libre Franklin\",[\"1a9\",\"1a9i\"]],[\"Licorice\",[4]],[\"Life Savers\",[4,7,8]],[\"Lilex\",[\"1a7\",\"1a7i\"]],[\"Lilita One\",[4]],[\"Lily Script One\",[4]],[\"Limelight\",[4]],[\"Linden Hill\",[4,\"4i\"]],[\"Linefont\",[\"1a10\"]],[\"Lisu Bosa\",[2,3,4,5,6,7,8,9,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Liter\",[4]],[\"Literata\",[\"2a9\",\"2a9i\"]],[\"Liu Jian Mao Cao\",[4]],[\"Livvic\",[1,2,3,4,5,6,7,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"9i\"]],[\"Lobster\",[4]],[\"Lobster Two\",[4,7,\"4i\",\"7i\"]],[\"Londrina Outline\",[4]],[\"Londrina Shadow\",[4]],[\"Londrina Sketch\",[4]],[\"Londrina Solid\",[1,3,4,9]],[\"Long Cang\",[4]],[\"Lora\",[\"4a7\",\"4a7i\"]],[\"Love Light\",[4]],[\"Love Ya Like A Sister\",[4]],[\"Loved by the King\",[4]],[\"Lovers Quarrel\",[4]],[\"Luckiest Guy\",[4]],[\"Lugrasimo\",[4]],[\"Lumanosimo\",[4]],[\"Lunasima\",[4,7]],[\"Lusitana\",[4,7]],[\"Lustria\",[4]],[\"Luxurious Roman\",[4]],[\"Luxurious Script\",[4]],[\"M PLUS 1\",[\"1a9\"]],[\"M PLUS 1 Code\",[\"1a7\"]],[\"M PLUS 1p\",[1,3,4,5,7,8,9]],[\"M PLUS 2\",[\"1a9\"]],[\"M PLUS Code Latin\",[\"1a7\"]],[\"M PLUS Rounded 1c\",[1,3,4,5,7,8,9]],[\"M PLUS U\",[\"1a9\"]],[\"Ma Shan Zheng\",[4]],[\"Macondo\",[4]],[\"Macondo Swash Caps\",[4]],[\"Mada\",[\"2a9\"]],[\"Madimi One\",[4]],[\"Magra\",[4,7]],[\"Maiden Orange\",[4]],[\"Maitree\",[2,3,4,5,6,7]],[\"Major Mono Display\",[4]],[\"Mako\",[4]],[\"Mali\",[2,3,4,5,6,7,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Mallanna\",[4]],[\"Maname\",[4]],[\"Mandali\",[4]],[\"Manjari\",[1,4,7]],[\"Manrope\",[\"2a8\"]],[\"Mansalva\",[4]],[\"Manuale\",[\"3a8\",\"3a8i\"]],[\"Manufacturing Consent\",[4]],[\"Marcellus\",[4]],[\"Marcellus SC\",[4]],[\"Marck Script\",[4]],[\"Margarine\",[4]],[\"Marhey\",[\"3a7\"]],[\"Markazi Text\",[\"4a7\"]],[\"Marko One\",[4]],[\"Marmelad\",[4]],[\"Martel\",[2,3,4,6,7,8,9]],[\"Martel Sans\",[2,3,4,6,7,8,9]],[\"Martian Mono\",[\"1a8\"]],[\"Marvel\",[4,7,\"4i\",\"7i\"]],[\"Matangi\",[\"3a9\"]],[\"Mate\",[4,\"4i\"]],[\"Mate SC\",[4]],[\"Matemasie\",[4]],[\"Maven Pro\",[\"4a9\"]],[\"McLaren\",[4]],[\"Mea Culpa\",[4]],[\"Meddon\",[4]],[\"MedievalSharp\",[4]],[\"Medula One\",[4]],[\"Meera Inimai\",[4]],[\"Megrim\",[4]],[\"Meie Script\",[4]],[\"Menbere\",[\"1a7\"]],[\"Meow Script\",[4]],[\"Merienda\",[\"3a9\"]],[\"Merriweather\",[\"3a9\",\"3a9i\"]],[\"Merriweather Sans\",[\"3a8\",\"3a8i\"]],[\"Metal\",[4]],[\"Metal Mania\",[4]],[\"Metamorphous\",[4]],[\"Metrophobic\",[4]],[\"Michroma\",[4]],[\"Micro 5\",[4]],[\"Micro 5 Charted\",[4]],[\"Milonga\",[4]],[\"Miltonian\",[4]],[\"Miltonian Tattoo\",[4]],[\"Mina\",[4,7]],[\"Mingzat\",[4]],[\"Miniver\",[4]],[\"Miranda Sans\",[\"4a7\",\"4a7i\"]],[\"Miriam Libre\",[\"4a7\"]],[\"Mirza\",[4,5,6,7]],[\"Miss Fajardose\",[4]],[\"Mitr\",[2,3,4,5,6,7]],[\"Mochiy Pop One\",[4]],[\"Mochiy Pop P One\",[4]],[\"Modak\",[4]],[\"Modern Antiqua\",[4]],[\"Moderustic\",[\"3a8\"]],[\"Mogra\",[4]],[\"Mohave\",[\"3a7\",\"3a7i\"]],[\"Moirai One\",[4]],[\"Molengo\",[4]],[\"Molle\",[\"4i\"]],[\"Momo Signature\",[4]],[\"Momo Trust Display\",[4]],[\"Momo Trust Sans\",[\"2a8\"]],[\"Mona Sans\",[\"2a9\",\"2a9i\"]],[\"Monda\",[\"4a7\"]],[\"Monofett\",[4]],[\"Monomakh\",[4]],[\"Monomaniac One\",[4]],[\"Monoton\",[4]],[\"Monsieur La Doulaise\",[4]],[\"Montaga\",[4]],[\"Montagu Slab\",[\"1a7\"]],[\"MonteCarlo\",[4]],[\"Montenegrin Gothic One\",[4]],[\"Montez\",[4]],[\"Montserrat\",[\"1a9\",\"1a9i\"]],[\"Montserrat Alternates\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Montserrat Underline\",[\"1a9\",\"1a9i\"]],[\"Moo Lah Lah\",[4]],[\"Mooli\",[4]],[\"Moon Dance\",[4]],[\"Moul\",[4]],[\"Moulpali\",[4]],[\"Mountains of Christmas\",[4,7]],[\"Mouse Memoirs\",[4]],[\"Mozilla Headline\",[\"2a7\"]],[\"Mozilla Text\",[\"2a7\"]],[\"Mr Bedfort\",[4]],[\"Mr Dafoe\",[4]],[\"Mr De Haviland\",[4]],[\"Mrs Saint Delafield\",[4]],[\"Mrs Sheppards\",[4]],[\"Ms Madi\",[4]],[\"Mukta\",[2,3,4,5,6,7,8]],[\"Mukta Mahee\",[2,3,4,5,6,7,8]],[\"Mukta Malar\",[2,3,4,5,6,7,8]],[\"Mukta Vaani\",[2,3,4,5,6,7,8]],[\"Mulish\",[\"2a10\",\"2a10i\"]],[\"Murecho\",[\"1a9\"]],[\"MuseoModerno\",[\"1a9\",\"1a9i\"]],[\"My Soul\",[4]],[\"Mynerve\",[4]],[\"Mystery Quest\",[4]],[\"NTR\",[4]],[\"Nabla\",[4]],[\"Namdhinggo\",[4,5,6,7,8]],[\"Nanum Brush Script\",[4]],[\"Nanum Gothic\",[4,7,8]],[\"Nanum Gothic Coding\",[4,7]],[\"Nanum Myeongjo\",[4,7,8]],[\"Nanum Pen Script\",[4]],[\"Narnoor\",[4,5,6,7,8]],[\"Nata Sans\",[\"1a9\"]],[\"National Park\",[\"2a8\"]],[\"Neonderthaw\",[4]],[\"Nerko One\",[4]],[\"Neucha\",[4]],[\"Neuton\",[2,3,4,7,8,\"4i\"]],[\"New Amsterdam\",[4]],[\"New Rocker\",[4]],[\"New Tegomin\",[4]],[\"News Cycle\",[4,7]],[\"Newsreader\",[\"2a8\",\"2a8i\"]],[\"Niconne\",[4]],[\"Niramit\",[2,3,4,5,6,7,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Nixie One\",[4]],[\"Nobile\",[4,5,7,\"4i\",\"5i\",\"7i\"]],[\"Nokora\",[\"1a9\"]],[\"Norican\",[4]],[\"Nosifer\",[4]],[\"Notable\",[4]],[\"Nothing You Could Do\",[4]],[\"Noticia Text\",[4,7,\"4i\",\"7i\"]],[\"Noto Color Emoji\",[4]],[\"Noto Emoji\",[\"3a7\"]],[\"Noto Kufi Arabic\",[\"1a9\"]],[\"Noto Music\",[4]],[\"Noto Naskh Arabic\",[\"4a7\"]],[\"Noto Nastaliq Urdu\",[\"4a7\"]],[\"Noto Rashi Hebrew\",[\"1a9\"]],[\"Noto Sans\",[\"1a9\",\"1a9i\"]],[\"Noto Sans Adlam\",[\"4a7\"]],[\"Noto Sans Adlam Unjoined\",[\"4a7\"]],[\"Noto Sans Anatolian Hieroglyphs\",[4]],[\"Noto Sans Arabic\",[\"1a9\"]],[\"Noto Sans Armenian\",[\"1a9\"]],[\"Noto Sans Avestan\",[4]],[\"Noto Sans Balinese\",[\"4a7\"]],[\"Noto Sans Bamum\",[\"4a7\"]],[\"Noto Sans Bassa Vah\",[\"4a7\"]],[\"Noto Sans Batak\",[4]],[\"Noto Sans Bengali\",[\"1a9\"]],[\"Noto Sans Bhaiksuki\",[4]],[\"Noto Sans Brahmi\",[4]],[\"Noto Sans Buginese\",[4]],[\"Noto Sans Buhid\",[4]],[\"Noto Sans Canadian Aboriginal\",[\"1a9\"]],[\"Noto Sans Carian\",[4]],[\"Noto Sans Caucasian Albanian\",[4]],[\"Noto Sans Chakma\",[4]],[\"Noto Sans Cham\",[\"1a9\"]],[\"Noto Sans Cherokee\",[\"1a9\"]],[\"Noto Sans Chorasmian\",[4]],[\"Noto Sans Coptic\",[4]],[\"Noto Sans Cuneiform\",[4]],[\"Noto Sans Cypriot\",[4]],[\"Noto Sans Cypro Minoan\",[4]],[\"Noto Sans Deseret\",[4]],[\"Noto Sans Devanagari\",[\"1a9\"]],[\"Noto Sans Display\",[\"1a9\",\"1a9i\"]],[\"Noto Sans Duployan\",[4,7]],[\"Noto Sans Egyptian Hieroglyphs\",[4]],[\"Noto Sans Elbasan\",[4]],[\"Noto Sans Elymaic\",[4]],[\"Noto Sans Ethiopic\",[\"1a9\"]],[\"Noto Sans Georgian\",[\"1a9\"]],[\"Noto Sans Glagolitic\",[4]],[\"Noto Sans Gothic\",[4]],[\"Noto Sans Grantha\",[4]],[\"Noto Sans Gujarati\",[\"1a9\"]],[\"Noto Sans Gunjala Gondi\",[\"4a7\"]],[\"Noto Sans Gurmukhi\",[\"1a9\"]],[\"Noto Sans HK\",[\"1a9\"]],[\"Noto Sans Hanifi Rohingya\",[\"4a7\"]],[\"Noto Sans Hanunoo\",[4]],[\"Noto Sans Hatran\",[4]],[\"Noto Sans Hebrew\",[\"1a9\"]],[\"Noto Sans Imperial Aramaic\",[4]],[\"Noto Sans Indic Siyaq Numbers\",[4]],[\"Noto Sans Inscriptional Pahlavi\",[4]],[\"Noto Sans Inscriptional Parthian\",[4]],[\"Noto Sans JP\",[\"1a9\"]],[\"Noto Sans Javanese\",[\"4a7\"]],[\"Noto Sans KR\",[\"1a9\"]],[\"Noto Sans Kaithi\",[4]],[\"Noto Sans Kannada\",[\"1a9\"]],[\"Noto Sans Kawi\",[\"4a7\"]],[\"Noto Sans Kayah Li\",[\"4a7\"]],[\"Noto Sans Kharoshthi\",[4]],[\"Noto Sans Khmer\",[\"1a9\"]],[\"Noto Sans Khojki\",[4]],[\"Noto Sans Khudawadi\",[4]],[\"Noto Sans Lao\",[\"1a9\"]],[\"Noto Sans Lao Looped\",[\"1a9\"]],[\"Noto Sans Lepcha\",[4]],[\"Noto Sans Limbu\",[4]],[\"Noto Sans Linear A\",[4]],[\"Noto Sans Linear B\",[4]],[\"Noto Sans Lisu\",[\"4a7\"]],[\"Noto Sans Lycian\",[4]],[\"Noto Sans Lydian\",[4]],[\"Noto Sans Mahajani\",[4]],[\"Noto Sans Malayalam\",[\"1a9\"]],[\"Noto Sans Mandaic\",[4]],[\"Noto Sans Manichaean\",[4]],[\"Noto Sans Marchen\",[4]],[\"Noto Sans Masaram Gondi\",[4]],[\"Noto Sans Math\",[4]],[\"Noto Sans Mayan Numerals\",[4]],[\"Noto Sans Medefaidrin\",[\"4a7\"]],[\"Noto Sans Meetei Mayek\",[\"1a9\"]],[\"Noto Sans Mende Kikakui\",[4]],[\"Noto Sans Meroitic\",[4]],[\"Noto Sans Miao\",[4]],[\"Noto Sans Modi\",[4]],[\"Noto Sans Mongolian\",[4]],[\"Noto Sans Mono\",[\"1a9\"]],[\"Noto Sans Mro\",[4]],[\"Noto Sans Multani\",[4]],[\"Noto Sans Myanmar\",[\"1a9\"]],[\"Noto Sans NKo\",[4]],[\"Noto Sans NKo Unjoined\",[\"4a7\"]],[\"Noto Sans Nabataean\",[4]],[\"Noto Sans Nag Mundari\",[\"4a7\"]],[\"Noto Sans Nandinagari\",[4]],[\"Noto Sans New Tai Lue\",[\"4a7\"]],[\"Noto Sans Newa\",[4]],[\"Noto Sans Nushu\",[4]],[\"Noto Sans Ogham\",[4]],[\"Noto Sans Ol Chiki\",[\"4a7\"]],[\"Noto Sans Old Hungarian\",[4]],[\"Noto Sans Old Italic\",[4]],[\"Noto Sans Old North Arabian\",[4]],[\"Noto Sans Old Permic\",[4]],[\"Noto Sans Old Persian\",[4]],[\"Noto Sans Old Sogdian\",[4]],[\"Noto Sans Old South Arabian\",[4]],[\"Noto Sans Old Turkic\",[4]],[\"Noto Sans Oriya\",[\"1a9\"]],[\"Noto Sans Osage\",[4]],[\"Noto Sans Osmanya\",[4]],[\"Noto Sans Pahawh Hmong\",[4]],[\"Noto Sans Palmyrene\",[4]],[\"Noto Sans Pau Cin Hau\",[4]],[\"Noto Sans PhagsPa\",[4]],[\"Noto Sans Phoenician\",[4]],[\"Noto Sans Psalter Pahlavi\",[4]],[\"Noto Sans Rejang\",[4]],[\"Noto Sans Runic\",[4]],[\"Noto Sans SC\",[\"1a9\"]],[\"Noto Sans Samaritan\",[4]],[\"Noto Sans Saurashtra\",[4]],[\"Noto Sans Sharada\",[4]],[\"Noto Sans Shavian\",[4]],[\"Noto Sans Siddham\",[4]],[\"Noto Sans SignWriting\",[4]],[\"Noto Sans Sinhala\",[\"1a9\"]],[\"Noto Sans Sogdian\",[4]],[\"Noto Sans Sora Sompeng\",[\"4a7\"]],[\"Noto Sans Soyombo\",[4]],[\"Noto Sans Sundanese\",[\"4a7\"]],[\"Noto Sans Sunuwar\",[4]],[\"Noto Sans Syloti Nagri\",[4]],[\"Noto Sans Symbols\",[\"1a9\"]],[\"Noto Sans Symbols 2\",[4]],[\"Noto Sans Syriac\",[\"1a9\"]],[\"Noto Sans Syriac Eastern\",[\"1a9\"]],[\"Noto Sans Syriac Western\",[\"1a9\"]],[\"Noto Sans TC\",[\"1a9\"]],[\"Noto Sans Tagalog\",[4]],[\"Noto Sans Tagbanwa\",[4]],[\"Noto Sans Tai Le\",[4]],[\"Noto Sans Tai Tham\",[\"4a7\"]],[\"Noto Sans Tai Viet\",[4]],[\"Noto Sans Takri\",[4]],[\"Noto Sans Tamil\",[\"1a9\"]],[\"Noto Sans Tamil Supplement\",[4]],[\"Noto Sans Tangsa\",[\"4a7\"]],[\"Noto Sans Telugu\",[\"1a9\"]],[\"Noto Sans Thaana\",[\"1a9\"]],[\"Noto Sans Thai\",[\"1a9\"]],[\"Noto Sans Thai Looped\",[\"1a9\"]],[\"Noto Sans Tifinagh\",[4]],[\"Noto Sans Tirhuta\",[4]],[\"Noto Sans Ugaritic\",[4]],[\"Noto Sans Vai\",[4]],[\"Noto Sans Vithkuqi\",[\"4a7\"]],[\"Noto Sans Wancho\",[4]],[\"Noto Sans Warang Citi\",[4]],[\"Noto Sans Yi\",[4]],[\"Noto Sans Zanabazar Square\",[4]],[\"Noto Serif\",[\"1a9\",\"1a9i\"]],[\"Noto Serif Ahom\",[4]],[\"Noto Serif Armenian\",[\"1a9\"]],[\"Noto Serif Balinese\",[4]],[\"Noto Serif Bengali\",[\"1a9\"]],[\"Noto Serif Devanagari\",[\"1a9\"]],[\"Noto Serif Display\",[\"1a9\",\"1a9i\"]],[\"Noto Serif Dives Akuru\",[4]],[\"Noto Serif Dogra\",[4]],[\"Noto Serif Ethiopic\",[\"1a9\"]],[\"Noto Serif Georgian\",[\"1a9\"]],[\"Noto Serif Grantha\",[4]],[\"Noto Serif Gujarati\",[\"1a9\"]],[\"Noto Serif Gurmukhi\",[\"1a9\"]],[\"Noto Serif HK\",[\"2a9\"]],[\"Noto Serif Hebrew\",[\"1a9\"]],[\"Noto Serif Hentaigana\",[\"2a9\"]],[\"Noto Serif JP\",[\"2a9\"]],[\"Noto Serif KR\",[\"2a9\"]],[\"Noto Serif Kannada\",[\"1a9\"]],[\"Noto Serif Khitan Small Script\",[4]],[\"Noto Serif Khmer\",[\"1a9\"]],[\"Noto Serif Khojki\",[\"4a7\"]],[\"Noto Serif Lao\",[\"1a9\"]],[\"Noto Serif Makasar\",[4]],[\"Noto Serif Malayalam\",[\"1a9\"]],[\"Noto Serif Myanmar\",[1,2,3,4,5,6,7,8,9]],[\"Noto Serif NP Hmong\",[\"4a7\"]],[\"Noto Serif Old Uyghur\",[4]],[\"Noto Serif Oriya\",[\"4a7\"]],[\"Noto Serif Ottoman Siyaq\",[4]],[\"Noto Serif SC\",[\"2a9\"]],[\"Noto Serif Sinhala\",[\"1a9\"]],[\"Noto Serif TC\",[\"2a9\"]],[\"Noto Serif Tamil\",[\"1a9\",\"1a9i\"]],[\"Noto Serif Tangut\",[4]],[\"Noto Serif Telugu\",[\"1a9\"]],[\"Noto Serif Thai\",[\"1a9\"]],[\"Noto Serif Tibetan\",[\"1a9\"]],[\"Noto Serif Todhri\",[4]],[\"Noto Serif Toto\",[\"4a7\"]],[\"Noto Serif Vithkuqi\",[\"4a7\"]],[\"Noto Serif Yezidi\",[\"4a7\"]],[\"Noto Traditional Nushu\",[\"3a7\"]],[\"Noto Znamenny Musical Notation\",[4]],[\"Nova Cut\",[4]],[\"Nova Flat\",[4]],[\"Nova Mono\",[4]],[\"Nova Oval\",[4]],[\"Nova Round\",[4]],[\"Nova Script\",[4]],[\"Nova Slim\",[4]],[\"Nova Square\",[4]],[\"Numans\",[4]],[\"Nunito\",[\"2a10\",\"2a10i\"]],[\"Nunito Sans\",[\"2a10\",\"2a10i\"]],[\"Nuosu SIL\",[4]],[\"Odibee Sans\",[4]],[\"Odor Mean Chey\",[4]],[\"Offside\",[4]],[\"Oi\",[4]],[\"Ojuju\",[\"2a8\"]],[\"Old Standard TT\",[4,7,\"4i\"]],[\"Oldenburg\",[4]],[\"Ole\",[4]],[\"Oleo Script\",[4,7]],[\"Oleo Script Swash Caps\",[4,7]],[\"Onest\",[\"1a9\"]],[\"Oooh Baby\",[4]],[\"Open Sans\",[\"3a8\",\"3a8i\"]],[\"Oranienbaum\",[4]],[\"Orbit\",[4]],[\"Orbitron\",[\"4a9\"]],[\"Oregano\",[4,\"4i\"]],[\"Orelega One\",[4]],[\"Orienta\",[4]],[\"Original Surfer\",[4]],[\"Oswald\",[\"2a7\"]],[\"Outfit\",[\"1a9\"]],[\"Over the Rainbow\",[4]],[\"Overlock\",[4,7,9,\"4i\",\"7i\",\"9i\"]],[\"Overlock SC\",[4]],[\"Overpass\",[\"1a9\",\"1a9i\"]],[\"Overpass Mono\",[\"3a7\"]],[\"Ovo\",[4]],[\"Oxanium\",[\"2a8\"]],[\"Oxygen\",[3,4,7]],[\"Oxygen Mono\",[4]],[\"PT Mono\",[4]],[\"PT Sans\",[4,7,\"4i\",\"7i\"]],[\"PT Sans Caption\",[4,7]],[\"PT Sans Narrow\",[4,7]],[\"PT Serif\",[4,7,\"4i\",\"7i\"]],[\"PT Serif Caption\",[4,\"4i\"]],[\"Pacifico\",[4]],[\"Padauk\",[4,7]],[\"Padyakke Expanded One\",[4]],[\"Palanquin\",[1,2,3,4,5,6,7]],[\"Palanquin Dark\",[4,5,6,7]],[\"Palette Mosaic\",[4]],[\"Pangolin\",[4]],[\"Paprika\",[4]],[\"Parastoo\",[\"4a7\"]],[\"Parisienne\",[4]],[\"Parkinsans\",[\"3a8\"]],[\"Passero One\",[4]],[\"Passion One\",[4,7,9]],[\"Passions Conflict\",[4]],[\"Pathway Extreme\",[\"1a9\",\"1a9i\"]],[\"Pathway Gothic One\",[4]],[\"Patrick Hand\",[4]],[\"Patrick Hand SC\",[4]],[\"Pattaya\",[4]],[\"Patua One\",[4]],[\"Pavanam\",[4]],[\"Paytone One\",[4]],[\"Peddana\",[4]],[\"Peralta\",[4]],[\"Permanent Marker\",[4]],[\"Petemoss\",[4]],[\"Petit Formal Script\",[4]],[\"Petrona\",[\"1a9\",\"1a9i\"]],[\"Phetsarath\",[4,7]],[\"Philosopher\",[4,7,\"4i\",\"7i\"]],[\"Phudu\",[\"3a9\"]],[\"Piazzolla\",[\"1a9\",\"1a9i\"]],[\"Piedra\",[4]],[\"Pinyon Script\",[4]],[\"Pirata One\",[4]],[\"Pixelify Sans\",[\"4a7\"]],[\"Plaster\",[4]],[\"Platypi\",[\"3a8\",\"3a8i\"]],[\"Play\",[4,7]],[\"Playball\",[4]],[\"Playfair\",[\"3a9\",\"3a9i\"]],[\"Playfair Display\",[\"4a9\",\"4a9i\"]],[\"Playfair Display SC\",[4,7,9,\"4i\",\"7i\",\"9i\"]],[\"Playpen Sans\",[\"1a8\"]],[\"Playpen Sans Arabic\",[\"1a8\"]],[\"Playpen Sans Deva\",[\"1a8\"]],[\"Playpen Sans Hebrew\",[\"1a8\"]],[\"Playpen Sans Thai\",[\"1a8\"]],[\"Playwrite AR\",[\"1a4\"]],[\"Playwrite AR Guides\",[4]],[\"Playwrite AT\",[\"1a4\",\"1a4i\"]],[\"Playwrite AT Guides\",[4,\"4i\"]],[\"Playwrite AU NSW\",[\"1a4\"]],[\"Playwrite AU NSW Guides\",[4]],[\"Playwrite AU QLD\",[\"1a4\"]],[\"Playwrite AU QLD Guides\",[4]],[\"Playwrite AU SA\",[\"1a4\"]],[\"Playwrite AU SA Guides\",[4]],[\"Playwrite AU TAS\",[\"1a4\"]],[\"Playwrite AU TAS Guides\",[4]],[\"Playwrite AU VIC\",[\"1a4\"]],[\"Playwrite AU VIC Guides\",[4]],[\"Playwrite BE VLG\",[\"1a4\"]],[\"Playwrite BE VLG Guides\",[4]],[\"Playwrite BE WAL\",[\"1a4\"]],[\"Playwrite BE WAL Guides\",[4]],[\"Playwrite BR\",[\"1a4\"]],[\"Playwrite BR Guides\",[4]],[\"Playwrite CA\",[\"1a4\"]],[\"Playwrite CA Guides\",[4]],[\"Playwrite CL\",[\"1a4\"]],[\"Playwrite CL Guides\",[4]],[\"Playwrite CO\",[\"1a4\"]],[\"Playwrite CO Guides\",[4]],[\"Playwrite CU\",[\"1a4\"]],[\"Playwrite CU Guides\",[4]],[\"Playwrite CZ\",[\"1a4\"]],[\"Playwrite CZ Guides\",[4]],[\"Playwrite DE Grund\",[\"1a4\"]],[\"Playwrite DE Grund Guides\",[4]],[\"Playwrite DE LA\",[\"1a4\"]],[\"Playwrite DE LA Guides\",[4]],[\"Playwrite DE SAS\",[\"1a4\"]],[\"Playwrite DE SAS Guides\",[4]],[\"Playwrite DE VA\",[\"1a4\"]],[\"Playwrite DE VA Guides\",[4]],[\"Playwrite DK Loopet\",[\"1a4\"]],[\"Playwrite DK Loopet Guides\",[4]],[\"Playwrite DK Uloopet\",[\"1a4\"]],[\"Playwrite DK Uloopet Guides\",[4]],[\"Playwrite ES\",[\"1a4\"]],[\"Playwrite ES Deco\",[\"1a4\"]],[\"Playwrite ES Deco Guides\",[4]],[\"Playwrite ES Guides\",[4]],[\"Playwrite FR Moderne\",[\"1a4\"]],[\"Playwrite FR Moderne Guides\",[4]],[\"Playwrite FR Trad\",[\"1a4\"]],[\"Playwrite FR Trad Guides\",[4]],[\"Playwrite GB J\",[\"1a4\",\"1a4i\"]],[\"Playwrite GB J Guides\",[4,\"4i\"]],[\"Playwrite GB S\",[\"1a4\",\"1a4i\"]],[\"Playwrite GB S Guides\",[4,\"4i\"]],[\"Playwrite HR\",[\"1a4\"]],[\"Playwrite HR Guides\",[4]],[\"Playwrite HR Lijeva\",[\"1a4\"]],[\"Playwrite HR Lijeva Guides\",[4]],[\"Playwrite HU\",[\"1a4\"]],[\"Playwrite HU Guides\",[4]],[\"Playwrite ID\",[\"1a4\"]],[\"Playwrite ID Guides\",[4]],[\"Playwrite IE\",[\"1a4\"]],[\"Playwrite IE Guides\",[4]],[\"Playwrite IN\",[\"1a4\"]],[\"Playwrite IN Guides\",[4]],[\"Playwrite IS\",[\"1a4\"]],[\"Playwrite IS Guides\",[4]],[\"Playwrite IT Moderna\",[\"1a4\"]],[\"Playwrite IT Moderna Guides\",[4]],[\"Playwrite IT Trad\",[\"1a4\"]],[\"Playwrite IT Trad Guides\",[4]],[\"Playwrite MX\",[\"1a4\"]],[\"Playwrite MX Guides\",[4]],[\"Playwrite NG Modern\",[\"1a4\"]],[\"Playwrite NG Modern Guides\",[4]],[\"Playwrite NL\",[\"1a4\"]],[\"Playwrite NL Guides\",[4]],[\"Playwrite NO\",[\"1a4\"]],[\"Playwrite NO Guides\",[4]],[\"Playwrite NZ\",[\"1a4\"]],[\"Playwrite NZ Basic\",[\"1a4\"]],[\"Playwrite NZ Basic Guides\",[4]],[\"Playwrite NZ Guides\",[4]],[\"Playwrite PE\",[\"1a4\"]],[\"Playwrite PE Guides\",[4]],[\"Playwrite PL\",[\"1a4\"]],[\"Playwrite PL Guides\",[4]],[\"Playwrite PT\",[\"1a4\"]],[\"Playwrite PT Guides\",[4]],[\"Playwrite RO\",[\"1a4\"]],[\"Playwrite RO Guides\",[4]],[\"Playwrite SK\",[\"1a4\"]],[\"Playwrite SK Guides\",[4]],[\"Playwrite TZ\",[\"1a4\"]],[\"Playwrite TZ Guides\",[4]],[\"Playwrite US Modern\",[\"1a4\"]],[\"Playwrite US Modern Guides\",[4]],[\"Playwrite US Trad\",[\"1a4\"]],[\"Playwrite US Trad Guides\",[4]],[\"Playwrite VN\",[\"1a4\"]],[\"Playwrite VN Guides\",[4]],[\"Playwrite ZA\",[\"1a4\"]],[\"Playwrite ZA Guides\",[4]],[\"Pliant\",[\"1a9\",\"1a9i\"]],[\"Plus Jakarta Sans\",[\"2a8\",\"2a8i\"]],[\"Pochaevsk\",[4]],[\"Podkova\",[\"4a8\"]],[\"Poetsen One\",[4]],[\"Poiret One\",[4]],[\"Poller One\",[4]],[\"Poltawski Nowy\",[\"4a7\",\"4a7i\"]],[\"Poly\",[4,\"4i\"]],[\"Pompiere\",[4]],[\"Ponnala\",[4]],[\"Ponomar\",[4]],[\"Pontano Sans\",[\"3a7\"]],[\"Poor Story\",[4]],[\"Poppins\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Port Lligat Sans\",[4]],[\"Port Lligat Slab\",[4]],[\"Potta One\",[4]],[\"Pragati Narrow\",[4,7]],[\"Praise\",[4]],[\"Prata\",[4]],[\"Preahvihear\",[4]],[\"Press Start 2P\",[4]],[\"Pridi\",[2,3,4,5,6,7]],[\"Princess Sofia\",[4]],[\"Prociono\",[4]],[\"Prompt\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Prosto One\",[4]],[\"Protest Guerrilla\",[4]],[\"Protest Revolution\",[4]],[\"Protest Riot\",[4]],[\"Protest Strike\",[4]],[\"Proza Libre\",[4,5,6,7,8,\"4i\",\"5i\",\"6i\",\"7i\",\"8i\"]],[\"Public Sans\",[\"1a9\",\"1a9i\"]],[\"Puppies Play\",[4]],[\"Puritan\",[4,7,\"4i\",\"7i\"]],[\"Purple Purse\",[4]],[\"Qahiri\",[4]],[\"Quando\",[4]],[\"Quantico\",[4,7,\"4i\",\"7i\"]],[\"Quattrocento\",[4,7]],[\"Quattrocento Sans\",[4,7,\"4i\",\"7i\"]],[\"Questrial\",[4]],[\"Quicksand\",[\"3a7\"]],[\"Quintessential\",[4]],[\"Qwigley\",[4]],[\"Qwitcher Grypen\",[4,7]],[\"REM\",[\"1a9\",\"1a9i\"]],[\"Racing Sans One\",[4]],[\"Radio Canada\",[\"3a7\",\"3a7i\"]],[\"Radio Canada Big\",[\"4a7\",\"4a7i\"]],[\"Radley\",[4,\"4i\"]],[\"Rajdhani\",[3,4,5,6,7]],[\"Rakkas\",[4]],[\"Raleway\",[\"1a9\",\"1a9i\"]],[\"Raleway Dots\",[4]],[\"Ramabhadra\",[4]],[\"Ramaraja\",[4]],[\"Rambla\",[4,7,\"4i\",\"7i\"]],[\"Rammetto One\",[4]],[\"Rampart One\",[4]],[\"Ramsina\",[4]],[\"Ranchers\",[4]],[\"Rancho\",[4]],[\"Ranga\",[4,7]],[\"Rasa\",[\"3a7\",\"3a7i\"]],[\"Rationale\",[4]],[\"Ravi Prakash\",[4]],[\"Readex Pro\",[\"2a7\"]],[\"Recursive\",[\"3a10\"]],[\"Red Hat Display\",[\"3a9\",\"3a9i\"]],[\"Red Hat Mono\",[\"3a7\",\"3a7i\"]],[\"Red Hat Text\",[\"3a7\",\"3a7i\"]],[\"Red Rose\",[\"3a7\"]],[\"Redacted\",[4]],[\"Redacted Script\",[3,4,7]],[\"Reddit Mono\",[\"2a9\"]],[\"Reddit Sans\",[\"2a9\",\"2a9i\"]],[\"Reddit Sans Condensed\",[\"2a9\"]],[\"Redressed\",[4]],[\"Reem Kufi\",[\"4a7\"]],[\"Reem Kufi Fun\",[\"4a7\"]],[\"Reem Kufi Ink\",[4]],[\"Reenie Beanie\",[4]],[\"Reggae One\",[4]],[\"Rethink Sans\",[\"4a8\",\"4a8i\"]],[\"Revalia\",[4]],[\"Rhodium Libre\",[4]],[\"Ribeye\",[4]],[\"Ribeye Marrow\",[4]],[\"Righteous\",[4]],[\"Risque\",[4]],[\"Road Rage\",[4]],[\"Roboto\",[\"1a9\",\"1a9i\"]],[\"Roboto Condensed\",[\"1a9\",\"1a9i\"]],[\"Roboto Flex\",[\"1a10\"]],[\"Roboto Mono\",[\"1a7\",\"1a7i\"]],[\"Roboto Serif\",[\"1a9\",\"1a9i\"]],[\"Roboto Slab\",[\"1a9\"]],[\"Rochester\",[4]],[\"Rock 3D\",[4]],[\"Rock Salt\",[4]],[\"RocknRoll One\",[4]],[\"Rokkitt\",[\"1a9\",\"1a9i\"]],[\"Romanesco\",[4]],[\"Ropa Sans\",[4,\"4i\"]],[\"Rosario\",[\"3a7\",\"3a7i\"]],[\"Rosarivo\",[4,\"4i\"]],[\"Rouge Script\",[4]],[\"Rowdies\",[3,4,7]],[\"Rozha One\",[4]],[\"Rubik\",[\"3a9\",\"3a9i\"]],[\"Rubik 80s Fade\",[4]],[\"Rubik Beastly\",[4]],[\"Rubik Broken Fax\",[4]],[\"Rubik Bubbles\",[4]],[\"Rubik Burned\",[4]],[\"Rubik Dirt\",[4]],[\"Rubik Distressed\",[4]],[\"Rubik Doodle Shadow\",[4]],[\"Rubik Doodle Triangles\",[4]],[\"Rubik Gemstones\",[4]],[\"Rubik Glitch\",[4]],[\"Rubik Glitch Pop\",[4]],[\"Rubik Iso\",[4]],[\"Rubik Lines\",[4]],[\"Rubik Maps\",[4]],[\"Rubik Marker Hatch\",[4]],[\"Rubik Maze\",[4]],[\"Rubik Microbe\",[4]],[\"Rubik Mono One\",[4]],[\"Rubik Moonrocks\",[4]],[\"Rubik Pixels\",[4]],[\"Rubik Puddles\",[4]],[\"Rubik Scribble\",[4]],[\"Rubik Spray Paint\",[4]],[\"Rubik Storm\",[4]],[\"Rubik Vinyl\",[4]],[\"Rubik Wet Paint\",[4]],[\"Ruda\",[\"4a9\"]],[\"Rufina\",[4,7]],[\"Ruge Boogie\",[4]],[\"Ruluko\",[4]],[\"Rum Raisin\",[4]],[\"Ruslan Display\",[4]],[\"Russo One\",[4]],[\"Ruthie\",[4]],[\"Ruwudu\",[4,5,6,7]],[\"Rye\",[4]],[\"SN Pro\",[\"2a9\",\"2a9i\"]],[\"STIX Two Math\",[4]],[\"STIX Two Text\",[\"4a7\",\"4a7i\"]],[\"SUSE\",[\"1a9\",\"1a9i\"]],[\"SUSE Mono\",[\"1a8\",\"1a8i\"]],[\"Sacramento\",[4]],[\"Sahitya\",[4,7]],[\"Sail\",[4]],[\"Saira\",[\"1a9\",\"1a9i\"]],[\"Saira Condensed\",[1,2,3,4,5,6,7,8,9]],[\"Saira Extra Condensed\",[1,2,3,4,5,6,7,8,9]],[\"Saira Semi Condensed\",[1,2,3,4,5,6,7,8,9]],[\"Saira Stencil\",[\"1a9\",\"1a9i\"]],[\"Salsa\",[4]],[\"Sanchez\",[4,\"4i\"]],[\"Sancreek\",[4]],[\"Sankofa Display\",[4]],[\"Sansation\",[3,4,7,\"3i\",\"4i\",\"7i\"]],[\"Sansita\",[4,7,8,9,\"4i\",\"7i\",\"8i\",\"9i\"]],[\"Sansita Swashed\",[\"3a9\"]],[\"Sarabun\",[1,2,3,4,5,6,7,8,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\"]],[\"Sarala\",[4,7]],[\"Sarina\",[4]],[\"Sarpanch\",[4,5,6,7,8,9]],[\"Sassy Frass\",[4]],[\"Satisfy\",[4]],[\"Savate\",[\"2a9\",\"2a9i\"]],[\"Sawarabi Gothic\",[4]],[\"Sawarabi Mincho\",[4]],[\"Scada\",[4,7,\"4i\",\"7i\"]],[\"Scheherazade New\",[4,5,6,7]],[\"Schibsted Grotesk\",[\"4a9\",\"4a9i\"]],[\"Schoolbell\",[4]],[\"Science Gothic\",[\"1a9\"]],[\"Scope One\",[4]],[\"Scoutie Sans\",[\"2a8\",\"2a8i\"]],[\"Seaweed Script\",[4]],[\"Secular One\",[4]],[\"Sedan\",[4,\"4i\"]],[\"Sedan SC\",[4]],[\"Sedgwick Ave\",[4]],[\"Sedgwick Ave Display\",[4]],[\"Sekuya\",[4]],[\"Sen\",[\"4a8\"]],[\"Send Flowers\",[4]],[\"Sevillana\",[4]],[\"Seymour One\",[4]],[\"Shadows Into Light\",[4]],[\"Shadows Into Light Two\",[4]],[\"Shafarik\",[4]],[\"Shalimar\",[4]],[\"Shantell Sans\",[\"3a8\",\"3a8i\"]],[\"Shanti\",[4]],[\"Share\",[4,7,\"4i\",\"7i\"]],[\"Share Tech\",[4]],[\"Share Tech Mono\",[4]],[\"Shippori Antique\",[4]],[\"Shippori Antique B1\",[4]],[\"Shippori Mincho\",[4,5,6,7,8]],[\"Shippori Mincho B1\",[4,5,6,7,8]],[\"Shizuru\",[4]],[\"Shojumaru\",[4]],[\"Short Stack\",[4]],[\"Shrikhand\",[4]],[\"Siemreap\",[4]],[\"Sigmar\",[4]],[\"Sigmar One\",[4]],[\"Signika\",[\"3a7\"]],[\"Signika Negative\",[\"3a7\"]],[\"Silkscreen\",[4,7]],[\"Simonetta\",[4,9,\"4i\",\"9i\"]],[\"Single Day\",[4]],[\"Sintony\",[4,7]],[\"Sirin Stencil\",[4]],[\"Sirivennela\",[4]],[\"Six Caps\",[4]],[\"Sixtyfour\",[4]],[\"Sixtyfour Convergence\",[4]],[\"Skranji\",[4,7]],[\"Slabo 13px\",[4]],[\"Slabo 27px\",[4]],[\"Slackey\",[4]],[\"Slackside One\",[4]],[\"Smokum\",[4]],[\"Smooch\",[4]],[\"Smooch Sans\",[\"1a9\"]],[\"Smythe\",[4]],[\"Sniglet\",[4,8]],[\"Snippet\",[4]],[\"Snowburst One\",[4]],[\"Sofadi One\",[4]],[\"Sofia\",[4]],[\"Sofia Sans\",[\"1a10\",\"1a10i\"]],[\"Sofia Sans Condensed\",[\"1a10\",\"1a10i\"]],[\"Sofia Sans Extra Condensed\",[\"1a10\",\"1a10i\"]],[\"Sofia Sans Semi Condensed\",[\"1a10\",\"1a10i\"]],[\"Solitreo\",[4]],[\"Solway\",[3,4,5,7,8]],[\"Sometype Mono\",[\"4a7\",\"4a7i\"]],[\"Song Myung\",[4]],[\"Sono\",[\"2a8\"]],[\"Sonsie One\",[4]],[\"Sora\",[\"1a8\"]],[\"Sorts Mill Goudy\",[4,\"4i\"]],[\"Sour Gummy\",[\"1a9\",\"1a9i\"]],[\"Source Code Pro\",[\"2a9\",\"2a9i\"]],[\"Source Sans 3\",[\"2a9\",\"2a9i\"]],[\"Source Serif 4\",[\"2a9\",\"2a9i\"]],[\"Space Grotesk\",[\"3a7\"]],[\"Space Mono\",[4,7,\"4i\",\"7i\"]],[\"Special Elite\",[4]],[\"Special Gothic\",[\"4a7\"]],[\"Special Gothic Condensed One\",[4]],[\"Special Gothic Expanded One\",[4]],[\"Spectral\",[2,3,4,5,6,7,8,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\"]],[\"Spectral SC\",[2,3,4,5,6,7,8,\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\"]],[\"Spicy Rice\",[4]],[\"Spinnaker\",[4]],[\"Spirax\",[4]],[\"Splash\",[4]],[\"Spline Sans\",[\"3a7\"]],[\"Spline Sans Mono\",[\"3a7\",\"3a7i\"]],[\"Squada One\",[4]],[\"Square Peg\",[4]],[\"Sree Krushnadevaraya\",[4]],[\"Sriracha\",[4]],[\"Srisakdi\",[4,7]],[\"Staatliches\",[4]],[\"Stack Sans Headline\",[\"2a7\"]],[\"Stack Sans Notch\",[\"2a7\"]],[\"Stack Sans Text\",[\"2a7\"]],[\"Stalemate\",[4]],[\"Stalinist One\",[4]],[\"Stardos Stencil\",[4,7]],[\"Stick\",[4]],[\"Stick No Bills\",[\"2a8\"]],[\"Stint Ultra Condensed\",[4]],[\"Stint Ultra Expanded\",[4]],[\"Stoke\",[3,4]],[\"Story Script\",[4]],[\"Strait\",[4]],[\"Strichpunkt Sans\",[\"4a9\"]],[\"Style Script\",[4]],[\"Stylish\",[4]],[\"Sue Ellen Francisco\",[4]],[\"Suez One\",[4]],[\"Sulphur Point\",[3,4,7]],[\"Sumana\",[4,7]],[\"Sunflower\",[3,5,7]],[\"Sunshiney\",[4]],[\"Supermercado One\",[4]],[\"Sura\",[4,7]],[\"Suranna\",[4]],[\"Suravaram\",[4]],[\"Suwannaphum\",[1,3,4,7,9]],[\"Swanky and Moo Moo\",[4]],[\"Syncopate\",[4,7]],[\"Syne\",[\"4a8\"]],[\"Syne Mono\",[4]],[\"Syne Tactile\",[4]],[\"TASA Explorer\",[\"4a8\"]],[\"TASA Orbiter\",[\"4a8\"]],[\"Tac One\",[4]],[\"Tagesschrift\",[4]],[\"Tai Heritage Pro\",[4,7]],[\"Tajawal\",[2,3,4,5,7,8,9]],[\"Tangerine\",[4,7]],[\"Tapestry\",[4]],[\"Taprom\",[4]],[\"Tauri\",[4]],[\"Taviraj\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Teachers\",[\"4a8\",\"4a8i\"]],[\"Teko\",[\"3a7\"]],[\"Tektur\",[\"4a9\"]],[\"Telex\",[4]],[\"Tenali Ramakrishna\",[4]],[\"Tenor Sans\",[4]],[\"Text Me One\",[4]],[\"Texturina\",[\"1a9\",\"1a9i\"]],[\"Thasadith\",[4,7,\"4i\",\"7i\"]],[\"The Girl Next Door\",[4]],[\"The Nautigal\",[4,7]],[\"Tienne\",[4,7,9]],[\"TikTok Sans\",[\"3a9\"]],[\"Tillana\",[4,5,6,7,8]],[\"Tilt Neon\",[4]],[\"Tilt Prism\",[4]],[\"Tilt Warp\",[4]],[\"Timmana\",[4]],[\"Tinos\",[4,7,\"4i\",\"7i\"]],[\"Tiny5\",[4]],[\"Tiro Bangla\",[4,\"4i\"]],[\"Tiro Devanagari Hindi\",[4,\"4i\"]],[\"Tiro Devanagari Marathi\",[4,\"4i\"]],[\"Tiro Devanagari Sanskrit\",[4,\"4i\"]],[\"Tiro Gurmukhi\",[4,\"4i\"]],[\"Tiro Kannada\",[4,\"4i\"]],[\"Tiro Tamil\",[4,\"4i\"]],[\"Tiro Telugu\",[4,\"4i\"]],[\"Tirra\",[4,5,6,7,8,9]],[\"Titan One\",[4]],[\"Titillium Web\",[2,3,4,6,7,9,\"2i\",\"3i\",\"4i\",\"6i\",\"7i\"]],[\"Tomorrow\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Tourney\",[\"1a9\",\"1a9i\"]],[\"Trade Winds\",[4]],[\"Train One\",[4]],[\"Triodion\",[4]],[\"Trirong\",[1,2,3,4,5,6,7,8,9,\"1i\",\"2i\",\"3i\",\"4i\",\"5i\",\"6i\",\"7i\",\"8i\",\"9i\"]],[\"Trispace\",[\"1a8\"]],[\"Trocchi\",[4]],[\"Trochut\",[4,7,\"4i\"]],[\"Truculenta\",[\"1a9\"]],[\"Trykker\",[4]],[\"Tsukimi Rounded\",[3,4,5,6,7]],[\"Tuffy\",[4,7,\"4i\",\"7i\"]],[\"Tulpen One\",[4]],[\"Turret Road\",[2,3,4,5,7,8]],[\"Twinkle Star\",[4]],[\"Ubuntu\",[3,4,5,7,\"3i\",\"4i\",\"5i\",\"7i\"]],[\"Ubuntu Condensed\",[4]],[\"Ubuntu Mono\",[4,7,\"4i\",\"7i\"]],[\"Ubuntu Sans\",[\"1a8\",\"1a8i\"]],[\"Ubuntu Sans Mono\",[\"4a7\",\"4a7i\"]],[\"Uchen\",[4]],[\"Ultra\",[4]],[\"Unbounded\",[\"2a9\"]],[\"Uncial Antiqua\",[4]],[\"Underdog\",[4]],[\"Unica One\",[4]],[\"UnifrakturCook\",[7]],[\"UnifrakturMaguntia\",[4]],[\"Unkempt\",[4,7]],[\"Unlock\",[4]],[\"Unna\",[4,7,\"4i\",\"7i\"]],[\"UoqMunThenKhung\",[4]],[\"Updock\",[4]],[\"Urbanist\",[\"1a9\",\"1a9i\"]],[\"VT323\",[4]],[\"Valley Sans\",[\"1a9\",\"1a9i\"]],[\"Vampiro One\",[4]],[\"Varela\",[4]],[\"Varela Round\",[4]],[\"Varta\",[\"3a7\"]],[\"Vast Shadow\",[4]],[\"Vazirmatn\",[\"1a9\"]],[\"Vend Sans\",[\"3a7\",\"3a7i\"]],[\"Vesper Libre\",[4,5,7,9]],[\"Viaoda Libre\",[4]],[\"Vibes\",[4]],[\"Vibur\",[4]],[\"Victor Mono\",[\"1a7\",\"1a7i\"]],[\"Vidaloka\",[4]],[\"Viga\",[4]],[\"Vina Sans\",[4]],[\"Voces\",[4]],[\"Volkhov\",[4,7,\"4i\",\"7i\"]],[\"Vollkorn\",[\"4a9\",\"4a9i\"]],[\"Vollkorn SC\",[4,6,7,9]],[\"Voltaire\",[4]],[\"Vujahday Script\",[4]],[\"WDXL Lubrifont JP N\",[4]],[\"WDXL Lubrifont SC\",[4]],[\"WDXL Lubrifont TC\",[4]],[\"Waiting for the Sunrise\",[4]],[\"Wallpoet\",[4]],[\"Walter Turncoat\",[4]],[\"Warnes\",[4]],[\"Water Brush\",[4]],[\"Waterfall\",[4]],[\"Wavefont\",[\"1a10\"]],[\"Wellfleet\",[4]],[\"Wendy One\",[4]],[\"Whisper\",[4]],[\"WindSong\",[4,5]],[\"Winky Rough\",[\"3a9\",\"3a9i\"]],[\"Winky Sans\",[\"3a9\",\"3a9i\"]],[\"Wire One\",[4]],[\"Wittgenstein\",[\"4a9\",\"4a9i\"]],[\"Wix Madefor Display\",[\"4a8\"]],[\"Wix Madefor Text\",[\"4a8\",\"4a8i\"]],[\"Work Sans\",[\"1a9\",\"1a9i\"]],[\"Workbench\",[4]],[\"Xanh Mono\",[4,\"4i\"]],[\"Yaldevi\",[\"2a7\"]],[\"Yanone Kaffeesatz\",[\"2a7\"]],[\"Yantramanav\",[1,3,4,5,7,9]],[\"Yarndings 12\",[4]],[\"Yarndings 12 Charted\",[4]],[\"Yarndings 20\",[4]],[\"Yarndings 20 Charted\",[4]],[\"Yatra One\",[4]],[\"Yellowtail\",[4]],[\"Yeon Sung\",[4]],[\"Yeseva One\",[4]],[\"Yesteryear\",[4]],[\"Yomogi\",[4]],[\"Young Serif\",[4]],[\"Yrsa\",[\"3a7\",\"3a7i\"]],[\"Ysabeau\",[\"1a10\",\"1a10i\"]],[\"Ysabeau Infant\",[\"1a10\",\"1a10i\"]],[\"Ysabeau Office\",[\"1a10\",\"1a10i\"]],[\"Ysabeau SC\",[\"1a10\"]],[\"Yuji Boku\",[4]],[\"Yuji Hentaigana Akari\",[4]],[\"Yuji Hentaigana Akebono\",[4]],[\"Yuji Mai\",[4]],[\"Yuji Syuku\",[4]],[\"Yusei Magic\",[4]],[\"Yuyu\",[4]],[\"Yuyu Short\",[4]],[\"ZCOOL KuaiLe\",[4]],[\"ZCOOL QingKe HuangYou\",[4]],[\"ZCOOL XiaoWei\",[4]],[\"Zain\",[2,3,4,7,8,9,\"3i\",\"4i\"]],[\"Zalando Sans\",[\"2a9\",\"2a9i\"]],[\"Zalando Sans Expanded\",[\"2a9\",\"2a9i\"]],[\"Zalando Sans SemiExpanded\",[\"2a9\",\"2a9i\"]],[\"Zen Antique\",[4]],[\"Zen Antique Soft\",[4]],[\"Zen Dots\",[4]],[\"Zen Kaku Gothic Antique\",[3,4,5,7,9]],[\"Zen Kaku Gothic New\",[3,4,5,7,9]],[\"Zen Kurenaido\",[4]],[\"Zen Loop\",[4,\"4i\"]],[\"Zen Maru Gothic\",[3,4,5,7,9]],[\"Zen Old Mincho\",[4,5,6,7,9]],[\"Zen Tokyo Zoo\",[4]],[\"Zeyada\",[4]],[\"Zhi Mang Xing\",[4]],[\"Zilla Slab\",[3,4,5,6,7,\"3i\",\"4i\",\"5i\",\"6i\",\"7i\"]],[\"Zilla Slab Highlight\",[4,7]]]");
	}));
	"queryLocalFonts" in globalThis && globalThis.queryLocalFonts;
	const xa = {
		100: "Thin",
		200: "ExtraLight",
		300: "Light",
		400: "Regular",
		500: "Medium",
		600: "SemiBold",
		700: "Bold",
		800: "ExtraBold",
		900: "Black",
		1e3: "UltraBlack"
	};
	var Ve = /* @__PURE__ */ new WeakMap(), Ce = /* @__PURE__ */ new WeakMap(), Da = class {
		constructor({ family: i, weight: r, isItalic: s, weightName: h }) {
			S(this, "family", void 0), S(this, "fullName", void 0), S(this, "postscriptName", void 0), S(this, "style", void 0), Na(this, Ve, void 0), Na(this, Ce, void 0), this.family = i, Ba(Ve, this, r), Ba(Ce, this, s), h === "Regular" ? s ? this.style = "Italic" : this.style = "Regular" : this.style = h + (s ? " Italic" : ""), this.style === "Regular" ? this.fullName = i : this.fullName = `${i} ${this.style}`, this.postscriptName = this.fullName.replace(/ /g, "-");
		}
		async blob() {
			const i = await (await fetch(`https://fonts.googleapis.com/css2?family=${this.family}:${We(Ce, this) ? "ital," : ""}wght@${We(Ce, this) ? "1," : ""}${We(Ve, this)}`)).text();
			for (const r of [
				/\/\* latin-ext \*\/[\s\S]+url\(([^)]+)\)/,
				/\/\* latin \*\/[\s\S]+url\(([^)]+)\)/,
				/url\(([^)]+)\)/
			]) {
				const s = i.match(r)?.[1];
				if (s) {
					const h = await fetch(s);
					if (h.ok) return h.blob();
				}
			}
			throw new Error("Failed to load font blob");
		}
	};
	function _i([i, r]) {
		const s = [];
		for (const h of r) {
			const m = "" + h, v = m.endsWith("i"), f = m.replace("i", "");
			if (f.includes("a")) {
				const [p, C] = f.split("a").map((P) => parseInt(P) * 100);
				for (let P = p; P <= C; P += 100) s.push(new Da({
					family: i,
					weight: `${p}..${C}`,
					isItalic: v,
					weightName: xa[P]
				}));
			} else s.push(new Da({
				family: i,
				weight: `${f}00`,
				isItalic: v,
				weightName: xa[parseInt(f) * 100]
			}));
		}
		return s;
	}
	let Ke = /* @__PURE__ */ new Map();
	async function pi() {
		if (Ke.size === 0) {
			const i = await Promise.resolve().then(() => (yi(), mi));
			for (const r of i.default.flatMap((s) => _i(s))) Ke.set(r.postscriptName.toLowerCase().replace(/-/g, ""), r);
		}
		return Ke;
	}
	async function bi({ postscriptNames: i } = {}) {
		const r = await pi();
		return i ? i.length === 0 ? [] : i.reduce((s, h) => {
			const m = r.get(h.toLowerCase().replace(/regular$/, "").replace(/-reg$/, "").replace(/[- ]/g, ""));
			return m && s.push(m), s;
		}, []) : [...r.values()];
	}
	if (self.name.startsWith("em-pthread")) {
		const i = self.name.split("-").slice(2).join("-"), r = globalThis.fetch;
		globalThis.fetch = (s) => r(i), self.name = "em-pthread";
	}
	async function La(i = {}) {
		var r = i, s = !0, h = !!globalThis.WorkerGlobalScope, m = (...e) => console.log(...e), v = (...e) => console.error(...e);
		function f() {
			p && Ja();
		}
		var p = h && globalThis.name == "em-pthread";
		h && (C = self.location.href);
		var C;
		const P = !!WebAssembly.Memory.prototype.toResizableBuffer;
		if (oe = () => {
			if (!(P && self.HEAPU8RAW)) {
				var e = P ? W.toResizableBuffer() : W.buffer;
				ee = new Int8Array(e), ce = new Int16Array(e), F = new Uint8Array(e), he = new Uint16Array(e), z = new Int32Array(e), B = new Uint32Array(e), la = new Float32Array(e), ae = new Float64Array(e), K = new BigInt64Array(e), aa = new BigUint64Array(e), self.HEAPU8RAW = new Uint8Array(e), self.WASMMEMORY = W;
			}
		}, i.__out && (m = i.__out), i.__err && (v = i.__err), !self.name.startsWith("em-pthread")) {
			const e = globalThis.Worker;
			globalThis.Worker = class extends e {
				constructor(a, t = {}) {
					super(a, {
						...t,
						name: "em-pthread-" + i.__url
					});
				}
			};
		}
		const b = Ue;
		Ue = function() {
			b(), Sa = new Proxy({}, { get() {
				return (e) => rt(D.toValue(e));
			} });
		};
		function N(e) {
			throw e;
		}
		class k {}
		class G extends k {}
		function g() {
			W.buffer != ee.buffer && oe();
		}
		var Ja;
		if (p) {
			let e = function(a) {
				try {
					var t = a.data, n = t.cmd;
					if (n == 1) {
						let o = [];
						self.onmessage = (l) => o.push(l), Ja = () => {
							postMessage({ cmd: 3 });
							for (let l of o) e(l);
							self.onmessage = e;
						};
						for (const l of t.handlers) (!r[l] || r[l].proxy) && (r[l] = (...c) => {
							postMessage({
								cmd: 9,
								handler: l,
								args: c
							});
						}, l == "print" && (m = r[l]), l == "printErr" && (v = r[l]));
						W = t.wasmMemory, oe(), r.wasm = t.wasmModule, Ue();
					} else if (n == 2) {
						Oi(t.pthread_ptr), va(t.pthread_ptr, 0, 0, 1, 0, 0), T.threadInitTLS(), ca(t.pthread_ptr), qe || (Ft(), qe = !0);
						try {
							Gi(t.start_routine, t.arg);
						} catch (o) {
							if (o != "unwind") throw o;
						}
					} else n == 4 ? qe && Fe() : n && (v(`worker: received unknown command ${n}`), v(t));
				} catch (o) {
					throw Za && Gt(), o;
				}
			};
			var qe = !1;
			self.onunhandledrejection = (a) => {
				throw a.reason || a;
			}, self.onmessage = e;
		}
		var Za = !1;
		function Bi() {
			return W.buffer;
		}
		function oe() {
			if (!ee?.buffer?.growable) {
				var e = Bi();
				ee = new Int8Array(e), ce = new Int16Array(e), F = new Uint8Array(e), he = new Uint16Array(e), z = new Int32Array(e), B = new Uint32Array(e), la = new Float32Array(e), ae = new Float64Array(e), K = new BigInt64Array(e), aa = new BigUint64Array(e);
			}
		}
		function Ei() {
			p || (W = new WebAssembly.Memory({
				initial: 512,
				maximum: 32768,
				shared: !0
			}), oe());
		}
		Ei();
		var ee, Qa = (e) => {
			e.terminate(), e.onmessage = (a) => {};
		}, et = (e) => {
			var a = T.pthreads[e];
			T.returnWorkerToPool(a);
		}, xi = (e) => {
			for (; e.length > 0;) e.shift()(r);
		}, Di = [], at = (e) => {
			var a = T.getNewWorker();
			if (!a) return 6;
			T.pthreads[e.pthread_ptr] = a, a.pthread_ptr = e.pthread_ptr;
			var t = {
				cmd: 2,
				start_routine: e.startRoutine,
				arg: e.arg,
				pthread_ptr: e.pthread_ptr
			};
			return a.postMessage(t, e.transferList), 0;
		}, Re = () => jt(), se = (e) => zt(e), Li = (e) => Yt(e), ae, K, I = (e, a, t, ...n) => {
			var o = 8 * n.length * 2, l = Re(), c = Li(o), u = c >> 3;
			for (var d of n) typeof d == "bigint" ? ((g(), K)[u++] = 1n, (g(), K)[u++] = d) : ((g(), K)[u++] = 0n, (g(), ae)[u++] = d);
			var y = Ut(e, a, o, c, t);
			return se(l), y;
		};
		function Xe(e) {
			if (p) return I(0, 0, 1, e);
			throw `exit(${e})`;
		}
		var Fi = Xe, tt = !Atomics.waitAsync || globalThis.navigator?.userAgent && Number((navigator.userAgent.match(/Chrom(e|ium)\/([0-9]+)\./) || [])[2]) < 91, z, T = {
			unusedWorkers: [],
			tlsInitFunctions: [],
			pthreads: {},
			init() {
				p || T.initMainThread();
			},
			initMainThread() {
				for (var e = !navigator.userAgent.toLowerCase().includes("firefox") && self.crossOriginIsolated ? Math.min(Math.max(0, navigator.hardwareConcurrency - 2), 8) : 0; e--;) T.allocateUnusedWorker();
			},
			terminateAllThreads: () => {
				for (var e of Object.values(T.pthreads)) Qa(e);
				for (var e of T.unusedWorkers) Qa(e);
				T.unusedWorkers = [], T.pthreads = {};
			},
			terminateRuntime: () => {
				T.terminateAllThreads();
				var e = Oe();
				Ot(0, 0, 0, 1), tt || Atomics.notify((g(), z), e >> 2);
			},
			returnWorkerToPool: (e) => {
				var a = e.pthread_ptr;
				delete T.pthreads[a], T.unusedWorkers.push(e), e.pthread_ptr = 0, $t(a);
			},
			threadInitTLS() {
				T.tlsInitFunctions.forEach((e) => e());
			},
			loadWasmModuleToWorker: (e) => new Promise((a) => {
				e.onmessage = (o) => {
					var l = o.data, c = l.cmd;
					if (l.targetThread && l.targetThread != Oe()) {
						T.pthreads[l.targetThread]?.postMessage(l);
						return;
					}
					if (l === "setimmediate" || l === "_si") {
						e.postMessage(l);
						return;
					}
					switch (c) {
						case 4:
							Fe();
							break;
						case 5:
							at(l);
							break;
						case 6:
							ua(() => et(l.thread));
							break;
						case 3:
							a(e);
							break;
						case 9:
							r[l.handler](...l.args);
							break;
						default: c && v(`worker sent an unknown command ${c}`);
					}
				}, e.onerror = (o) => {
					throw v(`worker sent an error! ${o.filename}:${o.lineno}: ${o.message}`), o;
				};
				var t = [];
				for (var n of []) r.propertyIsEnumerable(n) && t.push(n);
				e.postMessage({
					cmd: 1,
					handlers: t,
					wasmMemory: W,
					wasmModule: Zt
				});
			}),
			async loadWasmModuleToAllWorkers() {
				if (!p) return Promise.all(T.unusedWorkers.map(T.loadWasmModuleToWorker));
			},
			allocateUnusedWorker() {
				var e = new Worker(new URL("/fluxa/assets/jassub-worker-CHE4UPzj.js", "" + self.location.href), {
					type: "module",
					name: "em-pthread"
				});
				return T.unusedWorkers.push(e), e;
			},
			getNewWorker() {
				if (T.unusedWorkers.length == 0) {
					var e = T.allocateUnusedWorker();
					T.loadWasmModuleToWorker(e);
				}
				return T.unusedWorkers.pop();
			}
		}, B;
		function Oi(e) {
			var a = (g(), B)[e + 48 >> 2], t = a - (g(), B)[e + 52 >> 2];
			Kt(a, t), se(a);
		}
		var it = [], le = (e) => {
			var a = it[e];
			return a || (it[e] = a = qt.get(e)), a;
		}, Gi = (e, a) => {
			var t = le(e)(a);
			function n(o) {
				Ht(o);
			}
			n(t);
		}, Je = (e) => {
			for (var a = 0, t = 0; t < e.length; ++t) {
				var n = e.charCodeAt(t);
				n <= 127 ? a++ : n <= 2047 ? a += 2 : n >= 55296 && n <= 57343 ? (a += 4, ++t) : a += 3;
			}
			return a;
		}, Ii = (e, a, t, n) => {
			if (!(n > 0)) return 0;
			for (var o = t, l = t + n - 1, c = 0; c < e.length; ++c) {
				var u = e.codePointAt(c);
				if (u <= 127) {
					if (t >= l) break;
					a[t++] = u;
				} else if (u <= 2047) {
					if (t + 1 >= l) break;
					a[t++] = 192 | u >> 6, a[t++] = 128 | u & 63;
				} else if (u <= 65535) {
					if (t + 2 >= l) break;
					a[t++] = 224 | u >> 12, a[t++] = 128 | u >> 6 & 63, a[t++] = 128 | u & 63;
				} else {
					if (t + 3 >= l) break;
					a[t++] = 240 | u >> 18, a[t++] = 128 | u >> 12 & 63, a[t++] = 128 | u >> 6 & 63, a[t++] = 128 | u & 63, c++;
				}
			}
			return a[t] = 0, t - o;
		}, F, Ze = (e, a, t) => Ii(e, (g(), F), a, t), rt = (e) => {
			var a = Je(e) + 1, t = Ge(a);
			return t && Ze(e, t, a), t;
		}, W, nt = new TextDecoder(), Qe = (e, a, t, n) => {
			var o = a + t;
			if (n) return o;
			for (; e[a] && !(a >= o);) ++a;
			return a;
		}, te = (e, a, t) => {
			if (!e) return "";
			var n = Qe((g(), F), e, a, t);
			return nt.decode((g(), F).slice(e, n));
		}, Ui = (e, a, t, n) => N(`Assertion failed: ${te(e)}, at: ` + [
			a ? te(a) : "unknown filename",
			t,
			n ? te(n) : "unknown function"
		]);
		function ot(e, a, t, n) {
			return p ? I(1, 0, 1, e, a, t, n) : st(e, a, t, n);
		}
		var $i = () => !!globalThis.SharedArrayBuffer, st = (e, a, t, n) => {
			if (!$i()) return 6;
			var o = [], l = 0;
			if (p && (o.length === 0 || l)) return ot(e, a, t, n);
			if (l) return l;
			var c = {
				startRoutine: t,
				pthread_ptr: e,
				arg: n,
				transferList: o
			};
			return p ? (c.cmd = 5, postMessage(c, o), 0) : at(c);
		}, ea = {
			varargs: void 0,
			getStr(e) {
				return te(e);
			}
		};
		function lt(e, a, t) {
			return p ? I(2, 0, 1, e, a, t) : (ea.varargs = t, 0);
		}
		function ut(e, a, t) {
			if (p) return I(3, 0, 1, e, a, t);
		}
		function ct(e, a, t) {
			return p ? I(4, 0, 1, e, a, t) : (ea.varargs = t, 0);
		}
		function ht(e, a, t, n) {
			if (p) return I(5, 0, 1, e, a, t, n);
			ea.varargs = n;
		}
		var Hi = () => N(""), O = (e) => {
			for (var a = "";;) {
				var t = (g(), F)[e++];
				if (!t) return a;
				a += String.fromCharCode(t);
			}
		}, ie = {}, X = {}, Me = {};
		class ue extends Error {
			constructor(a) {
				super(a), this.name = "BindingError";
			}
		}
		var w = (e) => {
			throw new ue(e);
		};
		function Wi(e, a, t = {}) {
			var n = a.name;
			if (e || w(`type "${n}" must have a positive integer typeid pointer`), X.hasOwnProperty(e)) {
				if (t.ignoreDuplicateRegistrations) return;
				w(`Cannot register type '${n}' twice`);
			}
			if (X[e] = a, delete Me[e], ie.hasOwnProperty(e)) {
				var o = ie[e];
				delete ie[e], o.forEach((l) => l());
			}
		}
		function V(e, a, t = {}) {
			return Wi(e, a, t);
		}
		var ce, he, aa, dt = (e, a, t) => {
			switch (a) {
				case 1: return t ? (n) => (g(), ee)[n] : (n) => (g(), F)[n];
				case 2: return t ? (n) => (g(), ce)[n >> 1] : (n) => (g(), he)[n >> 1];
				case 4: return t ? (n) => (g(), z)[n >> 2] : (n) => (g(), B)[n >> 2];
				case 8: return t ? (n) => (g(), K)[n >> 3] : (n) => (g(), aa)[n >> 3];
				default: throw new TypeError(`invalid integer width (${a}): ${e}`);
			}
		}, Vi = (e, a, t, n, o) => {
			a = O(a);
			const l = n === 0n;
			let c = (u) => u;
			if (l) {
				const u = t * 8;
				c = (d) => BigInt.asUintN(u, d), o = c(o);
			}
			V(e, {
				name: a,
				fromWireType: c,
				toWireType: (u, d) => (typeof d == "number" && (d = BigInt(d)), d),
				readValueFromPointer: dt(a, t, !l),
				destructorFunction: null
			});
		}, Ki = (e, a, t, n) => {
			a = O(a), V(e, {
				name: a,
				fromWireType: function(o) {
					return !!o;
				},
				toWireType: function(o, l) {
					return l ? t : n;
				},
				readValueFromPointer: function(o) {
					return this.fromWireType((g(), F)[o]);
				},
				destructorFunction: null
			});
		}, zi = (e) => ({
			count: e.count,
			deleteScheduled: e.deleteScheduled,
			preservePointerOnDelete: e.preservePointerOnDelete,
			ptr: e.ptr,
			ptrType: e.ptrType,
			smartPtr: e.smartPtr,
			smartPtrType: e.smartPtrType
		}), ta = (e) => {
			function a(t) {
				return t.$$.ptrType.registeredClass.name;
			}
			w(a(e) + " instance already deleted");
		}, ia = !1, ft = (e) => {}, Yi = (e) => {
			e.smartPtr ? e.smartPtrType.rawDestructor(e.smartPtr) : e.ptrType.registeredClass.rawDestructor(e.ptr);
		}, gt = (e) => {
			e.count.value -= 1, e.count.value === 0 && Yi(e);
		}, de = (e) => globalThis.FinalizationRegistry ? (ia = new FinalizationRegistry((a) => {
			gt(a.$$);
		}), de = (a) => {
			var t = a.$$;
			if (t.smartPtr) {
				var n = { $$: t };
				ia.register(a, n, a);
			}
			return a;
		}, ft = (a) => ia.unregister(a), de(e)) : (de = (a) => a, e), Ne = [], ji = () => {
			for (; Ne.length;) {
				var e = Ne.pop();
				e.$$.deleteScheduled = !1, e.delete();
			}
		}, St, qi = () => {
			let e = ke.prototype;
			Object.assign(e, {
				isAliasOf(t) {
					if (!(this instanceof ke) || !(t instanceof ke)) return !1;
					var n = this.$$.ptrType.registeredClass, o = this.$$.ptr;
					t.$$ = t.$$;
					for (var l = t.$$.ptrType.registeredClass, c = t.$$.ptr; n.baseClass;) o = n.upcast(o), n = n.baseClass;
					for (; l.baseClass;) c = l.upcast(c), l = l.baseClass;
					return n === l && o === c;
				},
				clone() {
					if (this.$$.ptr || ta(this), this.$$.preservePointerOnDelete) return this.$$.count.value += 1, this;
					var t = de(Object.create(Object.getPrototypeOf(this), { $$: { value: zi(this.$$) } }));
					return t.$$.count.value += 1, t.$$.deleteScheduled = !1, t;
				},
				delete() {
					this.$$.ptr || ta(this), this.$$.deleteScheduled && !this.$$.preservePointerOnDelete && w("Object already scheduled for deletion"), ft(this), gt(this.$$), this.$$.preservePointerOnDelete || (this.$$.smartPtr = void 0, this.$$.ptr = void 0);
				},
				isDeleted() {
					return !this.$$.ptr;
				},
				deleteLater() {
					return this.$$.ptr || ta(this), this.$$.deleteScheduled && !this.$$.preservePointerOnDelete && w("Object already scheduled for deletion"), Ne.push(this), Ne.length === 1 && St && St(ji), this.$$.deleteScheduled = !0, this;
				}
			});
			const a = Symbol.dispose;
			a && (e[a] = e.delete);
		};
		function ke() {}
		var ra = (e, a) => Object.defineProperty(a, "name", { value: e }), vt = {}, mt = (e, a, t) => {
			if (e[a].overloadTable === void 0) {
				var n = e[a];
				e[a] = function(...o) {
					return e[a].overloadTable.hasOwnProperty(o.length) || w(`Function '${t}' called with an invalid number of arguments (${o.length}) - expects one of (${e[a].overloadTable})!`), e[a].overloadTable[o.length].apply(this, o);
				}, e[a].overloadTable = [], e[a].overloadTable[n.argCount] = n;
			}
		}, Xi = (e, a, t) => {
			r.hasOwnProperty(e) ? ((t === void 0 || r[e].overloadTable !== void 0 && r[e].overloadTable[t] !== void 0) && w(`Cannot register public name '${e}' twice`), mt(r, e, e), r[e].overloadTable.hasOwnProperty(t) && w(`Cannot register multiple overloads of a function with the same number of arguments (${t})!`), r[e].overloadTable[t] = a) : (r[e] = a, r[e].argCount = t);
		}, Ji = 48, Zi = 57, Qi = (e) => {
			e = e.replace(/[^a-zA-Z0-9_]/g, "$");
			var a = e.charCodeAt(0);
			return a >= Ji && a <= Zi ? `_${e}` : e;
		};
		function er(e, a, t, n, o, l, c, u) {
			this.name = e, this.constructor = a, this.instancePrototype = t, this.rawDestructor = n, this.baseClass = o, this.getActualType = l, this.upcast = c, this.downcast = u, this.pureVirtualFunctions = [];
		}
		var Be = (e, a, t) => {
			for (; a !== t;) a.upcast || w(`Expected null or instance of ${t.name}, got an instance of ${a.name}`), e = a.upcast(e), a = a.baseClass;
			return e;
		}, na = (e) => {
			if (e === null) return "null";
			var a = typeof e;
			return a === "object" || a === "array" || a === "function" ? e.toString() : "" + e;
		};
		function ar(e, a) {
			if (a === null) return this.isReference && w(`null is not a valid ${this.name}`), 0;
			a.$$ || w(`Cannot pass "${na(a)}" as a ${this.name}`), a.$$.ptr || w(`Cannot pass deleted object as a pointer of type ${this.name}`);
			var t = a.$$.ptrType.registeredClass;
			return Be(a.$$.ptr, t, this.registeredClass);
		}
		function tr(e, a) {
			var t;
			if (a === null) return this.isReference && w(`null is not a valid ${this.name}`), this.isSmartPointer ? (t = this.rawConstructor(), e !== null && e.push(this.rawDestructor, t), t) : 0;
			(!a || !a.$$) && w(`Cannot pass "${na(a)}" as a ${this.name}`), a.$$.ptr || w(`Cannot pass deleted object as a pointer of type ${this.name}`), !this.isConst && a.$$.ptrType.isConst && w(`Cannot convert argument of type ${a.$$.smartPtrType ? a.$$.smartPtrType.name : a.$$.ptrType.name} to parameter type ${this.name}`);
			var n = a.$$.ptrType.registeredClass;
			if (t = Be(a.$$.ptr, n, this.registeredClass), this.isSmartPointer) switch (a.$$.smartPtr === void 0 && w("Passing raw pointer to smart pointer is illegal"), this.sharingPolicy) {
				case 0:
					a.$$.smartPtrType === this ? t = a.$$.smartPtr : w(`Cannot convert argument of type ${a.$$.smartPtrType ? a.$$.smartPtrType.name : a.$$.ptrType.name} to parameter type ${this.name}`);
					break;
				case 1:
					t = a.$$.smartPtr;
					break;
				case 2:
					if (a.$$.smartPtrType === this) t = a.$$.smartPtr;
					else {
						var o = a.clone();
						t = this.rawShare(t, D.toHandle(() => o.delete())), e !== null && e.push(this.rawDestructor, t);
					}
					break;
				default: w("Unsupported sharing policy");
			}
			return t;
		}
		function ir(e, a) {
			if (a === null) return this.isReference && w(`null is not a valid ${this.name}`), 0;
			a.$$ || w(`Cannot pass "${na(a)}" as a ${this.name}`), a.$$.ptr || w(`Cannot pass deleted object as a pointer of type ${this.name}`), a.$$.ptrType.isConst && w(`Cannot convert argument of type ${a.$$.ptrType.name} to parameter type ${this.name}`);
			var t = a.$$.ptrType.registeredClass;
			return Be(a.$$.ptr, t, this.registeredClass);
		}
		function Ee(e) {
			return this.fromWireType((g(), B)[e >> 2]);
		}
		var yt = (e, a, t) => {
			if (a === t) return e;
			if (t.baseClass === void 0) return null;
			var n = yt(e, a, t.baseClass);
			return n === null ? null : t.downcast(n);
		}, rr = {}, nr = (e, a) => {
			for (a === void 0 && w("ptr should not be undefined"); e.baseClass;) a = e.upcast(a), e = e.baseClass;
			return a;
		}, or = (e, a) => (a = nr(e, a), rr[a]);
		class sr extends Error {
			constructor(a) {
				super(a), this.name = "InternalError";
			}
		}
		var xe = (e) => {
			throw new sr(e);
		}, De = (e, a) => ((!a.ptrType || !a.ptr) && xe("makeClassHandle requires ptr and ptrType"), !!a.smartPtrType != !!a.smartPtr && xe("Both smartPtrType and smartPtr must be specified"), a.count = { value: 1 }, de(Object.create(e, { $$: {
			value: a,
			writable: !0
		} })));
		function lr(e) {
			var a = this.getPointee(e);
			if (!a) return this.destructor(e), null;
			var t = or(this.registeredClass, a);
			if (t !== void 0) {
				if (t.$$.count.value === 0) return t.$$.ptr = a, t.$$.smartPtr = e, t.clone();
				var n = t.clone();
				return this.destructor(e), n;
			}
			function o() {
				return this.isSmartPointer ? De(this.registeredClass.instancePrototype, {
					ptrType: this.pointeeType,
					ptr: a,
					smartPtrType: this,
					smartPtr: e
				}) : De(this.registeredClass.instancePrototype, {
					ptrType: this,
					ptr: e
				});
			}
			var l = vt[this.registeredClass.getActualType(a)];
			if (!l) return o.call(this);
			var c;
			this.isConst ? c = l.constPointerType : c = l.pointerType;
			var u = yt(a, this.registeredClass, c.registeredClass);
			return u === null ? o.call(this) : this.isSmartPointer ? De(c.registeredClass.instancePrototype, {
				ptrType: c,
				ptr: u,
				smartPtrType: this,
				smartPtr: e
			}) : De(c.registeredClass.instancePrototype, {
				ptrType: c,
				ptr: u
			});
		}
		var ur = () => {
			Object.assign(Le.prototype, {
				getPointee(e) {
					return this.rawGetPointee && (e = this.rawGetPointee(e)), e;
				},
				destructor(e) {
					this.rawDestructor?.(e);
				},
				readValueFromPointer: Ee,
				fromWireType: lr
			});
		};
		function Le(e, a, t, n, o, l, c, u, d, y, _) {
			this.name = e, this.registeredClass = a, this.isReference = t, this.isConst = n, this.isSmartPointer = o, this.pointeeType = l, this.sharingPolicy = c, this.rawGetPointee = u, this.rawConstructor = d, this.rawShare = y, this.rawDestructor = _, !o && a.baseClass === void 0 ? n ? (this.toWireType = ar, this.destructorFunction = null) : (this.toWireType = ir, this.destructorFunction = null) : this.toWireType = tr;
		}
		var cr = (e, a, t) => {
			r.hasOwnProperty(e) || xe("Replacing nonexistent public symbol"), r[e].overloadTable !== void 0 && t !== void 0 ? r[e].overloadTable[t] = a : (r[e] = a, r[e].argCount = t);
		}, j = (e, a, t = !1) => {
			e = O(e);
			function n() {
				return le(a);
			}
			var o = n();
			return typeof o != "function" && w(`unknown function pointer with signature ${e}: ${a}`), o;
		};
		class hr extends Error {}
		var _t = (e) => {
			var a = Lt(e), t = O(a);
			return Y(a), t;
		}, fe = (e, a) => {
			var t = [], n = {};
			function o(l) {
				if (!n[l] && !X[l]) {
					if (Me[l]) {
						Me[l].forEach(o);
						return;
					}
					t.push(l), n[l] = !0;
				}
			}
			throw a.forEach(o), new hr(`${e}: ` + t.map(_t).join([", "]));
		}, J = (e, a, t) => {
			e.forEach((u) => Me[u] = a);
			function n(u) {
				var d = t(u);
				d.length !== e.length && xe("Mismatched type converter count");
				for (var y = 0; y < e.length; ++y) V(e[y], d[y]);
			}
			var o = new Array(a.length), l = [], c = 0;
			for (let [u, d] of a.entries()) X.hasOwnProperty(d) ? o[u] = X[d] : (l.push(d), ie.hasOwnProperty(d) || (ie[d] = []), ie[d].push(() => {
				o[u] = X[d], ++c, c === l.length && n(o);
			}));
			l.length === 0 && n(o);
		}, dr = (e, a, t, n, o, l, c, u, d, y, _, A, R) => {
			_ = O(_), l = j(o, l), u &&= j(c, u), y &&= j(d, y), R = j(A, R);
			var M = Qi(_);
			Xi(M, function() {
				fe(`Cannot construct ${_} due to unbound types`, [n]);
			}), J([
				e,
				a,
				t
			], n ? [n] : [], (E) => {
				E = E[0];
				var x, L;
				n ? (x = E.registeredClass, L = x.instancePrototype) : L = ke.prototype;
				var H = ra(_, function(...ma) {
					if (Object.getPrototypeOf(this) !== ve) throw new ue(`Use 'new' to construct ${_}`);
					if (U.constructor_body === void 0) throw new ue(`${_} has no accessible constructor`);
					var ai = U.constructor_body[ma.length];
					if (ai === void 0) throw new ue(`Tried to invoke ctor of ${_} with invalid number of parameters (${ma.length}) - expected (${Object.keys(U.constructor_body).toString()}) parameters instead!`);
					return ai.apply(this, ma);
				}), ve = Object.create(L, { constructor: { value: H } });
				H.prototype = ve;
				var U = new er(_, H, ve, R, x, l, u, y);
				U.baseClass && (U.baseClass.__derivedClasses ??= [], U.baseClass.__derivedClasses.push(U));
				var En = new Le(_, U, !0, !1, !1), Qt = new Le(_ + "*", U, !1, !1, !1), ei = new Le(_ + " const*", U, !1, !0, !1);
				return vt[e] = {
					pointerType: Qt,
					constPointerType: ei
				}, cr(M, H), [
					En,
					Qt,
					ei
				];
			});
		}, pt = (e, a) => {
			for (var t = [], n = 0; n < e; n++) t.push((g(), B)[a + n * 4 >> 2]);
			return t;
		}, oa = (e) => {
			for (; e.length;) {
				var a = e.pop();
				e.pop()(a);
			}
		};
		function fr(e) {
			for (var a = 1; a < e.length; ++a) if (e[a] !== null && e[a].destructorFunction === void 0) return !0;
			return !1;
		}
		var gr = {
			ftfnnn: function(a, t, n, o, l, c, u, d, y, _) {
				return function(A, R, M) {
					return c(n(o, d(null, A), y(null, R), _(null, M)));
				};
			},
			tffn: function(a, t, n, o, l, c, u) {
				return function() {
					n(o, u(null, this));
				};
			},
			tffnn: function(a, t, n, o, l, c, u, d) {
				return function(y) {
					n(o, u(null, this), d(null, y));
				};
			},
			tffnnnnn: function(a, t, n, o, l, c, u, d, y, _, A) {
				return function(R, M, E, x) {
					n(o, u(null, this), d(null, R), y(null, M), _(null, E), A(null, x));
				};
			},
			tffnnn: function(a, t, n, o, l, c, u, d, y) {
				return function(_, A) {
					n(o, u(null, this), d(null, _), y(null, A));
				};
			},
			ttfnn: function(a, t, n, o, l, c, u, d) {
				return function(y) {
					return c(n(o, u(null, this), d(null, y)));
				};
			},
			tffnt: function(a, t, n, o, l, c, u, d, y) {
				return function(_) {
					var A = u(null, this), R = d(null, _);
					n(o, A, R), y(R);
				};
			},
			tffntnn: function(a, t, n, o, l, c, u, d, y, _, A) {
				return function(R, M, E) {
					var x = u(null, this), L = d(null, R);
					n(o, x, L, y(null, M), _(null, E)), A(L);
				};
			},
			tffnnnn: function(a, t, n, o, l, c, u, d, y, _) {
				return function(A, R, M) {
					n(o, u(null, this), d(null, A), y(null, R), _(null, M));
				};
			},
			ttfn: function(a, t, n, o, l, c, u) {
				return function() {
					return c(n(o, u(null, this)));
				};
			},
			ttfnnn: function(a, t, n, o, l, c, u, d, y) {
				return function(_, A) {
					return c(n(o, u(null, this), d(null, _), y(null, A)));
				};
			}
		};
		function Sr(e, a, t, n) {
			const o = [
				a ? "t" : "f",
				t ? "t" : "f",
				n ? "t" : "f"
			];
			for (let l = a ? 1 : 2; l < e.length; ++l) {
				const c = e[l];
				let u = "";
				c.destructorFunction === void 0 ? u = "u" : c.destructorFunction === null ? u = "n" : u = "t", o.push(u);
			}
			return o.join("");
		}
		function bt(e, a, t, n, o, l) {
			var c = a.length;
			c < 2 && w("argTypes array size mismatch! Must at least get return value and receiver (this) types!");
			for (var u = a[1] !== null && t !== null, d = fr(a), y = !a[0].isVoid, _ = a[0], A = a[1], R = [
				e,
				w,
				n,
				o,
				oa,
				_.fromWireType.bind(_),
				A?.toWireType.bind(A)
			], M = 2; M < c; ++M) {
				var E = a[M];
				R.push(E.toWireType.bind(E));
			}
			if (!d) for (var M = u ? 1 : 2; M < a.length; ++M) a[M].destructorFunction !== null && R.push(a[M].destructorFunction);
			return ra(e, gr[Sr(a, u, y, l)](...R));
		}
		var vr = (e, a, t, n, o, l) => {
			var c = pt(a, t);
			o = j(n, o), J([], [e], (u) => {
				u = u[0];
				var d = `constructor ${u.name}`;
				if (u.registeredClass.constructor_body === void 0 && (u.registeredClass.constructor_body = []), u.registeredClass.constructor_body[a - 1] !== void 0) throw new ue(`Cannot register multiple constructors with identical number of parameters (${a - 1}) for class '${u.name}'! Overload resolution is currently only performed using the parameter count, not actual type info!`);
				return u.registeredClass.constructor_body[a - 1] = () => {
					fe(`Cannot construct ${u.name} due to unbound types`, c);
				}, J([], c, (y) => (y.splice(1, 0, null), u.registeredClass.constructor_body[a - 1] = bt(d, y, null, o, l), [])), [];
			});
		}, mr = (e) => {
			e = e.trim();
			const a = e.indexOf("(");
			return a === -1 ? e : e.slice(0, a);
		}, yr = (e, a, t, n, o, l, c, u, d, y) => {
			var _ = pt(t, n);
			a = O(a), a = mr(a), l = j(o, l, d), J([], [e], (A) => {
				A = A[0];
				var R = `${A.name}.${a}`;
				a.startsWith("@@") && (a = Symbol[a.substring(2)]), u && A.registeredClass.pureVirtualFunctions.push(a);
				function M() {
					fe(`Cannot call ${R} due to unbound types`, _);
				}
				var E = A.registeredClass.instancePrototype, x = E[a];
				return x === void 0 || x.overloadTable === void 0 && x.className !== A.name && x.argCount === t - 2 ? (M.argCount = t - 2, M.className = A.name, E[a] = M) : (mt(E, a, R), E[a].overloadTable[t - 2] = M), J([], _, (L) => {
					var H = bt(R, L, A, l, c, d);
					return E[a].overloadTable === void 0 ? (H.argCount = t - 2, E[a] = H) : E[a].overloadTable[t - 2] = H, [];
				}), [];
			});
		}, At = (e, a, t) => (e instanceof Object || w(`${t} with invalid "this": ${e}`), e instanceof a.registeredClass.constructor || w(`${t} incompatible with "this" of type ${e.constructor.name}`), e.$$.ptr || w(`cannot call emscripten binding method ${t} on deleted object`), Be(e.$$.ptr, e.$$.ptrType.registeredClass, a.registeredClass)), _r = (e, a, t, n, o, l, c, u, d, y) => {
			a = O(a), o = j(n, o), J([], [e], (_) => {
				_ = _[0];
				var A = `${_.name}.${a}`, R = {
					get() {
						fe(`Cannot access ${A} due to unbound types`, [t, c]);
					},
					enumerable: !0,
					configurable: !0
				};
				return d ? R.set = () => fe(`Cannot access ${A} due to unbound types`, [t, c]) : R.set = (M) => w(A + " is a read-only property"), Object.defineProperty(_.registeredClass.instancePrototype, a, R), J([], d ? [t, c] : [t], (M) => {
					var E = M[0], x = {
						get() {
							var H = At(this, _, A + " getter");
							return E.fromWireType(o(l, H));
						},
						enumerable: !0
					};
					if (d) {
						d = j(u, d);
						var L = M[1];
						x.set = function(H) {
							var ve = At(this, _, A + " setter"), U = [];
							d(y, ve, L.toWireType(U, H)), oa(U);
						};
					}
					return Object.defineProperty(_.registeredClass.instancePrototype, a, x), [];
				}), [];
			});
		}, wt = [], q = [
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
		], sa = (e) => {
			e > 9 && --q[e + 1] === 0 && (q[e], q[e] = void 0, wt.push(e));
		}, D = {
			toValue: (e) => (e || w(`Cannot use deleted val. handle = ${e}`), q[e]),
			toHandle: (e) => {
				switch (e) {
					case void 0: return 2;
					case null: return 4;
					case !0: return 6;
					case !1: return 8;
					default: {
						const a = wt.pop() || q.length;
						return q[a] = e, q[a + 1] = 1, a;
					}
				}
			}
		}, pr = {
			name: "emscripten::val",
			fromWireType: (e) => {
				var a = D.toValue(e);
				return sa(e), a;
			},
			toWireType: (e, a) => D.toHandle(a),
			readValueFromPointer: Ee,
			destructorFunction: null
		}, br = (e) => V(e, pr), la, Ar = (e, a) => {
			switch (a) {
				case 4: return function(t) {
					return this.fromWireType((g(), la)[t >> 2]);
				};
				case 8: return function(t) {
					return this.fromWireType((g(), ae)[t >> 3]);
				};
				default: throw new TypeError(`invalid float width (${a}): ${e}`);
			}
		}, wr = (e, a, t) => {
			a = O(a), V(e, {
				name: a,
				fromWireType: (n) => n,
				toWireType: (n, o) => o,
				readValueFromPointer: Ar(a, t),
				destructorFunction: null
			});
		}, Pr = (e, a, t, n, o) => {
			a = O(a);
			const l = n === 0;
			let c = (d) => d;
			if (l) {
				var u = 32 - 8 * t;
				c = (d) => d << u >>> u, o = c(o);
			}
			V(e, {
				name: a,
				fromWireType: c,
				toWireType: (d, y) => y,
				readValueFromPointer: dt(a, t, n !== 0),
				destructorFunction: null
			});
		}, Cr = (e, a, t) => {
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
			][a];
			function o(l) {
				var c = (g(), B)[l >> 2], u = (g(), B)[l + 4 >> 2];
				return new n((g(), ee).buffer, u, c);
			}
			t = O(t), V(e, {
				name: t,
				fromWireType: o,
				readValueFromPointer: o
			}, { ignoreDuplicateRegistrations: !0 });
		}, Tr = (e, a) => {
			a = O(a);
			var t = !0;
			V(e, {
				name: a,
				fromWireType(n) {
					var o = (g(), B)[n >> 2], l = n + 4, c;
					if (t) c = te(l, o, !0);
					else {
						c = "";
						for (var u = 0; u < o; ++u) c += String.fromCharCode((g(), F)[l + u]);
					}
					return Y(n), c;
				},
				toWireType(n, o) {
					o instanceof ArrayBuffer && (o = new Uint8Array(o));
					var l, c = typeof o == "string";
					c || ArrayBuffer.isView(o) && o.BYTES_PER_ELEMENT == 1 || w("Cannot pass non-string to std::string"), t && c ? l = Je(o) : l = o.length;
					var u = Ge(4 + l + 1), d = u + 4;
					if ((g(), B)[u >> 2] = l, c) if (t) Ze(o, d, l + 1);
					else for (var y = 0; y < l; ++y) {
						var _ = o.charCodeAt(y);
						_ > 255 && (Y(u), w("String has UTF-16 code units that do not fit in 8 bits")), (g(), F)[d + y] = _;
					}
					else (g(), F).set(o, d);
					return n !== null && n.push(Y, u), u;
				},
				readValueFromPointer: Ee,
				destructorFunction(n) {
					Y(n);
				}
			});
		}, Rr = new TextDecoder("utf-16le"), Mr = (e, a, t) => {
			var n = e >> 1, o = Qe((g(), he), n, a / 2, t);
			return Rr.decode((g(), he).slice(n, o));
		}, Nr = (e, a, t = 2147483647) => {
			if (t < 2) return 0;
			t -= 2;
			for (var n = a, o = t < e.length * 2 ? t / 2 : e.length, l = 0; l < o; ++l) {
				var c = e.charCodeAt(l);
				(g(), ce)[a >> 1] = c, a += 2;
			}
			return (g(), ce)[a >> 1] = 0, a - n;
		}, kr = (e) => e.length * 2, Br = (e, a, t) => {
			for (var n = "", o = e >> 2, l = 0; !(l >= a / 4); l++) {
				var c = (g(), B)[o + l];
				if (!c && !t) break;
				n += String.fromCodePoint(c);
			}
			return n;
		}, Er = (e, a, t = 2147483647) => {
			if (t < 4) return 0;
			for (var n = a, o = n + t - 4, l = 0; l < e.length; ++l) {
				var c = e.codePointAt(l);
				if (c > 65535 && l++, (g(), z)[a >> 2] = c, a += 4, a + 4 > o) break;
			}
			return (g(), z)[a >> 2] = 0, a - n;
		}, xr = (e) => {
			for (var a = 0, t = 0; t < e.length; ++t) e.codePointAt(t) > 65535 && t++, a += 4;
			return a;
		}, Dr = (e, a, t) => {
			t = O(t);
			var n, o, l;
			a === 2 ? (n = Mr, o = Nr, l = kr) : (n = Br, o = Er, l = xr), V(e, {
				name: t,
				fromWireType: (c) => {
					var u = (g(), B)[c >> 2], d = n(c + 4, u * a, !0);
					return Y(c), d;
				},
				toWireType: (c, u) => {
					typeof u != "string" && w(`Cannot pass non-string to C++ string type ${t}`);
					var d = l(u), y = Ge(4 + d + a);
					return (g(), B)[y >> 2] = d / a, o(u, y + 4, d + a), c !== null && c.push(Y, y), y;
				},
				readValueFromPointer: Ee,
				destructorFunction(c) {
					Y(c);
				}
			});
		}, Lr = (e, a) => {
			a = O(a), V(e, {
				isVoid: !0,
				name: a,
				fromWireType: () => {},
				toWireType: (t, n) => {}
			});
		}, Fr = (e) => {
			va(e, !h, 1, !s, 262144, !1), T.threadInitTLS();
		}, ua = (e) => {
			e();
		}, ca = (e) => {
			if (!tt) {
				Atomics.waitAsync((g(), z), e >> 2, e).value.then(Fe);
				var a = e + 112;
				Atomics.store((g(), z), a >> 2, 1);
			}
		}, Fe = () => {
			var e = Oe();
			e && ua(() => {
				ca(e), Vt();
			});
		}, Or = (e, a) => {
			if (e == a) setTimeout(Fe);
			else if (p) postMessage({
				targetThread: e,
				cmd: 4
			});
			else {
				var t = T.pthreads[e];
				if (!t) return;
				t.postMessage({ cmd: 4 });
			}
		}, ha = [], Gr = (e, a, t, n, o, l, c) => {
			ha.length = 0;
			for (var u = o >> 3, d = o + n >> 3; u < d;) {
				var y;
				(g(), K)[u++] ? y = (g(), K)[u++] : y = (g(), ae)[u++], ha.push(y);
			}
			var _ = a ? Sa[a] : Cn[e];
			T.currentProxiedOperationCallerThread = t;
			var A = _(...ha);
			if (T.currentProxiedOperationCallerThread = 0, l) {
				A.then((R) => It(l, c, R));
				return;
			}
			return A;
		}, Ir = () => {}, Ur = (e) => {
			p ? postMessage({
				cmd: 6,
				thread: e
			}) : et(e);
		}, $r = (e) => {}, Hr = () => {
			throw new G();
		}, da = [], Wr = (e) => {
			var a = da.length;
			return da.push(e), a;
		}, Vr = (e, a) => {
			var t = X[e];
			return t === void 0 && w(`${a} has unknown type ${_t(e)}`), t;
		}, Kr = (e, a) => {
			for (var t = new Array(e), n = 0; n < e; ++n) t[n] = Vr((g(), B)[a + n * 4 >> 2], `parameter ${n}`);
			return t;
		}, zr = (e, a, t) => {
			var n = [], o = e(n, t);
			return n.length && ((g(), B)[a >> 2] = D.toHandle(n)), o;
		}, Yr = {}, Pt = (e) => {
			var a = Yr[e];
			return a === void 0 ? O(e) : a;
		}, jr = (e, a, t) => {
			var n = 8, [o, ...l] = Kr(e, a), c = o.toWireType.bind(o), u = l.map((_) => _.readValueFromPointer.bind(_));
			e--;
			var d = new Array(e), y = (_, A, R, M) => {
				for (var E = 0, x = 0; x < e; ++x) d[x] = u[x](M + E), E += n;
				var L;
				switch (t) {
					case 0:
						L = D.toValue(_).apply(null, d);
						break;
					case 2:
						L = Reflect.construct(D.toValue(_), d);
						break;
					case 3:
						L = d[0];
						break;
					case 1: L = D.toValue(_)[Pt(A)](...d);
				}
				return zr(c, R, L);
			};
			return Wr(ra(`methodCaller<(${l.map((_) => _.name)}) => ${o.name}>`, y));
		}, qr = (e, a) => (e = D.toValue(e), a = D.toValue(a), D.toHandle(e[a])), Xr = (e) => {
			e > 9 && (q[e + 1] += 1);
		}, Jr = (e, a, t, n, o) => da[e](a, t, n, o), Zr = () => D.toHandle([]), Qr = (e) => D.toHandle(Pt(e)), en = () => D.toHandle({}), an = (e) => {
			oa(D.toValue(e)), sa(e);
		}, tn = (e, a, t) => {
			e = D.toValue(e), a = D.toValue(a), t = D.toValue(t), e[a] = t;
		}, ge = {}, Ct = () => performance.timeOrigin + performance.now();
		function Tt(e, a) {
			return p ? I(6, 0, 1, e, a) : (ge[e] && (clearTimeout(ge[e].id), delete ge[e]), a && (ge[e] = {
				id: setTimeout(() => {
					delete ge[e], ua(() => Wt(e, Ct()));
				}, a),
				timeout_ms: a
			}), 0);
		}
		var rn = () => Date.now(), nn = 9007199254740992, on = -9007199254740992, sn = (e) => e < on || e > nn ? NaN : Number(e), fa = [], ln = (e, a) => {
			fa.length = 0;
			for (var t; t = (g(), F)[e++];) {
				var n = t != 105;
				n &= t != 112, a += n && a % 8 ? 4 : 0, fa.push(t == 112 ? (g(), B)[a >> 2] : t == 106 ? (g(), K)[a >> 3] : t == 105 ? (g(), z)[a >> 2] : (g(), ae)[a >> 3]), a += n ? 8 : 4;
			}
			return fa;
		}, un = (e, a, t) => {
			var n = ln(a, t);
			return Sa[e](...n);
		}, cn = (e, a, t) => un(e, a, t), hn = () => {}, dn = (e) => v(te(e)), fn = () => {
			throw "unwind";
		}, Rt = () => 2147483648, gn = () => Rt(), Sn = () => navigator.hardwareConcurrency, vn = (e, a) => Math.ceil(e / a) * a, mn = (e) => {
			var a = (e - W.buffer.byteLength + 65535) / 65536 | 0;
			try {
				return W.grow(a), oe(), 1;
			} catch {}
		}, yn = (e) => {
			var a = (g(), F).length;
			if (e >>>= 0, e <= a) return !1;
			var t = Rt();
			if (e > t) return !1;
			for (var n = 1; n <= 4; n *= 2) {
				var o = a * (1 + .2 / n);
				if (o = Math.min(o, e + 100663296), mn(Math.min(t, vn(Math.max(e, o), 65536)))) return !0;
			}
			return !1;
		}, ga = {}, _n = () => "./this.program", Se = () => {
			if (!Se.strings) {
				var e = {
					USER: "web_user",
					LOGNAME: "web_user",
					PATH: "/",
					PWD: "/",
					HOME: "/home/web_user",
					LANG: (globalThis.navigator?.language ?? "C").replace("-", "_") + ".UTF-8",
					_: _n()
				};
				for (var a in ga) ga[a] === void 0 ? delete e[a] : e[a] = ga[a];
				var t = [];
				for (var a in e) t.push(`${a}=${e[a]}`);
				Se.strings = t;
			}
			return Se.strings;
		};
		function Mt(e, a) {
			if (p) return I(7, 0, 1, e, a);
			var t = 0, n = 0;
			for (var o of Se()) {
				var l = a + t;
				(g(), B)[e + n >> 2] = l, t += Ze(o, l, 1 / 0) + 1, n += 4;
			}
			return 0;
		}
		function Nt(e, a) {
			if (p) return I(8, 0, 1, e, a);
			var t = Se();
			(g(), B)[e >> 2] = t.length;
			var n = 0;
			for (var o of t) n += Je(o) + 1;
			return (g(), B)[a >> 2] = n, 0;
		}
		function kt(e) {
			return p ? I(9, 0, 1, e) : 52;
		}
		function Bt(e, a, t, n) {
			return p ? I(10, 0, 1, e, a, t, n) : 52;
		}
		function Et(e, a, t, n) {
			return p ? I(11, 0, 1, e, a, t, n) : (a = sn(a), 70);
		}
		var pn = [
			null,
			[],
			[]
		], bn = (e, a = 0, t, n) => {
			var o = Qe(e, a, t, n);
			return nt.decode(e.buffer ? e.buffer instanceof ArrayBuffer ? e.subarray(a, o) : e.slice(a, o) : new Uint8Array(e.slice(a, o)));
		}, An = (e, a) => {
			var t = pn[e];
			a === 0 || a === 10 ? ((e === 1 ? m : v)(bn(t)), t.length = 0) : t.push(a);
		};
		function xt(e, a, t, n) {
			if (p) return I(12, 0, 1, e, a, t, n);
			for (var o = 0, l = 0; l < t; l++) {
				var c = (g(), B)[a >> 2], u = (g(), B)[a + 4 >> 2];
				a += 8;
				for (var d = 0; d < u; d++) An(e, (g(), F)[c + d]);
				o += u;
			}
			return (g(), B)[n >> 2] = o, 0;
		}
		var wn = () => (e) => (e.set(crypto.getRandomValues(new Uint8Array(e.byteLength))), 0), Dt = (e) => (Dt = wn())(e), Pn = (e, a) => Dt((g(), F).subarray(e, e + a));
		T.init(), qi(), ur();
		var Cn = [
			Xe,
			ot,
			lt,
			ut,
			ct,
			ht,
			Tt,
			Mt,
			Nt,
			kt,
			Bt,
			Et,
			xt
		], Sa = { 614285: (e) => rt(D.toValue(e)) }, Oe, Y, Ge, Lt, Ft, va, Ot, Gt, It, Ut, $t, Ht, Wt, Vt, Ie, Kt, zt, Yt, jt, qt;
		function Tn(e) {
			r.__ZdlPvm = e.ia, Oe = e.ja, r.__Znwm = e.ka, Y = e.la, Ge = r._malloc = e.ma, r._calloc = e.na, Lt = e.oa, Ft = e.pa, r._emscripten_builtin_free = e.qa, e.ra, va = e.ta, Ot = e.ua, Gt = e.va, r.___libc_free = e.wa, r._emscripten_builtin_malloc = e.xa, r.___libc_malloc = e.ya, It = e.za, Ut = e.Aa, $t = e.Ba, Ht = e.Ca, Wt = e.Da, Vt = e.Ea, r.__ZdaPv = e.Fa, r.__ZdaPvm = e.Ga, r.__ZdlPv = e.Ha, r.__Znam = e.Ia, r.__ZnamSt11align_val_t = e.Ja, r.__ZnwmSt11align_val_t = e.Ka, r.___libc_calloc = e.La, r.___libc_realloc = e.Ma, r._emscripten_builtin_calloc = e.Na, r._emscripten_builtin_realloc = e.Oa, r._malloc_size = e.Pa, r._malloc_usable_size = e.Qa, r._reallocf = e.Ra, Ie = e.Sa, Kt = e.Ta, zt = e.Ua, Yt = e.Va, jt = e.Wa, qt = e.sa;
		}
		var Xt;
		function Rn() {
			Xt = {
				b: Ui,
				Q: st,
				y: lt,
				P: ut,
				Y: ct,
				z: ht,
				L: Hi,
				B: Vi,
				ba: Ki,
				ga: dr,
				fa: vr,
				d: yr,
				da: _r,
				$: br,
				A: wr,
				l: Pr,
				e: Cr,
				aa: Tr,
				t: Dr,
				ca: Lr,
				T: Fr,
				M: Or,
				x: Gr,
				H: Ir,
				v: Ur,
				S: ca,
				_: $r,
				F: Hr,
				h: jr,
				c: sa,
				ea: qr,
				i: Xr,
				g: Jr,
				p: Zr,
				j: Qr,
				s: en,
				f: an,
				k: tn,
				I: Tt,
				n: cn,
				w: hn,
				m: rn,
				u: dn,
				Z: fn,
				N: gn,
				o: Ct,
				O: Sn,
				J: yn,
				U: Mt,
				V: Nt,
				q: Fi,
				r: kt,
				X: Bt,
				R: Et,
				W: xt,
				E: Mn,
				C: kn,
				D: Nn,
				a: W,
				G: Xe,
				K: Pn
			};
		}
		function Mn(e, a, t) {
			var n = Re();
			try {
				return le(e)(a, t);
			} catch (o) {
				if (se(n), !(o instanceof k)) throw o;
				Ie(1, 0);
			}
		}
		function Nn(e, a, t, n, o) {
			var l = Re();
			try {
				return le(e)(a, t, n, o);
			} catch (c) {
				if (se(l), !(c instanceof k)) throw c;
				Ie(1, 0);
			}
		}
		function kn(e, a, t, n) {
			var o = Re();
			try {
				return le(e)(a, t, n);
			} catch (l) {
				if (se(o), !(l instanceof k)) throw l;
				Ie(1, 0);
			}
		}
		function Bn(e) {
			Za = !0, T.tlsInitFunctions.push(e.ra), !p && e.ha();
		}
		var Jt, Zt;
		function Ue() {
			Rn();
			var e = { a: Xt };
			Jt = WebAssembly.instantiateStreaming(fetch(new URL("/fluxa/assets/jassub-worker-VfCk7loQ.wasm", "" + self.location.href)), e).then((a) => {
				var t = (a.instance || a).exports;
				return Zt = a.module || r.wasm, Tn(t), xi(Di), Bn(t), T.loadWasmModuleToAllWorkers().then(f);
			});
		}
		return p || Ue(), await Jt, r;
	}
	globalThis.name == "em-pthread" && La();
	var Fa = class {
		constructor() {
			S(this, "canvas", null), S(this, "ctx", null), S(this, "bufferCanvas", new OffscreenCanvas(1, 1)), S(this, "bufferCtx", this.bufferCanvas.getContext("2d", {
				alpha: !0,
				desynchronized: !0,
				willReadFrequently: !1
			})), S(this, "_scheduledResize", void 0);
		}
		resizeCanvas(i, r) {
			!i || !r || this.canvas?.width === i && this.canvas?.height === r || (this._scheduledResize = {
				width: i,
				height: r
			});
		}
		setCanvas(i) {
			if (this.canvas = i, this.ctx = i.getContext("2d", {
				alpha: !0,
				desynchronized: !0,
				willReadFrequently: !1
			}), !this.ctx) throw new Error("Could not get 2D context");
		}
		setColorMatrix(i, r) {}
		render(i, r) {
			if (!(!this.ctx || !this.canvas)) {
				if (this._scheduledResize) {
					const { width: s, height: h } = this._scheduledResize;
					this._scheduledResize = void 0, this.canvas.width = s, this.canvas.height = h;
				} else this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
				for (const s of i) {
					if (s.w <= 0 || s.h <= 0) continue;
					const h = new ImageData(s.w, s.h), m = new Uint32Array(h.data.buffer), v = s.color << 8 & 16711680 | s.color >> 8 & 65280 | s.color >> 24 & 255, f = (255 - (s.color & 255)) / 255, p = s.stride, C = s.h, P = s.w;
					for (let b = C + 1, N = s.bitmap, k = 0; --b; N += p) for (let G = 0; G < P; ++G, ++k) {
						const g = r[N + G];
						g !== 0 && (m[k] = f * g << 24 | v);
					}
					this.bufferCanvas.width = P, this.bufferCanvas.height = C, this.bufferCtx.putImageData(h, 0, 0), this.ctx.drawImage(this.bufferCanvas, s.dst_x, s.dst_y);
				}
			}
		}
		destroy() {
			this.ctx = null, this.canvas = null, this.bufferCtx = null, this.bufferCanvas = null;
		}
	};
	const Oa = [
		"thin",
		"extralight",
		"light",
		"regular",
		"medium",
		"semibold",
		"bold",
		"extrabold",
		"black",
		"ultrablack"
	], Ga = navigator.userAgent.toLowerCase(), ze = Ga.includes("firefox"), Ia = /applewebkit\/[\d.]+ \(khtml, like gecko\)(?!.*chrome\/)/.test(Ga), Ye = "BT601", Ua = "BT709", $a = "SMPTE240M", Ha = "FCC", Wa = [
		null,
		Ye,
		null,
		Ye,
		Ye,
		Ua,
		Ua,
		$a,
		$a,
		Ha,
		Ha
	], Va = globalThis.fetch;
	async function Ka(i) {
		return await (await Va(i)).text();
	}
	const za = !ze && self.crossOriginIsolated ? Math.min(Math.max(1, navigator.hardwareConcurrency - 2), 8) : 1, Ai = !!WebAssembly.Memory.prototype.toResizableBuffer, wi = !ze && !Ia && (Ai || za > 1), Q = new Float32Array([
		1,
		0,
		0,
		0,
		1,
		0,
		0,
		0,
		1
	]), Ya = {
		BT601: {
			BT709: new Float32Array([
				1.0863,
				.0965,
				-.01411,
				-.0723,
				.8451,
				-.0277,
				-.0141,
				.0584,
				1.0418
			]),
			BT601: Q
		},
		BT709: {
			BT601: new Float32Array([
				.9137,
				.0784,
				.0079,
				-.1049,
				1.1722,
				-.0671,
				.0096,
				.0322,
				.9582
			]),
			BT709: Q
		},
		FCC: {
			BT709: new Float32Array([
				1.0873,
				-.0736,
				-.0137,
				.0974,
				.8494,
				.0531,
				-.0127,
				-.0251,
				1.0378
			]),
			BT601: new Float32Array([
				1.001,
				-8e-4,
				-2e-4,
				9e-4,
				1.005,
				-.006,
				.0013,
				.0027,
				.996
			])
		},
		SMPTE240M: {
			BT709: new Float32Array([
				.9993,
				6e-4,
				1e-4,
				-4e-4,
				.9812,
				.0192,
				-.0034,
				-.0114,
				1.0148
			]),
			BT601: new Float32Array([
				.913,
				.0774,
				.0096,
				-.1051,
				1.1508,
				-.0456,
				.0063,
				.0207,
				.973
			])
		}
	}, Pi = `
precision mediump float;

// Quad position attribute (0,0), (1,0), (0,1), (1,0), (1,1), (0,1)
attribute vec2 a_quadPos;

uniform vec2 u_resolution;

// Instance attributes
attribute vec4 a_destRect;  // x, y, w, h
attribute vec4 a_color;     // r, g, b, a
attribute float a_texLayer;

varying vec2 v_destXY;
varying vec4 v_color;
varying vec2 v_texSize;
varying float v_texLayer;
varying vec2 v_texCoord;
varying vec2 v_normalizedImageSize;

uniform vec2 u_texDimensions;

void main() {
  vec2 pixelPos = a_destRect.xy + a_quadPos * a_destRect.zw;
  vec2 clipPos = (pixelPos / u_resolution) * 2.0 - 1.0;
  clipPos.y = -clipPos.y;

  gl_Position = vec4(clipPos, 0.0, 1.0);
  v_destXY = a_destRect.xy;
  v_color = a_color;
  v_texSize = a_destRect.zw;
  v_texLayer = a_texLayer;
  v_texCoord = a_quadPos;
  v_normalizedImageSize = a_destRect.zw / u_texDimensions;
}
`, Ci = `
precision mediump float;

uniform sampler2D u_tex;
uniform mat3 u_colorMatrix;
uniform vec2 u_resolution;

varying vec2 v_destXY;
varying vec4 v_color;
varying vec2 v_texSize;
varying float v_texLayer;
varying vec2 v_texCoord;
varying vec2 v_normalizedImageSize;

void main() {
  // Sample texture (r channel contains mask)
  float mask = texture2D(u_tex, v_texCoord * v_normalizedImageSize).r;

  // Apply color matrix conversion (identity if no conversion needed)
  vec3 correctedColor = u_colorMatrix * v_color.rgb;

  // libass color alpha: 0 = opaque, 255 = transparent (inverted)
  float colorAlpha = 1.0 - v_color.a;

  // Final alpha = colorAlpha * mask
  float a = colorAlpha * mask;

  // Premultiplied alpha output
  gl_FragColor = vec4(correctedColor * a, a);
}
`, je = 256;
	var Ti = class {
		constructor() {
			S(this, "canvas", null), S(this, "gl", null), S(this, "program", null), S(this, "instancedArraysExt", null), S(this, "u_resolution", null), S(this, "u_tex", null), S(this, "u_colorMatrix", null), S(this, "u_texDimensions", null), S(this, "a_quadPos", -1), S(this, "a_destRect", -1), S(this, "a_color", -1), S(this, "a_texLayer", -1), S(this, "quadPosBuffer", null), S(this, "instanceDestRectBuffer", null), S(this, "instanceColorBuffer", null), S(this, "instanceTexLayerBuffer", null), S(this, "instanceDestRectData", void 0), S(this, "instanceColorData", void 0), S(this, "instanceTexLayerData", void 0), S(this, "textureCache", /* @__PURE__ */ new Map()), S(this, "textureWidth", 0), S(this, "textureHeight", 0), S(this, "colorMatrix", Q), S(this, "_scheduledResize", void 0), this.instanceDestRectData = new Float32Array(je * 4), this.instanceColorData = new Float32Array(je * 4), this.instanceTexLayerData = new Float32Array(je);
		}
		resizeCanvas(i, r) {
			!i || !r || this.canvas?.width === i && this.canvas?.height === r || (this._scheduledResize = {
				width: i,
				height: r
			});
		}
		setCanvas(i) {
			if (this.canvas = i, this.gl = i.getContext("webgl", {
				alpha: !0,
				premultipliedAlpha: !0,
				antialias: !1,
				depth: !1,
				preserveDrawingBuffer: !1,
				stencil: !1,
				desynchronized: !0,
				powerPreference: "high-performance"
			}), !this.gl) throw new Error("Could not get WebGL context");
			if (this.instancedArraysExt = this.gl.getExtension("ANGLE_instanced_arrays"), !this.instancedArraysExt) throw new Error("ANGLE_instanced_arrays extension not supported");
			const r = this.createShader(this.gl.VERTEX_SHADER, Pi), s = this.createShader(this.gl.FRAGMENT_SHADER, Ci);
			if (!r || !s) throw new Error("Failed to create shaders");
			if (this.program = this.gl.createProgram(), this.gl.attachShader(this.program, r), this.gl.attachShader(this.program, s), this.gl.linkProgram(this.program), !this.gl.getProgramParameter(this.program, this.gl.LINK_STATUS)) {
				const m = this.gl.getProgramInfoLog(this.program);
				throw new Error("Failed to link program: " + m);
			}
			this.u_resolution = this.gl.getUniformLocation(this.program, "u_resolution"), this.u_tex = this.gl.getUniformLocation(this.program, "u_tex"), this.u_colorMatrix = this.gl.getUniformLocation(this.program, "u_colorMatrix"), this.u_texDimensions = this.gl.getUniformLocation(this.program, "u_texDimensions"), this.a_quadPos = this.gl.getAttribLocation(this.program, "a_quadPos"), this.a_destRect = this.gl.getAttribLocation(this.program, "a_destRect"), this.a_color = this.gl.getAttribLocation(this.program, "a_color"), this.a_texLayer = this.gl.getAttribLocation(this.program, "a_texLayer"), this.quadPosBuffer = this.gl.createBuffer(), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.quadPosBuffer);
			const h = new Float32Array([
				0,
				0,
				1,
				0,
				0,
				1,
				1,
				0,
				1,
				1,
				0,
				1
			]);
			this.gl.bufferData(this.gl.ARRAY_BUFFER, h, this.gl.STATIC_DRAW), this.instanceDestRectBuffer = this.gl.createBuffer(), this.instanceColorBuffer = this.gl.createBuffer(), this.instanceTexLayerBuffer = this.gl.createBuffer(), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.quadPosBuffer), this.gl.enableVertexAttribArray(this.a_quadPos), this.gl.vertexAttribPointer(this.a_quadPos, 2, this.gl.FLOAT, !1, 0, 0), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceDestRectBuffer), this.gl.enableVertexAttribArray(this.a_destRect), this.gl.vertexAttribPointer(this.a_destRect, 4, this.gl.FLOAT, !1, 0, 0), this.instancedArraysExt.vertexAttribDivisorANGLE(this.a_destRect, 1), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceColorBuffer), this.gl.enableVertexAttribArray(this.a_color), this.gl.vertexAttribPointer(this.a_color, 4, this.gl.FLOAT, !1, 0, 0), this.instancedArraysExt.vertexAttribDivisorANGLE(this.a_color, 1), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceTexLayerBuffer), this.gl.enableVertexAttribArray(this.a_texLayer), this.gl.vertexAttribPointer(this.a_texLayer, 1, this.gl.FLOAT, !1, 0, 0), this.instancedArraysExt.vertexAttribDivisorANGLE(this.a_texLayer, 1), this.gl.enable(this.gl.BLEND), this.gl.blendFunc(this.gl.ONE, this.gl.ONE_MINUS_SRC_ALPHA), this.gl.useProgram(this.program), this.gl.uniform1i(this.u_tex, 0), this.gl.uniformMatrix3fv(this.u_colorMatrix, !1, this.colorMatrix), this.gl.pixelStorei(this.gl.UNPACK_ALIGNMENT, 1), this.gl.clearColor(0, 0, 0, 0), this.gl.activeTexture(this.gl.TEXTURE0);
		}
		createShader(i, r) {
			const s = this.gl.createShader(i);
			if (this.gl.shaderSource(s, r), this.gl.compileShader(s), !this.gl.getShaderParameter(s, this.gl.COMPILE_STATUS)) {
				const h = this.gl.getShaderInfoLog(s);
				return console.log(h), this.gl.deleteShader(s), null;
			}
			return s;
		}
		setColorMatrix(i, r) {
			this.colorMatrix = (i && r && Ya[i]?.[r]) ?? Q, this.gl && this.u_colorMatrix && this.program && (this.gl.useProgram(this.program), this.gl.uniformMatrix3fv(this.u_colorMatrix, !1, this.colorMatrix));
		}
		createTexture(i, r) {
			const s = this.gl.createTexture();
			return this.gl.bindTexture(this.gl.TEXTURE_2D, s), this.gl.texImage2D(this.gl.TEXTURE_2D, 0, this.gl.LUMINANCE, i, r, 0, this.gl.LUMINANCE, this.gl.UNSIGNED_BYTE, null), this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MIN_FILTER, this.gl.NEAREST), this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_MAG_FILTER, this.gl.NEAREST), this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE), this.gl.texParameteri(this.gl.TEXTURE_2D, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE), s;
		}
		render(i, r) {
			if (!this.gl || !this.program || !this.instancedArraysExt) return;
			if (this._scheduledResize) {
				const { width: v, height: f } = this._scheduledResize;
				this._scheduledResize = void 0, this.canvas.width = v, this.canvas.height = f, this.gl.viewport(0, 0, v, f), this.gl.uniform2f(this.u_resolution, v, f);
			} else this.gl.clear(this.gl.COLOR_BUFFER_BIT);
			let s = this.textureWidth, h = this.textureHeight;
			const m = [];
			for (const v of i) v.w <= 0 || v.h <= 0 || (m.push(v), v.w > s && (s = v.w), v.h > h && (h = v.h));
			if (m.length !== 0) {
				if (s > this.textureWidth || h > this.textureHeight) {
					this.textureWidth = s, this.textureHeight = h;
					for (const v of this.textureCache.values()) this.gl.deleteTexture(v);
					this.textureCache.clear();
				}
				for (let v = 0; v < m.length; v++) {
					const f = m[v];
					let p = this.textureCache.get(v);
					p || (p = this.createTexture(this.textureWidth, this.textureHeight), this.textureCache.set(v, p)), this.gl.bindTexture(this.gl.TEXTURE_2D, p);
					const C = new Uint8Array(r.buffer, f.bitmap, f.stride * f.h), P = new Uint8Array(f.w * f.h);
					for (let b = 0; b < f.h; b++) {
						const N = b * f.stride, k = b * f.w;
						P.set(C.subarray(N, N + f.w), k);
					}
					this.gl.texSubImage2D(this.gl.TEXTURE_2D, 0, 0, 0, f.w, f.h, this.gl.LUMINANCE, this.gl.UNSIGNED_BYTE, P), this.instanceDestRectData[0] = f.dst_x, this.instanceDestRectData[1] = f.dst_y, this.instanceDestRectData[2] = f.w, this.instanceDestRectData[3] = f.h, this.instanceColorData[0] = (f.color >>> 24 & 255) / 255, this.instanceColorData[1] = (f.color >>> 16 & 255) / 255, this.instanceColorData[2] = (f.color >>> 8 & 255) / 255, this.instanceColorData[3] = (f.color & 255) / 255, this.instanceTexLayerData[0] = 0, this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceDestRectBuffer), this.gl.bufferData(this.gl.ARRAY_BUFFER, this.instanceDestRectData.subarray(0, 4), this.gl.DYNAMIC_DRAW), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceColorBuffer), this.gl.bufferData(this.gl.ARRAY_BUFFER, this.instanceColorData.subarray(0, 4), this.gl.DYNAMIC_DRAW), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceTexLayerBuffer), this.gl.bufferData(this.gl.ARRAY_BUFFER, this.instanceTexLayerData.subarray(0, 1), this.gl.DYNAMIC_DRAW), this.gl.uniform2f(this.u_texDimensions, this.textureWidth, this.textureHeight), this.instancedArraysExt.drawArraysInstancedANGLE(this.gl.TRIANGLES, 0, 6, 1);
				}
			}
		}
		destroy() {
			if (this.gl) {
				for (const i of this.textureCache.values()) this.gl.deleteTexture(i);
				this.textureCache.clear(), this.quadPosBuffer && (this.gl.deleteBuffer(this.quadPosBuffer), this.quadPosBuffer = null), this.instanceDestRectBuffer && (this.gl.deleteBuffer(this.instanceDestRectBuffer), this.instanceDestRectBuffer = null), this.instanceColorBuffer && (this.gl.deleteBuffer(this.instanceColorBuffer), this.instanceColorBuffer = null), this.instanceTexLayerBuffer && (this.gl.deleteBuffer(this.instanceTexLayerBuffer), this.instanceTexLayerBuffer = null), this.program && (this.gl.deleteProgram(this.program), this.program = null), this.gl = null;
			}
		}
	};
	const Ri = `#version 300 es
precision highp float;

const vec2 QUAD_POSITIONS[6] = vec2[6](
  vec2(0.0, 0.0),
  vec2(1.0, 0.0),
  vec2(0.0, 1.0),
  vec2(1.0, 0.0),
  vec2(1.0, 1.0),
  vec2(0.0, 1.0)
);

uniform vec2 u_resolution;

// Instance attributes
in vec4 a_destRect;  // x, y, w, h
in vec4 a_color;     // r, g, b, a
in float a_texLayer;

flat out vec2 v_destXY;
flat out vec4 v_color;
flat out vec2 v_texSize;
flat out float v_texLayer;

void main() {
  vec2 quadPos = QUAD_POSITIONS[gl_VertexID];
  vec2 pixelPos = a_destRect.xy + quadPos * a_destRect.zw;
  vec2 clipPos = (pixelPos / u_resolution) * 2.0 - 1.0;
  clipPos.y = -clipPos.y;

  gl_Position = vec4(clipPos, 0.0, 1.0);
  v_destXY = a_destRect.xy;
  v_color = a_color;
  v_texSize = a_destRect.zw;
  v_texLayer = a_texLayer;
}
`, Mi = `#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray u_texArray;
uniform mat3 u_colorMatrix;
uniform vec2 u_resolution;

flat in vec2 v_destXY;
flat in vec4 v_color;
flat in vec2 v_texSize;
flat in float v_texLayer;

out vec4 fragColor;

void main() {
  // Flip Y: WebGL's gl_FragCoord.y is 0 at bottom, but destXY.y is from top
  vec2 fragPos = vec2(gl_FragCoord.x, u_resolution.y - gl_FragCoord.y);

  // Calculate local position within the quad (screen coords)
  vec2 localPos = fragPos - v_destXY;

  // Convert to integer texel coordinates for texelFetch
  ivec2 texCoord = ivec2(floor(localPos));

  // Bounds check (prevents out-of-bounds access)
  ivec2 texSizeI = ivec2(v_texSize);
  if (texCoord.x < 0 || texCoord.y < 0 || texCoord.x >= texSizeI.x || texCoord.y >= texSizeI.y) {
    discard;
  }

  // texelFetch: integer coords, no interpolation, no precision issues
  float mask = texelFetch(u_texArray, ivec3(texCoord, int(v_texLayer)), 0).r;

  // Apply color matrix conversion (identity if no conversion needed)
  vec3 correctedColor = u_colorMatrix * v_color.rgb;

  // libass color alpha: 0 = opaque, 255 = transparent (inverted)
  float colorAlpha = 1.0 - v_color.a;

  // Final alpha = colorAlpha * mask
  float a = colorAlpha * mask;

  // Premultiplied alpha output
  fragColor = vec4(correctedColor * a, a);
}
`, ja = 64, qa = 256, Te = 256;
	var Ni = class {
		constructor() {
			S(this, "canvas", null), S(this, "gl", null), S(this, "program", null), S(this, "vao", null), S(this, "u_resolution", null), S(this, "u_texArray", null), S(this, "u_colorMatrix", null), S(this, "instanceDestRectBuffer", null), S(this, "instanceColorBuffer", null), S(this, "instanceTexLayerBuffer", null), S(this, "instanceDestRectData", void 0), S(this, "instanceColorData", void 0), S(this, "instanceTexLayerData", void 0), S(this, "texArray", null), S(this, "texArrayWidth", 0), S(this, "texArrayHeight", 0), S(this, "colorMatrix", Q), S(this, "_scheduledResize", void 0), this.instanceDestRectData = new Float32Array(Te * 4), this.instanceColorData = new Float32Array(Te * 4), this.instanceTexLayerData = new Float32Array(Te);
		}
		resizeCanvas(i, r) {
			!i || !r || this.canvas?.width === i && this.canvas?.height === r || (this._scheduledResize = {
				width: i,
				height: r
			});
		}
		setCanvas(i) {
			if (this.canvas = i, this.gl = i.getContext("webgl2", {
				alpha: !0,
				premultipliedAlpha: !0,
				antialias: !1,
				depth: !1,
				preserveDrawingBuffer: !1,
				stencil: !1,
				desynchronized: !0,
				powerPreference: "high-performance"
			}), !this.gl) throw new Error("Could not get WebGL2 context");
			const r = this.createShader(this.gl.VERTEX_SHADER, Ri), s = this.createShader(this.gl.FRAGMENT_SHADER, Mi);
			if (!r || !s) throw new Error("Failed to create shaders");
			if (this.program = this.gl.createProgram(), this.gl.attachShader(this.program, r), this.gl.attachShader(this.program, s), this.gl.linkProgram(this.program), !this.gl.getProgramParameter(this.program, this.gl.LINK_STATUS)) {
				const f = this.gl.getProgramInfoLog(this.program);
				throw new Error("Failed to link program: " + f);
			}
			this.gl.deleteShader(r), this.gl.deleteShader(s), this.u_resolution = this.gl.getUniformLocation(this.program, "u_resolution"), this.u_texArray = this.gl.getUniformLocation(this.program, "u_texArray"), this.u_colorMatrix = this.gl.getUniformLocation(this.program, "u_colorMatrix"), this.instanceDestRectBuffer = this.gl.createBuffer(), this.instanceColorBuffer = this.gl.createBuffer(), this.instanceTexLayerBuffer = this.gl.createBuffer(), this.vao = this.gl.createVertexArray(), this.gl.bindVertexArray(this.vao);
			const h = this.gl.getAttribLocation(this.program, "a_destRect");
			this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceDestRectBuffer), this.gl.enableVertexAttribArray(h), this.gl.vertexAttribPointer(h, 4, this.gl.FLOAT, !1, 0, 0), this.gl.vertexAttribDivisor(h, 1);
			const m = this.gl.getAttribLocation(this.program, "a_color");
			this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceColorBuffer), this.gl.enableVertexAttribArray(m), this.gl.vertexAttribPointer(m, 4, this.gl.FLOAT, !1, 0, 0), this.gl.vertexAttribDivisor(m, 1);
			const v = this.gl.getAttribLocation(this.program, "a_texLayer");
			this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceTexLayerBuffer), this.gl.enableVertexAttribArray(v), this.gl.vertexAttribPointer(v, 1, this.gl.FLOAT, !1, 0, 0), this.gl.vertexAttribDivisor(v, 1), this.gl.enable(this.gl.BLEND), this.gl.blendFunc(this.gl.ONE, this.gl.ONE_MINUS_SRC_ALPHA), this.gl.useProgram(this.program), this.gl.uniform1i(this.u_texArray, 0), this.gl.uniformMatrix3fv(this.u_colorMatrix, !1, this.colorMatrix), this.gl.pixelStorei(this.gl.UNPACK_ALIGNMENT, 1), this.gl.clearColor(0, 0, 0, 0), this.gl.activeTexture(this.gl.TEXTURE0), this.createTexArray(qa, qa);
		}
		createShader(i, r) {
			const s = this.gl.createShader(i);
			if (this.gl.shaderSource(s, r), this.gl.compileShader(s), !this.gl.getShaderParameter(s, this.gl.COMPILE_STATUS)) {
				const h = this.gl.getShaderInfoLog(s);
				return console.log(h), this.gl.deleteShader(s), null;
			}
			return s;
		}
		setColorMatrix(i, r) {
			this.colorMatrix = (i && r && Ya[i]?.[r]) ?? Q, this.gl && this.u_colorMatrix && this.program && (this.gl.useProgram(this.program), this.gl.uniformMatrix3fv(this.u_colorMatrix, !1, this.colorMatrix));
		}
		createTexArray(i, r) {
			this.texArray && this.gl.deleteTexture(this.texArray), this.texArray = this.gl.createTexture(), this.gl.bindTexture(this.gl.TEXTURE_2D_ARRAY, this.texArray), this.gl.texImage3D(this.gl.TEXTURE_2D_ARRAY, 0, this.gl.R8, i, r, ja, 0, this.gl.RED, this.gl.UNSIGNED_BYTE, null), this.gl.texParameteri(this.gl.TEXTURE_2D_ARRAY, this.gl.TEXTURE_MIN_FILTER, this.gl.NEAREST), this.gl.texParameteri(this.gl.TEXTURE_2D_ARRAY, this.gl.TEXTURE_MAG_FILTER, this.gl.NEAREST), this.gl.texParameteri(this.gl.TEXTURE_2D_ARRAY, this.gl.TEXTURE_WRAP_S, this.gl.CLAMP_TO_EDGE), this.gl.texParameteri(this.gl.TEXTURE_2D_ARRAY, this.gl.TEXTURE_WRAP_T, this.gl.CLAMP_TO_EDGE), this.texArrayWidth = i, this.texArrayHeight = r;
		}
		render(i, r) {
			if (!this.gl || !this.program || !this.vao || !this.texArray) return;
			if ((self.HEAPU8RAW.buffer !== self.WASMMEMORY.buffer || wi) && (r = self.HEAPU8RAW = new Uint8Array(self.WASMMEMORY.buffer)), this._scheduledResize) {
				const { width: f, height: p } = this._scheduledResize;
				this._scheduledResize = void 0, this.canvas.width = f, this.canvas.height = p, this.gl.viewport(0, 0, f, p), this.gl.uniform2f(this.u_resolution, f, p);
			} else this.gl.clear(this.gl.COLOR_BUFFER_BIT);
			let s = this.texArrayWidth, h = this.texArrayHeight;
			const m = [];
			for (const f of i) f.w <= 0 || f.h <= 0 || (m.push(f), f.w > s && (s = f.w), f.h > h && (h = f.h));
			if (m.length === 0) return;
			(s > this.texArrayWidth || h > this.texArrayHeight) && this.createTexArray(s, h);
			const v = Math.min(ja, Te);
			for (let f = 0; f < m.length; f += v) {
				const p = Math.min(f + v, m.length);
				let C = 0;
				for (let P = f; P < p; P++) {
					const b = m[P], N = C;
					if (this.gl.pixelStorei(this.gl.UNPACK_ROW_LENGTH, b.stride), ze || Ia) {
						const G = new Uint8Array(r.buffer, b.bitmap, b.stride * b.h), g = new Uint8Array(G);
						this.gl.texSubImage3D(this.gl.TEXTURE_2D_ARRAY, 0, 0, 0, N, b.w, b.h, 1, this.gl.RED, this.gl.UNSIGNED_BYTE, g);
					} else this.gl.texSubImage3D(this.gl.TEXTURE_2D_ARRAY, 0, 0, 0, N, b.w, b.h, 1, this.gl.RED, this.gl.UNSIGNED_BYTE, r, b.bitmap);
					const k = C * 4;
					this.instanceDestRectData[k] = b.dst_x, this.instanceDestRectData[k + 1] = b.dst_y, this.instanceDestRectData[k + 2] = b.w, this.instanceDestRectData[k + 3] = b.h, this.instanceColorData[k] = (b.color >>> 24 & 255) / 255, this.instanceColorData[k + 1] = (b.color >>> 16 & 255) / 255, this.instanceColorData[k + 2] = (b.color >>> 8 & 255) / 255, this.instanceColorData[k + 3] = (b.color & 255) / 255, this.instanceTexLayerData[C] = N, C++;
				}
				this.gl.pixelStorei(this.gl.UNPACK_ROW_LENGTH, 0), C !== 0 && (this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceDestRectBuffer), this.gl.bufferData(this.gl.ARRAY_BUFFER, this.instanceDestRectData.subarray(0, C * 4), this.gl.DYNAMIC_DRAW), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceColorBuffer), this.gl.bufferData(this.gl.ARRAY_BUFFER, this.instanceColorData.subarray(0, C * 4), this.gl.DYNAMIC_DRAW), this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.instanceTexLayerBuffer), this.gl.bufferData(this.gl.ARRAY_BUFFER, this.instanceTexLayerData.subarray(0, C), this.gl.DYNAMIC_DRAW), this.gl.drawArraysInstanced(this.gl.TRIANGLES, 0, 6, C));
			}
		}
		destroy() {
			this.gl && (this.texArray && (this.gl.deleteTexture(this.texArray), this.texArray = null), this.instanceDestRectBuffer && (this.gl.deleteBuffer(this.instanceDestRectBuffer), this.instanceDestRectBuffer = null), this.instanceColorBuffer && (this.gl.deleteBuffer(this.instanceColorBuffer), this.instanceColorBuffer = null), this.instanceTexLayerBuffer && (this.gl.deleteBuffer(this.instanceTexLayerBuffer), this.instanceTexLayerBuffer = null), this.vao && (this.gl.deleteVertexArray(this.vao), this.vao = null), this.program && (this.gl.deleteProgram(this.program), this.program = null), this.gl = null);
		}
	};
	const Xa = Symbol.for("constructor");
	var ki = class {
		constructor(...i) {
			return S(this, "_wasm", void 0), S(this, "_subtitleColorSpace", void 0), S(this, "_videoColorSpace", void 0), S(this, "_malloc", void 0), S(this, "_gpurender", void 0), S(this, "debug", !1), S(this, "_defaultFont", void 0), S(this, "_loadedInitialFonts", !1), S(this, "_getFont", void 0), S(this, "_availableFonts", {}), S(this, "_checkedFonts", /* @__PURE__ */ new Set()), S(this, "queryFonts", void 0), S(this, "_fontId", 0), this[Xa](...i).catch(console.error);
		}
		async [Xa](i, r, s) {
			this._availableFonts = Object.fromEntries(Object.entries(i.availableFonts).map(([f, p]) => [f.trim().toLowerCase(), p])), this.debug = i.debug, this.queryFonts = i.queryFonts, this._getFont = r, this._defaultFont = i.defaultFont.trim().toLowerCase();
			const h = globalThis.fetch;
			globalThis.fetch = (f) => h(i.wasmUrl);
			try {
				const f = new OffscreenCanvas(1, 1);
				f.getContext("webgl2") ? this._gpurender = new Ni() : this._gpurender = f.getContext("webgl")?.getExtension("ANGLE_instanced_arrays") ? new Ti() : new Fa();
			} catch {
				this._gpurender = new Fa();
			}
			this._gpurender.setCanvas(s), this._loadedInitialFonts = !i.fonts.length;
			const { _malloc: m, JASSUB: v } = await La({
				__url: i.wasmUrl,
				__out: (f) => this._log(f)
			});
			return this._malloc = m, this._wasm = new v(i.width, i.height, this._defaultFont), this._wasm.setThreads(za), this._loadedInitialFonts || await this._loadInitialFonts(i.fonts), this._wasm.createTrackMem(i.subContent ?? await Ka(i.subUrl)), this._subtitleColorSpace = Wa[this._wasm.trackColorSpace], (i.libassMemoryLimit > 0 || i.libassGlyphLimit > 0) && this._wasm.setMemoryLimits(i.libassGlyphLimit || 0, i.libassMemoryLimit || 0), this._checkColorSpace(), this;
		}
		processData(i) {
			this._wasm.processData(i);
		}
		processChunk(i, r, s) {
			this._wasm.processChunk(i, r, s);
		}
		createEvent(i) {
			this._wasm.createEvent(i);
		}
		getEvents() {
			return this._wasm.getEvents();
		}
		setEvent(i, r) {
			this._wasm.setEvent(r, i);
		}
		removeEvent(i) {
			this._wasm.removeEvent(i);
		}
		createStyle(i) {
			this._wasm.createStyle(i);
		}
		getStyles() {
			return this._wasm.getStyles();
		}
		setStyle(i, r) {
			this._wasm.setStyle(r, i);
		}
		removeStyle(i) {
			this._wasm.removeStyle(i);
		}
		styleOverride(i) {
			this._wasm.styleOverride(i);
		}
		disableStyleOverride() {
			this._wasm.disableStyleOverride();
		}
		setTrack(i) {
			this._wasm.createTrackMem(i), this._subtitleColorSpace = Wa[this._wasm.trackColorSpace];
		}
		freeTrack() {
			this._wasm.removeTrack();
		}
		async setTrackByUrl(i) {
			this.setTrack(await Ka(i));
		}
		_checkColorSpace() {
			!this._subtitleColorSpace || !this._videoColorSpace || this._gpurender.setColorMatrix(this._subtitleColorSpace, this._videoColorSpace);
		}
		setDefaultFont(i) {
			this._defaultFont = i.trim().toLowerCase(), this._wasm.setDefaultFont(this._defaultFont);
		}
		async _log(i) {
			console.debug(i);
			const r = i.match(/JASSUB: fontselect:[^(]+: \(([^,]+), (\d{1,4}), \d\)/);
			r && !await this._findAvailableFont(r[1].trim().toLowerCase(), Oa[Math.ceil(parseInt(r[2]) / 100) - 1]) && await this._findAvailableFont(this._defaultFont);
		}
		async addFonts(i) {
			if (!i.length) return !1;
			const r = [], s = [];
			for (const h of i) typeof h == "string" ? r.push(h) : s.push(h);
			return s.length && this._allocFonts(s), !!await Promise.allSettled(r.map((h) => this._asyncWrite(h)));
		}
		async _loadInitialFonts(i) {
			await this.addFonts(i), this._loadedInitialFonts = !0, this._wasm.reloadFonts();
		}
		async _findAvailableFont(i, r) {
			if (!this._loadedInitialFonts) return;
			for (const h of Oa) if (i.includes(h)) {
				i = i.replace(h, "").trim(), r ??= h;
				break;
			}
			r ??= "regular";
			const s = i + " " + r;
			if (!this._checkedFonts.has(s)) {
				this._checkedFonts.add(s);
				try {
					const h = this._availableFonts[s] ?? this._availableFonts[i] ?? await this._queryLocalFont(i, r) ?? await this._queryRemoteFont([s, i]);
					if (h) return await this.addFonts([h]);
				} catch (h) {
					console.warn("Error querying font", i, r, h);
				}
			}
		}
		async _queryLocalFont(i, r) {
			if (this.queryFonts) return await this._getFont(i, r);
		}
		async _queryRemoteFont(i) {
			if (this.queryFonts !== "localandremote") return;
			const r = await bi({ postscriptNames: i });
			if (!r.length) return;
			const s = await r[0].blob();
			return new Uint8Array(await s.arrayBuffer());
		}
		async _asyncWrite(i) {
			const r = await Va(i);
			this._allocFonts([new Uint8Array(await r.arrayBuffer())]);
		}
		_allocFonts(i) {
			for (const r of i) {
				const s = this._malloc(r.byteLength);
				self.HEAPU8RAW.set(r, s), this._wasm.addFont("font-" + this._fontId++, s, r.byteLength);
			}
			this._wasm.reloadFonts();
		}
		_resizeCanvas(i, r, s, h) {
			this._wasm.resizeCanvas(i, r, s, h), this._gpurender.resizeCanvas(i, r);
		}
		async [pa]() {
			this._wasm.quitLibrary(), this._gpurender.destroy(), this._wasm = null, this._gpurender = null, this._availableFonts = {};
		}
		_draw(i, r = !1) {
			const s = this._wasm.rawRender(i, Number(r));
			s && this._gpurender.render(s, self.HEAPU8RAW);
		}
		_setColorSpace(i) {
			i !== "RGB" && (this._videoColorSpace = i, this._checkColorSpace());
		}
	};
	self.name === "jassub-worker" && fi(ki);
})();
