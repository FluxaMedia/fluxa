package com.fluxa.app.shared.image

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.viewinterop.UIKitView
import platform.Foundation.NSError
import platform.Foundation.NSOperationQueue
import platform.Foundation.NSURL
import platform.Foundation.NSURLSession
import platform.Foundation.NSURLSessionConfiguration
import platform.Foundation.NSURLSessionDownloadDelegateProtocol
import platform.Foundation.NSURLSessionDownloadTask
import platform.Foundation.NSURLSessionTask
import platform.UIKit.UIImage
import platform.UIKit.UIImageView
import platform.UIKit.UIViewContentMode
import platform.darwin.NSObject

private val imageCache = mutableMapOf<String, UIImage>()

@Composable
actual fun FluxaRemoteImage(
    imageUrl: String?,
    cacheKey: String?,
    contentDescription: String?,
    modifier: Modifier,
    contentScale: ContentScale,
    onError: (() -> Unit)?
) {
    val imageView = remember(imageUrl, cacheKey) {
        UIImageView().apply {
            clipsToBounds = true
            contentMode = UIViewContentMode.UIViewContentModeScaleAspectFill
            imageUrl?.let { url -> loadImage(url, cacheKey, this, onError) }
        }
    }
    UIKitView(
        factory = { imageView },
        modifier = modifier,
        update = {
            it.contentMode = if (contentScale == ContentScale.Fit) {
                UIViewContentMode.UIViewContentModeScaleAspectFit
            } else {
                UIViewContentMode.UIViewContentModeScaleAspectFill
            }
        }
    )
}

private fun loadImage(
    imageUrl: String,
    cacheKey: String?,
    imageView: UIImageView,
    onError: (() -> Unit)?
) {
    val key = cacheKey ?: imageUrl
    imageCache[key]?.let {
        imageView.image = it
        return
    }
    val url = NSURL.URLWithString(imageUrl)
    if (url == null) {
        onError?.invoke()
        return
    }
    val delegate = ImageSessionDelegate(key, imageView, onError)
    NSURLSession.sessionWithConfiguration(
        configuration = NSURLSessionConfiguration.defaultSessionConfiguration(),
        delegate = delegate,
        delegateQueue = NSOperationQueue.mainQueue
    ).downloadTaskWithURL(url).resume()
}

private class ImageSessionDelegate(
    private val cacheKey: String,
    private val imageView: UIImageView,
    private val onError: (() -> Unit)?
) : NSObject(), NSURLSessionDownloadDelegateProtocol {

    override fun URLSession(
        session: NSURLSession,
        downloadTask: NSURLSessionDownloadTask,
        didFinishDownloadingToURL: NSURL
    ) {
        val image = didFinishDownloadingToURL.path?.let(::UIImage)
        if (image == null) {
            onError?.invoke()
        } else {
            imageCache[cacheKey] = image
            imageView.image = image
        }
    }

    override fun URLSession(
        session: NSURLSession,
        task: NSURLSessionTask,
        didCompleteWithError: NSError?
    ) {
        if (didCompleteWithError != null) {
            onError?.invoke()
        }
    }
}
