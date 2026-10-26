globalThis.__SYNC_BODY = true;
globalThis.LoadDoc = function(html) { return cheerio.load(html); };
globalThis.$toString = function(value) { return value === undefined || value === null ? '' : String(value); };
globalThis.$replace = function(text, pattern, replacement) { return String(text).split(pattern).join(replacement); };
globalThis.$sleep = function() {};
globalThis.$getUserPreference = function(key) {
    var settings = globalThis.SCRAPER_SETTINGS || {};
    return settings[key] === undefined ? '' : String(settings[key]);
};
globalThis.$habari = {
    parse: function(name) {
        var text = String(name || '');
        var episode = text.match(/(?:\bE|\bEP?\.?\s*|\s-\s)(\d{1,4})\b/i);
        var resolution = text.match(/\b(\d{3,4}p)\b/i);
        return {
            title: text.replace(/\[[^\]]*\]|\([^)]*\)/g, '').replace(/\s-\s\d+.*$/, '').trim(),
            episode_number: episode ? [episode[1]] : [],
            video_resolution: resolution ? resolution[1] : ''
        };
    }
};
