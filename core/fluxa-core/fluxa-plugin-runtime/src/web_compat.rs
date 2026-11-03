pub(crate) const WEB_COMPAT_POLYFILL: &str = r#"
(function () {
    if (typeof globalThis.Buffer === 'undefined' && typeof Uint8Array !== 'undefined') {
        var B64 = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
        var utf8Bytes = function (str) {
            var out = [];
            for (var i = 0; i < str.length; i++) {
                var c = str.charCodeAt(i);
                if (c >= 0xd800 && c < 0xdc00 && i + 1 < str.length) {
                    var d = str.charCodeAt(i + 1);
                    if (d >= 0xdc00 && d < 0xe000) { c = 0x10000 + ((c - 0xd800) << 10) + (d - 0xdc00); i++; }
                }
                if (c < 0x80) out.push(c);
                else if (c < 0x800) out.push(0xc0 | (c >> 6), 0x80 | (c & 63));
                else if (c < 0x10000) out.push(0xe0 | (c >> 12), 0x80 | ((c >> 6) & 63), 0x80 | (c & 63));
                else out.push(0xf0 | (c >> 18), 0x80 | ((c >> 12) & 63), 0x80 | ((c >> 6) & 63), 0x80 | (c & 63));
            }
            return out;
        };
        var utf8String = function (bytes) {
            var out = '';
            for (var i = 0; i < bytes.length;) {
                var b = bytes[i++], c;
                if (b < 0x80) c = b;
                else if (b >= 0xf0 && i + 2 < bytes.length + 0) { c = ((b & 7) << 18) | ((bytes[i++] & 63) << 12) | ((bytes[i++] & 63) << 6) | (bytes[i++] & 63); }
                else if (b >= 0xe0) { c = ((b & 15) << 12) | ((bytes[i++] & 63) << 6) | (bytes[i++] & 63); }
                else if (b >= 0xc0) { c = ((b & 31) << 6) | (bytes[i++] & 63); }
                else c = 0xfffd;
                if (c >= 0x10000) { c -= 0x10000; out += String.fromCharCode(0xd800 + (c >> 10), 0xdc00 + (c & 1023)); }
                else out += String.fromCharCode(c);
            }
            return out;
        };
        var b64Decode = function (str) {
            str = String(str).replace(/-/g, '+').replace(/_/g, '/').replace(/[^A-Za-z0-9+\/]/g, '');
            var out = [];
            for (var i = 0; i < str.length; i += 4) {
                var a = B64.indexOf(str.charAt(i)), b = B64.indexOf(str.charAt(i + 1));
                var c = i + 2 < str.length ? B64.indexOf(str.charAt(i + 2)) : -1;
                var d = i + 3 < str.length ? B64.indexOf(str.charAt(i + 3)) : -1;
                out.push((a << 2) | (b >> 4));
                if (c >= 0) out.push(((b & 15) << 4) | (c >> 2));
                if (d >= 0) out.push(((c & 3) << 6) | d);
            }
            return out;
        };
        var b64Encode = function (bytes, url) {
            var out = '';
            for (var i = 0; i < bytes.length; i += 3) {
                var a = bytes[i], b = bytes[i + 1], c = bytes[i + 2];
                out += B64.charAt(a >> 2) + B64.charAt(((a & 3) << 4) | ((b || 0) >> 4));
                out += i + 1 < bytes.length ? B64.charAt(((b & 15) << 2) | ((c || 0) >> 6)) : '=';
                out += i + 2 < bytes.length ? B64.charAt(c & 63) : '=';
            }
            return url ? out.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '') : out;
        };
        function Buffer(arg, offset, length) {
            return Buffer.from(arg, offset, length);
        }
        var wrap = function (bytes) {
            var buf = new Uint8Array(bytes.length);
            for (var i = 0; i < bytes.length; i++) buf[i] = bytes[i];
            Object.setPrototypeOf(buf, Buffer.prototype);
            return buf;
        };
        Buffer.prototype = Object.create(Uint8Array.prototype, { constructor: { value: Buffer, writable: true, configurable: true } });
        Buffer.alloc = function (size, fill) { var buf = wrap(new Array(size).fill(0)); if (fill !== undefined) buf.fill(fill); return buf; };
        Buffer.allocUnsafe = Buffer.alloc;
        Buffer.isBuffer = function (value) { return value instanceof Buffer; };
        Buffer.from = function (value, encoding) {
            if (typeof value === 'string') {
                encoding = String(encoding || 'utf8').toLowerCase();
                if (encoding === 'base64' || encoding === 'base64url') return wrap(b64Decode(value));
                if (encoding === 'hex') {
                    var hex = [];
                    for (var i = 0; i + 1 < value.length; i += 2) hex.push(parseInt(value.substr(i, 2), 16));
                    return wrap(hex);
                }
                if (encoding === 'latin1' || encoding === 'binary' || encoding === 'ascii') {
                    var raw = [];
                    for (var j = 0; j < value.length; j++) raw.push(value.charCodeAt(j) & 255);
                    return wrap(raw);
                }
                return wrap(utf8Bytes(value));
            }
            if (value instanceof ArrayBuffer) return wrap(new Uint8Array(value));
            return wrap(Array.prototype.slice.call(value));
        };
        Buffer.concat = function (list) {
            var all = [];
            list.forEach(function (part) { for (var i = 0; i < part.length; i++) all.push(part[i]); });
            return wrap(all);
        };
        Buffer.byteLength = function (value, encoding) { return typeof value === 'string' ? Buffer.from(value, encoding).length : value.length; };
        Buffer.prototype.toString = function (encoding, start, end) {
            var bytes = Array.prototype.slice.call(this, start || 0, end === undefined ? this.length : end);
            encoding = String(encoding || 'utf8').toLowerCase();
            if (encoding === 'base64') return b64Encode(bytes, false);
            if (encoding === 'base64url') return b64Encode(bytes, true);
            if (encoding === 'hex') return bytes.map(function (b) { return (b < 16 ? '0' : '') + b.toString(16); }).join('');
            if (encoding === 'latin1' || encoding === 'binary' || encoding === 'ascii') return bytes.map(function (b) { return String.fromCharCode(b); }).join('');
            return utf8String(bytes);
        };
        Buffer.prototype.slice = Buffer.prototype.subarray = function (start, end) {
            return wrap(Array.prototype.slice.call(this, start, end));
        };
        Buffer.prototype.equals = function (other) {
            if (this.length !== other.length) return false;
            for (var i = 0; i < this.length; i++) if (this[i] !== other[i]) return false;
            return true;
        };
        globalThis.Buffer = Buffer;
    }

    if (typeof globalThis.setTimeout === 'undefined') {
        var timerSeq = 0;
        var startTimer = function (callback, delay, args, repeat) {
            var id = ++timerSeq;
            var tick = function () {
                __timer_sleep(id, Number(delay) || 0).then(function (alive) {
                    if (!alive) return;
                    if (!repeat) __timer_cancel(id);
                    callback.apply(globalThis, args);
                    if (repeat) tick();
                });
            };
            tick();
            return id;
        };
        globalThis.setTimeout = function (callback, delay) {
            return startTimer(callback, delay, Array.prototype.slice.call(arguments, 2), false);
        };
        globalThis.setInterval = function (callback, delay) {
            return startTimer(callback, delay, Array.prototype.slice.call(arguments, 2), true);
        };
        globalThis.clearTimeout = globalThis.clearInterval = function (id) { if (id) __timer_cancel(id); };
    }
    if (typeof globalThis.self === 'undefined') globalThis.self = globalThis;

    if (typeof globalThis.Headers === 'undefined') {
        function Headers(init) {
            this._values = {};
            var self = this;
            if (!init) return;
            if (typeof init.forEach === 'function') {
                init.forEach(function (value, key) { self.set(key, value); });
            } else if (Array.isArray(init)) {
                init.forEach(function (pair) {
                    if (pair && pair.length >= 2) self.append(pair[0], pair[1]);
                });
            } else {
                Object.keys(init).forEach(function (key) { self.set(key, init[key]); });
            }
        }
        Headers.prototype.set = function (key, value) {
            this._values[String(key).toLowerCase()] = String(value);
        };
        Headers.prototype.append = function (key, value) {
            key = String(key).toLowerCase();
            value = String(value);
            this._values[key] = this._values[key] ? this._values[key] + ', ' + value : value;
        };
        Headers.prototype.get = function (key) {
            key = String(key).toLowerCase();
            return Object.prototype.hasOwnProperty.call(this._values, key) ? this._values[key] : null;
        };
        Headers.prototype.has = function (key) {
            return Object.prototype.hasOwnProperty.call(this._values, String(key).toLowerCase());
        };
        Headers.prototype.forEach = function (callback) {
            var self = this;
            Object.keys(this._values).forEach(function (key) { callback(self._values[key], key, self); });
        };
        globalThis.Headers = Headers;
    }

    globalThis.__normalize_fetch_headers = function (headers) {
        var out = {};
        if (!headers) return out;
        if (typeof headers.forEach === 'function') {
            headers.forEach(function (value, key) { out[key] = String(value); });
            return out;
        }
        if (Array.isArray(headers)) {
            headers.forEach(function (pair) {
                if (pair && pair.length >= 2) out[String(pair[0])] = String(pair[1]);
            });
            return out;
        }
        Object.keys(headers).forEach(function (key) { out[key] = String(headers[key]); });
        return out;
    };

    if (typeof globalThis.URLSearchParams === 'undefined') {
        function URLSearchParams(init) {
            this._pairs = [];
            var self = this;
            if (typeof init === 'string') {
                String(init).replace(/^\?/, '').split('&').forEach(function (part) {
                    if (!part) return;
                    var split = part.indexOf('=');
                    var key = split >= 0 ? part.slice(0, split) : part;
                    var value = split >= 0 ? part.slice(split + 1) : '';
                    self.append(decodeURIComponent(key.replace(/\+/g, ' ')), decodeURIComponent(value.replace(/\+/g, ' ')));
                });
            } else if (Array.isArray(init)) {
                init.forEach(function (pair) { if (pair && pair.length >= 2) self.append(pair[0], pair[1]); });
            } else if (init && typeof init === 'object') {
                Object.keys(init).forEach(function (key) { self.append(key, init[key]); });
            }
        }
        URLSearchParams.prototype.append = function (key, value) { this._pairs.push([String(key), String(value)]); };
        URLSearchParams.prototype.set = function (key, value) { key = String(key); this.delete(key); this.append(key, value); };
        URLSearchParams.prototype.get = function (key) {
            key = String(key);
            for (var i = 0; i < this._pairs.length; i++) if (this._pairs[i][0] === key) return this._pairs[i][1];
            return null;
        };
        URLSearchParams.prototype.getAll = function (key) { key = String(key); return this._pairs.filter(function (p) { return p[0] === key; }).map(function (p) { return p[1]; }); };
        URLSearchParams.prototype.has = function (key) { return this.get(key) !== null; };
        URLSearchParams.prototype.delete = function (key) { key = String(key); this._pairs = this._pairs.filter(function (p) { return p[0] !== key; }); };
        URLSearchParams.prototype.forEach = function (callback) { var self = this; this._pairs.forEach(function (p) { callback(p[1], p[0], self); }); };
        var formEncode = function (value) {
            return encodeURIComponent(value).replace(/%20/g, '+').replace(/[!'()~]/g, function (c) { return '%' + c.charCodeAt(0).toString(16).toUpperCase(); });
        };
        URLSearchParams.prototype.toString = function () { return this._pairs.map(function (p) { return formEncode(p[0]) + '=' + formEncode(p[1]); }).join('&'); };
        URLSearchParams.prototype.keys = function () { return this._pairs.map(function (p) { return p[0]; })[Symbol.iterator](); };
        URLSearchParams.prototype.values = function () { return this._pairs.map(function (p) { return p[1]; })[Symbol.iterator](); };
        URLSearchParams.prototype.entries = function () { return this._pairs.map(function (p) { return [p[0], p[1]]; })[Symbol.iterator](); };
        URLSearchParams.prototype[Symbol.iterator] = URLSearchParams.prototype.entries;
        URLSearchParams.prototype.sort = function () {
            this._pairs = this._pairs.map(function (p, i) { return [p, i]; }).sort(function (a, b) {
                return a[0][0] < b[0][0] ? -1 : a[0][0] > b[0][0] ? 1 : a[1] - b[1];
            }).map(function (x) { return x[0]; });
        };
        Object.defineProperty(URLSearchParams.prototype, 'size', { get: function () { return this._pairs.length; } });
        globalThis.URLSearchParams = URLSearchParams;
    }

    if (typeof globalThis.URL === 'undefined') {
        function URL(url, base) {
            var value = String(url || '');
            var baseValue = base && base.href ? String(base.href) : String(base || '');
            if (baseValue && !/^[a-z][a-z0-9+.-]*:\/\//i.test(value)) {
                var originMatch = baseValue.match(/^([a-z][a-z0-9+.-]*:\/\/[^\/?#]+)/i);
                if (value.charAt(0) === '/') value = (originMatch ? originMatch[1] : '') + value;
                else value = baseValue.replace(/[?#].*$/, '').replace(/\/[^\/]*$/, '/') + value;
            }
            var match = value.match(/^([a-z][a-z0-9+.-]*:)?\/\/([^\/?#]*)([^?#]*)(\?[^#]*)?(#.*)?$/i);
            this.href = value;
            this.protocol = match && match[1] ? match[1] : '';
            this.host = match ? match[2] : '';
            this.hostname = this.host.replace(/:\d+$/, '');
            var portMatch = this.host.match(/:(\d+)$/);
            this.port = portMatch ? portMatch[1] : '';
            this.pathname = match && match[3] ? match[3] : '/';
            this.search = match && match[4] ? match[4] : '';
            this.hash = match && match[5] ? match[5] : '';
            this.origin = this.protocol && this.host ? this.protocol + '//' + this.host : '';
            this.searchParams = new URLSearchParams(this.search);
        }
        URL.prototype.toString = function () { return this.href; };
        globalThis.URL = URL;
    }

    if (typeof globalThis.AbortController === 'undefined') {
        function AbortSignal() { this.aborted = false; this.reason = undefined; this._listeners = []; }
        AbortSignal.prototype.addEventListener = function (type, listener) { if (type === 'abort' && typeof listener === 'function') this._listeners.push(listener); };
        AbortSignal.prototype.removeEventListener = function (type, listener) { if (type === 'abort') this._listeners = this._listeners.filter(function (item) { return item !== listener; }); };
        function AbortController() { this.signal = new AbortSignal(); }
        AbortController.prototype.abort = function (reason) {
            if (this.signal.aborted) return;
            this.signal.aborted = true;
            this.signal.reason = reason;
            this.signal._listeners.slice().forEach(function (listener) { try { listener({ type: 'abort' }); } catch (_) {} });
        };
        globalThis.AbortSignal = AbortSignal;
        globalThis.AbortController = AbortController;
    }

    if (!Array.prototype.flat) Array.prototype.flat = function (depth) {
        depth = depth === undefined ? 1 : Math.max(0, Math.floor(depth));
        var flatten = function (items, remaining) {
            return items.reduce(function (out, item) {
                return out.concat(Array.isArray(item) && remaining > 0 ? flatten(item, remaining - 1) : item);
            }, []);
        };
        return flatten(this, depth);
    };
    if (!Array.prototype.flatMap) Array.prototype.flatMap = function (callback, thisArg) { return this.map(callback, thisArg).flat(1); };
    if (!Object.fromEntries) Object.fromEntries = function (entries) {
        var out = {};
        Array.from(entries || []).forEach(function (entry) { if (entry && entry.length >= 2) out[entry[0]] = entry[1]; });
        return out;
    };
    if (!String.prototype.replaceAll) String.prototype.replaceAll = function (search, replacement) { return this.split(search).join(replacement); };
})();
"#;
