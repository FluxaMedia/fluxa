package com.fluxa.app.data.platform

import kotlinx.cinterop.BetaInteropApi
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.allocArrayOf
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.usePinned
import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.suspendCancellableCoroutine
import platform.Foundation.NSData
import platform.Foundation.NSError
import platform.Foundation.NSHTTPURLResponse
import platform.Foundation.NSMutableData
import platform.Foundation.NSMutableURLRequest
import platform.Foundation.NSOperationQueue
import platform.Foundation.NSURL
import platform.Foundation.NSURLSession
import platform.Foundation.NSURLSessionConfiguration
import platform.Foundation.NSURLSessionDataDelegateProtocol
import platform.Foundation.NSURLSessionDataTask
import platform.Foundation.NSURLSessionTask
import platform.Foundation.appendData
import platform.Foundation.create
import platform.Foundation.setHTTPBody
import platform.Foundation.setHTTPMethod
import platform.Foundation.setValue
import platform.darwin.NSObject
import platform.posix.memcpy
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

class ApplePlatformHttpClient : PlatformHttpClient {
    @OptIn(ExperimentalForeignApi::class)
    override suspend fun execute(request: PlatformHttpRequest): PlatformHttpResponse = suspendCancellableCoroutine { continuation ->
        val nativeRequest = NSMutableURLRequest.requestWithURL(NSURL(string = request.url))
        nativeRequest.setHTTPMethod(request.method)
        request.headers.forEach { (name, value) -> nativeRequest.setValue(value, forHTTPHeaderField = name) }
        request.body?.let { nativeRequest.setHTTPBody(it.encodeToByteArray().toNSData()) }

        val delegate = DataTaskDelegate(continuation)
        val session = NSURLSession.sessionWithConfiguration(
            configuration = NSURLSessionConfiguration.defaultSessionConfiguration(),
            delegate = delegate,
            delegateQueue = NSOperationQueue.mainQueue
        )
        val task = session.dataTaskWithRequest(nativeRequest)
        continuation.invokeOnCancellation { task.cancel() }
        task.resume()
    }
}

private class DataTaskDelegate(
    private val continuation: CancellableContinuation<PlatformHttpResponse>
) : NSObject(), NSURLSessionDataDelegateProtocol {
    private val receivedData = NSMutableData()

    override fun URLSession(session: NSURLSession, dataTask: NSURLSessionDataTask, didReceiveData: NSData) {
        receivedData.appendData(didReceiveData)
    }

    override fun URLSession(session: NSURLSession, task: NSURLSessionTask, didCompleteWithError: NSError?) {
        if (didCompleteWithError != null) {
            continuation.resumeWithException(RuntimeException(didCompleteWithError.localizedDescription))
            return
        }
        val httpResponse = task.response as? NSHTTPURLResponse
        val statusCode = httpResponse?.statusCode?.toInt() ?: 0
        val headers = httpResponse?.allHeaderFields
            ?.mapKeys { it.key.toString() }
            ?.mapValues { it.value.toString() }
            ?: emptyMap()
        val bodyString = receivedData.toByteArray().decodeToString()
        continuation.resume(PlatformHttpResponse(statusCode = statusCode, headers = headers, body = bodyString))
    }
}

@OptIn(ExperimentalForeignApi::class, BetaInteropApi::class)
private fun ByteArray.toNSData(): NSData = if (isEmpty()) {
    NSData()
} else {
    memScoped {
        NSData.create(bytes = allocArrayOf(this@toNSData), length = this@toNSData.size.toULong())
    }
}

@OptIn(ExperimentalForeignApi::class)
private fun NSData.toByteArray(): ByteArray {
    val size = length.toInt()
    if (size == 0) return ByteArray(0)
    val result = ByteArray(size)
    result.usePinned { pinned ->
        memcpy(pinned.addressOf(0), bytes, length)
    }
    return result
}
