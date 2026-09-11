package com.example.data

import kotlinx.coroutines.flow.Flow

/**
 * Repositorio que desacopla la fuente de datos (DAO de Room) de la capa de presentación (ViewModel).
 * Gestiona el acceso seguro a los juegos J2ME almacenados.
 */
class J2meGameRepository(private val dao: J2meGameDao) {

    /**
     * Flujo reactivo con la lista completa de juegos J2ME del usuario.
     */
    val allGames: Flow<List<J2meGame>> = dao.getAllGames()

    /**
     * Inserta un juego nuevo o actualiza uno existente.
     */
    suspend fun insertGame(game: J2meGame): Long {
        return dao.insertGame(game)
    }

    /**
     * Obtiene un juego por su ID.
     */
    suspend fun getGameById(id: Long): J2meGame? {
        return dao.getGameById(id)
    }

    /**
     * Elimina un juego de la base de datos.
     */
    suspend fun deleteGame(game: J2meGame) {
        dao.deleteGame(game)
    }

    /**
     * Elimina un juego por su identificador.
     */
    suspend fun deleteGameById(id: Long) {
        dao.deleteGameById(id)
    }
}
