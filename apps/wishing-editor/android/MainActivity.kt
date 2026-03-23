package dev.dioxus.main

import android.os.Bundle
import io.github.wishingroom.editor.BuildConfig
import java.io.File
import java.io.PrintWriter
import java.io.StringWriter

typealias BuildConfig = BuildConfig

class MainActivity : WryActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        installCrashLogger()
        appendBootstrapLog("activity:onCreate:start")
        try {
            super.onCreate(savedInstanceState)
            appendBootstrapLog("activity:onCreate:ok")
        } catch (throwable: Throwable) {
            appendBootstrapLog("activity:onCreate:throw\n${stackTrace(throwable)}")
            throw throwable
        }
    }

    override fun onStart() {
        appendBootstrapLog("activity:onStart")
        super.onStart()
    }

    override fun onResume() {
        appendBootstrapLog("activity:onResume")
        super.onResume()
    }

    override fun onPause() {
        appendBootstrapLog("activity:onPause")
        super.onPause()
    }

    override fun onStop() {
        appendBootstrapLog("activity:onStop")
        super.onStop()
    }

    override fun onDestroy() {
        appendBootstrapLog("activity:onDestroy")
        super.onDestroy()
    }

    private fun installCrashLogger() {
        val previous = Thread.getDefaultUncaughtExceptionHandler()
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            appendBootstrapLog("uncaught:${thread.name}\n${stackTrace(throwable)}")
            previous?.uncaughtException(thread, throwable)
        }
    }

    private fun appendBootstrapLog(message: String) {
        try {
            val dir = getExternalFilesDir("logs") ?: File(filesDir, "logs")
            if (!dir.exists()) {
                dir.mkdirs()
            }
            File(dir, "wishing-editor.log").appendText(
                "[${System.currentTimeMillis()}] java: $message\n"
            )
        } catch (_: Throwable) {
        }
    }

    private fun stackTrace(throwable: Throwable): String {
        val writer = StringWriter()
        throwable.printStackTrace(PrintWriter(writer))
        return writer.toString()
    }
}
