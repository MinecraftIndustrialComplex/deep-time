// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api.climate;

import com.minecraftindustrialcomplex.deeptime.api.planet.DeepTimePlanet;
import com.minecraftindustrialcomplex.deeptime.api.planet.LocalFrame;
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
 * computed once per climate cell (16 blocks wide on a 16,384-block planet, 32 on a 32,768-block one:
 * the planet's circumference over 1,024 columns; ledger BIO-097). Minecraft's biomes only carry a coarse band of that climate;
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
 * exception is the planet's geometry ({@link #latitudeDeg}, {@link #longitudeDeg}, {@link
 * #northBearingDeg}, {@link #metresPerBlock}), which a client knows too: those are shorthands for
 * Deep Time's planet API, {@link DeepTimePlanet}, which reads the world as a sphere whatever its map
 * and has the rest of the geometry (radius, canonical positions, distances, bearings).
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
     * ({@code circumferenceBlocks(Level)}, removed in 6, and a Lambert-only latitude from z, removed in 5); 3 adds the
     * warm-ground signal of the geothermal features ({@link #geothermalWarmthC}); 4 adds the seamless
     * world (format 0.5): {@code projectionKind} (removed in 6), {@link #latitudeDeg(Level, double, double)} and
     * {@link #longitudeDeg(Level, double, double)}, whose answers depend on x and z on a Petroff–Guyou
     * world; 5 adds the cube world (format 0.7, projection {@code "cube"}): {@link #northBearingDeg}
     * and {@link #metresPerBlock}, and removes the static {@code latitudeDeg(double z, int
     * circumferenceBlocks)}, which was right on a Lambert world only (call {@link
     * #latitudeDeg(Level, double, double)} instead). A caller compiled against a newer
     * version than the installed Deep Time gets a {@link NoSuchMethodError} from a newer method; read
     * this field reflectively (javac inlines it) or catch that error. 6 moves the geometry to the planet
     * API ({@link DeepTimePlanet}, its own version): {@code circumferenceBlocks} and {@code
     * projectionKind} are gone (a mod has no use for a map's period or name; the planet API answers
     * on the sphere), and the four geometry statics answer through it, NaN wherever it says off the
     * planet (on a Lambert world that now includes past its poles, which read as the pole before).
     */
    int API_VERSION = 6;

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
     * Latitude in degrees at block (x, z) of the planet {@code level} (API 4), north positive: {@link
     * DeepTimePlanet#latitudeDeg}. NaN for a level that is not a planet (or on a client before the
     * planet info arrived) and at a position off the planet. Both sides; a few hundred nanoseconds, so
     * cache it per column if you call it per block.
     */
    static double latitudeDeg(Level level, double x, double z) {
        Optional<DeepTimePlanet> p = DeepTimePlanet.of(level);
        return p.isPresent() ? p.get().latitudeDeg(x, z) : Double.NaN;
    }

    /**
     * Longitude in degrees in [0, 360) at block (x, z) of the planet {@code level} (API 4), in the
     * planet's own frame, 0 at a pole: {@link DeepTimePlanet#longitudeDeg}. NaN as {@link
     * #latitudeDeg(Level, double, double)}.
     */
    static double longitudeDeg(Level level, double x, double z) {
        Optional<DeepTimePlanet> p = DeepTimePlanet.of(level);
        return p.isPresent() ? p.get().longitudeDeg(x, z) : Double.NaN;
    }

    /**
     * The direction of north at block (x, z) of the planet {@code level} (API 5), as a bearing in
     * degrees in [0, 360) in the block frame, clockwise seen from above from −z: 0 when north is −z,
     * 90 when it is +x, 180 when it is +z, 270 when it is −x ({@link LocalFrame#northBearingDeg} of
     * {@link DeepTimePlanet#frame}). It varies across the world and turns where the map turns (past an
     * edge the same planet point can read it turned), and is not defined at a pole drawn as a line or
     * where the map has no derivative. NaN there and as {@link #latitudeDeg(Level, double, double)}.
     * Both sides; allocates a little, so cache it per column.
     */
    static double northBearingDeg(Level level, double x, double z) {
        Optional<DeepTimePlanet> p = DeepTimePlanet.of(level);
        Optional<LocalFrame> f = p.isPresent() ? p.get().frame(x, z) : Optional.empty();
        return f.isPresent() ? f.get().northBearingDeg() : Double.NaN;
    }

    /**
     * The planet's horizontal metres per block at block (x, z) of {@code level} (API 5): the square root
     * of the block's ground area ({@link LocalFrame#metresPerBlock} of {@link DeepTimePlanet#frame}),
     * the world data's nominal scale (its manifest's {@code m_per_block_h}, e.g. about 300 m) times the
     * map's local scale there. On an equal-area map it is the nominal scale everywhere, though a block
     * can be longer one way than the other ({@link LocalFrame#metresPerBlockEast}, {@link
     * LocalFrame#metresPerBlockNorth}); on a conformal one it varies across the world. NaN as {@link
     * #northBearingDeg}. Both sides.
     */
    static double metresPerBlock(Level level, double x, double z) {
        Optional<DeepTimePlanet> p = DeepTimePlanet.of(level);
        Optional<LocalFrame> f = p.isPresent() ? p.get().frame(x, z) : Optional.empty();
        return f.isPresent() ? f.get().metresPerBlock() : Double.NaN;
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
