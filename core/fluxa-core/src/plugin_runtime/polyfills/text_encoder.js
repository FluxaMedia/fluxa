if (typeof TextEncoder === 'undefined') {
    globalThis.TextEncoder = function() {};
    TextEncoder.prototype.encode = function(str) {
        var hex = __crypto_utf8_to_hex(str);
        var bytes = new Uint8Array(hex.length / 2);
        for (var i = 0; i < hex.length; i += 2) {
            bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
        }
        return bytes;
    };
}
if (typeof TextDecoder === 'undefined') {
    globalThis.TextDecoder = function() {};
    TextDecoder.prototype.decode = function(data) {
        var bytes = data;
        if (data instanceof ArrayBuffer) bytes = new Uint8Array(data);
        var hex = '';
        for (var i = 0; i < bytes.length; i++) {
            hex += bytes[i].toString(16).padStart(2, '0');
        }
        return __crypto_hex_to_utf8(hex);
    };
}
