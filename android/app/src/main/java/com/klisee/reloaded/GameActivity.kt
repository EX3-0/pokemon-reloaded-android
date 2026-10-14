package com.klisee.reloaded

import android.app.NativeActivity
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.view.WindowManager
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import java.io.File

/**
 * The emulator activity: a plain `NativeActivity` (that is what loads `libgm8emulator.so`),
 * subclassed to take the screen once the game is really running.
 *
 * It starts with a transparent window, in the landscape the manifest asks for, and without touching
 * the bars: while the engine loads (it draws nothing for a while) the launcher and its loading screen
 * stay visible underneath. The engine drops a marker once it is really running, and only then are the
 * bars hidden.
 *
 * Landscape is fixed in the manifest rather than asked for here: the engine lays its screens out from
 * the size of the window it gets, so an activity that opened in portrait and rotated later left those
 * first screens built for a portrait window.
 *
 * Hiding the bars has to be done here rather than from Rust: the Context that `android-activity`
 * publishes for JNI is the *Application*, which has no `getWindow()`, so the native attempt always
 * failed with a NoSuchMethodError and the navigation bar stayed on screen.
 */
class GameActivity : NativeActivity() {

    private val handler = Handler(Looper.getMainLooper())
    private var onScreen = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (Preferences(applicationContext).keepScreenOn) {
            window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
        awaitFirstFrame()
    }

    /** Waits for the engine's marker, then hands the screen over to the game. */
    private fun awaitFirstFrame() {
        handler.postDelayed(
            object : Runnable {
                override fun run() {
                    if (onScreen) return
                    if (File(filesDir, READY_FILE).isFile) {
                        onScreen = true
                        hideSystemBars()
                        return
                    }
                    handler.postDelayed(this, 200)
                }
            },
            200,
        )
    }

    // The bars come back on some system events, so re-hide them whenever we regain focus.
    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus && onScreen) hideSystemBars()
    }

    override fun onDestroy() {
        handler.removeCallbacksAndMessages(null)
        super.onDestroy()
    }

    private fun hideSystemBars() {
        WindowCompat.setDecorFitsSystemWindows(window, false)
        WindowInsetsControllerCompat(window, window.decorView).apply {
            hide(WindowInsetsCompat.Type.systemBars())
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        }
    }

    companion object {
        /** Dropped by the engine the moment it presents its first frame. */
        const val READY_FILE = "game-ready"
    }
}
