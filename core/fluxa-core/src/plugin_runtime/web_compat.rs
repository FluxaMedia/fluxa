pub(super) const WEB_COMPAT_POLYFILL: &str = r#"
(function () {
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
        URLSearchParams.prototype.toString = function () { return this._pairs.map(function (p) { return encodeURIComponent(p[0]) + '=' + encodeURIComponent(p[1]); }).join('&'); };
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
