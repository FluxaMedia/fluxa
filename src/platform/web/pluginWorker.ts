import { newAsyncContext } from 'quickjs-emscripten';
import { parseHTML } from 'linkedom';

type PluginRequest = {
  id: number;
  code: string;
  scraperId: string;
  settingsJson: string;
  tmdbId: string;
  mediaType: string;
  season: number | null;
  episode: number | null;
};

const companionUrl = import.meta.env.VITE_FLUXA_COMPANION_URL || 'http://127.0.0.1:19876';
const contexts = new Map<string, Awaited<ReturnType<typeof newAsyncContext>>>();

const cheerioPolyfill = `
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
}`;

const compatPolyfill = `
globalThis.global = globalThis; globalThis.window = globalThis; globalThis.self = globalThis;
if (!globalThis.console) globalThis.console = {};
if (!globalThis.console.log) globalThis.console.log = function() {};
if (!globalThis.console.warn) globalThis.console.warn = function() {};
if (!globalThis.console.error) globalThis.console.error = function() {};
if (!globalThis.URLSearchParams) { globalThis.URLSearchParams = function(value) { this._pairs = []; var self = this; if (typeof value === 'string') value.replace(/^\\?/, '').split('&').forEach(function(part) { if (!part) return; var p = part.split('='); self.append(decodeURIComponent(p[0]), decodeURIComponent(p.slice(1).join('='))); }); }; URLSearchParams.prototype.append = function(k, v) { this._pairs.push([String(k), String(v)]); }; URLSearchParams.prototype.get = function(k) { var p = this._pairs.find(function(pair) { return pair[0] === String(k); }); return p ? p[1] : null; }; URLSearchParams.prototype.toString = function() { return this._pairs.map(function(p) { return encodeURIComponent(p[0]) + '=' + encodeURIComponent(p[1]); }).join('&'); }; }
if (!globalThis.atob) globalThis.atob = function(value) { var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='; var output = ''; var buffer = 0; var bits = 0; String(value).replace(/=+$/, '').split('').forEach(function(char) { var index = chars.indexOf(char); if (index < 0) return; buffer = (buffer << 6) | index; bits += 6; if (bits >= 8) { bits -= 8; output += String.fromCharCode((buffer >> bits) & 255); } }); return output; };
if (!globalThis.btoa) globalThis.btoa = function(value) { var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/='; var output = ''; var index = 0; while (index < String(value).length) { var a = String(value).charCodeAt(index++); var b = String(value).charCodeAt(index++); var c = String(value).charCodeAt(index++); var n = (a << 16) | ((b || 0) << 8) | (c || 0); output += chars[(n >> 18) & 63] + chars[(n >> 12) & 63] + (index - 2 > String(value).length ? '=' : chars[(n >> 6) & 63]) + (index - 1 > String(value).length ? '=' : chars[n & 63]); } return output; };
`;

const domDocuments = new Map<string, ReturnType<typeof parseHTML>['document']>();
const domElements = new Map<string, Element>();
let domCounter = 0;

function domId(): string { return `dom_${domCounter++}`; }

function registerDom(context: Awaited<ReturnType<typeof newAsyncContext>>) {
  const sync = (name: string, fn: (...args: any[]) => string) => {
    const handle = context.newFunction(name, (...args) => context.newString(fn(...args.map((arg) => context.getString(arg)))));
    context.setProp(context.global, name, handle);
    handle.dispose();
  };
  sync('__cheerio_load', (html) => { const id = domId(); domDocuments.set(id, parseHTML(html).document); return id; });
  sync('__cheerio_select', (id, selector) => JSON.stringify([...((domDocuments.get(id)?.querySelectorAll(selector)) || [])].map((element) => { const key = `${id}:${domCounter++}`; domElements.set(key, element); return key; })));
  sync('__cheerio_find', (id, elementId, selector) => JSON.stringify([...(domElements.get(elementId)?.querySelectorAll(selector) || [])].map((element) => { const key = `${id}:${domCounter++}`; domElements.set(key, element); return key; })));
  sync('__cheerio_text', (_id, ids) => ids.split(',').map((id: string) => domElements.get(id)?.textContent || '').join(' '));
  sync('__cheerio_html', (_id, elementId) => elementId ? (domElements.get(elementId)?.outerHTML || '') : '');
  sync('__cheerio_inner_html', (elementId) => domElements.get(elementId)?.innerHTML || '');
  sync('__cheerio_attr', (elementId, name) => domElements.get(elementId)?.getAttribute(name) ?? '__UNDEFINED__');
  sync('__cheerio_next', (id, elementId) => { const element = domElements.get(elementId)?.nextElementSibling; if (!element) return '__NONE__'; const key = `${id}:${domCounter++}`; domElements.set(key, element); return key; });
  sync('__cheerio_prev', (id, elementId) => { const element = domElements.get(elementId)?.previousElementSibling; if (!element) return '__NONE__'; const key = `${id}:${domCounter++}`; domElements.set(key, element); return key; });
}

async function execute(request: PluginRequest): Promise<string> {
  let context = contexts.get(request.scraperId);
  if (!context) {
    context = await newAsyncContext();
    contexts.set(request.scraperId, context);
    registerDom(context);
    const fetchHandle = context.newFunction('__native_fetch', (urlHandle, methodHandle, headersHandle, bodyHandle) => {
      const url = context!.getString(urlHandle);
      const method = context!.getString(methodHandle);
      const headers = JSON.parse(context!.getString(headersHandle));
      const body = context!.getString(bodyHandle);
      const promise = context!.newPromise();
      promise.settled.then(() => context!.runtime.executePendingJobs());
      const request = async () => {
        let response: Response | null = null;
        try {
          response = await fetch(url, { method, headers, body: body || undefined, signal: AbortSignal.timeout(8000) });
        } catch {
          try {
            response = await fetch(`${companionUrl}/proxy?url=${encodeURIComponent(url)}&h=${encodeURIComponent(JSON.stringify(headers))}`, { method, body: body || undefined, signal: AbortSignal.timeout(4000) });
          } catch {
            response = null;
          }
        }
        const payload = response
          ? { ok: response.ok, status: response.status, statusText: response.statusText, url: response.url || url, headers: {}, body: await response.text() }
          : { ok: false, status: 0, statusText: 'NetworkError', url, headers: {}, body: '' };
        promise.resolve(context!.newString(JSON.stringify(payload)));
        context!.runtime.executePendingJobs();
      };
      void request().catch(() => {
        promise.resolve(context!.newString(JSON.stringify({ ok: false, status: 0, statusText: 'NetworkError', url, headers: {}, body: '' })));
        context!.runtime.executePendingJobs();
      });
      return promise.handle;
    });
    context.setProp(context.global, '__native_fetch', fetchHandle);
    fetchHandle.dispose();
    const setup = `${compatPolyfill}${cheerioPolyfill}globalThis.fetch = function(url, options) { options = options || {}; return Promise.resolve(__native_fetch(String(url), String(options.method || 'GET'), JSON.stringify(options.headers || {}), options.body == null ? '' : String(options.body))).then(function(raw) { var data = JSON.parse(raw); return { ok: data.ok, status: data.status, statusText: data.statusText || '', url: data.url, headers: data.headers || {}, text: function() { return Promise.resolve(data.body); }, json: function() { try { return Promise.resolve(JSON.parse(data.body)); } catch (_) { return Promise.resolve(null); } }, clone: function() { return this; } }; }); }; globalThis.require = function(name) { if (String(name).indexOf('cheerio') !== -1) return cheerio; throw new Error('module not available: ' + name); };`;
    const setupResult = await context.evalCodeAsync(setup, 'plugin-setup.js');
    if ('error' in setupResult) { const errorHandle = setupResult.error; if (!errorHandle) throw new Error('plugin setup failed'); const error = context.dump(errorHandle); errorHandle.dispose(); throw new Error(String(error)); }
    setupResult.value.dispose();
  }
  const script = `(async function() { try { globalThis.SCRAPER_ID = ${JSON.stringify(request.scraperId)}; globalThis.SCRAPER_SETTINGS = JSON.parse(${JSON.stringify(request.settingsJson || '{}')}); var module = { exports: {} }; var exports = module.exports; (function() { ${request.code}\n })(); var getStreams = module.exports.getStreams || globalThis.getStreams; if (!getStreams) return '[]'; return JSON.stringify(await getStreams(${JSON.stringify(request.tmdbId)}, ${JSON.stringify(request.mediaType)}, ${request.season == null ? 'null' : request.season}, ${request.episode == null ? 'null' : request.episode}) || []); } catch (_) { return '[]'; } })()`;
  const result = await context.evalCodeAsync(script, `${request.scraperId}.js`);
  if ('error' in result) { const errorHandle = result.error; if (!errorHandle) throw new Error('plugin execution failed'); const error = context.dump(errorHandle); errorHandle.dispose(); throw new Error(String(error)); }
  const promise = context.unwrapResult(result);
  const settled = await context.resolvePromise(promise);
  promise.dispose();
  if ('error' in settled) { const errorHandle = settled.error; if (!errorHandle) throw new Error('plugin promise failed'); const error = context.dump(errorHandle); errorHandle.dispose(); throw new Error(String(error)); }
  const value = context.unwrapResult(settled);
  const output = context.getString(value);
  value.dispose();
  return output;
}

self.onmessage = (event: MessageEvent<PluginRequest>) => {
  void execute(event.data).then((result) => self.postMessage({ id: event.data.id, result })).catch((error) => self.postMessage({ id: event.data.id, error: error instanceof Error ? error.message : String(error) }));
};
