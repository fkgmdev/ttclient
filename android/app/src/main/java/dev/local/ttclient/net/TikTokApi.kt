package dev.local.ttclient.net

import android.content.Context
import androidx.core.content.edit
import dev.local.ttclient.core.Signer
import dev.local.ttclient.data.FeedResponse
import kotlinx.serialization.json.Json
import okhttp3.Headers
import okhttp3.OkHttpClient
import okhttp3.Protocol
import okhttp3.Request
import okhttp3.HttpUrl.Companion.toHttpUrl
import org.json.JSONObject
import java.net.URLEncoder
import java.util.concurrent.TimeUnit

/** Anonymous (no-login) TikTok private API client for the For You feed. */
class TikTokApi(context: Context) {

    companion object {
        private const val API_HOST = "api16-normal-c-useast2a.tiktokv.com"
        private const val PREFS_NAME = "tt_identity"
        private const val KEY_DEVICE = "device_json"

        private const val USER_AGENT =
            "com.zhiliaoapp.musically/2023700040 (Linux; U; Android 9; en_US; " +
                "2203121C; Build/PQ3A.190705.09121607;tt-ok/3.12.13.4-tiktok)"

        private val json = Json {
            ignoreUnknownKeys = true
            coerceInputValues = true
        }
    }

    private val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    /** Stable across app restarts so the account-less identity stays warm. */
    val deviceJson: String by lazy {
        prefs.getString(KEY_DEVICE, null) ?: Signer.newDevice().also { device ->
            prefs.edit { putString(KEY_DEVICE, device) }
        }
    }

    private val client = OkHttpClient.Builder()
        .connectTimeout(20, TimeUnit.SECONDS)
        .readTimeout(60, TimeUnit.SECONDS)
        .followRedirects(true)
        // TikTok's edge servers stall OkHttp's HTTP/2 streams from time to
        // time; HTTP/1.1 matches the transport used by the working reference.
        .protocols(listOf(Protocol.HTTP_1_1))
        .build()

    private fun urlEncode(value: String): String =
        URLEncoder.encode(value, "UTF-8").replace("*", "%2A")

    private fun buildFeedUrl(host: String, maxCursor: Long, count: Int): String {
        val nowMs = System.currentTimeMillis()
        val nowSec = nowMs / 1000
        val d = JSONObject(deviceJson)

        val params = linkedMapOf<String, String>().apply {
            this["device_platform"] = "android"
            this["os"] = "android"
            this["ssmix"] = "a"
            this["_rticket"] = nowMs.toString()
            this["cdid"] = d.getString("cdid")
            this["channel"] = d.getString("channel")
            this["aid"] = "1233"
            this["app_name"] = "musical_ly"
            this["version_code"] = "370004"
            this["version_name"] = "37.0.4"
            this["manifest_version_code"] = "2023700040"
            this["update_version_code"] = "2023700040"
            this["ab_version"] = "37.0.4"
            this["resolution"] = d.getString("resolution")
            this["dpi"] = d.getString("dpi")
            this["device_type"] = d.getString("device_type")
            this["device_brand"] = d.getString("device_brand")
            this["language"] = "en"
            this["os_api"] = "28"
            this["os_version"] = d.getString("os_version")
            this["ac"] = "wifi"
            this["is_pad"] = "0"
            this["current_region"] = "US"
            this["app_type"] = "normal"
            this["sys_region"] = "US"
            this["last_install_time"] = (nowSec - 86400).toString()
            this["mcc_mnc"] = d.getString("mcc_mnc")
            this["timezone_name"] = d.getString("timezone_name")
            this["carrier_region_v2"] = "310"
            this["residence"] = "US"
            this["app_language"] = "en"
            this["carrier_region"] = "US"
            this["timezone_offset"] = d.getString("timezone_offset")
            this["host_abi"] = "arm64-v8a"
            this["locale"] = d.getString("locale")
            this["ac2"] = "wifi"
            this["uoo"] = "0"
            this["op_region"] = "US"
            this["build_number"] = "37.0.4"
            this["region"] = "US"
            this["ts"] = nowSec.toString()
            this["iid"] = d.getString("iid")
            this["device_id"] = d.getString("device_id")
            this["openudid"] = d.getString("openudid")
            this["count"] = count.toString()
            this["max_cursor"] = maxCursor.toString()
            this["type"] = "0"
        }

        val query = params.entries.joinToString("&") { (key, value) ->
            "$key=${urlEncode(value)}"
        }
        return "https://$host/aweme/v1/feed/?$query"
    }

    /** Fetch one page of the For You feed, falling back to another edge host. */
    fun forYou(maxCursor: Long = 0, count: Int = 6): FeedResponse {
        var lastError: Exception? = null
        for (host in listOf(API_HOST, "api.tiktokv.com")) {
            try {
                return forYouHost(host, maxCursor, count)
            } catch (e: Exception) {
                lastError = e
                android.util.Log.w("TTClient", "feed attempt on $host failed: ${e.message}")
            }
        }
        throw lastError ?: error("no API hosts available")
    }

    private fun forYouHost(host: String, maxCursor: Long, count: Int): FeedResponse {
        val url = buildFeedUrl(host, maxCursor, count)
        // Sign the exact query that OkHttp will put on the wire.
        val encodedQuery = url.toHttpUrl().encodedQuery
            ?: error("failed to build feed URL")

        val signatures = Signer.signOrThrow(encodedQuery, device = deviceJson)

        val headers = Headers.Builder().apply {
            signatures.forEach { (name, value) -> add(name, value) }
            add("User-Agent", USER_AGENT)
            // Let OkHttp add Accept-Encoding itself: setting it manually
            // disables its transparent gzip decompression.
            add("sdk-version", "2")
            add("x-tt-dm-status", "login=0;ct=1;rt=7")
        }.build()

        val request = Request.Builder()
            .url(url)
            .headers(headers)
            .get()
            .build()

        client.newCall(request).execute().use { response ->
            val body = response.body?.string() ?: error("empty API response")
            if (!response.isSuccessful) {
                val hint = if (response.code == 429) " (rate limited — wait a few minutes)" else ""
                error("HTTP ${response.code}$hint: ${body.take(160)}")
            }
            val feed = json.decodeFromString<FeedResponse>(body)
            if (feed.statusCode != 0) {
                error("API status ${feed.statusCode}: ${body.take(240)}")
            }
            return feed
        }
    }
}
