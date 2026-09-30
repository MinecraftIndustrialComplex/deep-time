// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.api.climate.ClimateSample;
import com.minecraftindustrialcomplex.deeptime.api.climate.DeepTimeClimate;
import com.mojang.brigadier.CommandDispatcher;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.commands.arguments.coordinates.BlockPosArgument;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;

import java.util.Locale;

/**
 * {@code /deeptime climate [pos]} (P1-25): what the public climate API ({@link DeepTimeClimate})
 * answers at a block, as a chat line and as one JSON probe line in the server log
 * ({@code [deeptime-probe] {"probe": "climate", ...}}, what the smoke harness records) for scripts
 * comparing it with other mods' temperatures.
 */
public final class ClimateCommand {

    private ClimateCommand() {}

    public static void register(CommandDispatcher<CommandSourceStack> dispatcher) {
        dispatcher.register(Commands.literal("deeptime").requires(s -> s.hasPermission(2))
                .then(Commands.literal("climate")
                        .executes(c -> report(c.getSource(), BlockPos.containing(c.getSource().getPosition())))
                        .then(Commands.argument("pos", BlockPosArgument.blockPos())
                                .executes(c -> report(c.getSource(), BlockPosArgument.getBlockPos(c, "pos"))))));
    }

    static int report(CommandSourceStack source, BlockPos pos) {
        DeepTimeClimate climate = DeepTimeClimate.of(source.getLevel());
        JsonObject o = new JsonObject();
        o.addProperty("probe", "climate");
        o.addProperty("api", DeepTimeClimate.API_VERSION);
        o.addProperty("level", source.getLevel().dimension().location().toString());
        o.addProperty("x", pos.getX());
        o.addProperty("y", pos.getY());
        o.addProperty("z", pos.getZ());
        o.addProperty("deep_time_level", climate.isDeepTimeLevel());
        o.addProperty("has_climate", climate.hasClimate());
        ClimateSample s = climate.at(pos).orElse(null);
        String text;
        if (s == null) {
            text = climate.isDeepTimeLevel() ? "Deep Time: this level has no simulated climate"
                    : "Deep Time: not a Deep Time level";
        } else {
            o.addProperty("koppen", s.koppen());
            o.addProperty("elevation_m", s.elevationMetres());
            o.addProperty("mean_c", s.meanC());
            o.addProperty("warmest_c", s.warmestMonthC());
            o.addProperty("coldest_c", s.coldestMonthC());
            o.addProperty("warmest_month", s.warmestMonth());
            o.addProperty("coldest_month", s.coldestMonth());
            o.addProperty("precip_mm", s.annualPrecipitationMm());
            o.addProperty("precip_seasonality", s.precipitationSeasonality());
            JsonArray t = new JsonArray(), p = new JsonArray();
            for (int m = 0; m < DeepTimeClimate.MONTHS; m++) {
                t.add(Math.round(s.monthC(m) * 100.0) / 100.0);
                p.add(Math.round(s.monthPrecipitationMm(m) * 10.0) / 10.0);
            }
            o.add("month_c", t);
            o.add("month_mm", p);
            text = String.format(Locale.ROOT,
                    "Deep Time climate at %d %d %d: %s, mean %.1f C (warmest month %.1f, coldest %.1f), %.0f mm/yr, "
                            + "%.0f m above the sea",
                    pos.getX(), pos.getY(), pos.getZ(), s.koppen(), s.meanC(), s.warmestMonthC(), s.coldestMonthC(),
                    s.annualPrecipitationMm(), s.elevationMetres());
        }
        DeepTime.LOGGER.info("{}{}", DeepTimeCommands.PROBE_PREFIX, o);
        source.sendSuccess(() -> Component.literal(text), false);
        return s == null ? 0 : 1;
    }
}
