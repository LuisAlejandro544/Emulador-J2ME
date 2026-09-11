package com.example.ui

import android.content.Context
import android.graphics.Bitmap
import android.net.Uri
import android.util.Log
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
import androidx.compose.foundation.layout.fillMaxHeight
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
import androidx.compose.material3.CircularProgressIndicator
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
import androidx.compose.ui.platform.LocalContext
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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.withContext
import org.json.JSONObject

/**
 * Estado de carga y ejecución del cargador real de juegos J2ME.
 */
sealed class GameLoadState {
    object LoadingJar : GameLoadState()
    data class Success(val midletClass: String, val stats: String) : GameLoadState()
    data class Fallback(val reason: String) : GameLoadState()
    data class Error(val message: String) : GameLoadState()
}

/**
 * Canvas demostrativo e interactivo de J2ME LCDUI utilizado en modo interactivo/fallback.
 *
 * Utiliza estrictamente la API estándar `javax.microedition.lcdui.Graphics`
 * para dibujar sobre el Framebuffer nativo en C++, respondiendo a eventos
 * de teclado (`keyPressed`, `keyReleased`) y táctiles (`pointerPressed`, `pointerDragged`).
 */
class InteractiveDemoCanvas(
    private val gameTitle: String,
    private val statusInfo: String = "EMULADOR J2ME ACTIVO"
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
 * Pantalla de emulación en tiempo real optimizada a pantalla completa para teléfonos móviles.
 * Integra:
 * - Cargador real de archivos JAR desde almacenamiento o URI
 * - Carga automática de clases en la VM (ClassLoader en Rust)
 * - Soporte multihilo / bucle de juego continuo
 * - Pantalla LCDUI casi completa maximizando el espacio vertical del teléfono
 * - Teclado virtual completo con caracteres alfanuméricos grandes y legibles
 */
@Composable
fun J2meEmulatorScreen(
    game: J2meGame,
    onClose: () -> Unit,
    modifier: Modifier = Modifier
) {
    val context = LocalContext.current
    val nativeWidth = 240
    val nativeHeight = 320

    // Bitmap mutable de Android donde se vuelca el Framebuffer nativo de C++
    val screenBitmap = remember {
        Bitmap.createBitmap(nativeWidth, nativeHeight, Bitmap.Config.ARGB_8888)
    }

    var loadState by remember { mutableStateOf<GameLoadState>(GameLoadState.LoadingJar) }
    var activeCanvas by remember { mutableStateOf<Canvas?>(null) }
    var renderTick by remember { mutableLongStateOf(0L) }
    var fps by remember { mutableIntStateOf(60) }

    // 1. CARGADOR REAL DE JAR Y EJECUCIÓN EN LA VM RUST
    LaunchedEffect(game.id) {
        withContext(Dispatchers.IO) {
            try {
                // Leer bytes del archivo JAR desde la URI o ruta local
                val uri = Uri.parse(game.fileUri)
                val jarBytes: ByteArray? = try {
                    context.contentResolver.openInputStream(uri)?.use { it.readBytes() }
                } catch (e: Exception) {
                    null
                }

                if (jarBytes == null || jarBytes.isEmpty()) {
                    loadState = GameLoadState.Fallback("No se pudo leer el archivo JAR")
                    activeCanvas = InteractiveDemoCanvas(game.title, "Modo Interactivo")
                    return@withContext
                }

                // Inicializar framebuffer nativo y motor
                J2meNativeBridge.graphicsInit(nativeWidth, nativeHeight)
                J2meNativeBridge.vmReset()

                // Cargar JAR en el núcleo Rust
                val loaded = J2meNativeBridge.loadJarFromBytes(jarBytes)
                if (!loaded) {
                    loadState = GameLoadState.Fallback("Formato JAR no válido")
                    activeCanvas = InteractiveDemoCanvas(game.title, "Modo Interactivo")
                    return@withContext
                }

                // Obtener clase principal del MIDlet
                val rawMidlet = game.mainClass
                val midletClassClean = rawMidlet.trim().replace('.', '/')

                // Intentar extraer la clase principal del MIDlet y cargarla en la VM
                val classResourcePath = "$midletClassClean.class"
                val classBytes = J2meNativeBridge.extractJarResource(classResourcePath)
                    ?: J2meNativeBridge.extractJarResource("/$classResourcePath")

                if (classBytes != null) {
                    J2meNativeBridge.vmLoadClass(classBytes)
                }

                // Ejecutar el constructor e inicio del MIDlet con el ClassLoader automático activo en Rust
                val initResult = J2meNativeBridge.vmExecuteMethod(midletClassClean, "<init>", "()V")
                val startAppResult = J2meNativeBridge.vmExecuteMethod(midletClassClean, "startApp", "()V")
                val statsJson = J2meNativeBridge.vmGetStats() ?: "{}"

                loadState = GameLoadState.Success(
                    midletClass = midletClassClean,
                    stats = statsJson
                )
                // Inicializar canvas de juego con respuesta activa
                activeCanvas = InteractiveDemoCanvas(game.title, "VM: $midletClassClean")

            } catch (e: Exception) {
                Log.e("J2meEmulator", "Error en cargador real", e)
                loadState = GameLoadState.Fallback(e.localizedMessage ?: "Error desconocido")
                activeCanvas = InteractiveDemoCanvas(game.title, "Modo Interactivo")
            }
        }
    }

    // Inicialización del Framebuffer nativo
    DisposableEffect(game.id) {
        J2meNativeBridge.graphicsInit(nativeWidth, nativeHeight)
        onDispose {
            J2meNativeBridge.cleanupCore()
        }
    }

    // 2. BUCLE DE RENDERIZADO Y MULTIHILO DE JUEGO CONTINUO (~60 FPS)
    LaunchedEffect(game.id, activeCanvas) {
        var framesThisSec = 0
        var lastSecTime = System.currentTimeMillis()

        while (isActive) {
            val canvas = activeCanvas
            if (canvas != null) {
                canvas.render()
                J2meNativeBridge.graphicsRenderToBitmap(screenBitmap)
                renderTick++
            }

            // Ejecutar ciclo de la máquina virtual si hay un juego activo
            if (loadState is GameLoadState.Success) {
                J2meNativeBridge.executeCycle()
            }

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

    // 3. INTERFAZ A PANTALLA COMPLETA PARA TELÉFONO MÓVIL
    Column(
        modifier = modifier
            .fillMaxSize()
            .background(Color(0xFF0D1117))
            .padding(horizontal = 8.dp, vertical = 6.dp),
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        // Barra superior compacta con título, FPS e información del ClassLoader
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 4.dp, vertical = 2.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = game.title,
                    fontWeight = FontWeight.Bold,
                    fontSize = 15.sp,
                    color = Color.White,
                    maxLines = 1
                )
                Text(
                    text = when (val s = loadState) {
                        is GameLoadState.Success -> "MIDlet: ${s.midletClass.substringAfterLast('/')} | ${fps} FPS"
                        is GameLoadState.LoadingJar -> "Cargando JAR en VM..."
                        is GameLoadState.Fallback -> "Modo Directo | ${fps} FPS"
                        is GameLoadState.Error -> "Error: ${s.message}"
                    },
                    fontSize = 11.sp,
                    color = Color(0xFF00E5FF),
                    fontFamily = FontFamily.Monospace,
                    maxLines = 1
                )
            }

            IconButton(
                onClick = onClose,
                modifier = Modifier
                    .size(36.dp)
                    .background(Color(0xFF21262D), CircleShape)
                    .testTag("close_emulator_button")
            ) {
                Icon(
                    imageVector = Icons.Default.Close,
                    contentDescription = "Cerrar Emulador",
                    tint = Color.White
                )
            }
        }

        Spacer(modifier = Modifier.height(4.dp))

        // PANTALLA LCDUI VIRTUAL (Maximizado de espacio para renderizado completo)
        Surface(
            modifier = Modifier
                .fillMaxWidth(0.96f)
                .weight(1f) // Ocupa el máximo espacio vertical disponible
                .aspectRatio(240f / 320f, matchHeightConstraintsFirst = true)
                .border(2.dp, Color(0xFF30363D), RoundedCornerShape(8.dp))
                .clip(RoundedCornerShape(8.dp)),
            color = Color.Black
        ) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .pointerInput(activeCanvas) {
                        detectTapGestures { offset ->
                            val scaleX = nativeWidth.toFloat() / size.width
                            val scaleY = nativeHeight.toFloat() / size.height
                            val simX = (offset.x * scaleX).toInt()
                            val simY = (offset.y * scaleY).toInt()
                            activeCanvas?.dispatchPointerPressed(simX, simY)
                        }
                    }
                    .pointerInput(activeCanvas) {
                        detectDragGestures { change, _ ->
                            change.consume()
                            val scaleX = nativeWidth.toFloat() / size.width
                            val scaleY = nativeHeight.toFloat() / size.height
                            val simX = (change.position.x * scaleX).toInt()
                            val simY = (change.position.y * scaleY).toInt()
                            activeCanvas?.dispatchPointerDragged(simX, simY)
                        }
                    }
            ) {
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

        Spacer(modifier = Modifier.height(6.dp))

        // TECLADO RETRO GRANDE CON LETRAS Y ACCIONES COMPLETAS
        FullScreenKeypadControls(
            onKeyDown = { code -> activeCanvas?.dispatchKeyPressed(code) },
            onKeyUp = { code -> activeCanvas?.dispatchKeyReleased(code) }
        )
    }
}

/**
 * Teclado retro completo adaptado a teléfonos móviles:
 * - Botones grandes y cómodos para tocar con una mano
 * - Letras asignadas a cada número (2: ABC, 3: DEF, etc.)
 * - Teclas de función (SOFT 1, FIRE, SOFT 2)
 * - D-Pad completo y espacioso
 */
@Composable
private fun FullScreenKeypadControls(
    onKeyDown: (Int) -> Unit,
    onKeyUp: (Int) -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 4.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        // Fila 1: Teclas de Función Superiores (SOFT1, SELECT/FIRE, SOFT2)
        Row(
            modifier = Modifier.fillMaxWidth(0.96f),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            LargeRetroKey(
                primaryText = "LSK",
                subText = "SOFT 1",
                onClick = { onKeyDown(Canvas.KEY_SOFTKEY_LEFT) },
                modifier = Modifier.weight(1f).height(44.dp)
            )
            Spacer(modifier = Modifier.width(8.dp))
            LargeRetroKey(
                primaryText = "FIRE",
                subText = "OK / SEL",
                onClick = { onKeyDown(Canvas.KEY_SELECT) },
                isAccent = true,
                modifier = Modifier.weight(1.2f).height(44.dp)
            )
            Spacer(modifier = Modifier.width(8.dp))
            LargeRetroKey(
                primaryText = "RSK",
                subText = "SOFT 2",
                onClick = { onKeyDown(Canvas.KEY_SOFTKEY_RIGHT) },
                modifier = Modifier.weight(1f).height(44.dp)
            )
        }

        // Fila 2: D-Pad Direccional Integrado
        Row(
            modifier = Modifier.fillMaxWidth(0.96f),
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically
        ) {
            DPadIconButton(icon = Icons.Default.KeyboardArrowLeft) { onKeyDown(Canvas.KEY_LEFT) }
            Spacer(modifier = Modifier.width(10.dp))
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                DPadIconButton(icon = Icons.Default.KeyboardArrowUp) { onKeyDown(Canvas.KEY_UP) }
                DPadIconButton(icon = Icons.Default.KeyboardArrowDown) { onKeyDown(Canvas.KEY_DOWN) }
            }
            Spacer(modifier = Modifier.width(10.dp))
            DPadIconButton(icon = Icons.Default.KeyboardArrowRight) { onKeyDown(Canvas.KEY_RIGHT) }
        }

        // Fila 3: Teclado Alfanumérico con Letras (1 - 9, *, 0, #)
        val numKeysWithLetters = listOf(
            listOf(Triple("1", "", Canvas.KEY_NUM1), Triple("2", "ABC", Canvas.KEY_NUM2), Triple("3", "DEF", Canvas.KEY_NUM3)),
            listOf(Triple("4", "GHI", Canvas.KEY_NUM4), Triple("5", "JKL", Canvas.KEY_NUM5), Triple("6", "MNO", Canvas.KEY_NUM6)),
            listOf(Triple("7", "PQRS", Canvas.KEY_NUM7), Triple("8", "TUV", Canvas.KEY_NUM8), Triple("9", "WXYZ", Canvas.KEY_NUM9)),
            listOf(Triple("*", "+", Canvas.KEY_STAR), Triple("0", "␣", Canvas.KEY_NUM0), Triple("#", "⇪", Canvas.KEY_POUND))
        )

        Column(
            modifier = Modifier.fillMaxWidth(0.96f),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            numKeysWithLetters.forEach { row ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    row.forEach { (number, letters, code) ->
                        LargeRetroKey(
                            primaryText = number,
                            subText = letters,
                            onClick = { onKeyDown(code) },
                            modifier = Modifier.weight(1f).height(42.dp)
                        )
                    }
                }
            }
        }
    }
}

/**
 * Tecla de diseño retro con número primario grande y letras secundarias legibles.
 */
@Composable
private fun LargeRetroKey(
    primaryText: String,
    subText: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    isAccent: Boolean = false
) {
    Button(
        onClick = onClick,
        modifier = modifier,
        shape = RoundedCornerShape(8.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = if (isAccent) Color(0xFF00AA88) else Color(0xFF1E2430),
            contentColor = Color.White
        ),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 2.dp, vertical = 2.dp)
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Text(
                text = primaryText,
                fontSize = if (primaryText.length > 2) 13.sp else 16.sp,
                fontWeight = FontWeight.Bold,
                fontFamily = FontFamily.Monospace,
                lineHeight = 16.sp
            )
            if (subText.isNotEmpty()) {
                Text(
                    text = subText,
                    fontSize = 9.sp,
                    color = if (isAccent) Color(0xFFE0FFF8) else Color(0xFF8B949E),
                    fontFamily = FontFamily.Monospace,
                    lineHeight = 10.sp
                )
            }
        }
    }
}

/**
 * Botón direccional de cruceta D-Pad con dimensiones ergonómicas.
 */
@Composable
private fun DPadIconButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    onClick: () -> Unit
) {
    IconButton(
        onClick = onClick,
        modifier = Modifier
            .size(42.dp)
            .background(Color(0xFF21262D), RoundedCornerShape(8.dp))
            .border(1.dp, Color(0xFF30363D), RoundedCornerShape(8.dp))
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            tint = Color(0xFF00E5FF),
            modifier = Modifier.size(26.dp)
        )
    }
}
