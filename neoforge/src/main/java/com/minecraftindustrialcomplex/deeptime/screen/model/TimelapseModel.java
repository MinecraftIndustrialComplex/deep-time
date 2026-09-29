// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.nativebridge.CancelledException;
import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.server.Staging;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;

import java.io.IOException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.CompletionException;
import java.util.concurrent.Executor;

/**
 * The run page's state (plan §7.12 page 2, §6.2): one run's keyframes, events and progress, the
 * scrubber, the event cursor, the layer choice and a bounded ring of decoded frames. The render
 * thread calls {@link #poll()} once per tick; decoding happens on {@code loader}, never on the
 * render thread (plan §6.2: the render thread does no file I/O).
 *
 * <p><b>Scrubbing.</b> While the run goes the view follows the newest frame ("live") until the
 * player scrubs back; scrubbing to the last frame follows again. After the run the player can
 * scrub freely and play the frames back.
 *
 * <p><b>Event cursor.</b> Events are shown up to the displayed frame's time (all of them while
 * following a live run); an event is "fresh" when it happened after the previous frame.
 */
public final class TimelapseModel {

    public enum State { RUNNING, DONE, CANCELLED, FAILED }

    /** Milliseconds per frame when playing back. */
    public static final long PLAY_FRAME_MS = 400;

    /** Bytes of decoded frames kept (1024 × 512 frames are 2 MiB per layer). */
    public static final long RING_BYTES = 64L << 20;

    /** A decoded frame layer. */
    public record Frame(KeyframeInfo info, int layer, int[] abgr) {}

    private record Key(int n, int layer) {}

    private final RunHandle run;
    private final String stagingId;
    private final String presetJson;
    private final int circumference;
    private final List<String> stageNames;
    private final Executor loader;
    private final Executor io;

    private final List<KeyframeInfo> frames = new ArrayList<>();
    private final List<GeoEvent> events = new ArrayList<>();
    private final List<String> layers = new ArrayList<>();
    private String layer;
    private int index = -1;
    private boolean follow = true;
    private boolean playing;
    private long playAccumulator;
    private SimSession.Progress progress = SimSession.Progress.INITIAL;
    private State state = State.RUNNING;
    private String failure;
    private String manifestJson;
    private final java.util.concurrent.atomic.AtomicBoolean cleanedUp = new java.util.concurrent.atomic.AtomicBoolean();

    // The ring: guarded by `ring`.
    private final Map<Key, int[]> ring = new LinkedHashMap<>(16, 0.75f, true);
    private final Set<Key> pending = new HashSet<>();
    private long ringBytes;
    private Frame shown;
    private long version;

    /**
     * @param stageNames the preset's stage names in order (from its estimate), for the progress line
     * @param loader     decodes frames (a background thread in the game, direct in tests)
     * @param io         deletes the staging folder of a cancelled or failed run
     */
    public TimelapseModel(RunHandle run, String stagingId, String presetJson, int circumference, List<String> stageNames,
            Executor loader, Executor io) {
        this.run = run;
        this.stagingId = stagingId;
        this.presetJson = presetJson;
        this.circumference = circumference;
        this.stageNames = List.copyOf(stageNames);
        this.loader = loader;
        this.io = io;
        // A cancelled or failed run leaves nothing worth keeping: delete its folder once it stopped,
        // whether or not anyone still polls this model.
        run.completion().whenComplete((manifest, error) -> {
            if (error != null)
                deleteFolder();
        });
    }

    public String stagingId() {
        return stagingId;
    }

    public String presetJson() {
        return presetJson;
    }

    public int circumference() {
        return circumference;
    }

    public RunHandle run() {
        return run;
    }

    public State state() {
        return state;
    }

    public Optional<String> failure() {
        return Optional.ofNullable(failure);
    }

    public Optional<String> manifestJson() {
        return Optional.ofNullable(manifestJson);
    }

    public SimSession.Progress progress() {
        return progress;
    }

    // ------------------------------------------------------------------------------------------
    // Polling (render thread, once per tick)
    // ------------------------------------------------------------------------------------------

    /** Takes new progress, frames and events, and notices the end of the run. */
    public void poll() {
        progress = run.progress();
        List<KeyframeInfo> fresh = run.newKeyframes();
        for (KeyframeInfo f : fresh) {
            frames.add(f);
            for (String l : f.layers())
                if (!layers.contains(l))
                    layers.add(l);
        }
        if (layer == null && !layers.isEmpty())
            layer = layers.get(0);
        List<GeoEvent> ev = run.newEvents();
        if (!ev.isEmpty()) {
            events.addAll(ev);
            events.sort(Comparator.comparingDouble(GeoEvent::tStart).thenComparingLong(GeoEvent::id));
        }
        if (follow && !frames.isEmpty())
            index = frames.size() - 1;
        if (state == State.RUNNING && run.completion().isDone()) {
            // Frames and events the worker collected before it finished arrive with the next poll too.
            try {
                manifestJson = run.completion().join();
                state = State.DONE;
                progress = run.progress();
            } catch (CompletionException | java.util.concurrent.CancellationException e) {
                Throwable cause = e.getCause() == null ? e : e.getCause();
                if (cause instanceof CancelledException || e instanceof java.util.concurrent.CancellationException) {
                    state = State.CANCELLED;
                } else {
                    state = State.FAILED;
                    failure = cause.getClass().getSimpleName() + ": " + cause.getMessage();
                }
                follow = false;
            }
        }
        requestDisplayed();
    }

    /** Cancels the run; its staging folder is deleted once it stopped. */
    public void cancel() {
        run.cancel();
    }

    private void deleteFolder() {
        if (!cleanedUp.compareAndSet(false, true))
            return;
        io.execute(() -> {
            try {
                Staging.deleteTree(run.outDir());
            } catch (IOException ignored) {
                // left for Staging.cleanupAbandoned
            }
        });
    }

    // ------------------------------------------------------------------------------------------
    // Frames, scrubbing, playback
    // ------------------------------------------------------------------------------------------

    public List<KeyframeInfo> frames() {
        return Collections.unmodifiableList(frames);
    }

    public int frameCount() {
        return frames.size();
    }

    /** The displayed frame's index, −1 before the first frame. */
    public int index() {
        return index;
    }

    public Optional<KeyframeInfo> current() {
        return index < 0 ? Optional.empty() : Optional.of(frames.get(index));
    }

    /** True while the view follows the newest frame of a running run. */
    public boolean following() {
        return follow && state == State.RUNNING;
    }

    /** Shows frame {@code i} (clamped); the last frame of a running run follows it again. */
    public void scrubTo(int i) {
        if (frames.isEmpty())
            return;
        index = Math.max(0, Math.min(frames.size() - 1, i));
        follow = state == State.RUNNING && index == frames.size() - 1;
        requestDisplayed();
    }

    public void step(int delta) {
        playing = false;
        scrubTo(index + delta);
    }

    public boolean playing() {
        return playing;
    }

    /** Plays the frames back from the current one (from the first when at the end). */
    public void togglePlay() {
        if (playing) {
            playing = false;
            return;
        }
        if (frames.size() < 2)
            return;
        if (index >= frames.size() - 1)
            scrubTo(0);
        follow = false;
        playing = true;
        playAccumulator = 0;
    }

    /** Advances playback by {@code ms} of wall time. */
    public void tick(long ms) {
        if (!playing)
            return;
        playAccumulator += ms;
        while (playAccumulator >= PLAY_FRAME_MS && playing) {
            playAccumulator -= PLAY_FRAME_MS;
            if (index >= frames.size() - 1) {
                playing = false;
                follow = state == State.RUNNING;
            } else {
                index++;
            }
        }
        requestDisplayed();
        if (playing && index + 1 < frames.size())
            request(new Key(index + 1, layerIndex(frames.get(index + 1))));
    }

    // ------------------------------------------------------------------------------------------
    // Layers
    // ------------------------------------------------------------------------------------------

    /** Every layer name seen in a frame, in first-seen order. */
    public List<String> layers() {
        return Collections.unmodifiableList(layers);
    }

    public Optional<String> layer() {
        return Optional.ofNullable(layer);
    }

    public void setLayer(String name) {
        if (layers.contains(name)) {
            layer = name;
            requestDisplayed();
        }
    }

    /** The layer shown for frame {@code f}: the chosen one, or the frame's first when it lacks it. */
    int layerIndex(KeyframeInfo f) {
        int i = layer == null ? -1 : f.layerIndex(layer);
        return Math.max(0, i);
    }

    // ------------------------------------------------------------------------------------------
    // Pixels: the ring of decoded frames
    // ------------------------------------------------------------------------------------------

    private void requestDisplayed() {
        if (index >= 0)
            request(new Key(index, layerIndex(frames.get(index))));
    }

    private void request(Key k) {
        KeyframeInfo info = frames.get(k.n());
        synchronized (ring) {
            if (ring.containsKey(k) || !pending.add(k))
                return;
        }
        loader.execute(() -> {
            int[] px = null;
            try {
                px = run.pixels(info, k.layer());
            } catch (RuntimeException e) {
                // unreadable frame: leave the previous one on screen
            }
            synchronized (ring) {
                pending.remove(k);
                if (px != null) {
                    ring.put(k, px);
                    ringBytes += px.length * 4L;
                    var it = ring.entrySet().iterator();
                    while (ringBytes > RING_BYTES && ring.size() > 1 && it.hasNext()) {
                        var e = it.next();
                        ringBytes -= e.getValue().length * 4L;
                        it.remove();
                    }
                }
            }
        });
    }

    /**
     * The frame to draw: the displayed frame once decoded, else the one drawn before (so the view
     * never flashes empty while a frame loads); empty before the first frame is decoded.
     */
    public Optional<Frame> displayed() {
        if (index >= 0) {
            KeyframeInfo f = frames.get(index);
            Key k = new Key(index, layerIndex(f));
            int[] px;
            synchronized (ring) {
                px = ring.get(k);
            }
            if (px != null && (shown == null || shown.abgr() != px)) {
                shown = new Frame(f, k.layer(), px);
                version++;
            }
        }
        return Optional.ofNullable(shown);
    }

    /** Changes whenever {@link #displayed()} returns another frame (the texture re-uploads then). */
    public long displayVersion() {
        displayed();
        return version;
    }

    /** Frames decoded and held in the ring. */
    public int ringSize() {
        synchronized (ring) {
            return ring.size();
        }
    }

    // ------------------------------------------------------------------------------------------
    // Events
    // ------------------------------------------------------------------------------------------

    public List<GeoEvent> events() {
        return Collections.unmodifiableList(events);
    }

    /** The time events are shown up to: the displayed frame's, or everything while following live. */
    public double cursorTime() {
        if (index < 0 || following())
            return Double.POSITIVE_INFINITY;
        return frames.get(index).t();
    }

    /** The previous frame's time: events after it are fresh. */
    double previousTime() {
        if (following()) {
            return frames.size() >= 2 ? frames.get(frames.size() - 2).t()
                    : frames.isEmpty() ? Double.NEGATIVE_INFINITY : frames.get(0).t();
        }
        return index <= 0 ? Double.NEGATIVE_INFINITY : frames.get(index - 1).t();
    }

    /** At most {@code max} events up to the cursor, newest first (the ticker). */
    public List<GeoEvent> visibleEvents(int max) {
        double t = cursorTime();
        List<GeoEvent> out = new ArrayList<>();
        for (int i = events.size() - 1; i >= 0 && out.size() < max; i--) {
            GeoEvent e = events.get(i);
            if (e.tStart() <= t)
                out.add(e);
        }
        return out;
    }

    /** True when {@code e} happened after the previous frame (drawn larger, highlighted in the ticker). */
    public boolean fresh(GeoEvent e) {
        return e.tStart() > previousTime();
    }

    /** Point events up to the cursor (the map's markers). */
    public List<GeoEvent> markers() {
        double t = cursorTime();
        List<GeoEvent> out = new ArrayList<>();
        for (GeoEvent e : events)
            if (e.hasPoint() && e.tStart() <= t)
                out.add(e);
        return out;
    }

    // ------------------------------------------------------------------------------------------
    // Progress text
    // ------------------------------------------------------------------------------------------

    /** The current stage's name. */
    public String stageName() {
        int i = progress.stageIndex();
        if (i >= 0 && i < stageNames.size())
            return stageNames.get(i);
        return "stage " + (i + 1) + "/" + Math.max(progress.stageCount(), i + 1);
    }

    /**
     * The progress line: {@code dummy_drift 37% (−312 Ma) · total 18% · 12s elapsed · ~40s left ·
     * native 1.3 MiB of 4.0 GiB} (the server's line, P1-13, with the cap given).
     */
    public String progressLine(long capBytes) {
        SimSession.Progress p = progress;
        StringBuilder b = new StringBuilder();
        switch (state) {
            case RUNNING -> {
                if (p.stageCount() == 0) {
                    b.append("starting");
                } else if (p.state() == SimSession.Progress.RUNNING) {
                    b.append(stageName()).append(' ').append(Formats.percent(p.stagePermille()))
                            .append(" (").append(Formats.ma(p.simulatedTime())).append(')')
                            .append(" · total ").append(Formats.percent(p.totalPermille()));
                } else {
                    b.append("writing world data");
                }
            }
            case DONE -> b.append("done · ").append(frames.size()).append(" keyframes · ").append(events.size()).append(" events");
            case CANCELLED -> b.append("cancelled");
            case FAILED -> b.append("failed: ").append(failure);
        }
        b.append(" · ").append(Formats.duration(p.elapsedMs())).append(" elapsed");
        if (state == State.RUNNING && p.stageCount() > 0)
            b.append(" · ").append(p.etaMs() < 0 ? "time left unknown" : "~" + Formats.duration(p.etaMs()) + " left");
        b.append(" · native ").append(Formats.bytes(state == State.RUNNING ? p.nativeBytesInUse() : p.nativeBytesPeak()))
                .append(" of ").append(Formats.gib(capBytes)).append(" GiB");
        return b.toString();
    }

    /** Total progress 0…1 (1 once done). */
    public double fraction() {
        if (state == State.DONE)
            return 1;
        return Math.max(0, Math.min(1, progress.totalPermille() / 1000.0));
    }
}
