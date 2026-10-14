package com.klisee.reloaded

import android.content.Context
import android.content.Intent
import java.io.File

/** Starts the native activity that runs the bundled `libgm8emulator.so`. */
object Emulator {
    private const val GAME_ACTIVITY = "com.klisee.reloaded.GameActivity"

    fun start(context: Context) {
        // The emulator reads the pad layout and the video mode once, at startup, so they have to be
        // on disk by now.
        val prefs = Preferences(context)
        prefs.writeControls(context.filesDir, connectedPads(context).isNotEmpty())
        prefs.writeVideo(context.filesDir)
        // A marker left over from a previous run would hand the screen over before the game paints.
        runCatching { File(context.filesDir, GameActivity.READY_FILE).delete() }
        context.startActivity(Intent().setClassName(context.packageName, GAME_ACTIVITY))
    }
}
