package com.example.ui

import android.app.Application
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.example.data.AppDatabase
import com.example.data.J2meGame
import com.example.data.J2meGameRepository
import com.example.util.J2meJarParser
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.io.File

/**
 * Estado de la interfaz de usuario del menú del emulador J2ME.
 */
data class MenuUiState(
    /** Lista de juegos filtrados por búsqueda */
    val games: List<J2meGame> = emptyList(),
    /** Total de juegos en la biblioteca sin filtrar */
    val totalGamesCount: Int = 0,
    /** Indica si se está importando o procesando un archivo .jar */
    val isLoading: Boolean = false,
    /** Juego actualmente seleccionado para ver detalles */
    val selectedGame: J2meGame? = null,
    /** Juego actualmente en ejecución en la pantalla del emulador */
    val activeRunningGame: J2meGame? = null,
    /** Juego pendiente de confirmación de eliminación */
    val gameToDelete: J2meGame? = null,
    /** Texto de búsqueda actual para filtrar juegos */
    val searchQuery: String = "",
    /** Mensaje de notificación o estado para el usuario */
    val userMessage: String? = null
)

/**
 * ViewModel que administra el estado del menú principal del emulador J2ME.
 * Coordina la importación de archivos .jar, el almacenamiento persistente con Room,
 * y la selección de juegos por parte del usuario.
 */
class J2meMenuViewModel(application: Application) : AndroidViewModel(application) {

    private val repository: J2meGameRepository

    init {
        val database = AppDatabase.getDatabase(application)
        repository = J2meGameRepository(database.j2meGameDao())
    }

    private val _isLoading = MutableStateFlow(false)
    private val _searchQuery = MutableStateFlow("")
    private val _selectedGame = MutableStateFlow<J2meGame?>(null)
    private val _activeRunningGame = MutableStateFlow<J2meGame?>(null)
    private val _gameToDelete = MutableStateFlow<J2meGame?>(null)
    private val _userMessage = MutableStateFlow<String?>(null)

    /**
     * Estado consolidado de la UI observado por la pantalla de Jetpack Compose.
     */
    val uiState: StateFlow<MenuUiState> = combine(
        repository.allGames,
        _searchQuery,
        _isLoading,
        _selectedGame,
        _activeRunningGame
    ) { allGames, query, loading, selected, running ->
        val filteredGames = if (query.isBlank()) {
            allGames
        } else {
            allGames.filter { game ->
                game.title.contains(query, ignoreCase = true) ||
                game.vendor.contains(query, ignoreCase = true) ||
                game.fileName.contains(query, ignoreCase = true)
            }
        }

        MenuUiState(
            games = filteredGames,
            totalGamesCount = allGames.size,
            isLoading = loading,
            selectedGame = selected,
            activeRunningGame = running,
            searchQuery = query,
            userMessage = null
        )
    }.combine(_gameToDelete) { baseState, toDelete ->
        baseState.copy(gameToDelete = toDelete)
    }.combine(_userMessage) { baseState, message ->
        baseState.copy(userMessage = message)
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = MenuUiState(isLoading = true)
    )

    /**
     * Importa y analiza un archivo .jar seleccionado por el usuario desde el almacenamiento.
     */
    fun importJarFile(context: Context, uri: Uri) {
        viewModelScope.launch(Dispatchers.IO) {
            _isLoading.value = true
            try {
                // Solicitar permisos de lectura persistentes para el archivo seleccionado mediante SAF
                try {
                    context.contentResolver.takePersistableUriPermission(
                        uri,
                        Intent.FLAG_GRANT_READ_URI_PERMISSION
                    )
                } catch (e: SecurityException) {
                    // Algunos proveedores no admiten permisos persistentes, se continúa con acceso temporal
                }

                // Analizar el archivo JAR y extraer información del manifiesto
                val newGame = J2meJarParser.parseJarFromUri(context, uri)

                // Guardar en la base de datos Room
                repository.insertGame(newGame)
                _userMessage.value = "¡\"${newGame.title}\" importado con éxito!"
            } catch (e: Exception) {
                _userMessage.value = "No se pudo leer el archivo .jar: ${e.localizedMessage ?: "Error desconocido"}"
            } finally {
                _isLoading.value = false
            }
        }
    }

    /**
     * Selecciona un juego para mostrar su información y preparar la ejecución.
     */
    fun onGameSelected(game: J2meGame) {
        _selectedGame.value = game
    }

    /**
     * Lanza el juego seleccionado en la pantalla interactiva del emulador LCDUI.
     */
    fun launchGame(game: J2meGame) {
        _selectedGame.value = null
        _activeRunningGame.value = game
    }

    /**
     * Cierra la pantalla de emulación activa y regresa a la biblioteca de juegos.
     */
    fun closeRunningGame() {
        _activeRunningGame.value = null
    }

    /**
     * Cierra el diálogo de información del juego seleccionado.
     */
    fun dismissGameDetail() {
        _selectedGame.value = null
    }

    /**
     * Abre el diálogo de confirmación para eliminar un juego.
     */
    fun requestDeleteGame(game: J2meGame) {
        _gameToDelete.value = game
    }

    /**
     * Cancela la eliminación del juego.
     */
    fun dismissDeleteDialog() {
        _gameToDelete.value = null
    }

    /**
     * Confirma y ejecuta la eliminación del juego seleccionado de la base de datos
     * y elimina su archivo de icono en caché.
     */
    fun confirmDeleteGame() {
        val game = _gameToDelete.value ?: return
        viewModelScope.launch(Dispatchers.IO) {
            try {
                // Eliminar icono local si existe
                game.iconPath?.let { path ->
                    val file = File(path)
                    if (file.exists()) {
                        file.delete()
                    }
                }
                repository.deleteGame(game)
                _userMessage.value = "Se eliminó \"${game.title}\""
            } catch (e: Exception) {
                _userMessage.value = "Error al eliminar el juego"
            } finally {
                _gameToDelete.value = null
            }
        }
    }

    /**
     * Actualiza el término de búsqueda para filtrar la biblioteca.
     */
    fun onSearchQueryChanged(query: String) {
        _searchQuery.value = query
    }

    /**
     * Limpia el mensaje de estado una vez mostrado.
     */
    fun clearUserMessage() {
        _userMessage.value = null
    }
}
