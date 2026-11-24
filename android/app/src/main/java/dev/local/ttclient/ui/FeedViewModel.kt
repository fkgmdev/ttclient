package dev.local.ttclient.ui

import android.app.Application
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dev.local.ttclient.data.Aweme
import dev.local.ttclient.net.TikTokApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class FeedViewModel(application: Application) : AndroidViewModel(application) {

    companion object {
        private const val TAG = "TTClient"
    }

    private val api = TikTokApi(application)

    private val _items = MutableStateFlow<List<Aweme>>(emptyList())
    val items: StateFlow<List<Aweme>> = _items.asStateFlow()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    private val _error = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = _error.asStateFlow()

    private var cursor = 0L
    private var hasMore = true

    init {
        loadMore()
    }

    fun loadMore() {
        if (_isLoading.value || !hasMore) return
        // Set the flag synchronously so rapid calls from the pager can't race.
        _isLoading.value = true
        viewModelScope.launch {
            _error.value = null
            try {
                val page = withContext(Dispatchers.IO) { api.forYou(cursor) }
                val existing = _items.value.map { it.awemeId }.toSet()
                val fresh = page.awemeList.filterNot { it.awemeId in existing }
                _items.value = _items.value + fresh
                cursor = page.maxCursor
                hasMore = page.hasMore == 1 && page.awemeList.isNotEmpty()
            } catch (e: Exception) {
                Log.e(TAG, "feed load failed", e)
                _error.value = e.message ?: "Unknown error"
            } finally {
                _isLoading.value = false
            }
        }
    }

    fun retry() {
        _error.value = null
        if (_items.value.isEmpty()) {
            cursor = 0L
            hasMore = true
        }
        loadMore()
    }
}
