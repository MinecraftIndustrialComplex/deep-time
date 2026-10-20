// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * The rule that keeps Distant Horizons' world generator on while on a Deep Time planet
 * ({@link DhGenerationGate}): it lifts only Distant Horizons' Chunky override, puts it back off the
 * planet, and never touches the player's file or another mod's later "off".
 */
class DhGenerationGateTest {

    /** Distant Horizons' two layers of the setting: the player's file and the API override. */
    static final class FakeSlot implements DhGenerationGate.Slot {
        boolean player = true;
        Boolean layer;
        boolean overridable = true;
        int writes;

        FakeSlot(Boolean layer) {
            this.layer = layer;
        }

        boolean effective() {
            return layer != null ? layer : player;
        }

        @Override
        public boolean playerValue() {
            return player;
        }

        @Override
        public Boolean apiValue() {
            return layer;
        }

        @Override
        public boolean overridable() {
            return overridable;
        }

        @Override
        public boolean setApi(boolean enabled) {
            if (!overridable)
                return false;
            writes++;
            layer = enabled;
            return true;
        }

        @Override
        public boolean clearApi() {
            if (!overridable)
                return false;
            writes++;
            layer = null;
            return true;
        }
    }

    @Test
    void chunkysOverrideIsLiftedOnThePlanetAndPutBackOffIt() {
        FakeSlot slot = new FakeSlot(false); // what Distant Horizons sets at client start with Chunky installed
        DhGenerationGate.Controller gate = new DhGenerationGate.Controller();
        assertEquals(DhGenerationGate.Outcome.UNCHANGED, gate.reconcile(slot, false, true), "the title screen is not the planet");
        assertFalse(slot.effective());
        assertEquals(DhGenerationGate.Outcome.LIFTED, gate.reconcile(slot, true, true));
        assertTrue(slot.effective(), "generation follows the player's file again");
        assertTrue(gate.lifted());
        assertEquals(DhGenerationGate.Outcome.UNCHANGED, gate.reconcile(slot, true, true));
        assertEquals(1, slot.writes, "nothing is written while nothing changes");
        assertEquals(DhGenerationGate.Outcome.RESTORED, gate.reconcile(slot, false, true), "leaving the planet");
        assertEquals(Boolean.FALSE, slot.layer);
        // Back on the planet (the Nether and back, or another join): its own "off" is lifted again.
        assertEquals(DhGenerationGate.Outcome.LIFTED, gate.reconcile(slot, true, true));
        assertNull(slot.layer);
        assertTrue(slot.player, "the player's file is never written");
    }

    @Test
    void withoutChunkyAnOffOverrideIsAnotherModsAndStays() {
        FakeSlot slot = new FakeSlot(false);
        DhGenerationGate.Controller gate = new DhGenerationGate.Controller();
        assertEquals(DhGenerationGate.Outcome.FOREIGN_OVERRIDE, gate.reconcile(slot, true, false));
        assertEquals(Boolean.FALSE, slot.layer);
        assertEquals(0, slot.writes);
        assertEquals(DhGenerationGate.Outcome.UNCHANGED, gate.reconcile(slot, false, false));
        assertEquals(0, slot.writes, "nothing of ours to put back");
    }

    @Test
    void anOffSetAgainAfterTheLiftIsLeftAloneForTheSession() {
        FakeSlot slot = new FakeSlot(false);
        DhGenerationGate.Controller gate = new DhGenerationGate.Controller();
        assertEquals(DhGenerationGate.Outcome.LIFTED, gate.reconcile(slot, true, true));
        slot.layer = false; // another mod (or Distant Horizons again) wants it off
        assertEquals(DhGenerationGate.Outcome.FOREIGN_OVERRIDE, gate.reconcile(slot, true, true));
        assertEquals(DhGenerationGate.Outcome.FOREIGN_OVERRIDE, gate.reconcile(slot, true, true));
        assertEquals(Boolean.FALSE, slot.layer);
        assertFalse(gate.lifted());
        assertEquals(DhGenerationGate.Outcome.UNCHANGED, gate.reconcile(slot, false, true), "theirs is not ours to put back");
        assertEquals(1, slot.writes);
    }

    @Test
    void anOnOverrideOrNoneIsLeftAlone() {
        DhGenerationGate.Controller gate = new DhGenerationGate.Controller();
        FakeSlot on = new FakeSlot(true);
        assertEquals(DhGenerationGate.Outcome.UNCHANGED, gate.reconcile(on, true, true));
        FakeSlot none = new FakeSlot(null);
        assertEquals(DhGenerationGate.Outcome.UNCHANGED, gate.reconcile(none, true, true));
        assertEquals(DhGenerationGate.Outcome.UNCHANGED, gate.reconcile(none, false, true));
        assertEquals(0, on.writes + none.writes);
    }

    @Test
    void thePlayersOwnOffIsReportedNotChanged() {
        FakeSlot slot = new FakeSlot(false);
        slot.player = false; // generatorPlan = DISABLED in the player's own file
        DhGenerationGate.Controller gate = new DhGenerationGate.Controller();
        assertEquals(DhGenerationGate.Outcome.LIFTED, gate.reconcile(slot, true, true));
        assertFalse(slot.effective(), "the player's DISABLED decides");
        assertEquals(DhGenerationGate.Outcome.PLAYER_OFF, gate.reconcile(slot, true, true));
        assertFalse(slot.player);
    }

    @Test
    void aSettingModsCannotOverrideIsReported() {
        FakeSlot slot = new FakeSlot(false);
        slot.overridable = false;
        DhGenerationGate.Controller gate = new DhGenerationGate.Controller();
        assertEquals(DhGenerationGate.Outcome.NOT_OVERRIDABLE, gate.reconcile(slot, true, true));
        assertEquals(Boolean.FALSE, slot.layer);
        assertFalse(gate.lifted());
    }
}
