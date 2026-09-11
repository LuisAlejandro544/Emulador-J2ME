package com.example

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import com.example.ui.J2meMenuScreen
import com.example.ui.J2meMenuViewModel
import com.example.ui.theme.J2meEmulatorTheme

/**
 * Actividad principal del emulador J2ME.
 *
 * Configura la experiencia visual Edge-to-Edge nativa de Android,
 * inicializa el tema retro y monta la pantalla principal del menú de juegos J2ME.
 */
class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Habilita el diseño inmersivo Edge-to-Edge aprovechando toda la pantalla
        enableEdgeToEdge()

        setContent {
            J2meEmulatorTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    val viewModel: J2meMenuViewModel = viewModel()
                    J2meMenuScreen(viewModel = viewModel)
                }
            }
        }
    }
}
