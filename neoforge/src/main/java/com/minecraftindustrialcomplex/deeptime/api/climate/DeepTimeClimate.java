// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.climate;

import net.minecraft.core.BlockPos;
import net.minecraft.world.level.Level;

import java.util.Optional;
import java.util.OptionalDouble;

/**
 * The simulated climate of a Deep Time level, for other mods (P1-25; MIC's mic-climate reads it to
 * give machines, players and chemistry the planet's own temperatures).
 *
 * <p>A Deep Time world is generated from a simulated planet whose climate (an energy-balance model
 * with seasons, wind belts, continentality, rain shadows and orographic rain; plan §8.3) was
 * computed for every 16-block cell. Minecraft's biomes only carry a coarse band of that climate;
 * this interface hands out the numbers themselves, read from the world data's bake: per block, the
 * twelve monthly mean temperatures at the block's height (the simulation's lapse rate applied over
 * the block's height above the sea), the annual mean, the warmest and coldest month, annual and
 * monthly precipitation and the Köppen class. Values are bilinear between cells (the Köppen class
 * is the nearest cell's), deterministic, and constant for the life of the world: they are the
 * <em>climate</em> (long-term monthly means), not the weather. Weather, the time of day and a
 * season calendar are the caller's to add; {@link ClimateSample#temperatureC(double)} gives the
 * mean temperature at any point of the year.
 *
 * <p>Server side only (the world data lives with the server): on a client level, or any level Deep
 * Time did not generate, {@link #isDeepTimeLevel()} is false and every query is empty. A Deep Time
 * level without a simulated climate (the analytic test terrains) has {@link #hasClimate()} false.
 * Thread-safe; lookups are cached per block column, so calling it from a tick is cheap. The one
 * exception is the planet's geometry ({@link #circumferenceBlocks(Level)}, {@link #latitudeDeg}),
 * which a client knows too.
 *
 * <pre>{@code
 * DeepTimeClimate climate = DeepTimeClimate.of(level);
 * if (climate.hasClimate())
 *     climate.at(pos).ifPresent(c -> use(c.temperatureC(yearFraction), c.koppen()));
 * }</pre>
 *
 * <p><b>Stability.</b> This package is Deep Time's public climate API. {@link #API_VERSION} goes up
 * by one whenever a method is added; nothing is removed or changes meaning within a major version of
 * the mod. Soft-depend on it: check {@code ModList.get().isLoaded("deeptime")} before touching the
 * class, and {@link #API_VERSION} for newer methods.
 *
 * <p><b>The year.</b> The simulated year has twelve equal months. Month 0 starts at solar longitude
 * 280°, about 1 January on Earth; the planet's northern spring equinox falls at year fraction
 * {@link #NORTHERN_SPRING_EQUINOX} (early in month 2). The southern hemisphere's seasons are the
 * other way round, as the data shows: a caller mapping a global season calendar (e.g. Serene
 * Seasons) onto a year fraction gets opposite seasons north and south of the equator for free.
 */
public interface DeepTimeClimate {

    /**
     * This interface's version: 1 = P1-25; 2 adds the planet's geometry on both sides
     * ({@link #circumferenceBlocks(Level)}, {@link #latitudeDeg}); 3 adds the warm-ground signal of the
     * geothermal features ({@link #geothermalWarmthC}). A caller compiled against a newer
     * version than the installed Deep Time gets a {@link NoSuchMethodError} from a newer method; read
     * this field reflectively (javac inlines it) or catch that error.
     */
    int API_VERSION = 3;

    /** Months in the simulated year. */
    int MONTHS = 12;

    /** Year fraction of the northern spring equinox (solar longitude 0°). */
    double NORTHERN_SPRING_EQUINOX = 80.0 / 360.0;

    /** Year fraction of the northern summer solstice (solar longitude 90°). */
    double NORTHERN_SUMMER_SOLSTICE = 170.0 / 360.0;

    /**
     * The climate of {@code level}: its world data's for a Deep Time level on the server, and an
     * empty implementation ({@link #isDeepTimeLevel()} false, every query empty) for anything else.
     * Cheap: the same object is returned for the same world.
     */
    static DeepTimeClimate of(Level level) {
        return LevelClimate.of(level);
    }

    /**
     * The planet's circumference C in blocks (API 2): its east-west period, and the scale of latitude,
     * φ = −z · 360° / C ({@link #latitudeDeg}). A positive C means {@code level} is a Deep Time planet
     * (a real one or an analytic test terrain); any other level answers 0.
     *
     * <p>Unlike the rest of this interface it answers on both sides. On the server it is the level's
     * generator's; on a client it is the planet info the server sends at login, respawn and dimension
     * change, and 0 until that has arrived, for a level the info is not about, and after leaving the
     * server. Cheap and allocation-free on both sides, so per-block callers (colour resolvers on mesh
     * threads, per-column precipitation) may call it every time; it is safe from any thread.
     */
    static int circumferenceBlocks(Level level) {
        return LevelClimate.circumferenceBlocks(level);
    }

    /**
     * Latitude in degrees at block {@code z} on a planet of circumference {@code circumferenceBlocks}
     * (API 2): φ = −z · 360° / C, north (−z) positive, the equator at z = 0 and the poles at
     * |z| = C/4. NaN when the circumference is not positive (not a planet).
     */
    static double latitudeDeg(double z, int circumferenceBlocks) {
        return circumferenceBlocks > 0 ? -z * 360.0 / circumferenceBlocks : Double.NaN;
    }

    /** Year fraction [0, 1) at solar longitude {@code degrees} (0° = northern spring equinox). */
    static double yearFractionAtSolarLongitude(double degrees) {
        double f = (degrees - 280.0) / 360.0;
        return f - Math.floor(f);
    }

    /** True for a server level whose generator is Deep Time's. */
    boolean isDeepTimeLevel();

    /** True when the level's world data holds the simulated climate (every real planet does). */
    boolean hasClimate();

    /** The climate at the block (its y is the height the lapse rate is applied at), if any. */
    Optional<ClimateSample> at(BlockPos pos);

    /** The climate at block (x, y, z), if any. */
    Optional<ClimateSample> at(int x, int y, int z);

    /** Mean temperature at the block at year fraction {@code yearFraction} (°C), if any. */
    default OptionalDouble temperatureC(BlockPos pos, double yearFraction) {
        return at(pos).map(c -> OptionalDouble.of(c.temperatureC(yearFraction))).orElse(OptionalDouble.empty());
    }

    /**
     * The warm-ground signal of the planet's geothermal features (API 3, P2-14): how many kelvin
     * warmer than {@link #at}'s temperature the block at (x, y, z) is because it lies in or beside a
     * hot spring, a fumarole or a field of altered ground; 0 everywhere else, on a level without
     * geothermal features (a density of 0, a planet without active volcanoes) and on any level that
     * is not a Deep Time planet. At most 12 K, at the heart of a vent, falling off linearly to 0 a
     * few blocks beyond the feature's altered ground, within a few blocks of the ground's height.
     * Deterministic and constant for the life of the world, like the rest of this interface; the
     * caller decides what, if anything, it does with it.
     */
    default double geothermalWarmthC(int x, int y, int z) {
        return 0.0;
    }

    /** The simulation's lapse rate, °C per km (NaN without a climate). */
    double lapseRateCPerKm();

    /** The level's sea level, block y (the lapse rate applies above it). */
    int seaLevelY();
}
