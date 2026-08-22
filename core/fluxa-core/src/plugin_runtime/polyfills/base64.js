if (typeof atob === 'undefined') {
    globalThis.atob = function(input) {
        var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
        var str = String(input).replace(/=+$/, '');
        if (str.length % 4 === 1) throw new Error('InvalidCharacterError');
        var output = '';
        var bc = 0, bs, buffer, idx = 0;
        while ((buffer = str.charAt(idx++))) {
            buffer = chars.indexOf(buffer);
            if (buffer === -1) continue;
            bs = bc % 4 ? bs * 64 + buffer : buffer;
            if (bc++ % 4) output += String.fromCharCode(255 & (bs >> ((-2 * bc) & 6)));
        }
        return output;
    };
}

if (typeof btoa === 'undefined') {
    globalThis.btoa = function(input) {
        var chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=';
        var str = String(input);
        var output = '';
        for (var block, charCode, idx = 0, map = chars;
             str.charAt(idx | 0) || (map = '=', idx % 1);
             output += map.charAt(63 & (block >> (8 - (idx % 1) * 8)))) {
            charCode = str.charCodeAt(idx += 3 / 4);
            if (charCode > 0xFF) throw new Error('InvalidCharacterError');
            block = (block << 8) | charCode;
        }
        return output;
    };
}
