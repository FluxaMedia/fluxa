(function () {
  [window.WebGLRenderingContext, window.WebGL2RenderingContext].forEach(function (Ctx) {
    if (!Ctx) return;
    var get = Ctx.prototype.getExtension;
    Ctx.prototype.getExtension = function (name) {
      var ext = get.call(this, name);
      if (ext && /astc/i.test(String(name)) && typeof ext.getSupportedProfiles !== "function") {
        ext.getSupportedProfiles = function () { return ["ldr"]; };
      }
      return ext;
    };
  });
  window.loadWasm = function (url) {
    return new Promise(function (resolve, reject) {
      var request = new XMLHttpRequest();
      request.open("GET", url);
      request.responseType = "arraybuffer";
      request.onload = function () { resolve(request.response); };
      request.onerror = reject;
      request.send();
    });
  };
})();
