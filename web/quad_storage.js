// localStorage bridge for the quad-storage crate (the scoreboard). Strings
// cross the boundary as sapp_jsutils object ids. Registered as a miniquad
// plugin together with the sapp_jsutils string helpers.
miniquad_add_plugin({
  name: "quad_storage",
  version: (0 << 24) + (1 << 16) + 0,
  register_plugin: function (importObject) {
    if (typeof register_plugin === "function") register_plugin(importObject);
    importObject.env.quad_storage_length = function () { return localStorage.length; };
    importObject.env.quad_storage_has_key = function (i) { return localStorage.key(i) !== null ? 1 : 0; };
    importObject.env.quad_storage_key = function (i) { return js_object(localStorage.key(i)); };
    importObject.env.quad_storage_has_value = function (key) { return localStorage.getItem(js_objects[key]) !== null ? 1 : 0; };
    importObject.env.quad_storage_get = function (key) { var v = localStorage.getItem(js_objects[key]); return js_object(v === null ? "" : v); };
    importObject.env.quad_storage_set = function (key, value) { localStorage.setItem(js_objects[key], js_objects[value]); };
    importObject.env.quad_storage_remove = function (key) { localStorage.removeItem(js_objects[key]); };
    importObject.env.quad_storage_clear = function () { localStorage.clear(); };
  },
});
