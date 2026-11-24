package dev.local.ttclient.data

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class FeedResponse(
    @SerialName("status_code") val statusCode: Int = 0,
    @SerialName("max_cursor") val maxCursor: Long = 0,
    @SerialName("has_more") val hasMore: Int = 0,
    @SerialName("aweme_list") val awemeList: List<Aweme> = emptyList(),
)

@Serializable
data class Aweme(
    @SerialName("aweme_id") val awemeId: String = "",
    val desc: String = "",
    val author: Author = Author(),
    val video: Video = Video(),
    val statistics: Statistics = Statistics(),
)

@Serializable
data class Author(
    val uid: String = "",
    val nickname: String = "",
    @SerialName("unique_id") val uniqueId: String = "",
    @SerialName("avatar_thumb") val avatarThumb: ImageList? = null,
    @SerialName("avatar_medium") val avatarMedium: ImageList? = null,
)

@Serializable
data class ImageList(
    @SerialName("url_list") val urlList: List<String> = emptyList(),
) {
    val firstUrl: String? get() = urlList.firstOrNull()
}

@Serializable
data class Video(
    @SerialName("play_addr") val playAddr: VideoAddr = VideoAddr(),
    @SerialName("play_addr_h264") val playAddrH264: VideoAddr? = null,
    val cover: ImageList = ImageList(),
    val duration: Long = 0,
    val width: Int = 0,
    val height: Int = 0,
)

@Serializable
data class VideoAddr(
    @SerialName("url_list") val urlList: List<String> = emptyList(),
    val width: Int = 0,
    val height: Int = 0,
) {
    val firstUrl: String? get() = urlList.firstOrNull()
}

@Serializable
data class Statistics(
    @SerialName("digg_count") val diggCount: Long = 0,
    @SerialName("comment_count") val commentCount: Long = 0,
    @SerialName("share_count") val shareCount: Long = 0,
    @SerialName("play_count") val playCount: Long = 0,
    @SerialName("collect_count") val collectCount: Long = 0,
)
