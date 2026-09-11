package javax.microedition.midlet;

/**
 * Excepción estándar lanzada cuando un MIDlet no puede cambiar su estado de ciclo de vida.
 */
public class MIDletStateChangeException extends Exception {
    public MIDletStateChangeException() {
        super();
    }

    public MIDletStateChangeException(String s) {
        super(s);
    }
}
