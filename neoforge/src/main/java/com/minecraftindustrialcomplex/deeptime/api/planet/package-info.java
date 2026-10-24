// SPDX-License-Identifier: LGPL-3.0-or-later
/**
 * Deep Time's planet-geometry API: a Deep Time world read as the sphere it stands for, whatever map
 * lays that sphere out in blocks. Start at {@link
 * com.minecraftindustrialcomplex.deeptime.api.planet.DeepTimePlanet#of}; versioned by {@link
 * com.minecraftindustrialcomplex.deeptime.api.planet.DeepTimePlanet#API_VERSION}. Only {@code
 * DeepTimePlanet}, {@code GeoPoint}, {@code BlockXZ} and {@code LocalFrame} are API; everything else
 * here is internal. The page docs/api/planet.md says the same with examples.
 *
 * <h2>The contract</h2>
 *
 * <p>This contract is the whole promise. It does not change when Deep Time adds a map or changes
 * one: a new or changed map changes which block shows which planet point, never what a call means.
 * A mod written against it never needs to know the map's name, its seams, its faces, its period or
 * where it wraps, and must not guess them from the answers.
 *
 * <p><b>The sphere.</b> A planet is a sphere of radius {@link
 * com.minecraftindustrialcomplex.deeptime.api.planet.DeepTimePlanet#radiusMetres()} metres, fixed for
 * the life of the world. Every call answers on that sphere, at sea level: a block's height (y) never
 * enters, so only x and z are asked for.
 *
 * <p><b>Positions.</b> A position is a point (x, z) of the level's block frame, in blocks, as
 * doubles; block (bx, bz) has its centre at (bx + 0.5, bz + 0.5). Each position shows at most one
 * planet point. A position that shows none is <em>off the planet</em>; which positions are off it
 * is fixed for the life of the world. On a position off the planet every call that returns an
 * {@code Optional} returns it empty and every call that returns a double returns NaN. A position
 * whose x or z is not finite is off the planet.
 *
 * <p><b>Many positions, one point.</b> Several positions can show the same planet point (the world
 * repeats, or a stretch past one edge shows ground that also lies elsewhere). Every answer about the
 * planet point (latitude, longitude, distance, bearing, destination, canonical position) is the
 * same whichever of those positions was asked, to rounding. Only the local frame belongs to the
 * position itself: it describes the block frame where you stand, so at two positions showing the
 * same point it can be turned or mirrored relative to each other.
 *
 * <p><b>The canonical position.</b> {@code canonical(x, z)} names, out of all the positions that
 * show the planet point at (x, z), the one Deep Time keeps it under. It is the same for each of them,
 * and for a block centre it is a block centre, computed exactly (compare canonical block centres with
 * {@code ==}). Floor it to get the canonical block. A point exactly on a line where the map meets
 * itself (a set of zero area that no block centre lies on) can have two canonical positions; compare
 * such points with {@code distanceMetres}. {@code blockPoint} returns the canonical position of a
 * latitude and longitude; every planet point has one, so it never fails.
 *
 * <p><b>Latitude and longitude.</b> Degrees. Latitude in [−90, 90], north positive. Longitude in
 * [0, 360), east positive, in the planet's own frame: it is fixed for the life of the world and is
 * the frame of Deep Time's simulation and climate, but its zero is not tied to any block. At a pole
 * the longitude is 0.
 *
 * <p><b>Bearings.</b> Two kinds, never mixed. A <em>geographic bearing</em> (the distance calls,
 * {@code destination}, {@code LocalFrame.step}) is degrees clockwise from geographic north on the
 * ground, in [0, 360): 0 north, 90 east, 180 south, 270 west. A <em>block-frame bearing</em>
 * ({@code LocalFrame.northBearingDeg}) is degrees clockwise from the block frame's −z seen from
 * above, in [0, 360): 0 is −z, 90 is +x, 180 is +z, 270 is −x (Minecraft's compass directions, which
 * on a Deep Time planet are not the planet's). At a pole, where north has no direction, geographic
 * bearings are measured as if the pole had been reached along the meridian of longitude 0.
 *
 * <p><b>Distances.</b> Metres along the great circle on the sphere, in [0, πR]. Distance, initial
 * bearing and destination are the sphere's, so they are continuous everywhere on the planet: across
 * every edge, seam, corner and wrap of the map, walking from one position to the next one block
 * away always moves a short distance. The initial bearing between two points is NaN when they are
 * the same point or antipodal (no single great circle joins them). A destination is a canonical
 * position; a negative distance goes the opposite way.
 *
 * <p><b>The local frame.</b> {@code frame(x, z)} is the first-order link between a step in the
 * block frame at (x, z) and a step on the ground: where north and east point in the block frame,
 * how many metres a block is along each, and the conversions both ways. North and east are unit
 * vectors in the block plane that a short step along moves due north or due east on the ground.
 * They are not always perpendicular in the block plane, and a block along one is not always as long
 * on the ground as a block along the other ({@code metresPerBlockNorth} and {@code
 * metresPerBlockEast} can differ); use {@code step} and {@code bearingOfStep} for any other
 * direction. {@code metresPerBlock} is the square root of one block's ground area. The frame is
 * exact in the limit of short steps; over long distances use the distance calls. It is empty off
 * the planet and at the isolated points or zero-area lines where the map has no derivative (a pole
 * drawn as a line, a cone point where the map folds), and it changes smoothly elsewhere except
 * where the map is creased; by how much a crease or an edge turns it is the map's business, so
 * read the frame where you stand rather than carrying one across.
 *
 * <p><b>Sides and threads.</b> Both sides. On the server a level Deep Time generated is a planet as
 * soon as it exists. On a client a level is a planet once the server's planet info for it has
 * arrived (at login, respawn and dimension change), and not before, nor after leaving the server.
 * Every object here is immutable and safe from any thread. {@code DeepTimePlanet.of} returns the
 * same object for the same world without allocating; the calls on it are pure arithmetic over the
 * map (a fraction of a microsecond), so per-block callers on mesh or worker threads may call them
 * every time, or cache per column.
 *
 * <p><b>Stability.</b> {@code DeepTimePlanet.API_VERSION} goes up by one whenever a call is added;
 * within a major version of the mod nothing is removed and nothing changes meaning, and adding or
 * changing a map is never a reason to. Soft-depend on it: check {@code
 * ModList.get().isLoaded("deeptime")} before touching any class here, and read the version
 * reflectively (javac inlines the field) before calling anything newer than version 1.
 */
package com.minecraftindustrialcomplex.deeptime.api.planet;
