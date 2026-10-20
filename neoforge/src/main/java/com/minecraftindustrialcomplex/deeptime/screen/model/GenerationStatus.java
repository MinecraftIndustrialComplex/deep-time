// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;

import java.util.List;
import java.util.Optional;

/**
 * What the generation page says about the run (docs/design/create-world-ux.md): one sentence in
 * plain words ("Moving the continents, 640 million years ago"), the progress fraction and the time
 * left. The developer's line with stage percentages and native memory
 * ({@link TimelapseModel#progressLine}) stays behind Details.
 */
public final class GenerationStatus {

    private GenerationStatus() {}

    public enum State {
        /** Generate waits for the estimate. */
        PREPARING,
        /** Nothing runs and nothing will start by itself: a blocking problem or a failed launch ({@link EditorModel#problems}). */
        BLOCKED,
        /** The run is launched and waits for the previous run's natives, or for its first stage. */
        STARTING,
        /** The simulation steps through its stages. */
        RUNNING,
        /** The stages are done and the world data is being written. */
        BAKING,
        DONE,
        CANCELLED,
        FAILED
    }

    public static State state(EditorModel m) {
        Optional<TimelapseModel> run = m.run();
        if (run.isEmpty())
            return m.generating() ? State.PREPARING : State.BLOCKED;
        TimelapseModel r = run.get();
        return switch (r.state()) {
            case DONE -> State.DONE;
            case CANCELLED -> State.CANCELLED;
            case FAILED -> State.FAILED;
            case RUNNING -> {
                SimSession.Progress p = r.progress();
                if (p.stageCount() == 0)
                    yield State.STARTING;
                yield p.state() == SimSession.Progress.RUNNING ? State.RUNNING : State.BAKING;
            }
        };
    }

    /** The status sentence. */
    public static Phrase sentence(EditorModel m) {
        State s = state(m);
        return switch (s) {
            case PREPARING -> Phrase.of("gen.status.preparing");
            case BLOCKED -> Phrase.of("gen.status.blocked");
            case STARTING -> m.run().map(r -> r.run().waitingForPrevious()).orElse(false)
                    ? Phrase.of("gen.status.waiting_previous") : Phrase.of("gen.status.starting");
            case RUNNING -> {
                TimelapseModel r = m.run().orElseThrow();
                Phrase stage = stage(r.stageName());
                long ago = r.progress().simulatedTime(); // years × −1
                yield ago > 0 ? Phrase.of("gen.status.stage_at", stage, Phrase.yearsAgo(-ago)) : stage;
            }
            case BAKING -> Phrase.of("gen.status.baking");
            case DONE -> Phrase.of("gen.status.done");
            case CANCELLED -> Phrase.of("gen.status.cancelled");
            case FAILED -> Phrase.of("gen.status.failed");
        };
    }

    /** The core's stage names, each with words in the language file ({@code stage.<name>}). */
    public static final List<String> STAGES = List.of("plates", "climate", "surface", "volcanoes", "impacts", "structure",
            "deposits", "young", "amplify");

    /**
     * A stage's words ({@code stage.<name>}; the screen falls back to the humanised name for a stage
     * the language lacks), or {@code stage.other} when the run has not named its stage.
     */
    public static Phrase stage(String name) {
        return name != null && name.matches("[a-z0-9_]+") ? Phrase.of("stage." + name) : Phrase.of("stage.other");
    }

    /** The time left while the run goes. */
    public static Optional<Phrase> timeLeft(EditorModel m) {
        State s = state(m);
        if (s == State.RUNNING || s == State.BAKING)
            return Optional.of(Phrase.timeLeft(m.run().orElseThrow().timeLeftMs()));
        if (s == State.PREPARING || s == State.STARTING)
            return m.estimate().map(e -> Phrase.of("gen.status.expected", Phrase.duration(e.seconds())));
        return Optional.empty();
    }

    /** The progress bar, 0 to 1, weighted by the stages' estimated seconds and the bake ({@link TimelapseModel#fraction}). */
    public static double fraction(EditorModel m) {
        return m.run().map(TimelapseModel::fraction).orElse(0.0);
    }
}
