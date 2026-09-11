package com.example.data

import androidx.room.Dao
import androidx.room.Delete
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import kotlinx.coroutines.flow.Flow

/**
 * Objeto de Acceso a Datos (DAO) para la gestión de juegos J2ME en la base de datos Room.
 * Proporciona métodos reactivos mediante Kotlin Coroutines y Flow para observar la lista
 * de juegos en tiempo real cuando el usuario agregue o elimine títulos.
 */
@Dao
interface J2meGameDao {

    /**
     * Retorna el flujo continuo con todos los juegos guardados en la biblioteca,
     * ordenados del más reciente al más antiguo.
     */
    @Query("SELECT * FROM j2me_games ORDER BY addedTimestamp DESC")
    fun getAllGames(): Flow<List<J2meGame>>

    /**
     * Busca un juego por su identificador único.
     */
    @Query("SELECT * FROM j2me_games WHERE id = :id LIMIT 1")
    suspend fun getGameById(id: Long): J2meGame?

    /**
     * Inserta un nuevo juego en la biblioteca. Si ya existe un registro con el mismo ID,
     * se reemplaza.
     */
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertGame(game: J2meGame): Long

    /**
     * Actualiza los datos de un juego existente.
     */
    @Update
    suspend fun updateGame(game: J2meGame)

    /**
     * Elimina un juego de la biblioteca.
     */
    @Delete
    suspend fun deleteGame(game: J2meGame)

    /**
     * Elimina un juego por su identificador numérico.
     */
    @Query("DELETE FROM j2me_games WHERE id = :id")
    suspend fun deleteGameById(id: Long)
}
