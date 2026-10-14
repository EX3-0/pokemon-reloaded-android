package com.klisee.reloaded

import android.content.Context
import org.json.JSONObject
import java.io.File
import java.util.Locale

/** A control's default shape, as the engine generates it (see `tools/generate-layout.sh`). */
data class ControlDefault(
    val name: String,
    val kind: String,
    val cx: Float,
    val cy: Float,
    val r: Float,
    val w: Float,
    val h: Float,
    val label: String,
)

/** A player's adjustment to one control: where it sits, how big it is and whether it is shown. */
data class ControlLayout(val cx: Float, val cy: Float, val scale: Float, val visible: Boolean)

/**
 * The customisable on-screen controls' layout.
 *
 * The defaults come from the engine itself (`controls-layout.json`, generated from its control
 * table), and the player's version lives in `layout.conf`, the same file the engine watches while
 * the game runs — so an edit here shows up in the game without restarting it.
 */
object Layout {
    const val JOYSTICK = "joystick"
    private const val MIN_SCALE = 0.3f

    fun defaults(context: Context): List<ControlDefault> {
        val text = context.assets.open("controls-layout.json").bufferedReader().use { it.readText() }
        val controls = JSONObject(text).getJSONArray("controls")
        return (0 until controls.length()).map { index ->
            val o = controls.getJSONObject(index)
            ControlDefault(
                name = o.getString("name"),
                kind = o.getString("kind"),
                cx = o.getDouble("cx").toFloat(),
                cy = o.getDouble("cy").toFloat(),
                r = o.optDouble("r", 0.0).toFloat(),
                w = o.optDouble("w", 0.0).toFloat(),
                h = o.optDouble("h", 0.0).toFloat(),
                label = o.optString("label"),
            )
        }
    }

    /** Centre and radius of the joystick, as the engine defines it. */
    fun joystickDefault(context: Context): Triple<Float, Float, Float>? {
        val text = context.assets.open("controls-layout.json").bufferedReader().use { it.readText() }
        val o = JSONObject(text).optJSONObject("joystick") ?: return null
        return Triple(o.getDouble("cx").toFloat(), o.getDouble("cy").toFloat(), o.getDouble("r").toFloat())
    }

    /** Every control plus the joystick, at their built-in positions. */
    fun defaultOverrides(context: Context): Map<String, ControlLayout> {
        val controls = defaults(context).associate { it.name to ControlLayout(it.cx, it.cy, 1f, true) }
        val joystick = joystickDefault(context)?.let { (cx, cy, _) -> ControlLayout(cx, cy, 1f, true) }
        return if (joystick == null) controls else controls + (JOYSTICK to joystick)
    }

    /** The saved layout, falling back to the defaults for anything the file does not mention. */
    fun read(context: Context, filesDir: File): Map<String, ControlLayout> {
        val layout = defaultOverrides(context).toMutableMap()
        val file = File(filesDir, "layout.conf")
        if (!file.isFile) return layout
        for (line in file.readLines()) {
            val trimmed = line.trim()
            if (trimmed.isEmpty() || trimmed.startsWith("#")) continue
            val split = trimmed.split('=', limit = 2)
            if (split.size != 2) continue
            val parts = split[1].split(',').map { it.trim() }
            if (parts.size < 4) continue
            val cx = parts[0].toFloatOrNull() ?: continue
            val cy = parts[1].toFloatOrNull() ?: continue
            val scale = parts[2].toFloatOrNull() ?: continue
            layout[split[0].trim().lowercase()] = ControlLayout(cx, cy, scale.coerceAtLeast(MIN_SCALE), parts[3] != "0")
        }
        return layout
    }

    /**
     * Writes every control, not only the ones that moved: the engine resets anything the file does
     * not mention back to its built-in position, so a partial file would quietly undo old choices.
     * The decimal separator is forced to `.`, which is what the engine's parser expects.
     */
    fun write(context: Context, filesDir: File, layout: Map<String, ControlLayout>) {
        val order = defaults(context).map { it.name } + JOYSTICK
        val text = buildString {
            appendLine("# Layout de los controles tactiles; lo escriben el editor del juego y la app.")
            for (name in order) {
                val entry = layout[name] ?: continue
                val visible = if (entry.visible) 1 else 0
                appendLine(String.format(Locale.US, "%s=%.4f,%.4f,%.4f,%d", name, entry.cx, entry.cy, entry.scale, visible))
            }
        }
        runCatching { File(filesDir, "layout.conf").writeText(text) }
    }

    fun restoreDefaults(context: Context, filesDir: File): Map<String, ControlLayout> {
        val layout = defaultOverrides(context)
        write(context, filesDir, layout)
        return layout
    }
}
