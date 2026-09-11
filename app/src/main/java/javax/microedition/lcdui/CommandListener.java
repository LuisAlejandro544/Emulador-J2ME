package javax.microedition.lcdui;

/**
 * Interfaz para recibir eventos de activación de comandos de usuario (botones de menú y softkeys).
 */
public interface CommandListener {
    void commandAction(Command c, Displayable d);
}
