package javax.microedition.lcdui;

import java.util.ArrayList;
import java.util.List;

/**
 * ============================================================================
 * CLASE BASE Displayable (J2ME LCDUI)
 * ============================================================================
 *
 * Representa cualquier objeto que puede colocarse en la pantalla (Display),
 * como `Canvas`, `Form`, `List`, `TextBox`, etc.
 * Maneja el título, comandos asociados y el oyente de comandos.
 */
public abstract class Displayable {

    private String title = null;
    private final List<Command> commands = new ArrayList<>();
    private CommandListener listener = null;
    private int width = 240;
    private int height = 320;
    private boolean isShown = false;

    public String getTitle() {
        return title;
    }

    public void setTitle(String s) {
        this.title = s;
    }

    public boolean isShown() {
        return isShown;
    }

    public void setShown(boolean shown) {
        this.isShown = shown;
    }

    public int getWidth() {
        return width;
    }

    public int getHeight() {
        return height;
    }

    public void setSize(int w, int h) {
        if (w > 0 && h > 0) {
            this.width = w;
            this.height = h;
        }
    }

    public void addCommand(Command cmd) {
        if (cmd != null && !commands.contains(cmd)) {
            commands.add(cmd);
        }
    }

    public void removeCommand(Command cmd) {
        commands.remove(cmd);
    }

    public void setCommandListener(CommandListener l) {
        this.listener = l;
    }

    public CommandListener getCommandListener() {
        return listener;
    }

    public List<Command> getCommands() {
        return new ArrayList<>(commands);
    }

    /**
     * Dispara la acción de un comando registrado hacia el CommandListener.
     */
    public void dispatchCommand(Command cmd) {
        if (listener != null && cmd != null) {
            listener.commandAction(cmd, this);
        }
    }
}
