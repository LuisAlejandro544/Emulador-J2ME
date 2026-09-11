package com.example.ui

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowLeft
import androidx.compose.material.icons.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.example.data.J2meGame
import javax.microedition.lcdui.Canvas
import javax.microedition.lcdui.Display
import javax.microedition.lcdui.Graphics
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive

/**
 * Canvas demostrativo e interactivo de J2ME LCDUI.
 *
 * Utiliza estrictamente la API estándar `javax.microedition.lcdui.Graphics`
 * para dibujar sobre el Framebuffer nativo en C++, respondiendo a eventos
 * de teclado (`keyPressed`, `keyReleased`) y táctiles (`pointerPressed`, `pointerDragged`).
 */
class InteractiveDemoCanvas(
    private val gameTitle: String
) : Canvas() {

    private var playerX = 120
    private var playerY = 160
    private var playerColor = 0x00FFCC
    private var score = 0
    private var lastKeyName = "NINGUNA"
    private var frameCount = 0

    init {
        setSize(240, 320)
    }

    override fun paint(g: Graphics) {
        frameCount++

        // 1. Limpieza de fondo con degradado retro oscuro
        g.setColor(0x0A, 0x0E, 0x17)
        g.fillRect(0, 0, width, height)

        // 2. Marco perimetral decorativo
        g.setColor(0x1F, 0x29, 0x3D)
        g.drawRect(2, 2, width - 5, height - 5)

        // 3. Encabezado del juego
        g.setColor(0x00, 0xE5, 0xFF)
        g.drawString("J2ME LCDUI ACTIVE", width / 2, 8, Graphics.HCENTER or Graphics.TOP)

        g.setColor(0x88, 0x99, 0xAA)
        val shortTitle = if (gameTitle.length > 20) gameTitle.substring(0, 18) + ".." else gameTitle
        g.drawString(shortTitle, width / 2, 22, Graphics.HCENTER or Graphics.TOP)

        // 4. Barra divisoria
        g.setColor(0x00, 0xAA, 0x88)
        g.drawLine(10, 34, width - 10, 34)

        // 5. Entorno de juego interactivo (área de simulación)
        g.setColor(0x15, 0x1D, 0x2A)
        g.fillRect(10, 38, width - 20, 200)
        g.setColor(0x2A, 0x38, 0x4E)
        g.drawRect(10, 38, width - 20, 200)

        // Cuadrícula retro
        g.setColor(0x1A, 0x22, 0x33)
        var gx = 20
        while (gx < width - 10) {
            g.drawLine(gx, 40, gx, 236)
            gx += 20
        }
        var gy = 48
        while (gy < 236) {
            g.drawLine(12, gy, width - 12, gy)
            gy += 20
        }

        // 6. Elemento interactivo del jugador (nave / sprite vectorial)
        g.setColor(playerColor)
        g.fillRect(playerX - 8, playerY - 8, 16, 16)
        g.setColor(0xFF, 0xFF, 0xFF)
        g.drawRect(playerX - 9, playerY - 9, 18, 18)
        g.drawLine(playerX - 12, playerY, playerX + 12, playerY)
        g.drawLine(playerX, playerY - 12, playerX, playerY + 12)

        // 7. Panel de telemetría y estado
        g.setColor(0x10, 0x17, 0x22)
        g.fillRect(10, 244, width - 20, 68)
        g.setColor(0x33, 0x44, 0x60)
        g.drawRect(10, 244, width - 20, 68)

        g.setColor(0x00, 0xFF, 0x66)
        g.drawString("PUNTOS: $score", 16, 250, Graphics.LEFT or Graphics.TOP)

        g.setColor(0xFF, 0xCC, 0x00)
        g.drawString("TECLA: $lastKeyName", 16, 264, Graphics.LEFT or Graphics.TOP)

        g.setColor(0xAA, 0xBB, 0xCC)
        g.drawString("POS: ($playerX, $playerY)", 16, 278, Graphics.LEFT or Graphics.TOP)

        g.setColor(0x66, 0x77, 0x88)
        g.drawString("FRAME: $frameCount", 16, 292, Graphics.LEFT or Graphics.TOP)
    }

    override fun keyPressed(keyCode: Int) {
        lastKeyName = getKeyName(keyCode)
        val action = getGameAction(keyCode)

        when (action) {
            UP -> {
                playerY = (playerY - 6).coerceAtLeast(50)
                score += 5
            }
            DOWN -> {
                playerY = (playerY + 6).coerceAtMost(226)
                score += 5
            }
            LEFT -> {
                playerX = (playerX - 6).coerceAtLeast(20)
                score += 5
            }
            RIGHT -> {
                playerX = (playerX + 6).coerceAtMost(width - 20)
                score += 5
            }
            FIRE -> {
                // Alternar color de la nave
                playerColor = when (playerColor) {
                    0x00FFCC -> 0xFF3366
                    0xFF3366 -> 0xFFCC00
                    0xFFCC00 -> 0x9933FF
                    else -> 0x00FFCC
                }
                score += 20
            }
        }

        // Soporte adicional para teclado numérico directo
        when (keyCode) {
            KEY_NUM2 -> { playerY = (playerY - 6).coerceAtLeast(50); score += 5 }
            KEY_NUM8 -> { playerY = (playerY + 6).coerceAtMost(226); score += 5 }
            KEY_NUM4 -> { playerX = (playerX - 6).coerceAtLeast(20); score += 5 }
            KEY_NUM6 -> { playerX = (playerX + 6).coerceAtMost(width - 20); score += 5 }
            KEY_NUM5 -> {
                playerColor = 0xFF5500
                score += 10
            }
        }
        repaint()
    }

    override fun pointerPressed(x: Int, y: Int) {
        if (x in 12..(width - 12) && y in 40..234) {
            playerX = x
            playerY = y
            lastKeyName = "TOUCH"
            score += 10
            repaint()
        }
    }

    override fun pointerDragged(x: Int, y: Int) {
        if (x in 12..(width - 12) && y in 40..234) {
            playerX = x
            playerY = y
            repaint()
        }
    }
}

/**
 * Pantalla de emulación en tiempo real con pantalla LCDUI y teclado virtual J2ME.
 */
@Composable
fun J2meEmulatorScreen(
    game: J2meGame,
    onClose: () -> Unit,
    modifier: Modifier = Modifier
) {
    val nativeWidth = 240
    val nativeHeight = 320

    // Bitmap mutable de Android donde se volcará el Framebuffer de C++
    val screenBitmap = remember {
        Bitmap.createBitmap(nativeWidth, nativeHeight, Bitmap.Config.ARGB_8888)
    }

    // Instancia del canvas demostrativo e interactivo
    val demoCanvas = remember(game.id) {
        InteractiveDemoCanvas(game.title)
    }

    var renderTick by remember { mutableLongStateOf(0L) }
    var fps by remember { mutableIntStateOf(60) }

    // Inicialización del Framebuffer nativo y registro del Canvas
    DisposableEffect(game.id) {
        J2meNativeBridge.graphicsInit(nativeWidth, nativeHeight)
        demoCanvas.render()
        J2meNativeBridge.graphicsRenderToBitmap(screenBitmap)

        onDispose {
            // Limpieza al salir de la pantalla de emulación
        }
    }

    // Bucle de renderizado continuo a ~60 FPS
    LaunchedEffect(game.id) {
        var framesThisSec = 0
        var lastSecTime = System.currentTimeMillis()

        while (isActive) {
            demoCanvas.render()
            J2meNativeBridge.graphicsRenderToBitmap(screenBitmap)
            renderTick++

            framesThisSec++
            val now = System.currentTimeMillis()
            if (now - lastSecTime >= 1000) {
                fps = framesThisSec
                framesThisSec = 0
                lastSecTime = now
            }

            delay(16) // ~60 FPS
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(Color(0xFF0F1117))
            .padding(12.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Barra superior con título y botón de cierre
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column {
                Text(
                    text = game.title,
                    fontWeight = FontWeight.Bold,
                    fontSize = 17.sp,
                    color = Color.White
                )
                Text(
                    text = "LCDUI Framebuffer 240x320 | ${fps} FPS",
                    fontSize = 12.sp,
                    color = Color(0xFF00E5FF),
                    fontFamily = FontFamily.Monospace
                )
            }

            IconButton(
                onClick = onClose,
                modifier = Modifier
                    .size(36.dp)
                    .background(Color(0xFF262C3D), CircleShape)
                    .testTag("close_emulator_button")
            ) {
                Icon(
                    imageVector = Icons.Default.Close,
                    contentDescription = "Cerrar Emulador",
                    tint = Color.White
                )
            }
        }

        Spacer(modifier = Modifier.height(10.dp))

        // Pantalla LCD virtual del teléfono retro (con borde estilizado)
        Surface(
            modifier = Modifier
                .width(260.dp)
                .aspectRatio(240f / 320f)
                .border(2.dp, Color(0xFF00E5FF).copy(alpha = 0.6f), RoundedCornerShape(8.dp))
                .clip(RoundedCornerShape(8.dp)),
            color = Color.Black
        ) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .pointerInput(Unit) {
                        detectTapGestures { offset ->
                            val scaleX = nativeWidth.toFloat() / size.width
                            val scaleY = nativeHeight.toFloat() / size.height
                            val simX = (offset.x * scaleX).toInt()
                            val simY = (offset.y * scaleY).toInt()
                            demoCanvas.dispatchPointerPressed(simX, simY)
                        }
                    }
                    .pointerInput(Unit) {
                        detectDragGestures { change, _ ->
                            change.consume()
                            val scaleX = nativeWidth.toFloat() / size.width
                            val scaleY = nativeHeight.toFloat() / size.height
                            val simX = (change.position.x * scaleX).toInt()
                            val simY = (change.position.y * scaleY).toInt()
                            demoCanvas.dispatchPointerDragged(simX, simY)
                        }
                    }
            ) {
                // La referencia mutable renderTick asegura recomposición reactiva del bitmap
                if (renderTick >= 0) {
                    Image(
                        bitmap = screenBitmap.asImageBitmap(),
                        contentDescription = "Pantalla LCDUI J2ME",
                        modifier = Modifier.fillMaxSize(),
                        filterQuality = FilterQuality.None // Pixel-art nítido retro
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(12.dp))

        // Controles virtuales: D-Pad, Softkeys y Teclado Numérico
        RetroKeypadControls(
            onKeyDown = { code -> demoCanvas.dispatchKeyPressed(code) },
            onKeyUp = { code -> demoCanvas.dispatchKeyReleased(code) }
        )
    }
}

/**
 * Controles de teclado retro estilo teléfono móvil clásico.
 */
@Composable
private fun RetroKeypadControls(
    onKeyDown: (Int) -> Unit,
    onKeyUp: (Int) -> Unit
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        // Teclas de función (SoftKey Left, FIRE, SoftKey Right)
        Row(
            modifier = Modifier.fillMaxWidth(0.85f),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            RetroKeyButton(label = "SOFT 1", onClick = { onKeyDown(Canvas.KEY_SOFTKEY_LEFT) }, width = 64)
            RetroKeyButton(label = "FIRE", onClick = { onKeyDown(Canvas.KEY_SELECT) }, width = 72, isAccent = true)
            RetroKeyButton(label = "SOFT 2", onClick = { onKeyDown(Canvas.KEY_SOFTKEY_RIGHT) }, width = 64)
        }

        // Cruceta Direccional D-Pad
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(2.dp)
        ) {
            RetroIconButton(icon = Icons.Default.KeyboardArrowUp) { onKeyDown(Canvas.KEY_UP) }
            Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                RetroIconButton(icon = Icons.Default.KeyboardArrowLeft) { onKeyDown(Canvas.KEY_LEFT) }
                RetroIconButton(icon = Icons.Default.PlayArrow, isAccent = true) { onKeyDown(Canvas.KEY_SELECT) }
                RetroIconButton(icon = Icons.Default.KeyboardArrowRight) { onKeyDown(Canvas.KEY_RIGHT) }
            }
            RetroIconButton(icon = Icons.Default.KeyboardArrowDown) { onKeyDown(Canvas.KEY_DOWN) }
        }

        // Teclado Numérico (1-9, *, 0, #)
        val numKeys = listOf(
            listOf("1" to Canvas.KEY_NUM1, "2" to Canvas.KEY_NUM2, "3" to Canvas.KEY_NUM3),
            listOf("4" to Canvas.KEY_NUM4, "5" to Canvas.KEY_NUM5, "6" to Canvas.KEY_NUM6),
            listOf("7" to Canvas.KEY_NUM7, "8" to Canvas.KEY_NUM8, "9" to Canvas.KEY_NUM9),
            listOf("*" to Canvas.KEY_STAR, "0" to Canvas.KEY_NUM0, "#" to Canvas.KEY_POUND)
        )

        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(3.dp)
        ) {
            numKeys.forEach { row ->
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    row.forEach { (label, code) ->
                        RetroKeyButton(
                            label = label,
                            onClick = { onKeyDown(code) },
                            width = 54
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun RetroKeyButton(
    label: String,
    onClick: () -> Unit,
    width: Int = 48,
    isAccent: Boolean = false
) {
    Button(
        onClick = onClick,
        modifier = Modifier
            .width(width.dp)
            .height(34.dp),
        shape = RoundedCornerShape(6.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = if (isAccent) Color(0xFF00AA88) else Color(0xFF222838),
            contentColor = Color.White
        ),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(0.dp)
    ) {
        Text(
            text = label,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            fontFamily = FontFamily.Monospace,
            textAlign = TextAlign.Center
        )
    }
}

@Composable
private fun RetroIconButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    isAccent: Boolean = false,
    onClick: () -> Unit
) {
    IconButton(
        onClick = onClick,
        modifier = Modifier
            .size(36.dp)
            .background(
                if (isAccent) Color(0xFF00AA88) else Color(0xFF222838),
                RoundedCornerShape(6.dp)
            )
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier.size(20.dp)
        )
    }
}
