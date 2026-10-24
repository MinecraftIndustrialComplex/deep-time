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
 * <p><b>The live map.</b> While the view follows a running run, the map shows the session's live
 * frame of the chosen layer whenever it is newer than the newest keyframe ({@code
 * deeptime_core::live}: drawn between the keyframes a few times a minute, at most 2 % of the run's
 * time). It is not a frame of the scrubber: {@link #index()}, {@link #current()}, the stop and the
 * epoch stay on the keyframes; {@link #mapTime()} is the time of the picture on the map.
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

    /**
     * How long a point event stays on the map after it ended: the workbench's {@code FADE_YEARS}
     * (24 Myr). A real 1,000-Myr planet logs some 10,000 events; without a window every one of them
     * would be a marker on every frame (game-choice: a display window, not a physical value).
     */
    public static final double MARKER_FADE_YEARS = 24e6;

    /** At most this many markers on the map (the newest by start time). */
    public static final int MAX_MARKERS = 400;

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
    /** True once the player picked a layer (until then the default follows the layers that arrive). */
    private boolean layerChosen;
    /** The group the layer list shows (null: the current layer's). */
    private String group;
    private int index = -1;
    private boolean follow = true;
    private boolean playing;
    private volatile boolean cancelRequested;
    private volatile boolean superseded;
    private volatile Estimate estimate;
    /** Wall milliseconds (tests set their own). */
    private java.util.function.LongSupplier clock = System::currentTimeMillis;
    /** When the stages were seen done and the bake began, by {@link #clock}; -1 before. */
    private long bakeStartedMs = -1;
    private long playAccumulator;
    private SimSession.Progress progress = SimSession.Progress.INITIAL;
    private State state = State.RUNNING;
    private String failure;
    private String manifestJson;
    /** A cube world's edges and corners, read from the finished run's world data (null before, or not a cube). */
    private volatile CubeOutline cubeOutline;
    private final java.util.concurrent.atomic.AtomicBoolean cleanedUp = new java.util.concurrent.atomic.AtomicBoolean();

    // The ring: guarded by `ring`.
    private final Map<Key, int[]> ring = new LinkedHashMap<>(16, 0.75f, true);
    private final Set<Key> pending = new HashSet<>();
    private long ringBytes;
    private Frame shown;
    private long version;
    /** The newest live frame decoded, and whether one is being fetched: guarded by `ring`. */
    private RunHandle.LiveFrame live;
    private boolean liveLoading;
    /** True while this model asks the run for live frames (render thread). */
    private boolean watching;

    /**
     * @param stageNames the preset's stage names in order (from its estimate), for the progress line
     * @param loader     decodes frames (a background thread in the game, direct in tests)
     * @param io         deletes the staging folder of a cancelled or failed run, and reads a finished cube world's outline
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
        // whether or not anyone still polls this model (unless a run picking it up took it over).
        run.completion().whenComplete((manifest, error) -> {
            if (error != null && !superseded)
                deleteFolder();
        });
    }

    /**
     * Another run picks this one up in the same folder (an epoch picked, a stop, a continue;
     * docs/design/playability-steering.md): when this one is cancelled its folder stays.
     */
    public void supersede() {
        superseded = true;
    }

    public boolean superseded() {
        return superseded;
    }

    /** The stage names the progress line names. */
    public List<String> stageNames() {
        return stageNames;
    }

    /** The estimate the run was started with, if any. */
    public Optional<Estimate> estimate() {
        return Optional.ofNullable(estimate);
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

    /** A finished cube world's edges and corners ({@link CubeOutline}, cube plan G13); empty before, and on the polar worlds. */
    public Optional<CubeOutline> cubeOutline() {
        return Optional.ofNullable(cubeOutline);
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
        if (bakeStartedMs < 0 && baking())
            bakeStartedMs = clock.getAsLong();
        List<KeyframeInfo> fresh = run.newKeyframes();
        for (KeyframeInfo f : fresh) {
            frames.add(f);
            for (String l : f.layers())
                if (!layers.contains(l))
                    layers.add(l);
        }
        if (!layerChosen && !layers.isEmpty())
            layer = LayerCatalog.defaultLayer(layers);
        List<GeoEvent> ev = run.newEvents();
        if (!ev.isEmpty()) {
            events.addAll(ev);
            events.sort(Comparator.comparingDouble(GeoEvent::tStart).thenComparingLong(GeoEvent::id));
        }
        if (follow && !frames.isEmpty())
            index = frames.size() - 1;
        pollLive();
        if (state == State.RUNNING && run.completion().isDone()) {
            // Frames and events the worker collected before it finished arrive with the next poll too.
            try {
                manifestJson = run.completion().join();
                state = State.DONE;
                progress = run.progress();
                // The cube's turn is chosen when the world data is baked: its edges are known now.
                io.execute(() -> cubeOutline = CubeOutline.read(run.outDir()).orElse(null));
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

    /**
     * Renews the request for live frames of the shown layer while the view follows the run (and
     * drops it once it does not), and fetches a newer live frame on {@code loader}.
     */
    private void pollLive() {
        boolean want = following() && layer != null && !frames.isEmpty();
        if (!want) {
            if (watching) {
                watching = false;
                run.watchLive(null);
            }
            return;
        }
        watching = true;
        long seq = run.watchLive(layer);
        if (seq <= 0)
            return;
        synchronized (ring) {
            if (liveLoading || (live != null && live.seq() >= seq))
                return;
            liveLoading = true;
        }
        KeyframeInfo like = frames.get(frames.size() - 1);
        loader.execute(() -> {
            RunHandle.LiveFrame f = null;
            try {
                f = run.liveFrame(like);
            } catch (RuntimeException e) {
                // no live frame this time: the keyframes stay on screen
            }
            synchronized (ring) {
                liveLoading = false;
                if (f != null && (live == null || f.seq() > live.seq()))
                    live = f;
            }
        });
    }

    /**
     * The live frame the map shows now, or null: while following a running run, the newest live
     * frame of the chosen layer when it is newer than the newest keyframe.
     */
    private RunHandle.LiveFrame liveShown() {
        if (!following() || layer == null || index < 0 || index != frames.size() - 1)
            return null;
        RunHandle.LiveFrame f;
        synchronized (ring) {
            f = live;
        }
        return f != null && f.layer().equals(layer) && f.info().t() > frames.get(index).t() ? f : null;
    }

    /** True while the map shows the live frame (the run is past the newest keyframe). */
    public boolean showingLive() {
        return liveShown() != null;
    }

    /**
     * The time of the picture on the map: the live frame's while it shows, else the displayed
     * keyframe's; empty before the first frame.
     */
    public java.util.OptionalDouble mapTime() {
        RunHandle.LiveFrame f = liveShown();
        if (f != null)
            return java.util.OptionalDouble.of(f.info().t());
        return index < 0 ? java.util.OptionalDouble.empty() : java.util.OptionalDouble.of(frames.get(index).t());
    }

    /** Cancels the run; its staging folder is deleted once it stopped. */
    public void cancel() {
        cancelRequested = true;
        run.cancel();
    }

    /** True once {@link #cancel} was called (the run may still be finishing its step). */
    public boolean cancelRequested() {
        return cancelRequested;
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

    /**
     * Shows layer {@code name}. When the displayed frame lacks it (the deposits, young and amplify
     * layers exist only in their stage's frame at the end), the view moves to the nearest frame
     * that has it.
     */
    public void setLayer(String name) {
        if (!layers.contains(name))
            return;
        layer = name;
        layerChosen = true;
        group = LayerCatalog.group(name);
        if (index >= 0 && frames.get(index).layerIndex(name) < 0) {
            int best = -1;
            for (int i = 0; i < frames.size(); i++)
                if (frames.get(i).layerIndex(name) >= 0 && (best < 0 || Math.abs(i - index) < Math.abs(best - index)))
                    best = i;
            if (best >= 0) {
                playing = false;
                scrubTo(best);
                return;
            }
        }
        requestDisplayed();
    }

    /** True when the displayed frame carries layer {@code name} (the page dims the others). */
    public boolean currentHas(String name) {
        return index >= 0 && frames.get(index).layerIndex(name) >= 0;
    }

    /** The layer groups seen so far ({@link LayerCatalog#groups}). */
    public List<String> groups() {
        return LayerCatalog.groups(layers);
    }

    /** The group the layer list shows: the one picked, else the current layer's. */
    public String group() {
        if (group != null && groups().contains(group))
            return group;
        return layer == null ? "" : LayerCatalog.group(layer);
    }

    /** Shows another group in the layer list (the displayed layer does not change). */
    public void selectGroup(String g) {
        if (groups().contains(g))
            group = g;
    }

    /** The next group after the shown one (wrapping), for the page's group button. */
    public void cycleGroup(int delta) {
        List<String> gs = groups();
        if (gs.isEmpty())
            return;
        int i = Math.max(0, gs.indexOf(group()));
        group = gs.get(Math.floorMod(i + delta, gs.size()));
    }

    /** The layers of the shown group, in first-seen order. */
    public List<String> groupLayers() {
        return LayerCatalog.inGroup(layers, group());
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
     * The frame to draw: the live frame while it shows ({@link #showingLive()}), else the displayed
     * frame once decoded, else the one drawn before (so the view never flashes empty while a frame
     * loads); empty before the first frame is decoded.
     */
    public Optional<Frame> displayed() {
        RunHandle.LiveFrame lf = liveShown();
        if (lf != null) {
            if (shown == null || shown.abgr() != lf.abgr()) {
                shown = new Frame(lf.info(), 0, lf.abgr());
                version++;
            }
            return Optional.of(shown);
        }
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

    /** True when time {@code t} lies after the previous frame (a History row that is new with this frame). */
    public boolean freshAt(double t) {
        return t > previousTime();
    }

    /** The run's length in years ({@code timeline.length_myr}), NaN when the preset has none. */
    public double lengthYears() {
        if (Double.isNaN(lengthYears)) {
            try {
                lengthYears = com.minecraftindustrialcomplex.deeptime.preset.Presets.info(
                                com.minecraftindustrialcomplex.deeptime.preset.Presets.parseObject(presetJson))
                        .timeline().map(t -> t.lengthMyr() * 1e6).orElse(Double.POSITIVE_INFINITY);
            } catch (RuntimeException e) {
                lengthYears = Double.POSITIVE_INFINITY;
            }
        }
        return Double.isInfinite(lengthYears) ? Double.NaN : lengthYears;
    }

    private double lengthYears = Double.NaN;

    /** True when {@code e} happened after the previous frame (drawn larger, highlighted in the History). */
    public boolean fresh(GeoEvent e) {
        return e.tStart() > previousTime();
    }

    /**
     * The time the map's markers are drawn at: the displayed frame's, or while following a live run
     * the newest of that frame and the newest event (events arrive between frames).
     */
    double markerTime() {
        if (index < 0)
            return Double.NEGATIVE_INFINITY;
        double t = frames.get(index).t();
        if (following() && !events.isEmpty())
            t = Math.max(t, events.get(events.size() - 1).tStart());
        return t;
    }

    /**
     * The map's markers: point events that began by {@link #markerTime()} and ended at most
     * {@link #MARKER_FADE_YEARS} before it, at most {@link #MAX_MARKERS} of them (the newest), oldest
     * first (fresh ones are drawn last, on top).
     */
    public List<GeoEvent> markers() {
        double t = markerTime();
        List<GeoEvent> out = new ArrayList<>();
        // Events are sorted by start time: walk back from the newest that began by t.
        for (int i = events.size() - 1; i >= 0 && out.size() < MAX_MARKERS; i--) {
            GeoEvent e = events.get(i);
            if (e.tStart() > t || !e.hasPoint())
                continue;
            double end = Math.min(Math.max(e.tEnd(), e.tStart()), t);
            if (t - end <= MARKER_FADE_YEARS)
                out.add(e);
            else if (t - e.tStart() > MARKER_FADE_YEARS + MAX_EVENT_SPAN_YEARS)
                break; // nothing older can still be within the window
        }
        Collections.reverse(out);
        return out;
    }

    /** No event is assumed to last longer than this (the walk back in {@link #markers()} stops past it). */
    static final double MAX_EVENT_SPAN_YEARS = 500e6;

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
     * The estimate the run was started with (its stages' seconds weight the time left; see
     * {@link Estimate#timeLeftMs}); without one the session's own figure is used.
     */
    public void setEstimate(Estimate estimate) {
        this.estimate = estimate;
    }

    /** Sets the wall clock (tests). */
    void clock(java.util.function.LongSupplier clock) {
        this.clock = clock;
    }

    /** True once the stages are done and the run writes its world data (the bake), until it is done. */
    public boolean baking() {
        return state == State.RUNNING && progress.stageCount() > 0 && progress.state() != SimSession.Progress.RUNNING;
    }

    /** The run's estimate when it describes this session's stages, else null. */
    private Estimate matching(SimSession.Progress p) {
        Estimate e = estimate;
        return e != null && e.stages().size() == p.stageCount() ? e : null;
    }

    private long bakeElapsedMs() {
        return bakeStartedMs < 0 ? 0 : clock.getAsLong() - bakeStartedMs;
    }

    /**
     * Milliseconds left: from the estimate while the stages step and while the bake runs (the
     * session's elapsed time stops with its stages), else the session's figure.
     */
    public long timeLeftMs() {
        SimSession.Progress p = progress;
        Estimate e = matching(p);
        if (e != null && p.state() == SimSession.Progress.RUNNING) {
            long t = e.timeLeftMs(p.stageIndex(), p.stagePermille(), p.elapsedMs());
            if (t >= 0)
                return t;
        }
        if (e != null && baking())
            return e.bakeLeftMs(p.elapsedMs(), bakeElapsedMs());
        return p.etaMs();
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
                    b.append(run.waitingForPrevious() ? "waiting for the previous run to stop" : "starting");
                } else if (p.state() == SimSession.Progress.RUNNING) {
                    b.append(stageName()).append(' ').append(Formats.percent(p.stagePermille()))
                            .append(" (").append(Formats.ma(p.simulatedTime())).append(')')
                            .append(" · total ").append(Formats.percent(permille()));
                } else {
                    b.append("writing world data");
                }
            }
            case DONE -> b.append("done · ").append(frames.size()).append(" keyframes · ").append(events.size()).append(" events");
            case CANCELLED -> b.append("cancelled");
            case FAILED -> b.append("failed: ").append(failure);
        }
        b.append(" · ").append(Formats.duration(p.elapsedMs())).append(" elapsed");
        if (state == State.RUNNING && p.stageCount() > 0) {
            long eta = timeLeftMs();
            b.append(" · ").append(eta < 0 ? "time left unknown" : "~" + Formats.duration(eta) + " left");
        }
        b.append(" · native ").append(Formats.bytes(state == State.RUNNING ? p.nativeBytesInUse() : p.nativeBytesPeak()))
                .append(" of ").append(Formats.gib(capBytes)).append(" GiB");
        return b.toString();
    }

    /**
     * Total progress 0…1 (1 once done): weighted by the estimate's seconds, bake included
     * ({@link Estimate#fraction}), else the session's own figure, which counts every stage alike.
     */
    public double fraction() {
        if (state == State.DONE)
            return 1;
        SimSession.Progress p = progress;
        Estimate e = matching(p);
        if (e != null && p.state() == SimSession.Progress.RUNNING) {
            double f = e.fraction(p.stageIndex(), p.stagePermille());
            if (f >= 0)
                return f;
        }
        if (e != null && baking())
            return e.bakeFraction(p.elapsedMs(), bakeElapsedMs());
        return Math.max(0, Math.min(1, p.totalPermille() / 1000.0));
    }

    /** {@link #fraction} in permille, for the percentage shown. */
    public int permille() {
        return (int) Math.round(fraction() * 1000);
    }
}
