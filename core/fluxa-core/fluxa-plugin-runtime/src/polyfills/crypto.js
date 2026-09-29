var WordArray = {
    init: function(words, sigBytes) {
        this.words = words || [];
        this.sigBytes = sigBytes != undefined ? sigBytes : this.words.length * 4;
    },
    toString: function(encoder) {
        return (encoder || CryptoJS.enc.Hex).stringify(this);
    },
    concat: function(wordArray) {
        var thisWords = this.words;
        var thatWords = wordArray.words;
        var thisSigBytes = this.sigBytes;
        var thatSigBytes = wordArray.sigBytes;

        this.clamp();

        for (var i = 0; i < thatSigBytes; i++) {
            var thatByte = (thatWords[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;
            thisWords[(thisSigBytes + i) >>> 2] |= thatByte << (24 - ((thisSigBytes + i) % 4) * 8);
        }
        this.sigBytes += thatSigBytes;
        return this;
    },
    clamp: function() {
        var words = this.words;
        var sigBytes = this.sigBytes;
        if (sigBytes % 4) {
            words[sigBytes >>> 2] &= 0xffffffff << (32 - (sigBytes % 4) * 8);
        }
        words.length = Math.ceil(sigBytes / 4);
        return this;
    },
    clone: function() {
        return __wordArrayCreate(this.words.slice(0), this.sigBytes);
    }
};

function __wordArrayCreate(words, sigBytes) {
    var wa = Object.create(WordArray);
    wa.init(words, sigBytes);
    return wa;
}

function __isWordArray(value) {
    return value && typeof value === 'object' && Array.isArray(value.words) && typeof value.sigBytes === 'number';
}

function __copyUint8Array(bytes) {
    bytes = __toUint8Array(bytes);
    var copy = new Uint8Array(bytes.length);
    copy.set(bytes);
    return copy;
}

function __toUint8Array(data) {
    if (!data) return new Uint8Array(0);
    if (data instanceof Uint8Array) return data;
    if (data instanceof ArrayBuffer) return new Uint8Array(data);
    if (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView && ArrayBuffer.isView(data)) {
        return new Uint8Array(data.buffer, data.byteOffset || 0, data.byteLength);
    }
    if (Array.isArray(data)) return new Uint8Array(data);
    if (typeof data.length === 'number') return new Uint8Array(Array.prototype.slice.call(data));
    return new Uint8Array(0);
}

function __bytesToArrayBuffer(bytes) {
    return __copyUint8Array(bytes).buffer;
}

function __wordArrayToBytes(wordArray) {
    if (!__isWordArray(wordArray)) return typeof wordArray === 'string' ? new TextEncoder().encode(wordArray) : __toUint8Array(wordArray);
    var bytes = new Uint8Array(wordArray.sigBytes);
    for (var i = 0; i < wordArray.sigBytes; i++) {
        bytes[i] = (wordArray.words[i >>> 2] >>> (24 - (i % 4) * 8)) & 0xff;
    }
    return bytes;
}

function __bytesToWordArray(bytes) {
    bytes = __toUint8Array(bytes);
    var words = [];
    for (var i = 0; i < bytes.length; i++) {
        words[i >>> 2] |= (bytes[i] & 0xff) << (24 - (i % 4) * 8);
    }
    return __wordArrayCreate(words, bytes.length);
}

function __normalizeWordArrayInput(value) {
    if (__isWordArray(value)) return __wordArrayToBytes(value);
    if (typeof value === 'string') return new TextEncoder().encode(value);
    return __toUint8Array(value);
}

function __bytesToHex(bytes) {
    bytes = __toUint8Array(bytes);
    var out = [];
    for (var i = 0; i < bytes.length; i++) {
        var hex = bytes[i].toString(16);
        out.push(hex.length < 2 ? '0' + hex : hex);
    }
    return out.join('');
}

function __hexToBytes(hex) {
    hex = String(hex || '').replace(/[^0-9a-fA-F]/g, '');
    if (hex.length % 2) hex = '0' + hex;
    var bytes = new Uint8Array(hex.length / 2);
    for (var i = 0; i < hex.length; i += 2) {
        bytes[i / 2] = parseInt(hex.substr(i, 2), 16) & 0xff;
    }
    return bytes;
}

function __concatBytes() {
    var total = 0;
    var parts = [];
    for (var i = 0; i < arguments.length; i++) {
        var part = __toUint8Array(arguments[i]);
        parts.push(part);
        total += part.length;
    }
    var out = new Uint8Array(total);
    var offset = 0;
    for (var j = 0; j < parts.length; j++) {
        out.set(parts[j], offset);
        offset += parts[j].length;
    }
    return out;
}

function __normalizeHashName(hash) {
    var name = hash && hash.name ? hash.name : hash;
    name = String(name || 'SHA-256').toUpperCase().replace(/[^A-Z0-9]/g, '');
    if (name === 'SHA1' || name === 'SHA256' || name === 'SHA384' || name === 'SHA512' || name === 'MD5') return name;
    throw new Error('Unsupported hash algorithm: ' + name);
}

function __normalizeAlgorithmName(algo) {
    var name = algo && algo.name ? algo.name : algo;
    name = String(name || '').toUpperCase();
    if (name.indexOf('AES-GCM') >= 0) return 'AES-GCM';
    if (name.indexOf('AES-CBC') >= 0) return 'AES-CBC';
    if (name.indexOf('AES-ECB') >= 0 || name === 'ECB') return 'AES-ECB';
    if (name.indexOf('PBKDF2') >= 0) return 'PBKDF2';
    if (name.indexOf('HMAC') >= 0) return 'HMAC';
    if (name.indexOf('RSASSA-PKCS1') >= 0) return 'RSASSA-PKCS1-V1_5';
    if (name.indexOf('ECDSA') >= 0) return 'ECDSA';
    return name;
}

function __aesModeName(mode, padding) {
    var normalized = __normalizeAlgorithmName(mode || 'AES-CBC');
    if (padding === CryptoJS.pad.NoPadding || padding === 'NoPadding') normalized += '-NoPadding';
    return normalized;
}

function __nativeDigestBytes(hash, dataBytes) {
    if (typeof __crypto_digest_hex_raw === 'undefined') throw new Error('Native digest bridge is unavailable');
    return __hexToBytes(__crypto_digest_hex_raw(__normalizeHashName(hash), __bytesToHex(dataBytes)));
}

function __nativeHmacBytes(hash, keyBytes, dataBytes) {
    if (typeof __crypto_hmac_hex_raw === 'undefined') throw new Error('Native HMAC bridge is unavailable');
    return __hexToBytes(__crypto_hmac_hex_raw(__normalizeHashName(hash), __bytesToHex(keyBytes), __bytesToHex(dataBytes)));
}

function __nativePbkdf2Bytes(passwordBytes, saltBytes, iterations, keySizeBits, hash) {
    if (typeof __crypto_pbkdf2_hex === 'undefined') throw new Error('Native PBKDF2 bridge is unavailable');
    return __hexToBytes(__crypto_pbkdf2_hex(__bytesToHex(passwordBytes), __bytesToHex(saltBytes), iterations, keySizeBits, __normalizeHashName(hash)));
}

function __nativeAesBytes(encrypt, mode, keyBytes, ivBytes, dataBytes) {
    var fn = encrypt ? __crypto_aes_encrypt_hex : __crypto_aes_decrypt_hex;
    if (typeof fn === 'undefined') throw new Error('Native AES bridge is unavailable');
    return __hexToBytes(fn(mode, __bytesToHex(keyBytes), __bytesToHex(ivBytes), __bytesToHex(dataBytes)));
}

function __evpKdf(passwordBytes, saltBytes, keySizeBytes, ivSizeBytes) {
    var targetSize = keySizeBytes + ivSizeBytes;
    var derived = new Uint8Array(targetSize);
    var block = new Uint8Array(0);
    var offset = 0;
    while (offset < targetSize) {
        block = __nativeDigestBytes('MD5', __concatBytes(block, passwordBytes, saltBytes || new Uint8Array(0)));
        var take = Math.min(block.length, targetSize - offset);
        derived.set(block.subarray(0, take), offset);
        offset += take;
    }
    return {
        key: derived.subarray(0, keySizeBytes),
        iv: derived.subarray(keySizeBytes, keySizeBytes + ivSizeBytes)
    };
}

function __opensslSaltHeader() {
    return new Uint8Array([83, 97, 108, 116, 101, 100, 95, 95]);
}

function __hasOpenSslSaltHeader(bytes) {
    var header = __opensslSaltHeader();
    if (!bytes || bytes.length < 16) return false;
    for (var i = 0; i < header.length; i++) {
        if (bytes[i] !== header[i]) return false;
    }
    return true;
}

function __makeCipherParams(ciphertext, key, iv, salt, mode) {
    return {
        ciphertext: __bytesToWordArray(ciphertext),
        key: key ? __bytesToWordArray(key) : undefined,
        iv: iv ? __bytesToWordArray(iv) : undefined,
        salt: salt ? __bytesToWordArray(salt) : undefined,
        mode: mode,
        toString: function(formatter) {
            return (formatter || CryptoJS.format.OpenSSL).stringify(this);
        }
    };
}

var CryptoJS = {
    enc: {
        Hex: {
            stringify: function(wordArray) {
                return __bytesToHex(__wordArrayToBytes(wordArray));
            },
            parse: function(hexStr) {
                return __bytesToWordArray(__hexToBytes(hexStr));
            }
        },
        Utf8: {
            stringify: function(wordArray) {
                return new TextDecoder('utf-8').decode(__wordArrayToBytes(wordArray));
            },
            parse: function(utf8Str) {
                return __bytesToWordArray(new TextEncoder().encode(String(utf8Str)));
            }
        },
        Latin1: {
            stringify: function(wordArray) {
                var bytes = __wordArrayToBytes(wordArray);
                var out = '';
                for (var i = 0; i < bytes.length; i++) out += String.fromCharCode(bytes[i]);
                return out;
            },
            parse: function(str) {
                str = String(str || '');
                var bytes = new Uint8Array(str.length);
                for (var i = 0; i < str.length; i++) bytes[i] = str.charCodeAt(i) & 0xff;
                return __bytesToWordArray(bytes);
            }
        },
        Base64: {
            stringify: function(wordArray) {
                var bytes = __wordArrayToBytes(wordArray);
                var binaryStr = '';
                for (var j = 0; j < bytes.length; j++) binaryStr += String.fromCharCode(bytes[j]);
                return btoa(binaryStr);
            },
            parse: function(base64Str) {
                var binaryStr = atob(String(base64Str || ''));
                var bytes = new Uint8Array(binaryStr.length);
                for (var i = 0; i < binaryStr.length; i++) bytes[i] = binaryStr.charCodeAt(i) & 0xff;
                return __bytesToWordArray(bytes);
            }
        },
        Base64url: {
            stringify: function(wordArray) {
                return CryptoJS.enc.Base64.stringify(wordArray).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
            },
            parse: function(str) {
                str = String(str || '').replace(/-/g, '+').replace(/_/g, '/');
                while (str.length % 4) str += '=';
                return CryptoJS.enc.Base64.parse(str);
            }
        }
    },
    lib: {
        WordArray: {
            create: function(words, sigBytes) {
                if (words == null) return __wordArrayCreate([], sigBytes || 0);
                if (__isWordArray(words)) return words.clone();
                if (typeof words === 'string') return CryptoJS.enc.Utf8.parse(words);
                if (words instanceof ArrayBuffer || (typeof ArrayBuffer !== 'undefined' && ArrayBuffer.isView && ArrayBuffer.isView(words))) {
                    var bytes = __toUint8Array(words);
                    return __bytesToWordArray(sigBytes != undefined ? bytes.subarray(0, sigBytes) : bytes);
                }
                return __wordArrayCreate(words, sigBytes);
            },
            random: function(nBytes) {
                var bytes = new Uint8Array(nBytes || 0);
                globalThis.crypto.getRandomValues(bytes);
                return __bytesToWordArray(bytes);
            }
        },
        CipherParams: {
            create: function(params) {
                params = params || {};
                params.toString = params.toString || function(formatter) {
                    return (formatter || CryptoJS.format.OpenSSL).stringify(this);
                };
                return params;
            }
        }
    },
    format: {
        OpenSSL: {
            stringify: function(cipherParams) {
                var cipherBytes = __wordArrayToBytes(cipherParams.ciphertext);
                var out = cipherParams.salt
                    ? __concatBytes(__opensslSaltHeader(), __wordArrayToBytes(cipherParams.salt), cipherBytes)
                    : cipherBytes;
                return CryptoJS.enc.Base64.stringify(__bytesToWordArray(out));
            },
            parse: function(str) {
                var bytes = __wordArrayToBytes(CryptoJS.enc.Base64.parse(str));
                if (__hasOpenSslSaltHeader(bytes)) {
                    return CryptoJS.lib.CipherParams.create({
                        salt: __bytesToWordArray(bytes.subarray(8, 16)),
                        ciphertext: __bytesToWordArray(bytes.subarray(16))
                    });
                }
                return CryptoJS.lib.CipherParams.create({ ciphertext: __bytesToWordArray(bytes) });
            }
        }
    },
    mode: { CBC: 'AES-CBC', GCM: 'AES-GCM', ECB: 'AES-ECB' },
    pad: { Pkcs7: 'Pkcs7', NoPadding: 'NoPadding' },
    algo: { MD5: 'MD5', SHA1: 'SHA1', SHA256: 'SHA256', SHA384: 'SHA384', SHA512: 'SHA512', AES: 'AES' },
    MD5: function(m) { return __bytesToWordArray(__nativeDigestBytes('MD5', __normalizeWordArrayInput(m))); },
    SHA1: function(m) { return __bytesToWordArray(__nativeDigestBytes('SHA1', __normalizeWordArrayInput(m))); },
    SHA256: function(m) { return __bytesToWordArray(__nativeDigestBytes('SHA256', __normalizeWordArrayInput(m))); },
    SHA384: function(m) { return __bytesToWordArray(__nativeDigestBytes('SHA384', __normalizeWordArrayInput(m))); },
    SHA512: function(m) { return __bytesToWordArray(__nativeDigestBytes('SHA512', __normalizeWordArrayInput(m))); },
    HmacMD5: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('MD5', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    HmacSHA1: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('SHA1', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    HmacSHA256: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('SHA256', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    HmacSHA384: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('SHA384', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    HmacSHA512: function(m, k) { return __bytesToWordArray(__nativeHmacBytes('SHA512', __normalizeWordArrayInput(k), __normalizeWordArrayInput(m))); },
    PBKDF2: function(pass, salt, options) {
        options = options || {};
        var pBytes = __normalizeWordArrayInput(pass);
        var sBytes = __normalizeWordArrayInput(salt);
        var iter = options.iterations || 1000;
        var kSize = options.keySize || 8;
        var algo = options.hasher || 'SHA1';
        return __bytesToWordArray(__nativePbkdf2Bytes(pBytes, sBytes, iter, kSize * 32, algo));
    },
    AES: {
        encrypt: function(message, key, options) {
            options = options || {};
            var data = __normalizeWordArrayInput(message);
            var kBytes;
            var ivBytes;
            var saltBytes;
            var isPassphrase = typeof key === 'string';
            if (isPassphrase) {
                saltBytes = options.salt ? __wordArrayToBytes(options.salt) : __wordArrayToBytes(CryptoJS.lib.WordArray.random(8));
                var derived = __evpKdf(new TextEncoder().encode(key), saltBytes, 32, 16);
                kBytes = derived.key;
                ivBytes = options.iv ? __wordArrayToBytes(options.iv) : derived.iv;
            } else {
                kBytes = __wordArrayToBytes(key);
                ivBytes = options.iv ? __wordArrayToBytes(options.iv) : new Uint8Array(0);
            }
            var mode = __aesModeName(options.mode || 'AES-CBC', options.padding);
            var resBytes = __nativeAesBytes(true, mode, kBytes, ivBytes, data);
            return __makeCipherParams(resBytes, kBytes, ivBytes, saltBytes, mode);
        },
        decrypt: function(cipher, key, options) {
            options = options || {};
            var cipherParams = typeof cipher === 'string' ? CryptoJS.format.OpenSSL.parse(cipher) : cipher;
            var data = cipherParams.ciphertext ? __wordArrayToBytes(cipherParams.ciphertext) : __toUint8Array(cipherParams);
            var kBytes;
            var ivBytes;
            var isPassphrase = typeof key === 'string';
            if (isPassphrase) {
                var saltBytes = options.salt ? __wordArrayToBytes(options.salt) : (cipherParams.salt ? __wordArrayToBytes(cipherParams.salt) : new Uint8Array(0));
                var derived = __evpKdf(new TextEncoder().encode(key), saltBytes, 32, 16);
                kBytes = derived.key;
                ivBytes = options.iv ? __wordArrayToBytes(options.iv) : derived.iv;
            } else {
                kBytes = __wordArrayToBytes(key);
                ivBytes = options.iv ? __wordArrayToBytes(options.iv) : new Uint8Array(0);
            }
            var mode = __aesModeName(options.mode || 'AES-CBC', options.padding);
            return __bytesToWordArray(__nativeAesBytes(false, mode, kBytes, ivBytes, data));
        }
    }
};
globalThis.CryptoJS = CryptoJS;

function __makeCryptoKey(type, algorithm, extractable, usages, rawBytes) {
    return {
        type: type,
        extractable: !!extractable,
        algorithm: algorithm,
        usages: usages || [],
        _raw: __copyUint8Array(rawBytes)
    };
}

function __webCryptoAlgorithm(algo) {
    var name = __normalizeAlgorithmName(algo);
    var out = { name: name };
    if (algo && typeof algo === 'object' && algo.length) out.length = algo.length;
    if (algo && typeof algo === 'object' && algo.hash) out.hash = { name: __normalizeHashName(algo.hash) };
    return out;
}

function __signatureAlgorithmName(algo, key) {
    var name = __normalizeAlgorithmName(algo || (key && key.algorithm));
    var hash = algo && algo.hash ? __normalizeHashName(algo.hash) : (key && key.algorithm && key.algorithm.hash ? key.algorithm.hash.name : 'SHA256');
    if (name === 'RSASSA-PKCS1-V1_5') return 'RSASSA-PKCS1-V1_5-' + hash;
    if (name === 'ECDSA') return 'ECDSA-' + hash;
    return name;
}

globalThis.crypto = {
    subtle: {
        digest: async function(algo, data) {
            return __bytesToArrayBuffer(__nativeDigestBytes(algo, __toUint8Array(data)));
        },
        importKey: async function(fmt, data, algo, extractable, usages) {
            fmt = String(fmt || 'raw').toLowerCase();
            if (fmt !== 'raw' && fmt !== 'pkcs8' && fmt !== 'spki') throw new Error('Unsupported key format: ' + fmt);
            var algorithm = __webCryptoAlgorithm(algo || {});
            var type = fmt === 'spki' ? 'public' : (fmt === 'pkcs8' ? 'private' : 'secret');
            return __makeCryptoKey(type, algorithm, extractable, usages || [], __toUint8Array(data));
        },
        exportKey: async function(fmt, key) {
            fmt = String(fmt || 'raw').toLowerCase();
            if (fmt !== 'raw' && fmt !== 'pkcs8' && fmt !== 'spki') throw new Error('Unsupported key format: ' + fmt);
            return __bytesToArrayBuffer(key._raw);
        },
        generateKey: async function(algo, extractable, usages) {
            var algorithm = __webCryptoAlgorithm(algo || {});
            if (algorithm.name !== 'AES-CBC' && algorithm.name !== 'AES-GCM' && algorithm.name !== 'HMAC') {
                throw new Error('Unsupported generateKey algorithm: ' + algorithm.name);
            }
            var length = algorithm.length || 256;
            var bytes = new Uint8Array(length / 8);
            globalThis.crypto.getRandomValues(bytes);
            return __makeCryptoKey('secret', algorithm, extractable, usages || [], bytes);
        },
        deriveBits: async function(params, key, len) {
            if (__normalizeAlgorithmName(params) !== 'PBKDF2') throw new Error('Only PBKDF2 deriveBits is supported');
            var pBytes = __toUint8Array(key._raw);
            var sBytes = __toUint8Array(params.salt);
            var hash = params.hash || 'SHA-256';
            return __bytesToArrayBuffer(__nativePbkdf2Bytes(pBytes, sBytes, params.iterations || 1000, len, hash));
        },
        deriveKey: async function(params, key, derivedKeyAlgo, extractable, usages) {
            var algorithm = __webCryptoAlgorithm(derivedKeyAlgo || {});
            var length = algorithm.length || 256;
            var raw = await globalThis.crypto.subtle.deriveBits(params, key, length);
            return __makeCryptoKey('secret', algorithm, extractable, usages || [], new Uint8Array(raw));
        },
        encrypt: async function(params, key, data) {
            var mode = __normalizeAlgorithmName(params);
            if (mode !== 'AES-CBC' && mode !== 'AES-GCM') throw new Error('Unsupported encrypt algorithm: ' + mode);
            var ivBytes = __toUint8Array(params.iv || new Uint8Array(0));
            return __bytesToArrayBuffer(__nativeAesBytes(true, mode, __toUint8Array(key._raw), ivBytes, __toUint8Array(data)));
        },
        decrypt: async function(params, key, data) {
            var mode = __normalizeAlgorithmName(params);
            if (mode !== 'AES-CBC' && mode !== 'AES-GCM') throw new Error('Unsupported decrypt algorithm: ' + mode);
            var ivBytes = __toUint8Array(params.iv || new Uint8Array(0));
            return __bytesToArrayBuffer(__nativeAesBytes(false, mode, __toUint8Array(key._raw), ivBytes, __toUint8Array(data)));
        },
        sign: async function(algo, key, data) {
            if (__normalizeAlgorithmName(algo || key.algorithm) === 'HMAC' || key.algorithm.name === 'HMAC') {
                var hash = (algo && algo.hash) || (key.algorithm && key.algorithm.hash) || 'SHA-256';
                return __bytesToArrayBuffer(__nativeHmacBytes(hash, __toUint8Array(key._raw), __toUint8Array(data)));
            }
            if (typeof __crypto_sign_hex === 'undefined') throw new Error('Native signature bridge is unavailable');
            var sigHex = __crypto_sign_hex(__signatureAlgorithmName(algo, key), __bytesToHex(key._raw), __bytesToHex(__toUint8Array(data)));
            return __bytesToArrayBuffer(__hexToBytes(sigHex));
        },
        verify: async function(algo, key, sig, data) {
            if (__normalizeAlgorithmName(algo || key.algorithm) === 'HMAC' || key.algorithm.name === 'HMAC') {
                var expected = __nativeHmacBytes((algo && algo.hash) || (key.algorithm && key.algorithm.hash) || 'SHA-256', __toUint8Array(key._raw), __toUint8Array(data));
                var actual = __toUint8Array(sig);
                if (expected.length !== actual.length) return false;
                var diff = 0;
                for (var i = 0; i < expected.length; i++) diff |= expected[i] ^ actual[i];
                return diff === 0;
            }
            if (typeof __crypto_verify_hex === 'undefined') throw new Error('Native signature bridge is unavailable');
            return __crypto_verify_hex(__signatureAlgorithmName(algo, key), __bytesToHex(key._raw), __bytesToHex(__toUint8Array(sig)), __bytesToHex(__toUint8Array(data)));
        }
    },
    getRandomValues: function(arr) {
        if (!arr) return arr;
        var byteLength = arr.byteLength != undefined ? arr.byteLength : arr.length;
        if (!byteLength) return arr;
        if (typeof __crypto_get_random_values_hex === 'undefined') throw new Error('Native random bridge is unavailable');
        var random = __hexToBytes(__crypto_get_random_values_hex(byteLength));
        if (arr.buffer && arr.byteLength != undefined) {
            new Uint8Array(arr.buffer, arr.byteOffset || 0, arr.byteLength).set(random);
        } else {
            for (var i = 0; i < arr.length; i++) arr[i] = random[i] || 0;
        }
        return arr;
    },
    randomUUID: function() {
        var b = new Uint8Array(16);
        globalThis.crypto.getRandomValues(b);
        b[6] = (b[6] & 0x0f) | 0x40;
        b[8] = (b[8] & 0x3f) | 0x80;
        var h = __bytesToHex(b);
        return h.substr(0, 8) + '-' + h.substr(8, 4) + '-' + h.substr(12, 4) + '-' + h.substr(16, 4) + '-' + h.substr(20);
    }
};
