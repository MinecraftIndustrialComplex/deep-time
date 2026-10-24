# The planet API (`api/planet`, version 1)

Deep Time lays its planet out in blocks with a map: Lambert's cylinder (the simulation's frame and
the analytic test terrains), Petroff's square in the Guyou layout (the seamless world, format 0.5)
or a cube net (format 0.7). Each map has its own seams, edges, corners, wraps and creases, and more
may come. The planet API lets another mod read any Deep Time world as the sphere it stands for, so
it never needs to know which map is in use or where its seams are (Ben, 2026-10-24: "call our world
as if it were a stable sphere regardless of what projections, seams, etc, we implement").

- Package: `com.minecraftindustrialcomplex.deeptime.api.planet`. Only `DeepTimePlanet`, `GeoPoint`,
  `BlockXZ` and `LocalFrame` are API; the rest of the package is internal.
- Version: `DeepTimePlanet.API_VERSION`, 1 since 2026-10-24. It has its own number, apart from the
  climate API's.
- The contract is the package documentation (`api/planet/package-info.java`). This page says the
  same with examples. Where the two differ, the package documentation wins and this page is wrong.
- Pure Java over the world's projection; no JNI, no world data on the client.

## Getting a planet

```java
// Soft dependency: never touch a Deep Time class unless the mod is there.
if (ModList.get().isLoaded("deeptime")) {
    DeepTimePlanet.of(level).ifPresent(planet -> {
        double lat = planet.latitudeDeg(pos.getX() + 0.5, pos.getZ() + 0.5);   // NaN off the planet
        // ...
    });
}
```

`DeepTimePlanet.of(level)` is empty for any level Deep Time did not generate, for `null`, and on a
client until the server's planet info for that level has arrived (at login, respawn and dimension
change). It returns the same object for the same world without allocating, from any thread.

Compile against the Deep Time jar with `compileOnly` and keep the classes that touch the API out of
the paths that run without Deep Time (mic-climate's `provider/DeepTimeSource` is a working example).
`API_VERSION` is a compile-time constant, so javac copies it into your class: read the installed
version reflectively before calling anything newer than version 1.

```java
int v = Class.forName("com.minecraftindustrialcomplex.deeptime.api.planet.DeepTimePlanet")
        .getField("API_VERSION").getInt(null);
```

## The calls

Positions are (x, z) in the level's block frame, as doubles; block (bx, bz) has its centre at
(bx + 0.5, bz + 0.5). Height never enters: every answer is on the sphere at sea level.

| call | contract |
|---|---|
| `radiusMetres()` | The sphere's radius in metres, fixed for the life of the world (ledger SCL-240). |
| `isOnPlanet(x, z)` | Whether the position shows a planet point. Which positions are off the planet is fixed for the life of the world; a non-finite x or z is off it. |
| `geoPoint(x, z)` | The latitude and longitude the position shows; empty off the planet. |
| `latitudeDeg(x, z)` | Degrees in [-90, 90], north positive; NaN off the planet. |
| `longitudeDeg(x, z)` | Degrees in [0, 360), east positive, in the planet's own frame (the simulation's and the climate's), 0 at a pole; NaN off the planet. Its zero is not tied to any block. |
| `blockPoint(GeoPoint)` | The canonical position of a latitude and longitude. Every planet point has one, so it never fails. |
| `canonical(x, z)` | Out of every position that shows the same planet point, the one Deep Time keeps it under: the same answer for each of them, and exactly a block centre for a block centre (compare with `==`, floor it for the block). Empty off the planet. |
| `frame(x, z)` | The local frame of the block frame at (x, z) (below). Empty off the planet and at the isolated points or zero-area lines where the map has no derivative (a pole drawn as a line, a cone point). |
| `distanceMetres(x0, z0, x1, z1)`, `distanceMetres(GeoPoint, GeoPoint)` | Great-circle metres between the two planet points, in [0, pi R]; NaN if either position is off the planet. |
| `initialBearingDeg(x0, z0, x1, z1)`, `initialBearingDeg(GeoPoint, GeoPoint)` | The geographic bearing in [0, 360) at the first point of the great circle to the second; NaN off the planet, or when the points are the same or antipodal. |
| `destination(x, z, bearingDeg, metres)` | The canonical position reached along the great circle leaving at that geographic bearing; a negative distance goes the other way. Empty off the planet or for a non-finite argument. |
| `destination(GeoPoint, bearingDeg, metres)` | The planet point reached the same way; throws `IllegalArgumentException` for a non-finite bearing or distance. |

`LocalFrame`, the first-order link between a step in the block frame and a step on the ground at
one position:

| call | contract |
|---|---|
| `northBearingDeg()` | Where geographic north points in the block frame, as a block-frame bearing in [0, 360): 0 is -z, 90 is +x, 180 is +z, 270 is -x. |
| `northX()`, `northZ()` | The unit vector in the block plane along which a short step goes due north. |
| `eastX()`, `eastZ()` | The unit vector in the block plane along which a short step goes due east. Not always perpendicular to north. |
| `metresPerBlockNorth()`, `metresPerBlockEast()` | Ground metres one block along north (or east) covers. They can differ. |
| `metresPerBlock()` | The square root of one block's ground area: the scale for areas and densities. |
| `metresOfStep(dx, dz)` | Ground metres of the block-frame step (dx, dz). |
| `bearingOfStep(dx, dz)` | The geographic bearing in [0, 360) of the block-frame step; NaN for no step. |
| `step(bearingDeg, metres)` | The block-frame step that goes that far on the ground at that geographic bearing. |

## The rules worth remembering

- **One point, many positions.** A world can show the same planet point at several positions (it
  repeats east to west, and a stretch past an edge shows ground that also lies elsewhere). Every
  answer about the point (latitude, longitude, distance, bearing, destination, canonical position)
  is the same at each of them. Only the local frame belongs to the position: at two positions that
  show the same point it can be turned or mirrored.
- **Two kinds of bearing.** Geographic bearings (distance calls, `destination`, `step`,
  `bearingOfStep`) are clockwise from north on the ground. Block-frame bearings
  (`northBearingDeg`) are clockwise from -z seen from above. Minecraft's compass directions are not
  the planet's.
- **Poles.** At a pole the longitude is 0, and geographic bearings are measured as if the pole had
  been reached along the meridian of longitude 0.
- **Glued lines and cuts.** Where the world glues one stretch of blocks to the next (a wrap, a
  seam, a turning edge), distances run on unbroken: two positions a block apart across the line are
  about a block apart on the ground. A world can also hold cuts, lines where two stretches showing
  ground from elsewhere meet without being glued. On the cube world these are the diagonals through
  the squares beside its corners. Positions either side of a cut show points that are far apart,
  and the distance calls say so. Never assume that one block is a short step on the ground; ask
  `distanceMetres`.
- **North is not uphill in latitude.** North is where a short step goes due north. On a map that
  keeps angles (Lambert, the cube) that is also the direction in which latitude climbs fastest; on
  an equal-area map that shears (Petroff-Guyou) the two can differ by tens of degrees (up to 44.9
  degrees in the contract test), though the latitude gradient is always perpendicular to east. Use
  `northX/northZ` to walk north and `step` for any other heading.
- **Read the frame where you stand.** The frame changes smoothly except where the map is creased
  (Petroff's map is piecewise) or at an edge. How much it turns there is the map's business, so
  never carry a frame from one position to another; ask again.
- **Threads and cost.** Every object is immutable and safe from any thread. The calls are pure
  arithmetic over the map, a fraction of a microsecond each; cache per column if you call them per
  block in a mesh.

## Examples

A machine's latitude for seasons, on either side:

```java
double lat = planet.latitudeDeg(x + 0.5, z + 0.5);
boolean south = lat < 0.0;
```

A compass needle on a client (the needle's angle in the block frame):

```java
planet.frame(player.getX(), player.getZ())
        .ifPresent(f -> needle.setAngleDeg(f.northBearingDeg()));
```

The distance between two machines and the heading from one to the other, correct across any seam:

```java
double km = planet.distanceMetres(ax + 0.5, az + 0.5, bx + 0.5, bz + 0.5) / 1000.0;
double heading = planet.initialBearingDeg(ax + 0.5, az + 0.5, bx + 0.5, bz + 0.5);   // 0 = north
```

A point 50 km due east, wherever the map's seams lie:

```java
planet.destination(x, z, 90.0, 50_000.0).ifPresent(p -> target(p.blockX(), p.blockZ()));
```

One kilometre north on foot (short steps, the frame re-read at each):

```java
double px = x, pz = z, left = 1000.0;
while (left > 0) {
    LocalFrame f = planet.frame(px, pz).orElseThrow();
    double m = Math.min(left, f.metresPerBlockNorth());
    BlockXZ s = f.step(0.0, m);
    px += s.x();
    pz += s.z();
    left -= m;
}
```

A density in a column (things per square kilometre) from a count per block:

```java
double km2PerBlock = Math.pow(f.metresPerBlock() / 1000.0, 2);
```

## The climate API's geometry

The climate API (`api/climate/DeepTimeClimate`, version 6) keeps four static shorthands that answer
through this API: `latitudeDeg`, `longitudeDeg`, `northBearingDeg` and `metresPerBlock`, each taking
a `Level`. Version 6 removed `circumferenceBlocks` and `projectionKind`: a mod has no use for a map's
period or name. Off the planet the shorthands answer NaN, which on a Lambert world now includes the
rows past its poles (they used to read as the pole).

## How the contract is tested

`neoforge/src/test/java/com/minecraftindustrialcomplex/deeptime/api/planet/DeepTimePlanetContractTest.java`
runs the whole contract on seven worlds: Lambert 16,384; Petroff-Guyou b 4,096 at lambda_c 37.25
and b 1,536 at 0; the cube with face 1,024 at three orientations (the identity, a polar tilt of 30
degrees and a general rotation). It checks round trips (position to latitude and longitude and
back, exact canonical block centres over every image a position has), the frame against distance
and latitude over half-block steps in eight directions, distances, bearings and destinations
across every seam, edge and wrap, rings round every cone point and corner, long-range agreement of
distance, bearing and destination, the pole convention, empty answers off the planet, and the
lookup. Each run prints its measurements on lines starting `[contract]`. A new map passes this
suite before it ships; the GameTests `climateApiOnTheRealPlanet`, `climateApiIsEmptyElsewhere`,
`climateApiOnTheCube` and `seamlessWorldLayoutAndFold` check the lookup on real levels.
