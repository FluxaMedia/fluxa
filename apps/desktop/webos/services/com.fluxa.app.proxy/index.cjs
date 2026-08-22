'use strict';

var http = require('http');
var https = require('https');

var PORT = 19876;
var BASE = 'http://127.0.0.1:' + PORT;

function resolveUrl(base, relative) {
  if (/^https?:\/\//i.test(relative)) return relative;
  try { return new URL(relative, base).href; } catch (e) { return relative; }
}

function rewriteHls(content, manifestUrl, headers) {
  var encoded = encodeURIComponent(JSON.stringify(headers));
  var result = content.replace(/^([^#\n][^\n]*)$/gm, function (line) {
    var trimmed = line.trim();
    if (!trimmed) return line;
    var abs = resolveUrl(manifestUrl, trimmed);
    return BASE + '/proxy?url=' + encodeURIComponent(abs) + '&h=' + encoded;
  });
  result = result.replace(/URI="([^"]+)"/g, function (match, uri) {
    var abs = resolveUrl(manifestUrl, uri);
    return 'URI="' + BASE + '/proxy?url=' + encodeURIComponent(abs) + '&h=' + encoded + '"';
  });
  return result;
}

function isHls(contentType, targetUrl) {
  var ct = (contentType || '').toLowerCase();
  return ct.indexOf('mpegurl') !== -1 || ct.indexOf('x-mpegurl') !== -1 ||
    /\.m3u8(\?|$)/i.test(targetUrl);
}

function proxyRequest(targetUrl, headers, req, res) {
  var parsed;
  try { parsed = new URL(targetUrl); } catch (e) {
    res.writeHead(400); res.end('Bad url'); return;
  }

  var driver = parsed.protocol === 'https:' ? https : http;
  var options = {
    hostname: parsed.hostname,
    port: parsed.port || (parsed.protocol === 'https:' ? 443 : 80),
    path: parsed.pathname + parsed.search,
    method: 'GET',
    headers: headers,
  };

  var upstream = driver.request(options, function (upRes) {
    var ct = upRes.headers['content-type'] || '';
    var cors = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Expose-Headers': '*',
    };

    if (isHls(ct, targetUrl)) {
      var body = '';
      upRes.setEncoding('utf8');
      upRes.on('data', function (chunk) { body += chunk; });
      upRes.on('end', function () {
        var out = rewriteHls(body, targetUrl, headers);
        res.writeHead(200, Object.assign({
          'Content-Type': ct || 'application/vnd.apple.mpegurl',
          'Cache-Control': 'no-cache',
        }, cors));
        res.end(out);
      });
    } else {
      var resHeaders = Object.assign({}, upRes.headers, cors);
      delete resHeaders['content-length'];
      res.writeHead(upRes.statusCode, resHeaders);
      upRes.pipe(res);
    }
  });

  upstream.on('error', function (err) {
    if (!res.headersSent) { res.writeHead(502); res.end(err.message); }
  });
  upstream.setTimeout(30000, function () { upstream.destroy(); });
  upstream.end();
}

var server = http.createServer(function (req, res) {
  var parsed = new URL(req.url, 'http://localhost');

  if (req.method === 'OPTIONS') {
    res.writeHead(200, {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, OPTIONS',
      'Access-Control-Allow-Headers': '*',
    });
    res.end();
    return;
  }

  if (parsed.pathname === '/health') {
    res.writeHead(200, { 'Content-Type': 'text/plain', 'Access-Control-Allow-Origin': '*' });
    res.end('ok');
    return;
  }

  if (parsed.pathname === '/proxy') {
    var targetUrl = parsed.searchParams.get('url') || '';
    var headers = {};
    try {
      var h = parsed.searchParams.get('h');
      if (h) headers = JSON.parse(h);
    } catch (e) {}

    if (!targetUrl || !/^https?:\/\//i.test(targetUrl)) {
      res.writeHead(400); res.end('Missing or invalid url param'); return;
    }

    proxyRequest(targetUrl, headers, req, res);
    return;
  }

  res.writeHead(404);
  res.end();
});

server.on('error', function (err) {
  if (err.code !== 'EADDRINUSE') process.exit(1);
});

server.listen(PORT, '127.0.0.1', function () {});

try {
  var Service = require('webos-service');
  var svc = new Service('com.fluxa.app.proxy');
  svc.register('start', function (message) {
    message.respond({ returnValue: true, port: PORT });
  });
} catch (e) {}
