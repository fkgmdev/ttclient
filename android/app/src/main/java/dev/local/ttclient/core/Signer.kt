package dev.local.ttclient.core

import org.json.JSONObject

/** JNI bridge to the Rust signing core. */
object Signer {
    init {
        System.loadLibrary("ttcore")
    }

    /** Create a fresh anonymous device identity as a JSON string. */
    external fun newDevice(): String

    /**
     * Sign a request. Returns a JSON object of header name -> value.
     * The returned object contains an "error" key when signing fails.
     */
    external fun sign(query: String, body: String?, cookie: String?, device: String): String

    fun signOrThrow(query: String, body: String? = null, cookie: String? = null, device: String): Map<String, String> {
        val json = JSONObject(sign(query, body, cookie, device))
        if (json.has("error")) {
            error("Signing failed: ${json.getString("error")}")
        }
        val out = mutableMapOf<String, String>()
        val keys = json.keys()
        while (keys.hasNext()) {
            val key = keys.next()
            out[key] = json.getString(key)
        }
        return out
    }
}
