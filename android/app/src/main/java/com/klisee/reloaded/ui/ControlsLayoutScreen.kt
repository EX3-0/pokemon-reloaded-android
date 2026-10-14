package com.klisee.reloaded.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.pm.ActivityInfo
import android.graphics.BitmapFactory
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.klisee.reloaded.ControlDefault
import com.klisee.reloaded.ControlLayout
import com.klisee.reloaded.Layout
import java.io.File
import kotlin.math.abs

private fun Context.findActivity(): Activity? {
    var context: Context? = this
    while (context is ContextWrapper) {
        if (context is Activity) return context
        context = context.baseContext
    }
    return null
}

/** Decodes one of the bundled control sprites, or null when it is missing. */
private fun loadAssetBitmap(context: Context, name: String): ImageBitmap? =
    runCatching { context.assets.open(name).use { BitmapFactory.decodeStream(it)?.asImageBitmap() } }.getOrNull()

/**
 * Places the on-screen controls: drag one to move it, pick one and adjust its size and whether it is
 * shown. Saves to the same `layout.conf` the engine watches, so the game picks it up without
 * restarting.
 *
 * The screen turns sideways while it is open: the game is landscape, so the preview fills the screen
 * instead of being a strip in the middle, which is what made it awkward to aim at.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ControlsLayoutScreen(filesDir: File, onBack: () -> Unit) {
    val context = LocalContext.current
    val activity = remember(context) { context.findActivity() }
    DisposableEffect(activity) {
        val previous = activity?.requestedOrientation
        activity?.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
        onDispose { if (previous != null) activity.requestedOrientation = previous }
    }

    val controls = remember { Layout.defaults(context) }
    val joystick = remember { Layout.joystickDefault(context) }
    // The same artwork the game draws, so the preview matches what ends up on screen.
    val joystickBase = remember { loadAssetBitmap(context, "joystick-base.png") }
    val joystickKnob = remember { loadAssetBitmap(context, "joystick-knob.png") }
    // Any control that ships its own artwork (`<name>.png`), so the preview matches the game.
    val sprites = remember(controls) {
        controls.mapNotNull { control -> loadAssetBitmap(context, "${control.name}.png")?.let { control.name to it } }.toMap()
    }
    var layout by remember { mutableStateOf(Layout.read(context, filesDir)) }
    var selected by remember { mutableStateOf<String?>(null) }
    var saved by remember { mutableStateOf(false) }

    Column(Modifier.fillMaxSize().background(Color(0xFF0B0B12))) {
        // The preview is the game window: the controls are normalised to it, so it fills everything
        // above the bar.
        Box(
            Modifier
                .fillMaxWidth()
                .weight(1f)
                .padding(6.dp)
                .background(Color(0xFF101018))
                .border(1.dp, Color(0xFF3A3A4A)),
        ) {
            LayoutPreview(
                controls = controls,
                joystick = joystick,
                joystickBase = joystickBase,
                joystickKnob = joystickKnob,
                sprites = sprites,
                layout = layout,
                selected = selected,
                onSelect = { selected = it },
                onMove = { name, nx, ny ->
                    val entry = layout[name] ?: return@LayoutPreview
                    layout = layout + (name to entry.copy(cx = nx, cy = ny))
                    saved = false
                },
            )
        }

        val entry = selected?.let { layout[it] }
        val default = selected?.let { name -> controls.firstOrNull { it.name == name } }
        Row(
            modifier = Modifier.fillMaxWidth().background(Color(0xFF16161F)).padding(horizontal = 12.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            IconButton(onClick = onBack) { Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Volver") }
            Text(
                when (selected) {
                    null -> "Toca un control"
                    Layout.JOYSTICK -> "Joystick"
                    else -> default?.label?.takeIf { it.isNotEmpty() } ?: selected ?: ""
                },
                style = MaterialTheme.typography.titleSmall,
                modifier = Modifier.width(96.dp),
            )
            Text("Tamano", style = MaterialTheme.typography.bodySmall)
            Slider(
                value = entry?.scale ?: 1f,
                onValueChange = { value ->
                    val name = selected ?: return@Slider
                    val current = layout[name] ?: return@Slider
                    layout = layout + (name to current.copy(scale = value.coerceAtLeast(0.3f)))
                    saved = false
                },
                valueRange = 0.4f..3f,
                enabled = entry != null,
                modifier = Modifier.weight(1f),
            )
            Text("Mostrar", style = MaterialTheme.typography.bodySmall)
            Switch(
                checked = entry?.visible ?: true,
                enabled = entry != null,
                onCheckedChange = { value ->
                    val name = selected ?: return@Switch
                    val current = layout[name] ?: return@Switch
                    layout = layout + (name to current.copy(visible = value))
                    saved = false
                },
            )
            Button(onClick = {
                layout = Layout.restoreDefaults(context, filesDir)
                selected = null
                saved = true
            }) { Text("Restaurar") }
            Button(onClick = {
                Layout.write(context, filesDir, layout)
                saved = true
            }) { Text(if (saved) "Guardado" else "Guardar") }
        }
    }
}

@Composable
private fun LayoutPreview(
    controls: List<ControlDefault>,
    joystick: Triple<Float, Float, Float>?,
    joystickBase: ImageBitmap?,
    joystickKnob: ImageBitmap?,
    sprites: Map<String, ImageBitmap>,
    layout: Map<String, ControlLayout>,
    selected: String?,
    onSelect: (String?) -> Unit,
    onMove: (String, Float, Float) -> Unit,
) {
    val measurer = rememberTextMeasurer()
    var dragging by remember { mutableStateOf<String?>(null) }
    // The gesture handler must not be rebuilt while a drag is in flight — rebuilding it cancels the
    // drag, which is why a button could only be nudged a few pixels before it stopped following the
    // finger. Reading the latest layout and callbacks through these keeps `pointerInput(Unit)` stable.
    val currentLayout by rememberUpdatedState(layout)
    val currentSelect by rememberUpdatedState(onSelect)
    val currentMove by rememberUpdatedState(onMove)

    fun controlAt(offset: Offset, w: Float, h: Float): String? {
        val nx = offset.x / w
        val ny = offset.y / h
        val value = currentLayout
        return controls.lastOrNull { hit(it, value[it.name], nx, ny, w, h) }?.name
            ?: joystick?.takeIf { hitCircle(it, value[Layout.JOYSTICK], nx, ny, w, h) }?.let { Layout.JOYSTICK }
    }

    Canvas(
        Modifier
            .fillMaxSize()
            // A plain tap just picks a control; the drag that follows is what moves it.
            .pointerInput(Unit) {
                detectTapGestures { offset ->
                    val w = size.width.toFloat().coerceAtLeast(1f)
                    val h = size.height.toFloat().coerceAtLeast(1f)
                    currentSelect(controlAt(offset, w, h))
                }
            }
            .pointerInput(Unit) {
                detectDragGestures(
                    onDragStart = { offset ->
                        val w = size.width.toFloat().coerceAtLeast(1f)
                        val h = size.height.toFloat().coerceAtLeast(1f)
                        val found = controlAt(offset, w, h)
                        dragging = found
                        currentSelect(found)
                    },
                    onDragEnd = { dragging = null },
                    onDragCancel = { dragging = null },
                    onDrag = { change, drag ->
                        val name = dragging ?: return@detectDragGestures
                        change.consume()
                        val entry = currentLayout[name] ?: return@detectDragGestures
                        val w = size.width.toFloat().coerceAtLeast(1f)
                        val h = size.height.toFloat().coerceAtLeast(1f)
                        currentMove(name, (entry.cx + drag.x / w).coerceIn(0f, 1f), (entry.cy + drag.y / h).coerceIn(0f, 1f))
                    },
                )
            },
    ) {
        val w = size.width
        val h = size.height
        for (control in controls) {
            val entry = layout[control.name] ?: continue
            val cx = entry.cx * w
            val cy = entry.cy * h
            val isSelected = selected == control.name
            val stroke = if (!entry.visible) Color(0xFFFF5555) else if (isSelected) Color(0xFFFFD633) else Color(0xFFFFFFFF)
            val fill = if (entry.visible) Color(0x22FFFFFF) else Color(0x22FF5555)
            val sprite = sprites[control.name]
            if (sprite != null) {
                // The control's own artwork, drawn at the size the game draws it.
                val half = if (control.kind == "circle") {
                    control.r * entry.scale * w
                } else {
                    (control.w * entry.scale * w + control.h * entry.scale * h) / 4f
                }
                drawImage(
                    image = sprite,
                    dstOffset = IntOffset((cx - half).toInt(), (cy - half).toInt()),
                    dstSize = IntSize((half * 2f).toInt().coerceAtLeast(1), (half * 2f).toInt().coerceAtLeast(1)),
                    filterQuality = FilterQuality.None,
                )
                drawCircle(color = stroke, radius = half + 3f, center = Offset(cx, cy), style = Stroke(width = if (isSelected) 4f else 2f))
                continue
            }
            if (control.kind == "circle") {
                val r = control.r * entry.scale * w
                drawCircle(color = fill, radius = r, center = Offset(cx, cy))
                drawCircle(color = stroke, radius = r, center = Offset(cx, cy), style = Stroke(width = if (isSelected) 4f else 2f))
            } else {
                val rw = control.w * entry.scale * w
                val rh = control.h * entry.scale * h
                val topLeft = Offset(cx - rw / 2f, cy - rh / 2f)
                drawRect(color = fill, topLeft = topLeft, size = Size(rw, rh))
                drawRect(color = stroke, topLeft = topLeft, size = Size(rw, rh), style = Stroke(width = if (isSelected) 4f else 2f))
            }
            if (control.label.isNotEmpty()) {
                drawText(
                    textMeasurer = measurer,
                    text = control.label,
                    topLeft = Offset(cx - 6f, cy - 7f),
                    style = TextStyle(color = stroke, fontSize = 11.sp),
                )
            }
        }
        if (joystick != null) {
            val entry = layout[Layout.JOYSTICK]
            if (entry != null) {
                val cx = entry.cx * w
                val cy = entry.cy * h
                val r = joystick.third * entry.scale * h
                val isSelected = selected == Layout.JOYSTICK
                val stroke = if (!entry.visible) Color(0xFFFF5555) else if (isSelected) Color(0xFFFFD633) else Color(0xCCFFFFFF)
                // The same artwork the game draws: a square base with the knob at its centre.
                val half = r * 1.05f
                if (joystickBase != null) {
                    drawImage(
                        image = joystickBase,
                        dstOffset = IntOffset((cx - half).toInt(), (cy - half).toInt()),
                        dstSize = IntSize((half * 2f).toInt().coerceAtLeast(1), (half * 2f).toInt().coerceAtLeast(1)),
                        filterQuality = FilterQuality.None,
                    )
                } else {
                    drawCircle(color = Color(0x2200FFFF), radius = r, center = Offset(cx, cy))
                }
                val kh = r * 0.88f
                if (joystickKnob != null) {
                    drawImage(
                        image = joystickKnob,
                        dstOffset = IntOffset((cx - kh).toInt(), (cy - kh).toInt()),
                        dstSize = IntSize((kh * 2f).toInt().coerceAtLeast(1), (kh * 2f).toInt().coerceAtLeast(1)),
                        filterQuality = FilterQuality.None,
                    )
                }
                drawCircle(color = stroke, radius = r, center = Offset(cx, cy), style = Stroke(width = if (isSelected) 4f else 2f))
            }
        }
    }
}

private fun hit(control: ControlDefault, entry: ControlLayout?, nx: Float, ny: Float, w: Float, h: Float): Boolean {
    val e = entry ?: return false
    val dx = (nx - e.cx) * w
    val dy = (ny - e.cy) * h
    return if (control.kind == "circle") {
        val r = control.r * e.scale * w
        dx * dx + dy * dy <= r * r
    } else {
        abs(dx) <= control.w * e.scale * w / 2f && abs(dy) <= control.h * e.scale * h / 2f
    }
}

private fun hitCircle(joystick: Triple<Float, Float, Float>, entry: ControlLayout?, nx: Float, ny: Float, w: Float, h: Float): Boolean {
    val e = entry ?: return false
    val dx = (nx - e.cx) * w
    val dy = (ny - e.cy) * h
    val r = joystick.third * e.scale * h
    return dx * dx + dy * dy <= r * r
}
