// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.server.MinecraftServer;

/**
 * Stops a dedicated server cleanly before {@code loadLevel()} (plan §6.6, §7.13), so no half-made
 * world is ever saved. {@link FirstStartGenerator} calls {@link #request} from
 * {@code ServerAboutToStartEvent}; two mixins then take the server down the normal shutdown path
 * without a level:
 *
 * <ol>
 *   <li>{@code DedicatedServerFirstStartMixin}: {@code initServer} returns right after the event,
 *       before "Preparing level" and {@code loadLevel()} (and before RCON, the watchdog and
 *       {@code ServerStartingEvent}).</li>
 *   <li>{@code MinecraftServerFirstStartMixin}: {@code runServer} skips
 *       {@code ServerStartedEvent} and {@code ServerStoppingEvent} (no mod expects them without a
 *       level), and {@code saveAllChunks} saves nothing (vanilla would throw on the missing
 *       overworld, an "Exception stopping the server"). {@code stopServer} still closes the network
 *       listener and releases {@code session.lock}; {@code ServerStoppedEvent} still fires.</li>
 * </ol>
 *
 * What the world folder holds afterwards is what vanilla wrote before any mod event:
 * {@code level.dat} (the world's settings, written by {@code net.minecraft.server.Main} before the
 * server thread starts, so it already records the level-type), {@code session.lock} and NeoForge's
 * {@code serverconfig/}. No region, entity, POI or {@code data/} file, and no world data.
 *
 * <p>The server is halted, so the tick loop never runs. No crash report is written. When the stop is
 * a failure (not a Ctrl-C during generation), a {@code deeptime-stop} thread waits for the server
 * thread to finish and exits the JVM with status 1, so service managers see that the server did
 * not start.
 */
public final class ServerStop {

    /** Exit status of a server that stopped before creating its world. */
    public static final int EXIT_STATUS = 1;

    private static volatile String reason;

    private ServerStop() {}

    /** True once a stop was requested; the mixins read it. */
    public static boolean requested() {
        return reason != null;
    }

    /** The one-line reason, or {@code null}. */
    public static String reason() {
        return reason;
    }

    /**
     * Logs {@code message} (every line prefixed {@code [deeptime]}) and stops the server before it
     * loads a level. Call on the server thread during {@code ServerAboutToStartEvent}.
     *
     * @param headline a one-line summary, repeated as the last log line
     * @param exitWithFailure exit the JVM with {@link #EXIT_STATUS} once the server thread ends
     */
    public static void request(MinecraftServer server, String headline, String message, boolean exitWithFailure) {
        reason = headline;
        StringBuilder b = new StringBuilder("[deeptime] ").append("The server stops before creating the world: ")
                .append(headline);
        for (String line : message.split("\n", -1))
            b.append('\n').append("[deeptime]   ").append(line);
        DeepTime.LOGGER.error(b.toString());
        server.halt(false);
        if (!exitWithFailure)
            return;
        Thread serverThread = server.getRunningThread();
        Thread exit = new Thread(() -> {
            try {
                serverThread.join();
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            DeepTime.LOGGER.error("[deeptime] Server stopped before the world was created: {} (exit status {})",
                    headline, EXIT_STATUS);
            System.exit(EXIT_STATUS);
        }, "deeptime-stop");
        exit.setDaemon(false); // the JVM must wait for it, or it could exit 0 first
        exit.start();
    }
}
